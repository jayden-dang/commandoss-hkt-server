use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;

use crate::Id;

// ================================================================================================
// Developer Ecosystem Models
// ================================================================================================

// ------------------------------------------------------------------------------------------------
// Developer Management
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Developer {
    pub id: Id,
    
    // GitHub identity
    pub github_username: String,
    pub github_user_id: i64,
    pub display_name: Option<String>,
    pub email: Option<String>,
    
    // Reputation scores (0-100 scale)
    pub coding_reputation_score: Decimal,
    pub security_awareness_score: Decimal,
    pub community_trust_score: Decimal,
    
    // Activity metrics
    pub total_contributions: i32,
    pub account_created_at: Option<DateTime<Utc>>,
    pub last_activity_at: Option<DateTime<Utc>>,
    
    // ZK integration
    pub zk_proof_hash: Option<String>,
    
    // Timestamps
    pub cid: Option<Id>,
    pub ctime: DateTime<Utc>,
    pub mid: Option<Id>,
    pub mtime: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperForCreate {
    pub github_username: String,
    pub github_user_id: i64,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub coding_reputation_score: Option<Decimal>,
    pub security_awareness_score: Option<Decimal>,
    pub community_trust_score: Option<Decimal>,
    pub total_contributions: Option<i32>,
    pub account_created_at: Option<DateTime<Utc>>,
    pub zk_proof_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperForUpdate {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub coding_reputation_score: Option<Decimal>,
    pub security_awareness_score: Option<Decimal>,
    pub community_trust_score: Option<Decimal>,
    pub total_contributions: Option<i32>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub zk_proof_hash: Option<String>,
}

// ------------------------------------------------------------------------------------------------
// GitHub Repository Management
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GitHubRepository {
    pub id: Id,
    
    // GitHub identity
    pub github_repo_id: i64,
    pub owner_username: String,
    pub repo_name: String,
    pub full_name: String,
    pub description: Option<String>,
    
    // Repository metadata
    pub primary_language: Option<String>,
    pub is_private: bool,
    pub star_count: i32,
    pub fork_count: i32,
    
    // Security analysis
    pub security_score: Option<Decimal>,
    pub last_analyzed_at: Option<DateTime<Utc>>,
    
    // Monitoring configuration
    pub webhook_secret: Option<String>,
    pub monitoring_enabled: bool,
    
    // Timestamps
    pub cid: Option<Id>,
    pub ctime: DateTime<Utc>,
    pub mid: Option<Id>,
    pub mtime: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepositoryForCreate {
    pub github_repo_id: i64,
    pub owner_username: String,
    pub repo_name: String,
    pub description: Option<String>,
    pub primary_language: Option<String>,
    pub is_private: bool,
    pub star_count: Option<i32>,
    pub fork_count: Option<i32>,
    pub webhook_secret: Option<String>,
    pub monitoring_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepositoryForUpdate {
    pub description: Option<String>,
    pub primary_language: Option<String>,
    pub is_private: Option<bool>,
    pub star_count: Option<i32>,
    pub fork_count: Option<i32>,
    pub security_score: Option<Decimal>,
    pub last_analyzed_at: Option<DateTime<Utc>>,
    pub webhook_secret: Option<String>,
    pub monitoring_enabled: Option<bool>,
}

// ------------------------------------------------------------------------------------------------
// Code Analysis Results
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CodeAnalysisResult {
    pub id: Id,
    
    // Relationships
    pub repository_id: Id,
    
    // Git information
    pub commit_sha: String,
    
    // Analysis metadata
    pub analysis_type: AnalysisType,
    pub security_score: Decimal,
    pub quality_score: Decimal,
    
    // Issue counts
    pub issues_found: i32,
    pub critical_issues: i32,
    
    // Performance metrics
    pub analysis_duration_ms: i32,
    pub analyzer_version: String,
    
    // Raw analysis data
    pub raw_results: JsonValue,
    
    // Timestamps
    pub ctime: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "analysis_type_enum")]
pub enum AnalysisType {
    #[sqlx(rename = "static_analysis")]
    StaticAnalysis,
    #[sqlx(rename = "llm_review")]
    LlmReview,
    #[sqlx(rename = "dependency_check")]
    DependencyCheck,
}

impl Default for AnalysisType {
    fn default() -> Self {
        Self::StaticAnalysis
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisResultForCreate {
    pub repository_id: Id,
    pub commit_sha: String,
    pub analysis_type: AnalysisType,
    pub security_score: Decimal,
    pub quality_score: Decimal,
    pub issues_found: i32,
    pub critical_issues: i32,
    pub analysis_duration_ms: i32,
    pub analyzer_version: String,
    pub raw_results: JsonValue,
}

// ------------------------------------------------------------------------------------------------
// Security Vulnerabilities
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SecurityVulnerability {
    pub id: Id,
    
    // Relationships
    pub repository_id: Id,
    pub analysis_result_id: Id,
    
    // Vulnerability classification
    pub vulnerability_type: VulnerabilityType,
    pub severity: Severity,
    pub confidence_score: Decimal,
    
    // Location information
    pub file_path: String,
    pub line_number: Option<i32>,
    pub code_snippet: Option<String>,
    
    // Vulnerability details
    pub description: String,
    pub recommendation: String,
    pub cve_id: Option<String>,
    
    // Status tracking
    pub is_false_positive: bool,
    pub fixed_at: Option<DateTime<Utc>>,
    
    // Timestamps
    pub ctime: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "vulnerability_type_enum")]
pub enum VulnerabilityType {
    #[sqlx(rename = "reentrancy")]
    Reentrancy,
    #[sqlx(rename = "overflow")]
    Overflow,
    #[sqlx(rename = "access_control")]
    AccessControl,
    #[sqlx(rename = "other")]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "severity_enum")]
pub enum Severity {
    #[sqlx(rename = "critical")]
    Critical,
    #[sqlx(rename = "high")]
    High,
    #[sqlx(rename = "medium")]
    Medium,
    #[sqlx(rename = "low")]
    Low,
}

impl Default for VulnerabilityType {
    fn default() -> Self {
        Self::Other
    }
}

impl Default for Severity {
    fn default() -> Self {
        Self::Medium
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityVulnerabilityForCreate {
    pub repository_id: Id,
    pub analysis_result_id: Id,
    pub vulnerability_type: VulnerabilityType,
    pub severity: Severity,
    pub confidence_score: Decimal,
    pub file_path: String,
    pub line_number: Option<i32>,
    pub code_snippet: Option<String>,
    pub description: String,
    pub recommendation: String,
    pub cve_id: Option<String>,
    pub is_false_positive: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityVulnerabilityForUpdate {
    pub is_false_positive: Option<bool>,
    pub fixed_at: Option<DateTime<Utc>>,
}

// ------------------------------------------------------------------------------------------------
// Patch Proposals
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PatchProposal {
    pub id: Id,
    
    // Relationships
    pub vulnerability_id: Id,
    pub repository_id: Id,
    pub proposed_by_developer_id: Option<Id>,
    
    // Patch metadata
    pub patch_type: PatchType,
    pub title: String,
    pub description: String,
    
    // Patch content
    pub diff_content: String,
    
    // GitHub integration
    pub github_pr_number: Option<i32>,
    
    // Status and approval
    pub status: PatchStatus,
    pub community_votes_for: i32,
    pub community_votes_against: i32,
    pub approval_threshold_met: bool,
    
    // Application tracking
    pub applied_at: Option<DateTime<Utc>>,
    
    // Timestamps
    pub cid: Option<Id>,
    pub ctime: DateTime<Utc>,
    pub mid: Option<Id>,
    pub mtime: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "patch_type_enum")]
pub enum PatchType {
    #[sqlx(rename = "ai_generated")]
    AiGenerated,
    #[sqlx(rename = "community")]
    Community,
    #[sqlx(rename = "automated")]
    Automated,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "patch_status_enum")]
pub enum PatchStatus {
    #[sqlx(rename = "proposed")]
    Proposed,
    #[sqlx(rename = "under_review")]
    UnderReview,
    #[sqlx(rename = "approved")]
    Approved,
    #[sqlx(rename = "rejected")]
    Rejected,
    #[sqlx(rename = "applied")]
    Applied,
}

impl Default for PatchType {
    fn default() -> Self {
        Self::Community
    }
}

impl Default for PatchStatus {
    fn default() -> Self {
        Self::Proposed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchProposalForCreate {
    pub vulnerability_id: Id,
    pub repository_id: Id,
    pub proposed_by_developer_id: Option<Id>,
    pub patch_type: PatchType,
    pub title: String,
    pub description: String,
    pub diff_content: String,
    pub github_pr_number: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchProposalForUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub diff_content: Option<String>,
    pub github_pr_number: Option<i32>,
    pub status: Option<PatchStatus>,
    pub community_votes_for: Option<i32>,
    pub community_votes_against: Option<i32>,
    pub approval_threshold_met: Option<bool>,
    pub applied_at: Option<DateTime<Utc>>,
}

// ================================================================================================
// Filter Structs for Queries
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperFilter {
    pub github_username: Option<String>,
    pub email: Option<String>,
    pub min_coding_reputation_score: Option<Decimal>,
    pub min_security_awareness_score: Option<Decimal>,
    pub min_community_trust_score: Option<Decimal>,
    pub min_total_contributions: Option<i32>,
    pub has_zk_proof: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepositoryFilter {
    pub owner_username: Option<String>,
    pub repo_name: Option<String>,
    pub primary_language: Option<String>,
    pub is_private: Option<bool>,
    pub min_star_count: Option<i32>,
    pub min_security_score: Option<Decimal>,
    pub monitoring_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisResultFilter {
    pub repository_id: Option<Id>,
    pub analysis_type: Option<AnalysisType>,
    pub min_security_score: Option<Decimal>,
    pub min_quality_score: Option<Decimal>,
    pub min_issues_found: Option<i32>,
    pub has_critical_issues: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityVulnerabilityFilter {
    pub repository_id: Option<Id>,
    pub vulnerability_type: Option<VulnerabilityType>,
    pub severity: Option<Severity>,
    pub min_confidence_score: Option<Decimal>,
    pub is_false_positive: Option<bool>,
    pub is_fixed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchProposalFilter {
    pub repository_id: Option<Id>,
    pub vulnerability_id: Option<Id>,
    pub proposed_by_developer_id: Option<Id>,
    pub patch_type: Option<PatchType>,
    pub status: Option<PatchStatus>,
    pub approval_threshold_met: Option<bool>,
    pub has_github_pr: Option<bool>,
}

// ================================================================================================
// String Conversion Implementations
// ================================================================================================

impl std::str::FromStr for AnalysisType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "static_analysis" => Ok(AnalysisType::StaticAnalysis),
            "llm_review" => Ok(AnalysisType::LlmReview),
            "dependency_check" => Ok(AnalysisType::DependencyCheck),
            _ => Err(format!("Invalid analysis type: {}", s)),
        }
    }
}

impl std::str::FromStr for VulnerabilityType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "reentrancy" => Ok(VulnerabilityType::Reentrancy),
            "overflow" => Ok(VulnerabilityType::Overflow),
            "access_control" => Ok(VulnerabilityType::AccessControl),
            "other" => Ok(VulnerabilityType::Other),
            _ => Err(format!("Invalid vulnerability type: {}", s)),
        }
    }
}

impl std::str::FromStr for Severity {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "critical" => Ok(Severity::Critical),
            "high" => Ok(Severity::High),
            "medium" => Ok(Severity::Medium),
            "low" => Ok(Severity::Low),
            _ => Err(format!("Invalid severity: {}", s)),
        }
    }
}

impl std::str::FromStr for PatchType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ai_generated" => Ok(PatchType::AiGenerated),
            "community" => Ok(PatchType::Community),
            "automated" => Ok(PatchType::Automated),
            _ => Err(format!("Invalid patch type: {}", s)),
        }
    }
}

impl std::str::FromStr for PatchStatus {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "proposed" => Ok(PatchStatus::Proposed),
            "under_review" => Ok(PatchStatus::UnderReview),
            "approved" => Ok(PatchStatus::Approved),
            "rejected" => Ok(PatchStatus::Rejected),
            "applied" => Ok(PatchStatus::Applied),
            _ => Err(format!("Invalid patch status: {}", s)),
        }
    }
}