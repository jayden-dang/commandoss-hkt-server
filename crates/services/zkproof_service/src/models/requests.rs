use serde::{Deserialize, Serialize};
use jd_domain::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateProofRequest {
    pub behavior_input: serde_json::Value,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyProofRequest {
    pub proof_data: String,
    pub verification_key: String,
    pub public_signals: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofQueryRequest {
    pub scoring_result_id: Option<Id>,
    pub verified: Option<bool>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}