use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::analytics_models::*;
use crate::Result;

#[async_trait]
pub trait AnalyticsRepository: Send + Sync {
    // Developer Analytics
    async fn get_developer_analytics(&self, developer_id: Uuid) -> Result<DeveloperAnalytics>;
    
    async fn get_reputation_history(
        &self,
        developer_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<(DateTime<Utc>, rust_decimal::Decimal)>>;
    
    async fn refresh_reputation_score(&self, developer_id: Uuid) -> Result<rust_decimal::Decimal>;
    
    // Repository Analytics
    async fn get_repository_analytics(&self, repository_id: Uuid) -> Result<RepositoryAnalytics>;
    
    async fn get_repository_trends(
        &self,
        repository_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<SecurityTrend>>;
    
    async fn get_repository_contributors(
        &self,
        repository_id: Uuid,
    ) -> Result<Vec<(Uuid, String, rust_decimal::Decimal)>>;
    
    // Team Analytics
    async fn get_team_performance(&self, team_id: Uuid) -> Result<TeamAnalytics>;
    
    async fn get_team_skills(&self, team_id: Uuid) -> Result<Vec<SkillDistribution>>;
    
    async fn get_team_collaboration_metrics(&self, team_id: Uuid) -> Result<CollaborationMetrics>;
    
    // Platform Overview
    async fn get_platform_overview(&self) -> Result<PlatformOverview>;
    
    async fn get_platform_trends(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<SecurityTrend>>;
}