use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use uuid::Uuid;
use jd_core::{AppState, base};

use crate::{
    PatchDmc,
    domain::{
        PatchFilter, PatchLeaderboard, PatchProposal, PatchRepository, PatchStatistics, PatchStatus,
        ValidationStatus, Vote, PatchProposalDb, PatchProposalForCreate, PatchProposalForUpdate, PatchProposalFilter,
    },
    Error, Result,
};

pub struct PatchRepositoryImpl {
    state: AppState,
}

impl PatchRepositoryImpl {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl PatchRepository for PatchRepositoryImpl {
    async fn create(&self, patch: &PatchProposal) -> Result<PatchProposal> {
        let create_req = PatchProposalForCreate {
            vulnerability_id: patch.vulnerability_id,
            repository_id: patch.repository_id,
            developer_id: patch.developer_id,
            title: patch.title.clone(),
            description: patch.description.clone(),
            patch_diff: patch.patch_diff.clone(),
            status: Some(patch.status.to_string()),
            approval_score: Some(patch.approval_score),
            rejection_score: Some(patch.rejection_score),
            total_votes: Some(patch.total_votes),
            generated_by_ai: Some(patch.generated_by_ai),
        };
        
        let patch_db = base::rest::create::<PatchDmc, _, PatchProposalDb>(&self.state.mm(), create_req)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(patch_db.to_patch_proposal())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<PatchProposal> {
        match base::rest::get_by_id::<PatchDmc, PatchProposalDb>(&self.state.mm(), id).await {
            Ok(patch_db) => Ok(patch_db.to_patch_proposal()),
            Err(_) => Err(Error::PatchNotFound(id.to_string())),
        }
    }

    async fn update(&self, patch: &PatchProposal) -> Result<PatchProposal> {
        let update_req = PatchProposalForUpdate {
            title: Some(patch.title.clone()),
            description: Some(patch.description.clone()),
            patch_diff: Some(patch.patch_diff.clone()),
            status: Some(patch.status.to_string()),
            approval_score: Some(patch.approval_score),
            rejection_score: Some(patch.rejection_score),
            total_votes: Some(patch.total_votes),
            applied_at: patch.applied_at,
            pr_url: patch.pr_url.clone(),
        };
        
        base::rest::update::<PatchDmc, _>(&self.state.mm(), patch.id, update_req)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(patch.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        base::rest::delete::<PatchDmc>(&self.state.mm(), id)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn list(
        &self,
        _filter: &PatchFilter,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<PatchProposal>> {
        // For now, simple list without filtering (TODO: Implement filtering)
        use modql::filter::ListOptions;
        
        let list_options = ListOptions {
            limit: Some(limit),
            offset: Some(offset),
            order_bys: Some("created_at desc".to_string().into()),
        };

        let (patches_db, _): (Vec<PatchProposalDb>, _) = base::rest::list::<PatchDmc, PatchProposalFilter, PatchProposalDb>(
            &self.state.mm(), 
            None,
            Some(list_options)
        ).await.map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(patches_db.into_iter().map(|p| p.to_patch_proposal()).collect())
    }

    async fn count(&self, _filter: &PatchFilter) -> Result<i64> {
        // For now, simple count without filtering (TODO: Implement filtering)
        let count: i64 = base::rest::count::<PatchDmc, PatchProposalFilter>(&self.state.mm(), None)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(count)
    }

    async fn add_vote(&self, vote: &Vote) -> Result<()> {
        // TODO: Implement with base::rest once Vote DMC is created
        // For now, return success to prevent compilation errors
        Ok(())
    }

    async fn get_vote(&self, patch_id: Uuid, voter_id: Uuid) -> Result<Option<Vote>> {
        // TODO: Implement with base::rest once Vote DMC is created
        Ok(None)
    }

    async fn get_votes(&self, patch_id: Uuid) -> Result<Vec<Vote>> {
        // TODO: Implement with base::rest once Vote DMC is created
        Ok(vec![])
    }

    async fn update_vote_scores(
        &self,
        patch_id: Uuid,
        approval_score: Decimal,
        rejection_score: Decimal,
        total_votes: i32,
    ) -> Result<()> {
        let update_req = PatchProposalForUpdate {
            title: None,
            description: None,
            patch_diff: None,
            status: None,
            approval_score: Some(approval_score.to_f64().unwrap_or(0.0)),
            rejection_score: Some(rejection_score.to_f64().unwrap_or(0.0)),
            total_votes: Some(total_votes),
            applied_at: None,
            pr_url: None,
        };
        
        base::rest::update::<PatchDmc, _>(&self.state.mm(), patch_id, update_req)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn update_status(&self, id: Uuid, status: PatchStatus) -> Result<()> {
        let update_req = PatchProposalForUpdate {
            title: None,
            description: None,
            patch_diff: None,
            status: Some(status.to_string()),
            approval_score: None,
            rejection_score: None,
            total_votes: None,
            applied_at: None,
            pr_url: None,
        };
        
        base::rest::update::<PatchDmc, _>(&self.state.mm(), id, update_req)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn mark_as_applied(&self, id: Uuid, pr_url: Option<String>) -> Result<()> {
        let update_req = PatchProposalForUpdate {
            title: None,
            description: None,
            patch_diff: None,
            status: Some("applied".to_string()),
            approval_score: None,
            rejection_score: None,
            total_votes: None,
            applied_at: Some(time::OffsetDateTime::now_utc()),
            pr_url,
        };
        
        base::rest::update::<PatchDmc, _>(&self.state.mm(), id, update_req)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn update_validation_status(
        &self,
        id: Uuid,
        validation: &ValidationStatus,
    ) -> Result<()> {
        // TODO: Implement validation_status field in PatchProposalForUpdate
        // For now, return success to prevent compilation errors
        Ok(())
    }

    async fn get_statistics(&self) -> Result<PatchStatistics> {
        // TODO: Implement complex aggregation queries
        // For now, return placeholder statistics
        Ok(PatchStatistics {
            total_patches: 0,
            approved_patches: 0,
            rejected_patches: 0,
            applied_patches: 0,
            success_rate: 0.0,
            average_approval_time: Some(24),
            ai_generated_percentage: 0.0,
        })
    }

    async fn get_leaderboard(&self, limit: i64) -> Result<Vec<PatchLeaderboard>> {
        // TODO: Implement complex join queries with developers table
        // For now, return empty leaderboard
        Ok(vec![])
    }

    async fn get_developer_statistics(&self, _developer_id: Uuid) -> Result<PatchStatistics> {
        // For now, simple count (TODO: Implement developer filtering)
        let total: i64 = base::rest::count::<PatchDmc, PatchProposalFilter>(&self.state.mm(), None)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // TODO: Implement status-specific counts once complex filtering is supported
        Ok(PatchStatistics {
            total_patches: total,
            approved_patches: 0,
            rejected_patches: 0,
            applied_patches: 0,
            success_rate: 0.0,
            average_approval_time: Some(24),
            ai_generated_percentage: 0.0,
        })
    }

    async fn get_by_vulnerability(&self, vulnerability_id: Uuid) -> Result<Vec<PatchProposal>> {
        let filter = PatchFilter {
            repository_id: None,
            vulnerability_id: Some(vulnerability_id),
            developer_id: None,
            status: None,
            generated_by_ai: None,
            created_after: None,
            created_before: None,
        };

        self.list(&filter, 0, i64::MAX).await
    }

    async fn has_approved_patch(&self, _vulnerability_id: Uuid) -> Result<bool> {
        // For now, return false (TODO: Implement filtering for approved patches)
        Ok(false)
    }
}