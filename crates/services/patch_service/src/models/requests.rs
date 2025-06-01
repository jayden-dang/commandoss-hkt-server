use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{GenerationStrategy, PatchStatus, VoteType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPatchesRequest {
    pub repository_id: Option<Uuid>,
    pub vulnerability_id: Option<Uuid>,
    pub status: Option<PatchStatus>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl Default for ListPatchesRequest {
    fn default() -> Self {
        Self {
            repository_id: None,
            vulnerability_id: None,
            status: None,
            page: Some(1),
            limit: Some(20),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePatchRequest {
    pub vulnerability_id: Uuid,
    pub title: String,
    pub description: String,
    pub patch_diff: String,
    pub files_changed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePatchRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub patch_diff: Option<String>,
    pub files_changed: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotePatchRequest {
    pub vote_type: VoteType,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratePatchRequest {
    pub vulnerability_id: Uuid,
    pub strategy: Option<GenerationStrategy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatePatchRequest {
    pub patch_diff: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyPatchRequest {
    pub create_pull_request: bool,
    pub pr_title: Option<String>,
    pub pr_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewPatchRequest {
    pub patch_diff: String,
}