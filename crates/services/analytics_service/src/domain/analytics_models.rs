use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperAnalytics {
    pub developer_id: Uuid,
    pub reputation_score: Decimal,
    pub reputation_components: ReputationComponents,
    pub recent_activities: Vec<ActivitySummary>,
    pub skill_trends: Vec<SkillTrend>,
    pub peer_comparison: PeerComparison,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationComponents {
    pub code_quality_score: Decimal,
    pub security_expertise_score: Decimal,
    pub collaboration_score: Decimal,
    pub contribution_frequency: Decimal,
    pub review_quality: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySummary {
    pub activity_type: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub impact_score: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTrend {
    pub skill_name: String,
    pub proficiency_level: Decimal,
    pub growth_rate: Decimal,
    pub last_demonstrated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerComparison {
    pub percentile_rank: Decimal,
    pub compared_to_count: i32,
    pub strengths: Vec<String>,
    pub improvement_areas: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryAnalytics {
    pub repository_id: Uuid,
    pub security_score: Decimal,
    pub vulnerability_count: VulnerabilityCount,
    pub patch_success_rate: Decimal,
    pub contributor_count: i32,
    pub last_analysis: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityCount {
    pub critical: i32,
    pub high: i32,
    pub medium: i32,
    pub low: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAnalytics {
    pub team_id: Uuid,
    pub team_name: String,
    pub performance_score: Decimal,
    pub collaboration_metrics: CollaborationMetrics,
    pub skill_distribution: Vec<SkillDistribution>,
    pub productivity_trends: Vec<ProductivityTrend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationMetrics {
    pub code_review_rate: Decimal,
    pub average_review_time: Decimal,
    pub knowledge_sharing_score: Decimal,
    pub communication_effectiveness: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDistribution {
    pub skill_category: String,
    pub member_count: i32,
    pub average_proficiency: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityTrend {
    pub period: String,
    pub commits_count: i32,
    pub issues_resolved: i32,
    pub average_cycle_time: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformOverview {
    pub total_developers: i32,
    pub total_repositories: i32,
    pub total_vulnerabilities_found: i32,
    pub total_patches_applied: i32,
    pub platform_security_score: Decimal,
    pub trending_technologies: Vec<TrendingTech>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingTech {
    pub technology: String,
    pub usage_count: i32,
    pub growth_percentage: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityTrend {
    pub date: DateTime<Utc>,
    pub vulnerabilities_discovered: i32,
    pub vulnerabilities_patched: i32,
    pub average_patch_time: Decimal,
    pub security_score: Decimal,
}