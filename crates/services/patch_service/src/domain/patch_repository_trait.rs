use async_trait::async_trait;
use rust_decimal::Decimal;
use uuid::Uuid;

use super::patch_models::*;
use crate::Result;

#[async_trait]
pub trait PatchRepository: Send + Sync {
    // CRUD Operations
    async fn create(&self, patch: &PatchProposal) -> Result<PatchProposal>;
    
    async fn get_by_id(&self, id: Uuid) -> Result<PatchProposal>;
    
    async fn update(&self, patch: &PatchProposal) -> Result<PatchProposal>;
    
    async fn delete(&self, id: Uuid) -> Result<()>;
    
    // List and Filter
    async fn list(
        &self,
        filter: &PatchFilter,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<PatchProposal>>;
    
    async fn count(&self, filter: &PatchFilter) -> Result<i64>;
    
    // Voting
    async fn add_vote(&self, vote: &Vote) -> Result<()>;
    
    async fn get_vote(&self, patch_id: Uuid, voter_id: Uuid) -> Result<Option<Vote>>;
    
    async fn get_votes(&self, patch_id: Uuid) -> Result<Vec<Vote>>;
    
    async fn update_vote_scores(
        &self,
        patch_id: Uuid,
        approval_score: Decimal,
        rejection_score: Decimal,
        total_votes: i32,
    ) -> Result<()>;
    
    // Status Management
    async fn update_status(&self, id: Uuid, status: PatchStatus) -> Result<()>;
    
    async fn mark_as_applied(&self, id: Uuid, pr_url: Option<String>) -> Result<()>;
    
    // Validation
    async fn update_validation_status(
        &self,
        id: Uuid,
        validation: &ValidationStatus,
    ) -> Result<()>;
    
    // Statistics
    async fn get_statistics(&self) -> Result<PatchStatistics>;
    
    async fn get_leaderboard(&self, limit: i64) -> Result<Vec<PatchLeaderboard>>;
    
    async fn get_developer_statistics(&self, developer_id: Uuid) -> Result<PatchStatistics>;
    
    // Vulnerability-specific queries
    async fn get_by_vulnerability(&self, vulnerability_id: Uuid) -> Result<Vec<PatchProposal>>;
    
    async fn has_approved_patch(&self, vulnerability_id: Uuid) -> Result<bool>;
}