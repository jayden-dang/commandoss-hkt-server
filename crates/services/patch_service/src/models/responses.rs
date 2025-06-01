use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{
    FilePreview, GeneratedPatch, PatchLeaderboard, PatchProposal, PatchStatistics,
    PreviewResult, ValidationResult, Vote,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchResponse {
    pub patch: PatchProposal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchListResponse {
    pub patches: Vec<PatchProposal>,
    pub total_count: i64,
    pub page: u32,
    pub limit: u32,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteResponse {
    pub patch_id: Uuid,
    pub voter_id: Uuid,
    pub vote_type: String,
    pub weight: Decimal,
    pub voted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingStatusResponse {
    pub patch_id: Uuid,
    pub approval_score: Decimal,
    pub rejection_score: Decimal,
    pub total_votes: i32,
    pub user_vote: Option<Vote>,
    pub votes: Vec<VoteDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteDetail {
    pub voter_id: Uuid,
    pub voter_username: String,
    pub vote_type: String,
    pub weight: Decimal,
    pub comment: Option<String>,
    pub voted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyPatchResponse {
    pub patch_id: Uuid,
    pub status: String,
    pub pr_url: Option<String>,
    pub applied_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratePatchResponse {
    pub generated_patch: GeneratedPatch,
    pub vulnerability_id: Uuid,
    pub generation_time_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatePatchResponse {
    pub validation_result: ValidationResult,
    pub validated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewPatchResponse {
    pub preview: PreviewResult,
    pub estimated_impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchStatisticsResponse {
    pub statistics: PatchStatistics,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchLeaderboardResponse {
    pub leaderboard: Vec<PatchLeaderboard>,
    pub period: String,
    pub generated_at: DateTime<Utc>,
}