use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use jd_domain::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateProofResponse {
    pub proof_id: Id,
    pub behavior_input_id: Id,
    pub scoring_result_id: Id,
    pub score: f64,
    pub proof_data: String,
    pub verification_key: String,
    pub public_signals: serde_json::Value,
    pub timestamp: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyProofResponse {
    pub valid: bool,
    pub proof_id: Option<Id>,
    pub timestamp: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProofResponse {
    pub id: Id,
    pub scoring_result_id: Id,
    pub proof_data: String,
    pub verification_key: String,
    pub verified: bool,
    pub blockchain_tx_hash: Option<String>,
    pub timestamp: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProofListResponse {
    pub items: Vec<ZkProofResponse>,
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
}