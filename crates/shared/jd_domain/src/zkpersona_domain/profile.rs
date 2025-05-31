use crate::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorInput {
  pub session_id: Option<String>,
  pub input_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringResult {
  pub behavior_input_id: Id,
  pub score: f64,
  pub model_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProof {
  pub scoring_result_id: Id,
  pub proof_data: Vec<u8>,
  pub verification_key: Vec<u8>,
  pub verified: bool,
  pub blockchain_tx_hash: Option<String>,
}
