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
pub struct BehaviorInputRecord {
    pub id: uuid::Uuid,
    pub session_id: Option<String>,
    pub input_data: String, // JSON stored as string
    pub timestamp: OffsetDateTime,
    pub processed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Fields)]
pub struct BehaviorInputForCreate {
    pub session_id: Option<String>,
    pub input_data: String, // JSON stored as string
    pub processed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Fields)]
pub struct BehaviorInputForUpdate {
    pub processed: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, FilterNodes)]
pub struct BehaviorInputFilter {
    pub session_id: Option<OpValsString>,
    pub processed: Option<OpValsValue>,
}

// Conversion implementations
impl From<BehaviorInputRecord> for responses::BehaviorInputResponse {
    fn from(record: BehaviorInputRecord) -> Self {
        Self {
            id: Id::new(record.id.to_string()),
            session_id: record.session_id,
            input_data: serde_json::from_str(&record.input_data).unwrap_or_default(),
            timestamp: record.timestamp,
            processed: record.processed,
        }
    }
}