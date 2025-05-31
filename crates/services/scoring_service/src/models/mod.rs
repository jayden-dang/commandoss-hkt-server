pub mod requests;
pub mod responses;

use modql::field::Fields;
use modql::filter::{FilterNodes, OpValsString, OpValsValue};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use jd_domain::Id;

// Database model structures for the REST pattern
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
pub struct ScoringResultRecord {
    pub id: uuid::Uuid,
    pub behavior_input_id: uuid::Uuid,
    pub score: f64,
    pub model_version: String,
    pub timestamp: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, Fields)]
pub struct ScoringResultForCreate {
    pub behavior_input_id: uuid::Uuid,
    pub score: f64,
    pub model_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Fields)]
pub struct ScoringResultForUpdate {
    pub score: Option<f64>,
    pub model_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, FilterNodes)]
pub struct ScoringResultFilter {
    pub behavior_input_id: Option<OpValsValue>,
    pub model_version: Option<OpValsString>,
}

// Conversion implementations
impl From<ScoringResultRecord> for responses::ScoringResponse {
    fn from(record: ScoringResultRecord) -> Self {
        Self {
            id: Id::new(record.id.to_string()),
            behavior_input_id: Id::new(record.behavior_input_id.to_string()),
            score: record.score,
            model_version: record.model_version,
            timestamp: record.timestamp,
            metadata: None,
        }
    }
}