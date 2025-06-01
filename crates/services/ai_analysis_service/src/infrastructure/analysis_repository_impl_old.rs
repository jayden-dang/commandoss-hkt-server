use crate::domain::analysis_models::{AnalysisResult, AnalysisType, VulnerabilityFinding, VulnerabilityType, Severity};
use crate::domain::analysis_repository_trait::{AnalysisRepository, VulnerabilityStatistics};
use crate::error::{Error, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use jd_storage::dbx::Dbx;
use serde_json;
use sqlx::{PgPool, types::Decimal, Row};
use uuid::Uuid;


pub struct AnalysisRepositoryImpl {
    db: PgPool,
}

impl AnalysisRepositoryImpl {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    fn decimal_to_f64(&self, decimal: Decimal) -> f64 {
        decimal.to_string().parse().unwrap_or(0.0)
    }

    fn offsetdatetime_to_utc(&self, dt: time::OffsetDateTime) -> DateTime<Utc> {
        DateTime::from_timestamp(dt.unix_timestamp(), dt.nanosecond()).unwrap_or_default()
    }

    fn map_analysis_type_to_db(&self, analysis_type: &AnalysisType) -> &'static str {
        match analysis_type {
            AnalysisType::StaticAnalysis => "static_analysis",
            AnalysisType::LLMReview => "llm_review",
            AnalysisType::VulnerabilityDetection => "static_analysis", // Map to existing enum
            AnalysisType::CodeQualityAssessment => "llm_review", // Map to existing enum
        }
    }

    fn map_db_to_analysis_type(&self, db_type: &str) -> AnalysisType {
        match db_type {
            "static_analysis" => AnalysisType::StaticAnalysis,
            "llm_review" => AnalysisType::LLMReview,
            "dependency_check" => AnalysisType::StaticAnalysis, // Fallback
            _ => AnalysisType::StaticAnalysis,
        }
    }

    fn map_vulnerability_type_to_db(&self, vuln_type: &VulnerabilityType) -> &'static str {
        match vuln_type {
            VulnerabilityType::UnauthorizedAccess | VulnerabilityType::AccessControl => "access_control",
            VulnerabilityType::IntegerOverflow => "overflow",
            VulnerabilityType::ReentrancyLike => "reentrancy",
            _ => "other",
        }
    }

    fn map_db_to_vulnerability_type(&self, db_type: &str) -> VulnerabilityType {
        match db_type {
            "access_control" => VulnerabilityType::AccessControl,
            "overflow" => VulnerabilityType::IntegerOverflow,
            "reentrancy" => VulnerabilityType::ReentrancyLike,
            "other" => VulnerabilityType::Other("Unknown".to_string()),
            _ => VulnerabilityType::Other(db_type.to_string()),
        }
    }

    fn map_severity_to_db(&self, severity: &Severity) -> &'static str {
        match severity {
            Severity::Critical => "critical",
            Severity::High => "high",
            Severity::Medium => "medium",
            Severity::Low => "low",
        }
    }

    fn map_db_to_severity(&self, db_severity: &str) -> Severity {
        match db_severity {
            "critical" => Severity::Critical,
            "high" => Severity::High,
            "medium" => Severity::Medium,
            "low" => Severity::Low,
            _ => Severity::Medium,
        }
    }
}

