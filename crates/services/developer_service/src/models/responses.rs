use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{
    CodeReview, Developer, DeveloperActivity, DeveloperCollaborator, DeveloperContribution,
    DeveloperLeaderboard, DeveloperNetwork, Skill,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperResponse {
    pub developer: Developer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperListResponse {
    pub developers: Vec<Developer>,
    pub total_count: i64,
    pub page: u32,
    pub limit: u32,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperProfileResponse {
    pub developer: Developer,
    pub recent_activities: Vec<DeveloperActivity>,
    pub top_skills: Vec<Skill>,
    pub contributions_summary: ContributionsSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionsSummary {
    pub total_commits: i32,
    pub total_pull_requests: i32,
    pub total_issues_resolved: i32,
    pub repositories_contributed: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitiesResponse {
    pub developer_id: Uuid,
    pub activities: Vec<DeveloperActivity>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionsResponse {
    pub developer_id: Uuid,
    pub contributions: Vec<DeveloperContribution>,
    pub total_repositories: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewsResponse {
    pub reviews: Vec<CodeReviewDetail>,
    pub total_count: i64,
    pub average_quality_score: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReviewDetail {
    pub review: CodeReview,
    pub repository_name: String,
    pub author_username: String,
    pub reviewer_username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaboratorsResponse {
    pub developer_id: Uuid,
    pub collaborators: Vec<CollaboratorDetail>,
    pub total_collaborators: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaboratorDetail {
    pub collaborator: DeveloperCollaborator,
    pub username: String,
    pub reputation_score: Decimal,
    pub common_skills: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkResponse {
    pub network: DeveloperNetwork,
    pub network_strength: Decimal,
    pub recommended_connections: Vec<Developer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    pub leaderboard: Vec<DeveloperLeaderboard>,
    pub updated_at: DateTime<Utc>,
    pub period: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResponse {
    pub developer_id: Uuid,
    pub verified: bool,
    pub verification_date: DateTime<Utc>,
    pub verification_type: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsResponse {
    pub developer_id: Uuid,
    pub skills: Vec<Skill>,
    pub verified_skills_count: i32,
    pub skill_categories: Vec<SkillCategorySummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCategorySummary {
    pub category: String,
    pub skills_count: i32,
    pub average_proficiency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenteesResponse {
    pub mentor_id: Uuid,
    pub mentees: Vec<MenteeDetail>,
    pub total_mentees: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenteeDetail {
    pub developer: Developer,
    pub mentorship_start_date: DateTime<Utc>,
    pub progress_score: Decimal,
}