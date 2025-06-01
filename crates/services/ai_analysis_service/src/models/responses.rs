use crate::domain::analysis_models::{AnalysisResult, VulnerabilityFinding, SecurityRecommendation};
use crate::domain::analysis_repository_trait::VulnerabilityStatistics;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResponse {
    pub analysis_id: Uuid,
    pub repository_id: Uuid,
    pub commit_sha: String,
    pub security_score: f64,
    pub quality_score: f64,
    pub vulnerabilities_found: u32,
    pub critical_vulnerabilities: u32,
    pub analysis_duration_ms: u64,
    pub analysis_types_completed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedAnalysisResponse {
    pub analysis_result: AnalysisResult,
    pub vulnerability_statistics: VulnerabilityStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisHistoryResponse {
    pub repository_id: Uuid,
    pub analyses: Vec<AnalysisSummary>,
    pub total_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub analysis_id: Uuid,
    pub commit_sha: String,
    pub analysis_type: String,
    pub security_score: f64,
    pub quality_score: f64,
    pub vulnerabilities_count: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityListResponse {
    pub repository_id: Uuid,
    pub vulnerabilities: Vec<VulnerabilityFinding>,
    pub statistics: VulnerabilityStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisResponse {
    pub security_score: f64,
    pub quality_score: f64,
    pub vulnerabilities: Vec<VulnerabilityFinding>,
    pub recommendations: Vec<SecurityRecommendation>,
    pub analysis_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisStatusResponse {
    pub repository_id: Uuid,
    pub latest_analysis: Option<AnalysisSummary>,
    pub total_analyses: u32,
    pub vulnerability_statistics: VulnerabilityStatistics,
}