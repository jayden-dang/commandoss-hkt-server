use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{
    CollaborationMetrics, DeveloperAnalytics, PlatformOverview, RepositoryAnalytics,
    SecurityTrend, SkillDistribution, TeamAnalytics,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperAnalyticsResponse {
    pub analytics: DeveloperAnalytics,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationHistoryResponse {
    pub developer_id: Uuid,
    pub history: Vec<ReputationPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationPoint {
    pub date: DateTime<Utc>,
    pub score: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshReputationResponse {
    pub developer_id: Uuid,
    pub new_score: Decimal,
    pub previous_score: Decimal,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryAnalyticsResponse {
    pub analytics: RepositoryAnalytics,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryTrendsResponse {
    pub repository_id: Uuid,
    pub trends: Vec<SecurityTrend>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryContributorsResponse {
    pub repository_id: Uuid,
    pub contributors: Vec<ContributorInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorInfo {
    pub developer_id: Uuid,
    pub username: String,
    pub contribution_score: Decimal,
    pub commits_count: i32,
    pub issues_resolved: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPerformanceResponse {
    pub analytics: TeamAnalytics,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamSkillsResponse {
    pub team_id: Uuid,
    pub skills: Vec<SkillDistribution>,
    pub total_members: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamCollaborationResponse {
    pub team_id: Uuid,
    pub metrics: CollaborationMetrics,
    pub collaboration_graph: Option<CollaborationGraph>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationGraph {
    pub nodes: Vec<CollaborationNode>,
    pub edges: Vec<CollaborationEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationNode {
    pub developer_id: Uuid,
    pub username: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationEdge {
    pub from_id: Uuid,
    pub to_id: Uuid,
    pub interaction_count: i32,
    pub interaction_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformOverviewResponse {
    pub overview: PlatformOverview,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformTrendsResponse {
    pub trends: Vec<SecurityTrend>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}