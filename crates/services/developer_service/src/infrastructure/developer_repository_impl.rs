use async_trait::async_trait;
use rust_decimal::Decimal;
use time::OffsetDateTime;
use uuid::Uuid;
use jd_core::{AppState, base};

use crate::{
    DeveloperDmc,
    domain::{
        CodeReview, Developer, DeveloperActivity, DeveloperCollaborator, DeveloperContribution,
        DeveloperLeaderboard, DeveloperNetwork, DeveloperProfile, DeveloperRepository, DeveloperSearch,
        Skill, SocialLinks, ZKProofVerification,
        DeveloperForCreate, DeveloperForUpdate, DeveloperFilter, DeveloperDb,
    },
    Error, Result,
};

pub struct DeveloperRepositoryImpl {
    state: AppState,
}

impl DeveloperRepositoryImpl {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl DeveloperRepository for DeveloperRepositoryImpl {
    async fn create(&self, developer: &Developer) -> Result<Developer> {
        let create_req = DeveloperForCreate {
            username: developer.username.clone(),
            email: developer.email.clone(),
            github_username: developer.github_username.clone(),
            wallet_address: developer.wallet_address.clone(),
            reputation_score: Some(developer.reputation_score),
            is_verified: Some(developer.is_verified),
        };
        
        base::rest::create::<DeveloperDmc, _, DeveloperDb>(&self.state.mm(), create_req)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // TODO: Handle profile insertion separately (requires separate table/model)
        Ok(developer.clone())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Developer> {
        match base::rest::get_by_id::<DeveloperDmc, DeveloperDb>(&self.state.mm(), id).await {
            Ok(developer_db) => Ok(developer_db.to_developer()),
            Err(_) => Err(Error::DeveloperNotFound(id.to_string())),
        }
    }

    async fn get_by_username(&self, username: &str) -> Result<Developer> {
        let filter = DeveloperFilter {
            username: Some(username.into()),
            email: None,
            github_username: None,
            wallet_address: None,
            is_verified: None,
            id: None,
        };

        match base::rest::first::<DeveloperDmc, _, DeveloperDb>(&self.state.mm(), Some(filter), None).await {
            Ok(Some(developer_db)) => Ok(developer_db.to_developer()),
            Ok(None) => Err(Error::DeveloperNotFound(username.to_string())),
            Err(_) => Err(Error::DeveloperNotFound(username.to_string())),
        }
    }

    async fn get_by_email(&self, email: &str) -> Result<Developer> {
        let filter = DeveloperFilter {
            email: Some(email.into()),
            username: None,
            github_username: None,
            wallet_address: None,
            is_verified: None,
            id: None,
        };

        match base::rest::first::<DeveloperDmc, _, DeveloperDb>(&self.state.mm(), Some(filter), None).await {
            Ok(Some(developer_db)) => Ok(developer_db.to_developer()),
            Ok(None) => Err(Error::DeveloperNotFound(email.to_string())),
            Err(_) => Err(Error::DeveloperNotFound(email.to_string())),
        }
    }

    async fn update(&self, developer: &Developer) -> Result<Developer> {
        let update_req = DeveloperForUpdate {
            username: Some(developer.username.clone()),
            email: Some(developer.email.clone()),
            github_username: developer.github_username.clone(),
            wallet_address: developer.wallet_address.clone(),
            reputation_score: Some(developer.reputation_score),
            is_verified: Some(developer.is_verified),
            verification_date: developer.verification_date,
        };
        
        base::rest::update::<DeveloperDmc, _>(&self.state.mm(), developer.id, update_req)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // TODO: Handle profile updates separately
        Ok(developer.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        base::rest::delete::<DeveloperDmc>(&self.state.mm(), id)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn update_profile(&self, _id: Uuid, _profile: &DeveloperProfile) -> Result<()> {
        // TODO: Implement profile updates with separate ProfileDmc
        Ok(())
    }

    async fn add_skill(&self, _developer_id: Uuid, _skill: &Skill) -> Result<()> {
        // TODO: Implement skill management with separate SkillDmc
        Ok(())
    }

    async fn remove_skill(&self, _developer_id: Uuid, _skill_name: &str) -> Result<()> {
        // TODO: Implement skill management with separate SkillDmc
        Ok(())
    }

    async fn update_social_links(&self, id: Uuid, links: &SocialLinks) -> Result<()> {
        // Update social links in profile
        let mut developer = self.get_by_id(id).await?;
        developer.profile.social_links = links.clone();
        self.update(&developer).await?;
        Ok(())
    }

    async fn search(&self, search: &DeveloperSearch) -> Result<Vec<Developer>> {
        // TODO: Implement complex search using base::rest::list with filters
        use modql::filter::ListOptions;
        
        let offset = (search.page - 1) * search.limit;
        let list_options = ListOptions {
            limit: Some(search.limit as i64),
            offset: Some(offset as i64),
            order_bys: Some("reputation_score desc".to_string().into()),
        };

        let (developers, _) = base::rest::list::<DeveloperDmc, DeveloperFilter, DeveloperDb>(
            &self.state.mm(), 
            None, // TODO: Convert DeveloperSearch to DeveloperFilter
            Some(list_options)
        ).await.map_err(|e| Error::DatabaseError(e.to_string()))?;

        // Convert to full Developer structs
        let results = developers.into_iter().map(|db| db.to_developer()).collect();

        Ok(results)
    }

    async fn count_search(&self, _search: &DeveloperSearch) -> Result<i64> {
        // TODO: Implement with base::rest::count
        base::rest::count::<DeveloperDmc, DeveloperFilter>(&self.state.mm(), None)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))
    }

    async fn get_leaderboard(&self, limit: i64) -> Result<Vec<DeveloperLeaderboard>> {
        // TODO: Implement leaderboard with complex query
        use modql::filter::ListOptions;
        
        let list_options = ListOptions {
            limit: Some(limit),
            offset: Some(0),
            order_bys: Some("reputation_score desc".to_string().into()),
        };

        let (developers, _) = base::rest::list::<DeveloperDmc, DeveloperFilter, DeveloperDb>(
            &self.state.mm(), 
            None,
            Some(list_options)
        ).await.map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(developers
            .into_iter()
            .enumerate()
            .map(|(i, dev)| DeveloperLeaderboard {
                developer_id: dev.id,
                username: dev.username,
                reputation_score: dev.reputation_score,
                rank: (i + 1) as i32,
                contributions_count: 0, // TODO: Calculate from activities
                specialization: "Full Stack".to_string(), // TODO: Derive from skills
            })
            .collect())
    }

    async fn get_by_skill(&self, _skill: &str, limit: i64) -> Result<Vec<Developer>> {
        // TODO: Implement with skill filtering when SkillDmc is available
        use modql::filter::ListOptions;
        
        let list_options = ListOptions {
            limit: Some(limit),
            offset: Some(0),
            order_bys: Some("reputation_score desc".to_string().into()),
        };

        let (developers, _) = base::rest::list::<DeveloperDmc, DeveloperFilter, DeveloperDb>(
            &self.state.mm(), 
            None,
            Some(list_options)
        ).await.map_err(|e| Error::DatabaseError(e.to_string()))?;

        // Convert to full Developer structs
        let results = developers.into_iter().map(|db| db.to_developer()).collect();

        Ok(results)
    }

    async fn add_activity(&self, _activity: &DeveloperActivity) -> Result<()> {
        // TODO: Implement with separate ActivityDmc
        Ok(())
    }

    async fn get_activities(
        &self,
        _developer_id: Uuid,
        _limit: i64,
    ) -> Result<Vec<DeveloperActivity>> {
        // TODO: Implement with separate ActivityDmc
        Ok(vec![])
    }

    async fn get_contributions(
        &self,
        _developer_id: Uuid,
    ) -> Result<Vec<DeveloperContribution>> {
        // TODO: Implement with separate ContributionDmc
        Ok(vec![])
    }

    async fn add_contribution(&self, _contribution: &DeveloperContribution) -> Result<()> {
        // TODO: Implement with separate ContributionDmc
        Ok(())
    }

    async fn add_code_review(&self, _review: &CodeReview) -> Result<()> {
        // TODO: Implement with separate CodeReviewDmc
        Ok(())
    }

    async fn get_reviews_given(
        &self,
        _reviewer_id: Uuid,
        _limit: i64,
    ) -> Result<Vec<CodeReview>> {
        // TODO: Implement with separate CodeReviewDmc
        Ok(vec![])
    }

    async fn get_reviews_received(
        &self,
        _author_id: Uuid,
        _limit: i64,
    ) -> Result<Vec<CodeReview>> {
        // TODO: Implement with separate CodeReviewDmc
        Ok(vec![])
    }

    async fn get_collaborators(
        &self,
        _developer_id: Uuid,
    ) -> Result<Vec<DeveloperCollaborator>> {
        // TODO: Implement with separate CollaboratorDmc
        Ok(vec![])
    }

    async fn add_collaboration(
        &self,
        _developer_id: Uuid,
        _collaborator_id: Uuid,
        _score: Decimal,
    ) -> Result<()> {
        // TODO: Implement with separate CollaboratorDmc
        Ok(())
    }

    async fn get_network(&self, developer_id: Uuid) -> Result<DeveloperNetwork> {
        // TODO: Implement with proper network analysis
        Ok(DeveloperNetwork {
            developer_id,
            collaborators: vec![],
            total_connections: 0,
            collaboration_strength: 0.0,
        })
    }

    async fn get_mentees(&self, _mentor_id: Uuid) -> Result<Vec<Developer>> {
        // TODO: Implement mentorship relationships
        Ok(vec![])
    }

    async fn verify_developer(
        &self,
        developer_id: Uuid,
        _verification: &ZKProofVerification,
    ) -> Result<()> {
        let update_req = DeveloperForUpdate {
            username: None,
            email: None,
            github_username: None,
            wallet_address: None,
            reputation_score: None,
            is_verified: Some(true),
            verification_date: Some(OffsetDateTime::now_utc()),
        };
        
        base::rest::update::<DeveloperDmc, _>(&self.state.mm(), developer_id, update_req)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // TODO: Store verification details in separate VerificationDmc
        Ok(())
    }

    async fn update_reputation(&self, developer_id: Uuid, new_score: f64) -> Result<()> {
        let update_req = DeveloperForUpdate {
            username: None,
            email: None,
            github_username: None,
            wallet_address: None,
            reputation_score: Some(new_score),
            is_verified: None,
            verification_date: None,
        };
        
        base::rest::update::<DeveloperDmc, _>(&self.state.mm(), developer_id, update_req)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_developer_count(&self) -> Result<i64> {
        base::rest::count::<DeveloperDmc, DeveloperFilter>(&self.state.mm(), None)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))
    }

    async fn get_verified_developer_count(&self) -> Result<i64> {
        use modql::filter::{OpValValue};
        
        let filter = DeveloperFilter {
            is_verified: Some(OpValValue::Eq(serde_json::Value::Bool(true)).into()),
            username: None,
            email: None,
            github_username: None,
            wallet_address: None,
            id: None,
        };

        base::rest::count::<DeveloperDmc, _>(&self.state.mm(), Some(filter))
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))
    }

    async fn get_active_developer_count(&self, _days: i32) -> Result<i64> {
        // TODO: Implement with activity tracking
        // For now, return total count as placeholder
        self.get_developer_count().await
    }
}