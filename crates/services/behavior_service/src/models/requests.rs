use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct BehaviorInputRequest {
    #[validate(length(min = 1, max = 100))]
    pub session_id: Option<String>,
    
    pub input_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorQueryRequest {
    pub session_id: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}