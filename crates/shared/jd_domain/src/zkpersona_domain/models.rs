use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use std::collections::HashMap;

use crate::Id;

// ================================================================================================
// Core User Management
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Id,
    pub wallet_address: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub status: UserStatus,
    pub privacy_settings: JsonValue,
    
    // Timestamps
    pub cid: Option<Id>,
    pub ctime: DateTime<Utc>,
    pub mid: Option<Id>,
    pub mtime: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum UserStatus {
    #[sqlx(rename = "active")]
    Active,
    #[sqlx(rename = "inactive")]
    Inactive,
    #[sqlx(rename = "suspended")]
    Suspended,
    #[sqlx(rename = "deleted")]
    Deleted,
}

impl Default for UserStatus {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    pub data_sharing: bool,
    pub public_profile: bool,
    pub analytics_opt_in: Option<bool>,
    pub marketing_opt_in: Option<bool>,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            data_sharing: false,
            public_profile: false,
            analytics_opt_in: None,
            marketing_opt_in: None,
        }
    }
}

// ================================================================================================
// Behavior Session Management
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BehaviorSession {
    pub id: Id,
    pub user_id: Option<Id>,
    pub session_token: String,
    pub session_type: SessionType,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i32>,
    pub metadata: JsonValue,
    pub status: SessionStatus,
    
    // Timestamps
    pub cid: Option<Id>,
    pub ctime: DateTime<Utc>,
    pub mid: Option<Id>,
    pub mtime: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum SessionType {
    #[sqlx(rename = "web")]
    Web,
    #[sqlx(rename = "mobile")]
    Mobile,
    #[sqlx(rename = "api")]
    Api,
    #[sqlx(rename = "blockchain")]
    Blockchain,
}

impl Default for SessionType {
    fn default() -> Self {
        Self::Web
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum SessionStatus {
    #[sqlx(rename = "active")]
    Active,
    #[sqlx(rename = "completed")]
    Completed,
    #[sqlx(rename = "expired")]
    Expired,
    #[sqlx(rename = "terminated")]
    Terminated,
}

impl Default for SessionStatus {
    fn default() -> Self {
        Self::Active
    }
}

// ================================================================================================
// Behavior Input Data
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BehaviorInput {
    pub id: Id,
    pub user_id: Option<Id>,
    pub behavior_session_id: Option<Id>,
    pub session_id: Option<String>,
    pub input_data: JsonValue,
    pub input_type: InputType,
    pub source: InputSource,
    pub timestamp: DateTime<Utc>,
    pub processed: bool,
    
    // Timestamps
    pub cid: Option<Id>,
    pub ctime: DateTime<Utc>,
    pub mid: Option<Id>,
    pub mtime: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Hash)]
#[sqlx(type_name = "text")]
pub enum InputType {
    #[sqlx(rename = "transaction")]
    Transaction,
    #[sqlx(rename = "interaction")]
    Interaction,
    #[sqlx(rename = "defi")]
    Defi,
    #[sqlx(rename = "nft")]
    Nft,
    #[sqlx(rename = "dao")]
    Dao,
    #[sqlx(rename = "social")]
    Social,
    #[sqlx(rename = "general")]
    General,
}

impl Default for InputType {
    fn default() -> Self {
        Self::General
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Hash)]
#[sqlx(type_name = "text")]
pub enum InputSource {
    #[sqlx(rename = "web")]
    Web,
    #[sqlx(rename = "mobile")]
    Mobile,
    #[sqlx(rename = "api")]
    Api,
    #[sqlx(rename = "blockchain")]
    Blockchain,
    #[sqlx(rename = "oracle")]
    Oracle,
}

impl Default for InputSource {
    fn default() -> Self {
        Self::Web
    }
}

// Parse enum from string for InputType
impl std::str::FromStr for InputType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "transaction" => Ok(InputType::Transaction),
            "interaction" => Ok(InputType::Interaction),
            "defi" => Ok(InputType::Defi),
            "nft" => Ok(InputType::Nft),
            "dao" => Ok(InputType::Dao),
            "social" => Ok(InputType::Social),
            "general" => Ok(InputType::General),
            _ => Err(format!("Invalid input type: {}", s)),
        }
    }
}

