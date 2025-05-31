use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisJob {
    pub id: Uuid,
    pub repository_id: u64,
    pub commit_sha: String,
    pub files_to_analyze: Vec<String>,
    pub analysis_type: AnalysisType,
    pub priority: AnalysisPriority,
    pub created_at: DateTime<Utc>,
    pub status: JobStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AnalysisType {
    InitialScan,
    SmartContract,
    SecurityFocus,
    FullAnalysis,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnalysisPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Serialize)]
pub struct QueueStatus {
    pub queued_jobs: usize,
    pub processing_jobs: usize,
    pub total_jobs: usize,
}