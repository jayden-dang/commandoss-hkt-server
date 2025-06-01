use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::patch_models::*;
use crate::Result;

#[async_trait]
pub trait PatchGenerator: Send + Sync {
    // Generate a patch for a vulnerability
    async fn generate_patch(
        &self,
        request: &PatchGenerationRequest,
    ) -> Result<GeneratedPatch>;
    
    // Validate that a patch can be applied
    async fn validate_patch(
        &self,
        patch_diff: &str,
        repository_id: Uuid,
    ) -> Result<ValidationResult>;
    
    // Preview patch application
    async fn preview_patch(
        &self,
        patch_diff: &str,
        repository_id: Uuid,
    ) -> Result<PreviewResult>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedPatch {
    pub title: String,
    pub description: String,
    pub patch_diff: String,
    pub files_changed: Vec<String>,
    pub confidence_score: Decimal,
    pub generation_metadata: GenerationMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetadata {
    pub model_version: String,
    pub generation_time_ms: i64,
    pub tokens_used: i32,
    pub strategy_used: GenerationStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub can_apply_cleanly: bool,
    pub conflicts: Vec<String>,
    pub syntax_errors: Vec<String>,
    pub security_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewResult {
    pub affected_files: Vec<FilePreview>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePreview {
    pub file_path: String,
    pub original_content: String,
    pub patched_content: String,
    pub changes_count: i32,
}