// Parse enum from string for InputSource
impl std::str::FromStr for InputSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "web" => Ok(InputSource::Web),
            "mobile" => Ok(InputSource::Mobile),
            "api" => Ok(InputSource::Api),
            "blockchain" => Ok(InputSource::Blockchain),
            "oracle" => Ok(InputSource::Oracle),
            _ => Err(format!("Invalid input source: {}", s)),
        }
    }
}

// ================================================================================================
// AI Scoring Results
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScoringResult {
    pub id: Id,
    pub behavior_input_id: Id,
    pub score: rust_decimal::Decimal,
    pub model_version: String,
    pub timestamp: DateTime<Utc>,
}

// ================================================================================================
// ZK Proof Management
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ZkProof {
    pub id: Id,
    pub user_id: Option<Id>,
    pub behavior_input_id: Option<Id>,
    pub scoring_result_id: Option<Id>,
    
    // Proof data
    pub proof_data: JsonValue,
    pub verification_key: JsonValue,
    pub public_signals: JsonValue,
    
    // Proof metadata
    pub proof_type: ProofType,
    pub protocol: ProofProtocol,
    pub circuit_version: String,
    
    // Verification status
    pub verification_status: VerificationStatus,
    pub verified_at: Option<DateTime<Utc>>,
    pub verifier_id: Option<Id>,
    
    // Blockchain integration
    pub blockchain_network: Option<String>,
    pub transaction_hash: Option<String>,
    pub block_number: Option<i64>,
    pub gas_used: Option<i32>,
    
    // Additional metadata
    pub metadata: JsonValue,
    pub expires_at: Option<DateTime<Utc>>,
    
    // Timestamps
    pub cid: Option<Id>,
    pub ctime: DateTime<Utc>,
    pub mid: Option<Id>,
    pub mtime: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum ProofType {
    #[sqlx(rename = "zkml")]
    Zkml,
    #[sqlx(rename = "zk-snark")]
    ZkSnark,
    #[sqlx(rename = "zk-stark")]
    ZkStark,
    #[sqlx(rename = "plonk")]
    Plonk,
    #[sqlx(rename = "custom")]
    Custom,
}

impl Default for ProofType {
    fn default() -> Self {
        Self::Zkml
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum ProofProtocol {
    #[sqlx(rename = "groth16")]
    Groth16,
    #[sqlx(rename = "plonk")]
    Plonk,
    #[sqlx(rename = "marlin")]
    Marlin,
    #[sqlx(rename = "sonic")]
    Sonic,
    #[sqlx(rename = "bulletproofs")]
    Bulletproofs,
}

impl Default for ProofProtocol {
    fn default() -> Self {
        Self::Groth16
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum VerificationStatus {
    #[sqlx(rename = "pending")]
    Pending,
    #[sqlx(rename = "verified")]
    Verified,
    #[sqlx(rename = "failed")]
    Failed,
    #[sqlx(rename = "expired")]
    Expired,
}

impl Default for VerificationStatus {
    fn default() -> Self {
        Self::Pending
    }
}

// ================================================================================================
// Reputation Management
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReputationRecord {
    pub id: Id,
    pub user_id: Id,
    pub zk_proof_id: Option<Id>,
    pub scoring_result_id: Option<Id>,
    
    // Reputation data
    pub reputation_score: rust_decimal::Decimal,
    pub previous_score: Option<rust_decimal::Decimal>,
    pub score_change: Option<rust_decimal::Decimal>,
    pub confidence_level: rust_decimal::Decimal,
    
    // Scoring context
    pub scoring_category: ScoringCategory,
    pub scoring_period: ScoringPeriod,
    pub weight: rust_decimal::Decimal,
    
    // Reputation metadata
    pub contributing_factors: JsonValue,
    pub reputation_badges: JsonValue,
    pub peer_validations: i32,
    
    // Time-based data
    pub effective_from: DateTime<Utc>,
    pub effective_until: Option<DateTime<Utc>>,
    pub calculation_method: String,
    
    // Status and flags
    pub status: ReputationStatus,
    pub is_verified: bool,
    pub is_public: bool,
    
    // Timestamps
    pub cid: Option<Id>,
    pub ctime: DateTime<Utc>,
    pub mid: Option<Id>,
    pub mtime: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Hash)]
#[sqlx(type_name = "text")]
pub enum ScoringCategory {
    #[sqlx(rename = "overall")]
    Overall,
    #[sqlx(rename = "defi")]
    Defi,
    #[sqlx(rename = "nft")]
    Nft,
    #[sqlx(rename = "dao")]
    Dao,
    #[sqlx(rename = "social")]
    Social,
    #[sqlx(rename = "trading")]
    Trading,
    #[sqlx(rename = "staking")]
    Staking,
    #[sqlx(rename = "lending")]
    Lending,
}

impl Default for ScoringCategory {
    fn default() -> Self {
        Self::Overall
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum ScoringPeriod {
    #[sqlx(rename = "current")]
    Current,
    #[sqlx(rename = "daily")]
    Daily,
    #[sqlx(rename = "weekly")]
    Weekly,
    #[sqlx(rename = "monthly")]
    Monthly,
    #[sqlx(rename = "quarterly")]
    Quarterly,
    #[sqlx(rename = "yearly")]
    Yearly,
    #[sqlx(rename = "all_time")]
    AllTime,
}

impl Default for ScoringPeriod {
    fn default() -> Self {
        Self::Current
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum ReputationStatus {
    #[sqlx(rename = "active")]
    Active,
    #[sqlx(rename = "archived")]
    Archived,
    #[sqlx(rename = "disputed")]
    Disputed,
    #[sqlx(rename = "invalidated")]
    Invalidated,
}

impl Default for ReputationStatus {
    fn default() -> Self {
        Self::Active
    }
}

// ================================================================================================
// Data Transfer Objects (DTOs)
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub wallet_address: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub privacy_settings: Option<PrivacySettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub username: Option<String>,
    pub status: Option<UserStatus>,
    pub privacy_settings: Option<PrivacySettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBehaviorSessionRequest {
    pub user_id: Option<Id>,
    pub session_type: Option<SessionType>,
    pub metadata: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBehaviorInputRequest {
    pub user_id: Option<Id>,
    pub behavior_session_id: Option<Id>,
    pub session_id: Option<String>,
    pub input_data: JsonValue,
    pub input_type: Option<InputType>,
    pub source: Option<InputSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateZkProofRequest {
    pub user_id: Option<Id>,
    pub behavior_input_id: Option<Id>,
    pub scoring_result_id: Option<Id>,
    pub proof_data: JsonValue,
    pub verification_key: JsonValue,
    pub public_signals: JsonValue,
    pub proof_type: Option<ProofType>,
    pub protocol: Option<ProofProtocol>,
    pub circuit_version: String,
    pub blockchain_network: Option<String>,
    pub metadata: Option<JsonValue>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReputationRecordRequest {
    pub user_id: Id,
    pub zk_proof_id: Option<Id>,
    pub scoring_result_id: Option<Id>,
    pub reputation_score: rust_decimal::Decimal,
    pub confidence_level: Option<rust_decimal::Decimal>,
    pub scoring_category: Option<ScoringCategory>,
    pub scoring_period: Option<ScoringPeriod>,
    pub weight: Option<rust_decimal::Decimal>,
    pub contributing_factors: Option<JsonValue>,
    pub reputation_badges: Option<JsonValue>,
    pub effective_until: Option<DateTime<Utc>>,
    pub calculation_method: Option<String>,
    pub is_public: Option<bool>,
}

// ================================================================================================
// Query Filters and Pagination
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorInputFilter {
    pub user_id: Option<Id>,
    pub behavior_session_id: Option<Id>,
    pub session_id: Option<String>,
    pub input_type: Option<InputType>,
    pub source: Option<InputSource>,
    pub processed: Option<bool>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

impl Default for BehaviorInputFilter {
    fn default() -> Self {
        Self {
            user_id: None,
            behavior_session_id: None,
            session_id: None,
            input_type: None,
            source: None,
            processed: None,
            start_date: None,
            end_date: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationFilter {
    pub user_id: Option<Id>,
    pub scoring_category: Option<ScoringCategory>,
    pub scoring_period: Option<ScoringPeriod>,
    pub status: Option<ReputationStatus>,
    pub is_verified: Option<bool>,
    pub is_public: Option<bool>,
    pub min_score: Option<rust_decimal::Decimal>,
    pub max_score: Option<rust_decimal::Decimal>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProofFilter {
    pub user_id: Option<Id>,
    pub proof_type: Option<ProofType>,
    pub verification_status: Option<VerificationStatus>,
    pub blockchain_network: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            limit: Some(20),
            offset: Some(0),
            sort_by: None,
            sort_order: Some(SortOrder::Desc),
        }
    }
}

// ================================================================================================
// Aggregation and Analytics
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserReputationSummary {
    pub user_id: Id,
    pub overall_score: rust_decimal::Decimal,
    pub category_scores: HashMap<ScoringCategory, rust_decimal::Decimal>,
    pub total_proofs: i64,
    pub verified_proofs: i64,
    pub reputation_trend: Vec<ReputationTrendPoint>,
    pub badges: Vec<String>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationTrendPoint {
    pub date: DateTime<Utc>,
    pub score: rust_decimal::Decimal,
    pub category: ScoringCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorAnalyticsSummary {
    pub total_inputs: i64,
    pub processed_inputs: i64,
    pub unique_sessions: i64,
    pub input_types_distribution: HashMap<InputType, i64>,
    pub source_distribution: HashMap<InputSource, i64>,
    pub date_range: (DateTime<Utc>, DateTime<Utc>),
}

// ================================================================================================
// Error Types
// ================================================================================================

// ================================================================================================
// Data Transfer Objects (DTOs) for REST API
// ================================================================================================

/// DTO for creating a new behavior input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBehaviorInputForRest {
    pub user_id: Option<Id>,
    pub behavior_session_id: Option<Id>,
    pub session_id: Option<String>,
    pub input_data: JsonValue,
    pub input_type: InputType,
    pub source: InputSource,
    pub processed: Option<bool>,
}

/// DTO for creating a new scoring result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScoringResultForRest {
    pub behavior_input_id: Id,
    pub score: rust_decimal::Decimal,
    pub model_version: String,
    pub confidence_level: Option<rust_decimal::Decimal>,
    pub metadata: Option<JsonValue>,
}

/// DTO for creating a new ZK proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateZkProofForRest {
    pub user_id: Option<Id>,
    pub behavior_input_id: Option<Id>,
    pub scoring_result_id: Option<Id>,
    pub proof_data: JsonValue,
    pub verification_key: JsonValue,
    pub public_signals: Option<JsonValue>,
    pub proof_type: ProofType,
    pub protocol: ProofProtocol,
    pub circuit_version: String,
    pub verification_status: Option<VerificationStatus>,
}

// ================================================================================================
// Error Types
// ================================================================================================

#[derive(Debug, thiserror::Error)]
pub enum ZkPersonaError {
    #[error("User not found: {user_id}")]
    UserNotFound { user_id: String },
    
    #[error("Behavior session not found: {session_id}")]
    SessionNotFound { session_id: String },
    
    #[error("ZK proof verification failed: {reason}")]
    ProofVerificationFailed { reason: String },
    
    #[error("Invalid reputation score: {score}. Must be between 0 and 100")]
    InvalidReputationScore { score: rust_decimal::Decimal },
    
    #[error("Expired proof: {proof_id}")]
    ExpiredProof { proof_id: String },
    
    #[error("Insufficient permissions for user: {user_id}")]
    InsufficientPermissions { user_id: String },
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Database transaction error: {0}")]
    DatabaseTransaction(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type ZkPersonaResult<T> = Result<T, ZkPersonaError>;