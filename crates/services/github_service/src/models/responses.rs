use serde::Serialize;
use uuid::Uuid;
use jd_domain::zkpersona_domain::developer_models::GitHubRepository;

#[derive(Debug, Serialize)]
pub struct RepositoryResponse {
    pub repository: GitHubRepository,
    pub webhook_configured: bool,
    pub initial_scan_queued: bool,
}

#[derive(Debug, Serialize)]
pub struct RepositoryListResponse {
    pub repositories: Vec<GitHubRepository>,
    pub total_count: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct RepositoryDetailResponse {
    pub repository: GitHubRepository,
    pub latest_analysis: Option<AnalysisSummary>,
    pub vulnerability_summary: VulnerabilitySummary,
    pub security_trends: Vec<SecurityTrend>,
}

#[derive(Debug, Serialize)]
pub struct AnalysisSummary {
    pub id: Uuid,
    pub analysis_type: String,
    pub completed_at: chrono::DateTime<chrono::Utc>,
    pub security_score: Option<f64>,
    pub issues_found: i32,
}

#[derive(Debug, Serialize)]
pub struct VulnerabilitySummary {
    pub total_vulnerabilities: i32,
    pub critical_count: i32,
    pub high_count: i32,
    pub medium_count: i32,
    pub low_count: i32,
}

#[derive(Debug, Serialize)]
pub struct SecurityTrend {
    pub date: chrono::NaiveDate,
    pub security_score: f64,
    pub vulnerability_count: i32,
}

#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    pub status: String,
    pub message: String,
}