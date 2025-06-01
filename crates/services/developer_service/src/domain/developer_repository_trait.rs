use async_trait::async_trait;
use rust_decimal::Decimal;
use uuid::Uuid;

use super::developer_models::*;
use crate::Result;

#[async_trait]
pub trait DeveloperRepository: Send + Sync {
    // CRUD Operations
    async fn create(&self, developer: &Developer) -> Result<Developer>;
    
    async fn get_by_id(&self, id: Uuid) -> Result<Developer>;
    
    async fn get_by_username(&self, username: &str) -> Result<Developer>;
    
    async fn get_by_email(&self, email: &str) -> Result<Developer>;
    
    async fn update(&self, developer: &Developer) -> Result<Developer>;
    
    async fn delete(&self, id: Uuid) -> Result<()>;
    
    // Profile Management
    async fn update_profile(&self, id: Uuid, profile: &DeveloperProfile) -> Result<()>;
    
    async fn add_skill(&self, developer_id: Uuid, skill: &Skill) -> Result<()>;
    
    async fn remove_skill(&self, developer_id: Uuid, skill_name: &str) -> Result<()>;
    
    async fn update_social_links(&self, id: Uuid, links: &SocialLinks) -> Result<()>;
    
    // Search and Discovery
    async fn search(&self, search: &DeveloperSearch) -> Result<Vec<Developer>>;
    
    async fn count_search(&self, search: &DeveloperSearch) -> Result<i64>;
    
    async fn get_leaderboard(&self, limit: i64) -> Result<Vec<DeveloperLeaderboard>>;
    
    async fn get_by_skill(&self, skill: &str, limit: i64) -> Result<Vec<Developer>>;
    
    // Activities and Contributions
    async fn add_activity(&self, activity: &DeveloperActivity) -> Result<()>;
    
    async fn get_activities(
        &self,
        developer_id: Uuid,
        limit: i64,
    ) -> Result<Vec<DeveloperActivity>>;
    
    async fn get_contributions(
        &self,
        developer_id: Uuid,
    ) -> Result<Vec<DeveloperContribution>>;
    
    async fn add_contribution(&self, contribution: &DeveloperContribution) -> Result<()>;
    
    // Code Reviews
    async fn add_code_review(&self, review: &CodeReview) -> Result<()>;
    
    async fn get_reviews_given(
        &self,
        reviewer_id: Uuid,
        limit: i64,
    ) -> Result<Vec<CodeReview>>;
    
    async fn get_reviews_received(
        &self,
        author_id: Uuid,
        limit: i64,
    ) -> Result<Vec<CodeReview>>;
    
    // Collaboration Network
    async fn get_collaborators(
        &self,
        developer_id: Uuid,
    ) -> Result<Vec<DeveloperCollaborator>>;
    
    async fn add_collaboration(
        &self,
        developer_id: Uuid,
        collaborator_id: Uuid,
        score: Decimal,
    ) -> Result<()>;
    
    async fn get_network(&self, developer_id: Uuid) -> Result<DeveloperNetwork>;
    
    async fn get_mentees(&self, mentor_id: Uuid) -> Result<Vec<Developer>>;
    
    // Verification
    async fn verify_developer(
        &self,
        developer_id: Uuid,
        verification: &ZKProofVerification,
    ) -> Result<()>;
    
    async fn update_reputation(&self, developer_id: Uuid, new_score: f64) -> Result<()>;
    
    // Statistics
    async fn get_developer_count(&self) -> Result<i64>;
    
    async fn get_verified_developer_count(&self) -> Result<i64>;
    
    async fn get_active_developer_count(&self, days: i32) -> Result<i64>;
}