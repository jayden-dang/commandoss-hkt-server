use serde::{Deserialize, Serialize};
use uuid::Uuid;
use modql::field::Fields;
use modql::filter::{FilterNodes, OpValsString, OpValsValue};
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatchStatus {
    Draft,
    UnderReview,
    Approved,
    Rejected,
    Applied,
    Failed,
}

impl From<String> for PatchStatus {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "draft" => PatchStatus::Draft,
            "under_review" => PatchStatus::UnderReview,
            "approved" => PatchStatus::Approved,
            "rejected" => PatchStatus::Rejected,
            "applied" => PatchStatus::Applied,
            "failed" => PatchStatus::Failed,
            _ => PatchStatus::Draft,
        }
    }
}

impl ToString for PatchStatus {
    fn to_string(&self) -> String {
        match self {
            PatchStatus::Draft => "draft".to_string(),
            PatchStatus::UnderReview => "under_review".to_string(),
            PatchStatus::Approved => "approved".to_string(),
            PatchStatus::Rejected => "rejected".to_string(),
            PatchStatus::Applied => "applied".to_string(),
            PatchStatus::Failed => "failed".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchProposal {
    pub id: Uuid,
    pub vulnerability_id: Uuid,
    pub repository_id: Uuid,
    pub developer_id: Uuid,
    pub title: String,
    pub description: String,
    pub patch_diff: String,
    pub files_changed: Vec<String>,
    pub status: PatchStatus,
    pub approval_score: f64,
    pub rejection_score: f64,
    pub total_votes: i32,
    pub generated_by_ai: bool,
    pub validation_status: Option<ValidationStatus>,
    pub applied_at: Option<OffsetDateTime>,
    pub pr_url: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStatus {
    pub tests_passed: bool,
    pub build_succeeded: bool,
    pub security_scan_passed: bool,
    pub validation_message: String,
    pub validated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub id: Uuid,
    pub patch_id: Uuid,
    pub voter_id: Uuid,
    pub vote_type: VoteType,
    pub weight: f64,
    pub comment: Option<String>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteType {
    Approve,
    Reject,
    Abstain,
}

impl From<String> for VoteType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "approve" => VoteType::Approve,
            "reject" => VoteType::Reject,
            "abstain" => VoteType::Abstain,
            _ => VoteType::Abstain,
        }
    }
}

impl ToString for VoteType {
    fn to_string(&self) -> String {
        match self {
            VoteType::Approve => "approve".to_string(),
            VoteType::Reject => "reject".to_string(),
            VoteType::Abstain => "abstain".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchStatistics {
    pub total_patches: i64,
    pub approved_patches: i64,
    pub rejected_patches: i64,
    pub applied_patches: i64,
    pub success_rate: f64,
    pub average_approval_time: Option<i64>, // in hours
    pub ai_generated_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchLeaderboard {
    pub developer_id: Uuid,
    pub username: String,
    pub patches_submitted: i64,
    pub patches_approved: i64,
    pub approval_rate: f64,
    pub reputation_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchFilter {
    pub repository_id: Option<Uuid>,
    pub vulnerability_id: Option<Uuid>,
    pub developer_id: Option<Uuid>,
    pub status: Option<PatchStatus>,
    pub generated_by_ai: Option<bool>,
    pub created_after: Option<OffsetDateTime>,
    pub created_before: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchGenerationRequest {
    pub vulnerability_id: Uuid,
    pub context: VulnerabilityContext,
    pub generation_strategy: GenerationStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityContext {
    pub vulnerability_type: String,
    pub severity: String,
    pub file_path: String,
    pub line_number: Option<i32>,
    pub code_snippet: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenerationStrategy {
    Conservative, // Minimal changes, high safety
    Balanced,     // Balance between safety and effectiveness
    Aggressive,   // More comprehensive fixes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchDiff {
    pub file_path: String,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: i32,
    pub old_lines: i32,
    pub new_start: i32,
    pub new_lines: i32,
    pub content: String,
}

// Database representation without complex nested fields
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
pub struct PatchProposalDb {
    pub id: Uuid,
    pub vulnerability_id: Uuid,
    pub repository_id: Uuid,
    pub developer_id: Uuid,
    pub title: String,
    pub description: String,
    pub patch_diff: String,
    pub status: String, // Stored as string in DB
    pub approval_score: f64,
    pub rejection_score: f64,
    pub total_votes: i32,
    pub generated_by_ai: bool,
    pub applied_at: Option<OffsetDateTime>,
    pub pr_url: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

// For database operations with base::rest
#[derive(Debug, Clone, Serialize, Deserialize, Fields)]
pub struct PatchProposalForCreate {
    pub vulnerability_id: Uuid,
    pub repository_id: Uuid,
    pub developer_id: Uuid,
    pub title: String,
    pub description: String,
    pub patch_diff: String,
    pub status: Option<String>,
    pub approval_score: Option<f64>,
    pub rejection_score: Option<f64>,
    pub total_votes: Option<i32>,
    pub generated_by_ai: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Fields)]
pub struct PatchProposalForUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub patch_diff: Option<String>,
    pub status: Option<String>,
    pub approval_score: Option<f64>,
    pub rejection_score: Option<f64>,
    pub total_votes: Option<i32>,
    pub applied_at: Option<OffsetDateTime>,
    pub pr_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, FilterNodes)]
pub struct PatchProposalFilter {
    pub id: Option<OpValsValue>,
    pub vulnerability_id: Option<OpValsValue>,
    pub repository_id: Option<OpValsValue>,
    pub developer_id: Option<OpValsValue>,
    pub status: Option<OpValsString>,
    pub generated_by_ai: Option<OpValsValue>,
}

impl PatchProposalDb {
    /// Convert database representation to full PatchProposal
    pub fn to_patch_proposal(self) -> PatchProposal {
        PatchProposal {
            id: self.id,
            vulnerability_id: self.vulnerability_id,
            repository_id: self.repository_id,
            developer_id: self.developer_id,
            title: self.title,
            description: self.description,
            patch_diff: self.patch_diff,
            files_changed: vec![], // TODO: Handle separately
            status: PatchStatus::from(self.status),
            approval_score: self.approval_score,
            rejection_score: self.rejection_score,
            total_votes: self.total_votes,
            generated_by_ai: self.generated_by_ai,
            validation_status: None, // TODO: Handle separately
            applied_at: self.applied_at,
            pr_url: self.pr_url,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}