#[async_trait]
impl AnalysisRepository for AnalysisRepositoryImpl {
    async fn save_analysis_result(&self, result: &AnalysisResult) -> Result<Uuid> {
        let analysis_type_db = self.map_analysis_type_to_db(&result.analysis_type);
        
        let row = sqlx::query(
            r#"
            INSERT INTO code_analysis_results (
                id, repository_id, commit_sha, analysis_type, security_score, quality_score,
                issues_found, critical_issues, analysis_duration_ms, analyzer_version, raw_results
            ) VALUES ($1, $2, $3, $4::analysis_type_enum, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id
            "#,
        )
        .bind(result.id)
        .bind(result.repository_id)
        .bind(&result.commit_sha)
        .bind(analysis_type_db)
        .bind(result.security_score)
        .bind(result.quality_score)
        .bind(result.vulnerabilities.len() as i32)
        .bind(result.vulnerabilities.iter().filter(|v| matches!(v.severity, Severity::Critical)).count() as i32)
        .bind(result.analysis_duration_ms as i32)
        .bind(&result.analyzer_version)
        .bind(&result.raw_results)
        .fetch_one(&self.db)
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;
        
        let analysis_id: Uuid = row.get("id");

        // Save vulnerabilities
        for vulnerability in &result.vulnerabilities {
            self.save_vulnerability(vulnerability, analysis_id).await?;
        }

        Ok(analysis_id)
    }

    async fn get_analysis_result(&self, id: Uuid) -> Result<Option<AnalysisResult>> {
        let row = sqlx::query!(
            r#"
            SELECT id, repository_id, commit_sha, analysis_type as "analysis_type: String",
                   security_score, quality_score, issues_found, critical_issues,
                   analysis_duration_ms, analyzer_version, raw_results, ctime
            FROM code_analysis_results
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        if let Some(row) = row {
            let vulnerabilities = self.get_vulnerabilities_for_analysis(id).await?;

            Ok(Some(AnalysisResult {
                id: row.id,
                repository_id: row.repository_id,
                commit_sha: row.commit_sha,
                analysis_type: self.map_db_to_analysis_type(&row.analysis_type),
                security_score: self.decimal_to_f64(row.security_score),
                quality_score: self.decimal_to_f64(row.quality_score),
                vulnerabilities,
                recommendations: Vec::new(), // TODO: Implement recommendations storage
                analysis_duration_ms: row.analysis_duration_ms as u64,
                analyzer_version: row.analyzer_version,
                raw_results: row.raw_results,
                created_at: self.offsetdatetime_to_utc(row.ctime),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_latest_analysis_for_repository(&self, repository_id: Uuid) -> Result<Option<AnalysisResult>> {
        let row = sqlx::query!(
            r#"
            SELECT id, repository_id, commit_sha, analysis_type as "analysis_type: String",
                   security_score, quality_score, issues_found, critical_issues,
                   analysis_duration_ms, analyzer_version, raw_results, ctime
            FROM code_analysis_results
            WHERE repository_id = $1
            ORDER BY ctime DESC
            LIMIT 1
            "#,
            repository_id
        )
        .fetch_optional(&self.db)
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        if let Some(row) = row {
            let vulnerabilities = self.get_vulnerabilities_for_analysis(row.id).await?;

            Ok(Some(AnalysisResult {
                id: row.id,
                repository_id: row.repository_id,
                commit_sha: row.commit_sha,
                analysis_type: self.map_db_to_analysis_type(&row.analysis_type),
                security_score: self.decimal_to_f64(row.security_score),
                quality_score: self.decimal_to_f64(row.quality_score),
                vulnerabilities,
                recommendations: Vec::new(),
                analysis_duration_ms: row.analysis_duration_ms as u64,
                analyzer_version: row.analyzer_version,
                raw_results: row.raw_results,
                created_at: self.offsetdatetime_to_utc(row.ctime),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_analysis_history(&self, repository_id: Uuid, limit: Option<u32>) -> Result<Vec<AnalysisResult>> {
        let limit = limit.unwrap_or(10) as i64;
        
        let rows = sqlx::query!(
            r#"
            SELECT id, repository_id, commit_sha, analysis_type as "analysis_type: String",
                   security_score, quality_score, issues_found, critical_issues,
                   analysis_duration_ms, analyzer_version, raw_results, ctime
            FROM code_analysis_results
            WHERE repository_id = $1
            ORDER BY ctime DESC
            LIMIT $2
            "#,
            repository_id,
            limit
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        let mut results = Vec::new();
        for row in rows {
            let vulnerabilities = self.get_vulnerabilities_for_analysis(row.id).await?;
            
            results.push(AnalysisResult {
                id: row.id,
                repository_id: row.repository_id,
                commit_sha: row.commit_sha,
                analysis_type: self.map_db_to_analysis_type(&row.analysis_type),
                security_score: self.decimal_to_f64(row.security_score),
                quality_score: self.decimal_to_f64(row.quality_score),
                vulnerabilities,
                recommendations: Vec::new(),
                analysis_duration_ms: row.analysis_duration_ms as u64,
                analyzer_version: row.analyzer_version,
                raw_results: row.raw_results,
                created_at: self.offsetdatetime_to_utc(row.ctime),
            });
        }

        Ok(results)
    }

    async fn save_vulnerability(&self, vulnerability: &VulnerabilityFinding, analysis_id: Uuid) -> Result<Uuid> {
        let vulnerability_type_db = self.map_vulnerability_type_to_db(&vulnerability.vulnerability_type);
        let severity_db = self.map_severity_to_db(&vulnerability.severity);

        // First, get the repository_id from the analysis
        let repository_id = sqlx::query_scalar!(
            "SELECT repository_id FROM code_analysis_results WHERE id = $1",
            analysis_id
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        let row = sqlx::query(
            r#"
            INSERT INTO security_vulnerabilities (
                id, repository_id, analysis_result_id, vulnerability_type, severity,
                confidence_score, file_path, line_number, code_snippet,
                description, recommendation, cve_id, is_false_positive
            ) VALUES ($1, $2, $3, $4::vulnerability_type_enum, $5::severity_enum, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id
            "#,
        )
        .bind(vulnerability.id)
        .bind(repository_id)
        .bind(analysis_id)
        .bind(vulnerability_type_db)
        .bind(severity_db)
        .bind(vulnerability.confidence_score)
        .bind(&vulnerability.file_path)
        .bind(vulnerability.line_number.map(|n| n as i32))
        .bind(&vulnerability.code_snippet)
        .bind(&vulnerability.description)
        .bind(&vulnerability.recommendation)
        .bind(&vulnerability.cve_id)
        .bind(vulnerability.is_false_positive)
        .fetch_one(&self.db)
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        let vulnerability_id: Uuid = row.get("id");

        Ok(vulnerability_id)
    }

    async fn get_vulnerabilities_for_analysis(&self, analysis_id: Uuid) -> Result<Vec<VulnerabilityFinding>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, vulnerability_type as "vulnerability_type: String", 
                   severity as "severity: String", confidence_score, file_path, line_number,
                   code_snippet, description, recommendation, cve_id, is_false_positive
            FROM security_vulnerabilities
            WHERE analysis_result_id = $1
            "#,
            analysis_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        let vulnerabilities = rows
            .into_iter()
            .map(|row| VulnerabilityFinding {
                id: row.id,
                vulnerability_type: self.map_db_to_vulnerability_type(&row.vulnerability_type),
                severity: self.map_db_to_severity(&row.severity),
                confidence_score: self.decimal_to_f64(row.confidence_score),
                file_path: row.file_path,
                line_number: row.line_number.map(|n| n as u32),
                code_snippet: row.code_snippet,
                description: row.description,
                recommendation: row.recommendation,
                cve_id: row.cve_id,
                is_false_positive: row.is_false_positive,
            })
            .collect();

        Ok(vulnerabilities)
    }

    async fn get_vulnerabilities_for_repository(&self, repository_id: Uuid) -> Result<Vec<VulnerabilityFinding>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, vulnerability_type as "vulnerability_type: String",
                   severity as "severity: String", confidence_score, file_path, line_number,
                   code_snippet, description, recommendation, cve_id, is_false_positive
            FROM security_vulnerabilities
            WHERE repository_id = $1 AND fixed_at IS NULL
            ORDER BY severity DESC, confidence_score DESC
            "#,
            repository_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        let vulnerabilities = rows
            .into_iter()
            .map(|row| VulnerabilityFinding {
                id: row.id,
                vulnerability_type: self.map_db_to_vulnerability_type(&row.vulnerability_type),
                severity: self.map_db_to_severity(&row.severity),
                confidence_score: self.decimal_to_f64(row.confidence_score),
                file_path: row.file_path,
                line_number: row.line_number.map(|n| n as u32),
                code_snippet: row.code_snippet,
                description: row.description,
                recommendation: row.recommendation,
                cve_id: row.cve_id,
                is_false_positive: row.is_false_positive,
            })
            .collect();

        Ok(vulnerabilities)
    }

    async fn mark_vulnerability_as_false_positive(&self, vulnerability_id: Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE security_vulnerabilities SET is_false_positive = true WHERE id = $1",
            vulnerability_id
        )
        .execute(&self.db)
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        Ok(())
    }

    async fn mark_vulnerability_as_fixed(&self, vulnerability_id: Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE security_vulnerabilities SET fixed_at = NOW() WHERE id = $1",
            vulnerability_id
        )
        .execute(&self.db)
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        Ok(())
    }

    async fn get_vulnerability_statistics(&self, repository_id: Uuid) -> Result<VulnerabilityStatistics> {
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
        .fetch_one(&self.db)
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