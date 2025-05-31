use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use jd_domain::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorInputResponse {
    pub id: Id,
    pub session_id: Option<String>,
    pub input_data: serde_json::Value,
    pub timestamp: OffsetDateTime,
    pub processed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorListResponse {
    pub items: Vec<BehaviorInputResponse>,
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
}