use serde::{Deserialize, Serialize};
use uuid::Uuid;
use modql::field::Fields;
use modql::filter::{FilterNodes, OpValsString, OpValsValue};
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Developer {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub github_username: Option<String>,
    pub wallet_address: Option<String>,
    pub reputation_score: f64,
    pub is_verified: bool,
    pub verification_date: Option<OffsetDateTime>,
    pub profile: DeveloperProfile,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperProfile {
    pub display_name: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    pub skills: Vec<Skill>,
    pub social_links: SocialLinks,
    pub preferences: DeveloperPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub category: SkillCategory,
    pub proficiency_level: ProficiencyLevel,
    pub years_experience: Option<i32>,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillCategory {
    ProgrammingLanguage,
    Framework,
    Security,
    DevOps,
    Database,
    Cloud,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProficiencyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialLinks {
    pub github: Option<String>,
    pub twitter: Option<String>,
    pub linkedin: Option<String>,
    pub personal_site: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperPreferences {
    pub email_notifications: bool,
    pub public_profile: bool,
    pub show_activity: bool,
    pub preferred_languages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperActivity {
    pub id: Uuid,
    pub developer_id: Uuid,
    pub activity_type: ActivityType,
    pub description: String,
    pub repository_id: Option<Uuid>,
    pub related_id: Option<Uuid>,
    pub impact_score: f64,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
    CodeCommit,
    PullRequest,
    CodeReview,
    IssueCreated,
    IssueResolved,
    VulnerabilityFound,
    PatchSubmitted,
    PatchApproved,
    Collaboration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperContribution {
    pub id: Uuid,
    pub developer_id: Uuid,
    pub repository_id: Uuid,
    pub contribution_type: String,
    pub commits_count: i32,
    pub lines_added: i32,
    pub lines_removed: i32,
    pub pull_requests_count: i32,
    pub issues_resolved_count: i32,
    pub last_activity: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReview {
    pub id: Uuid,
    pub reviewer_id: Uuid,
    pub author_id: Uuid,
    pub repository_id: Uuid,
    pub pull_request_id: String,
    pub review_quality_score: f64,
    pub comments_count: i32,
    pub suggestions_accepted: i32,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperCollaborator {
    pub developer_id: Uuid,
    pub collaborator_id: Uuid,
    pub collaboration_score: f64,
    pub projects_count: i32,
    pub last_collaboration: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperNetwork {
    pub developer_id: Uuid,
    pub collaborators: Vec<NetworkNode>,
    pub total_connections: i32,
    pub collaboration_strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkNode {
    pub developer_id: Uuid,
    pub username: String,
    pub connection_strength: f64,
    pub shared_projects: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperSearch {
    pub query: Option<String>,
    pub skills: Option<Vec<String>>,
    pub min_reputation: Option<f64>,
    pub verified_only: bool,
    pub sort_by: SortBy,
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    ReputationDesc,
    ReputationAsc,
    ActivityDesc,
    JoinDateDesc,
    JoinDateAsc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperLeaderboard {
    pub developer_id: Uuid,
    pub username: String,
    pub reputation_score: f64,
    pub rank: i32,
    pub contributions_count: i32,
    pub specialization: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZKProofVerification {
    pub developer_id: Uuid,
    pub proof_data: String,
    pub verification_type: String,
    pub verified_at: OffsetDateTime,
    pub expires_at: Option<OffsetDateTime>,
}

// Database representation without complex profile field
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
pub struct DeveloperDb {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub github_username: Option<String>,
    pub wallet_address: Option<String>,
    pub reputation_score: f64,
    pub is_verified: bool,
    pub verification_date: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

// For database operations with base::rest
#[derive(Debug, Clone, Serialize, Deserialize, Fields)]
pub struct DeveloperForCreate {
    pub username: String,
    pub email: String,
    pub github_username: Option<String>,
    pub wallet_address: Option<String>,
    pub reputation_score: Option<f64>,
    pub is_verified: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Fields)]
pub struct DeveloperForUpdate {
    pub username: Option<String>,
    pub email: Option<String>,
    pub github_username: Option<String>,
    pub wallet_address: Option<String>,
    pub reputation_score: Option<f64>,
    pub is_verified: Option<bool>,
    pub verification_date: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Deserialize, FilterNodes)]
pub struct DeveloperFilter {
    pub id: Option<OpValsValue>,
    pub username: Option<OpValsString>,
    pub email: Option<OpValsString>,
    pub github_username: Option<OpValsString>,
    pub wallet_address: Option<OpValsString>,
    pub is_verified: Option<OpValsValue>,
}

impl DeveloperDb {
    /// Convert database representation to full Developer with default profile
    pub fn to_developer(self) -> Developer {
        Developer {
            id: self.id,
            username: self.username.clone(),
            email: self.email.clone(),
            github_username: self.github_username.clone(),
            wallet_address: self.wallet_address.clone(),
            reputation_score: self.reputation_score,
            is_verified: self.is_verified,
            verification_date: self.verification_date,
            profile: DeveloperProfile {
                display_name: self.username.clone(),
                bio: None,
                avatar_url: None,
                location: None,
                website: None,
                skills: vec![],
                social_links: SocialLinks {
                    github: self.github_username.clone(),
                    twitter: None,
                    linkedin: None,
                    personal_site: None,
                },
                preferences: DeveloperPreferences {
                    email_notifications: true,
                    public_profile: true,
                    show_activity: true,
                    preferred_languages: vec![],
                },
            },
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}