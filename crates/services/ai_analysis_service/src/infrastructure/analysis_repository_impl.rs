use crate::domain::analysis_models::{AnalysisResult, VulnerabilityFinding};
use crate::domain::analysis_repository_trait::{AnalysisRepository, VulnerabilityStatistics};
use crate::error::{Error, Result};
use async_trait::async_trait;
use jd_core::AppState;
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use time;

pub struct AnalysisRepositoryImpl {
    state: AppState,
}

impl AnalysisRepositoryImpl {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    fn db(&self) -> &PgPool {
        self.state.mm().dbx().db()
    }

    fn decimal_to_f64(&self, decimal: Decimal) -> f64 {
        decimal.to_string().parse().unwrap_or(0.0)
    }

    fn offsetdatetime_to_utc(&self, dt: time::OffsetDateTime) -> DateTime<Utc> {
        DateTime::from_timestamp(dt.unix_timestamp(), dt.nanosecond()).unwrap_or_default()
    }

    fn map_analysis_type_to_db(&self, analysis_type: &crate::domain::analysis_models::AnalysisType) -> &'static str {
        match analysis_type {
            crate::domain::analysis_models::AnalysisType::StaticAnalysis => "static_analysis",
            crate::domain::analysis_models::AnalysisType::LLMReview => "llm_review",
            crate::domain::analysis_models::AnalysisType::VulnerabilityDetection => "static_analysis",
            crate::domain::analysis_models::AnalysisType::CodeQualityAssessment => "llm_review",
        }
    }

    fn map_db_to_analysis_type(&self, db_type: &str) -> crate::domain::analysis_models::AnalysisType {
        match db_type {
            "static_analysis" => crate::domain::analysis_models::AnalysisType::StaticAnalysis,
            "llm_review" => crate::domain::analysis_models::AnalysisType::LLMReview,
            "dependency_check" => crate::domain::analysis_models::AnalysisType::StaticAnalysis,
            _ => crate::domain::analysis_models::AnalysisType::StaticAnalysis,
        }
    }

    fn map_vulnerability_type_to_db(&self, vuln_type: &crate::domain::analysis_models::VulnerabilityType) -> &'static str {
        match vuln_type {
            crate::domain::analysis_models::VulnerabilityType::UnauthorizedAccess | 
            crate::domain::analysis_models::VulnerabilityType::AccessControl => "access_control",
            crate::domain::analysis_models::VulnerabilityType::IntegerOverflow => "overflow",
            crate::domain::analysis_models::VulnerabilityType::ReentrancyLike => "reentrancy",
            _ => "other",
        }
    }

    fn map_db_to_vulnerability_type(&self, db_type: &str) -> crate::domain::analysis_models::VulnerabilityType {
        match db_type {
            "access_control" => crate::domain::analysis_models::VulnerabilityType::AccessControl,
            "overflow" => crate::domain::analysis_models::VulnerabilityType::IntegerOverflow,
            "reentrancy" => crate::domain::analysis_models::VulnerabilityType::ReentrancyLike,
            "other" => crate::domain::analysis_models::VulnerabilityType::Other("Unknown".to_string()),
            _ => crate::domain::analysis_models::VulnerabilityType::Other(db_type.to_string()),
        }
    }

    fn map_severity_to_db(&self, severity: &crate::domain::analysis_models::Severity) -> &'static str {
        match severity {
            crate::domain::analysis_models::Severity::Critical => "critical",
            crate::domain::analysis_models::Severity::High => "high",
            crate::domain::analysis_models::Severity::Medium => "medium",
            crate::domain::analysis_models::Severity::Low => "low",
        }
    }

    fn map_db_to_severity(&self, db_severity: &str) -> crate::domain::analysis_models::Severity {
        match db_severity {
            "critical" => crate::domain::analysis_models::Severity::Critical,
            "high" => crate::domain::analysis_models::Severity::High,
            "medium" => crate::domain::analysis_models::Severity::Medium,
            "low" => crate::domain::analysis_models::Severity::Low,
            _ => crate::domain::analysis_models::Severity::Medium,
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
        .bind(result.vulnerabilities.iter().filter(|v| matches!(v.severity, crate::domain::analysis_models::Severity::Critical)).count() as i32)
        .bind(result.analysis_duration_ms as i32)
        .bind(&result.analyzer_version)
        .bind(&result.raw_results)
        .fetch_one(self.db())
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
        let row = sqlx::query(
            r#"
            SELECT id, repository_id, commit_sha, analysis_type,
                   security_score, quality_score, issues_found, critical_issues,
                   analysis_duration_ms, analyzer_version, raw_results, ctime
            FROM code_analysis_results
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.db())
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        if let Some(row) = row {
            let vulnerabilities = self.get_vulnerabilities_for_analysis(row.get::<Uuid, _>("id")).await?;

            Ok(Some(AnalysisResult {
                id: row.get("id"),
                repository_id: row.get("repository_id"),
                commit_sha: row.get("commit_sha"),
                analysis_type: self.map_db_to_analysis_type(&row.get::<String, _>("analysis_type")),
                security_score: self.decimal_to_f64(row.get("security_score")),
                quality_score: self.decimal_to_f64(row.get("quality_score")),
                vulnerabilities,
                recommendations: Vec::new(),
                analysis_duration_ms: row.get::<i32, _>("analysis_duration_ms") as u64,
                analyzer_version: row.get("analyzer_version"),
                raw_results: row.get("raw_results"),
                created_at: self.offsetdatetime_to_utc(row.get("ctime")),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_latest_analysis_for_repository(&self, repository_id: Uuid) -> Result<Option<AnalysisResult>> {
        let row = sqlx::query(
            r#"
            SELECT id, repository_id, commit_sha, analysis_type,
                   security_score, quality_score, issues_found, critical_issues,
                   analysis_duration_ms, analyzer_version, raw_results, ctime
            FROM code_analysis_results
            WHERE repository_id = $1
            ORDER BY ctime DESC
            LIMIT 1
            "#,
        )
        .bind(repository_id)
        .fetch_optional(self.db())
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        if let Some(row) = row {
            let vulnerabilities = self.get_vulnerabilities_for_analysis(row.get::<Uuid, _>("id")).await?;

            Ok(Some(AnalysisResult {
                id: row.get("id"),
                repository_id: row.get("repository_id"),
                commit_sha: row.get("commit_sha"),
                analysis_type: self.map_db_to_analysis_type(&row.get::<String, _>("analysis_type")),
                security_score: self.decimal_to_f64(row.get("security_score")),
                quality_score: self.decimal_to_f64(row.get("quality_score")),
                vulnerabilities,
                recommendations: Vec::new(),
                analysis_duration_ms: row.get::<i32, _>("analysis_duration_ms") as u64,
                analyzer_version: row.get("analyzer_version"),
                raw_results: row.get("raw_results"),
                created_at: self.offsetdatetime_to_utc(row.get("ctime")),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_analysis_history(&self, repository_id: Uuid, limit: Option<u32>) -> Result<Vec<AnalysisResult>> {
        let limit = limit.unwrap_or(10) as i64;
        
        let rows = sqlx::query(
            r#"
            SELECT id, repository_id, commit_sha, analysis_type,
                   security_score, quality_score, issues_found, critical_issues,
                   analysis_duration_ms, analyzer_version, raw_results, ctime
            FROM code_analysis_results
            WHERE repository_id = $1
            ORDER BY ctime DESC
            LIMIT $2
            "#,
        )
        .bind(repository_id)
        .bind(limit)
        .fetch_all(self.db())
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        let mut results = Vec::new();
        for row in rows {
            let vulnerabilities = self.get_vulnerabilities_for_analysis(row.get::<Uuid, _>("id")).await?;
            
            results.push(AnalysisResult {
                id: row.get("id"),
                repository_id: row.get("repository_id"),
                commit_sha: row.get("commit_sha"),
                analysis_type: self.map_db_to_analysis_type(&row.get::<String, _>("analysis_type")),
                security_score: self.decimal_to_f64(row.get("security_score")),
                quality_score: self.decimal_to_f64(row.get("quality_score")),
                vulnerabilities,
                recommendations: Vec::new(),
                analysis_duration_ms: row.get::<i32, _>("analysis_duration_ms") as u64,
                analyzer_version: row.get("analyzer_version"),
                raw_results: row.get("raw_results"),
                created_at: self.offsetdatetime_to_utc(row.get("ctime")),
            });
        }

        Ok(results)
    }

    async fn save_vulnerability(&self, vulnerability: &VulnerabilityFinding, analysis_id: Uuid) -> Result<Uuid> {
        let vulnerability_type_db = self.map_vulnerability_type_to_db(&vulnerability.vulnerability_type);
        let severity_db = self.map_severity_to_db(&vulnerability.severity);

        // First, get the repository_id from the analysis
        let repository_id: Uuid = sqlx::query_scalar(
            "SELECT repository_id FROM code_analysis_results WHERE id = $1"
        )
        .bind(analysis_id)
        .fetch_one(self.db())
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
        .fetch_one(self.db())
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        let vulnerability_id: Uuid = row.get("id");

        Ok(vulnerability_id)
    }

    async fn get_vulnerabilities_for_analysis(&self, analysis_id: Uuid) -> Result<Vec<VulnerabilityFinding>> {
        let rows = sqlx::query(
            r#"
            SELECT id, vulnerability_type::text, severity::text, confidence_score, 
                   file_path, line_number, code_snippet, description, recommendation, cve_id, is_false_positive
            FROM security_vulnerabilities
            WHERE analysis_result_id = $1
            "#,
        )
        .bind(analysis_id)
        .fetch_all(self.db())
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        let vulnerabilities = rows
            .into_iter()
            .map(|row| VulnerabilityFinding {
                id: row.get("id"),
                vulnerability_type: self.map_db_to_vulnerability_type(&row.get::<Option<String>, _>("vulnerability_type").unwrap()),
                severity: self.map_db_to_severity(&row.get::<Option<String>, _>("severity").unwrap()),
                confidence_score: self.decimal_to_f64(row.get("confidence_score")),
                file_path: row.get("file_path"),
                line_number: row.get::<Option<i32>, _>("line_number").map(|n| n as u32),
                code_snippet: row.get("code_snippet"),
                description: row.get("description"),
                recommendation: row.get("recommendation"),
                cve_id: row.get("cve_id"),
                is_false_positive: row.get("is_false_positive"),
            })
            .collect();

        Ok(vulnerabilities)
    }

    async fn get_vulnerabilities_for_repository(&self, repository_id: Uuid) -> Result<Vec<VulnerabilityFinding>> {
        let rows = sqlx::query(
            r#"
            SELECT id, vulnerability_type::text, severity::text, confidence_score,
                   file_path, line_number, code_snippet, description, recommendation, cve_id, is_false_positive
            FROM security_vulnerabilities
            WHERE repository_id = $1 AND fixed_at IS NULL
            ORDER BY severity DESC, confidence_score DESC
            "#,
        )
        .bind(repository_id)
        .fetch_all(self.db())
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        let vulnerabilities = rows
            .into_iter()
            .map(|row| VulnerabilityFinding {
                id: row.get("id"),
                vulnerability_type: self.map_db_to_vulnerability_type(&row.get::<Option<String>, _>("vulnerability_type").unwrap()),
                severity: self.map_db_to_severity(&row.get::<Option<String>, _>("severity").unwrap()),
                confidence_score: self.decimal_to_f64(row.get("confidence_score")),
                file_path: row.get("file_path"),
                line_number: row.get::<Option<i32>, _>("line_number").map(|n| n as u32),
                code_snippet: row.get("code_snippet"),
                description: row.get("description"),
                recommendation: row.get("recommendation"),
                cve_id: row.get("cve_id"),
                is_false_positive: row.get("is_false_positive"),
            })
            .collect();

        Ok(vulnerabilities)
    }

    async fn mark_vulnerability_as_false_positive(&self, vulnerability_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE security_vulnerabilities SET is_false_positive = true WHERE id = $1"
        )
        .bind(vulnerability_id)
        .execute(self.db())
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        Ok(())
    }

    async fn mark_vulnerability_as_fixed(&self, vulnerability_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE security_vulnerabilities SET fixed_at = NOW() WHERE id = $1"
        )
        .bind(vulnerability_id)
        .execute(self.db())
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        Ok(())
    }

    async fn get_vulnerability_statistics(&self, repository_id: Uuid) -> Result<VulnerabilityStatistics> {
        let stats = sqlx::query(
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
        )
        .bind(repository_id)
        .fetch_one(self.db())
        .await
        .map_err(|e| Error::DatabaseError { message: e.to_string() })?;

        Ok(VulnerabilityStatistics {
            total_vulnerabilities: stats.get::<Option<i64>, _>("total_vulnerabilities").unwrap_or(0),
            critical_count: stats.get::<Option<i64>, _>("critical_count").unwrap_or(0),
            high_count: stats.get::<Option<i64>, _>("high_count").unwrap_or(0),
            medium_count: stats.get::<Option<i64>, _>("medium_count").unwrap_or(0),
            low_count: stats.get::<Option<i64>, _>("low_count").unwrap_or(0),
            false_positive_count: stats.get::<Option<i64>, _>("false_positive_count").unwrap_or(0),
            fixed_count: stats.get::<Option<i64>, _>("fixed_count").unwrap_or(0),
        })
    }
}