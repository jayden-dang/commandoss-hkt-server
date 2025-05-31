use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use jd_domain::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringResponse {
    pub id: Id,
    pub behavior_input_id: Id,
    pub score: f64,
    pub model_version: String,
    pub timestamp: OffsetDateTime,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringListResponse {
    pub items: Vec<ScoringResponse>,
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
}