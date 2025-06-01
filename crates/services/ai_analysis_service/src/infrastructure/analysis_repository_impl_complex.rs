use crate::domain::analysis_models::{AnalysisResult, VulnerabilityFinding};
use crate::domain::analysis_repository_trait::{AnalysisRepository, VulnerabilityStatistics};
use crate::error::{Error, Result};
use crate::infrastructure::data_models::{CodeAnalysisResultDmc, SecurityVulnerabilityDmc, GitHubRepositoryDmc};
use crate::infrastructure::model_adapters::SecurityVulnerabilityForCreate;
use async_trait::async_trait;
use jd_core::{AppState, base};
use jd_domain::zkpersona_domain::developer_models::{
    CodeAnalysisResult, CodeAnalysisResultForCreate,
    SecurityVulnerability, SecurityVulnerabilityFilter,
    GitHubRepository
};
use jd_domain::Id;
use modql::filter::ListOptions;
use uuid::Uuid;

pub struct AnalysisRepositoryImpl {
    state: AppState,
}

impl AnalysisRepositoryImpl {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl AnalysisRepository for AnalysisRepositoryImpl {
    async fn save_analysis_result(&self, result: &AnalysisResult) -> Result<Uuid> {
        // Convert to domain model
        let create_req = CodeAnalysisResultForCreate::from(result);
        
        // Create the analysis result using jd_core::base
        let created_result = base::rest::create_with_enum_cast::<CodeAnalysisResultDmc, _, CodeAnalysisResult>(
            &self.state.mm(),
            create_req,
        )
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        // Save vulnerabilities
        for vulnerability in &result.vulnerabilities {
            self.save_vulnerability(vulnerability, created_result.id).await?;
        }

        Ok(created_result.id)
    }

    async fn get_analysis_result(&self, id: Uuid) -> Result<Option<AnalysisResult>> {
        // Get analysis result
        let analysis_result = match base::rest::get_by_id::<CodeAnalysisResultDmc, CodeAnalysisResult>(
            &self.state.mm(),
            id,
        )
        .await
        {
            Ok(result) => result,
            Err(jd_core::error::Error::EntityNotFound { .. }) => return Ok(None),
            Err(e) => return Err(Error::DatabaseError { message: e.to_string() }),
        };

        // Get vulnerabilities
        let vulnerabilities = self.get_vulnerabilities_for_analysis(id).await?;

        // Convert to domain model
        let mut result: AnalysisResult = analysis_result.into();
        result.vulnerabilities = vulnerabilities;

        Ok(Some(result))
    }

