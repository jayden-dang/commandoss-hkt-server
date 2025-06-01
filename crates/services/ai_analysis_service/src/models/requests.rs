use crate::domain::analysis_models::AnalysisType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeRepositoryRequest {
    pub repository_id: Uuid,
    pub commit_sha: String,
    pub files_to_analyze: Option<Vec<String>>, // If None, analyze all Move files
    pub analysis_types: Vec<AnalysisType>,
    pub enable_llm_analysis: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeCodeRequest {
    pub code: String,
    pub file_path: String,
    pub analysis_types: Vec<AnalysisType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAnalysisHistoryRequest {
    pub repository_id: Uuid,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkVulnerabilityRequest {
    pub vulnerability_id: Uuid,
    pub action: VulnerabilityAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VulnerabilityAction {
    MarkFalsePositive,
    MarkFixed,
    Reopen,
}