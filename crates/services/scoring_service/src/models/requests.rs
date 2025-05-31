use serde::{Deserialize, Serialize};
use jd_domain::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringRequest {
    pub behavior_input_id: Id,
    pub model_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringQueryRequest {
    pub behavior_input_id: Option<Id>,
    pub model_version: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}