    async fn get_latest_analysis_for_repository(&self, repository_id: Uuid) -> Result<Option<AnalysisResult>> {
        // Use list with limit 1 and descending order to get the latest
        let list_options = ListOptions {
            limit: Some(1),
            offset: None,
            order_bys: Some("ctime DESC".to_string()),
        };

        let filter = jd_domain::zkpersona_domain::developer_models::CodeAnalysisResultFilter {
            repository_id: Some(repository_id),
            analysis_type: None,
            min_security_score: None,
            min_quality_score: None,
            min_issues_found: None,
            has_critical_issues: None,
        };

        let (results, _) = base::rest::list::<CodeAnalysisResultDmc, _, CodeAnalysisResult>(
            &self.state.mm(),
            Some(filter),
            Some(list_options),
        )
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        if let Some(analysis_result) = results.into_iter().next() {
            let vulnerabilities = self.get_vulnerabilities_for_analysis(analysis_result.id).await?;
            let mut result: AnalysisResult = analysis_result.into();
            result.vulnerabilities = vulnerabilities;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    async fn get_analysis_history(&self, repository_id: Uuid, limit: Option<u32>) -> Result<Vec<AnalysisResult>> {
        let list_options = ListOptions {
            limit: Some(limit.unwrap_or(10) as i64),
            offset: None,
            order_bys: Some("ctime DESC".to_string()),
        };

        let filter = jd_domain::zkpersona_domain::developer_models::CodeAnalysisResultFilter {
            repository_id: Some(repository_id),
            analysis_type: None,
            min_security_score: None,
            min_quality_score: None,
            min_issues_found: None,
            has_critical_issues: None,
        };

        let (results, _) = base::rest::list::<CodeAnalysisResultDmc, _, CodeAnalysisResult>(
            &self.state.mm(),
            Some(filter),
            Some(list_options),
        )
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        let mut analysis_results = Vec::new();
        for analysis_result in results {
            let vulnerabilities = self.get_vulnerabilities_for_analysis(analysis_result.id).await?;
            let mut result: AnalysisResult = analysis_result.into();
            result.vulnerabilities = vulnerabilities;
            analysis_results.push(result);
        }

        Ok(analysis_results)
    }

    async fn save_vulnerability(&self, vulnerability: &VulnerabilityFinding, analysis_id: Uuid) -> Result<Uuid> {
        // First, get the repository_id from the analysis
        let analysis = base::rest::get_by_id::<CodeAnalysisResultDmc, CodeAnalysisResult>(
            &self.state.mm(),
            analysis_id,
        )
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        // Convert to domain model
        let create_req = SecurityVulnerabilityForCreate::from_vulnerability_and_analysis(
            vulnerability,
            analysis.repository_id,
            analysis_id,
        );

        // Create the vulnerability using jd_core::base
        let created_vulnerability = base::rest::create_with_enum_cast::<SecurityVulnerabilityDmc, _, SecurityVulnerability>(
            &self.state.mm(),
            create_req,
        )
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        Ok(created_vulnerability.id)
    }

    async fn get_vulnerabilities_for_analysis(&self, analysis_id: Uuid) -> Result<Vec<VulnerabilityFinding>> {
        let filter = SecurityVulnerabilityFilter {
            repository_id: None,
            vulnerability_type: None,
            severity: None,
            min_confidence_score: None,
            is_false_positive: None,
            is_fixed: None,
        };

        // We need to add analysis_result_id to the filter, but it's not in the existing filter
        // For now, we'll use a manual query approach or extend the filter
        // Let's manually query for this specific case
        use sqlx::FromRow;
        
        let vulnerabilities = sqlx::query_as!(
            SecurityVulnerability,
            r#"
            SELECT id, repository_id, analysis_result_id, 
                   vulnerability_type as "vulnerability_type: jd_domain::zkpersona_domain::developer_models::VulnerabilityType",
                   severity as "severity: jd_domain::zkpersona_domain::developer_models::Severity",
                   confidence_score, file_path, line_number, code_snippet,
                   description, recommendation, cve_id, is_false_positive, fixed_at, ctime
            FROM security_vulnerabilities
            WHERE analysis_result_id = $1
            "#,
            analysis_id
        )
        .fetch_all(self.state.mm().dbx().db())
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        Ok(vulnerabilities.into_iter().map(|v| v.into()).collect())
    }

    async fn get_vulnerabilities_for_repository(&self, repository_id: Uuid) -> Result<Vec<VulnerabilityFinding>> {
        let filter = SecurityVulnerabilityFilter {
            repository_id: Some(repository_id),
            vulnerability_type: None,
            severity: None,
            min_confidence_score: None,
            is_false_positive: Some(false),
            is_fixed: Some(false),
        };

        let list_options = ListOptions {
            limit: None,
            offset: None,
            order_bys: Some("severity DESC, confidence_score DESC".to_string()),
        };

        let (vulnerabilities, _) = base::rest::list::<SecurityVulnerabilityDmc, _, SecurityVulnerability>(
            &self.state.mm(),
            Some(filter),
            Some(list_options),
        )
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        Ok(vulnerabilities.into_iter().map(|v| v.into()).collect())
    }

    async fn mark_vulnerability_as_false_positive(&self, vulnerability_id: Uuid) -> Result<()> {
        let update_req = jd_domain::zkpersona_domain::developer_models::SecurityVulnerabilityForUpdate {
            is_false_positive: Some(true),
            fixed_at: None,
        };

        base::rest::update::<SecurityVulnerabilityDmc, _>(&self.state.mm(), vulnerability_id, update_req)
            .await
            .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        Ok(())
    }

    async fn mark_vulnerability_as_fixed(&self, vulnerability_id: Uuid) -> Result<()> {
        let update_req = jd_domain::zkpersona_domain::developer_models::SecurityVulnerabilityForUpdate {
            is_false_positive: None,
            fixed_at: Some(chrono::Utc::now()),
        };

        base::rest::update::<SecurityVulnerabilityDmc, _>(&self.state.mm(), vulnerability_id, update_req)
            .await
            .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        Ok(())
    }

    async fn get_vulnerability_statistics(&self, repository_id: Uuid) -> Result<VulnerabilityStatistics> {
        // Manual query for statistics
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_vulnerabilities,
                COUNT(*) FILTER (WHERE severity = 'critical') as critical_count,
                COUNT(*) FILTER (WHERE severity = 'high') as high_count,
                COUNT(*) FILTER (WHERE severity = 'medium') as medium_count,
                COUNT(*) FILTER (WHERE severity = 'low') as low_count,
                COUNT(*) FILTER (WHERE is_false_positive = true) as false_positive_count,
                COUNT(*) FILTER (WHERE fixed_at IS NOT NULL) as fixed_count
            FROM security_vulnerabilities
            WHERE repository_id = $1
            "#,
            repository_id
        )
        .fetch_one(self.state.mm().dbx().db())
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        Ok(VulnerabilityStatistics {
            total_vulnerabilities: stats.total_vulnerabilities.unwrap_or(0),
            critical_count: stats.critical_count.unwrap_or(0),
            high_count: stats.high_count.unwrap_or(0),
            medium_count: stats.medium_count.unwrap_or(0),
            low_count: stats.low_count.unwrap_or(0),
            false_positive_count: stats.false_positive_count.unwrap_or(0),
            fixed_count: stats.fixed_count.unwrap_or(0),
        })
    }
}