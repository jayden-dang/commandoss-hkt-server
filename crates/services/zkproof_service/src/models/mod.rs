pub mod requests;
pub mod responses;

use modql::field::Fields;
use modql::filter::{FilterNodes, OpValsValue};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use jd_domain::Id;

// Database model structures for the REST pattern
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
pub struct ZkProofRecord {
    pub id: uuid::Uuid,
    pub scoring_result_id: uuid::Uuid,
    pub proof_data: Vec<u8>, // Binary data stored as bytea
    pub verification_key: Vec<u8>, // Binary data stored as bytea
    pub verified: bool,
    pub blockchain_tx_hash: Option<String>,
    pub timestamp: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, Fields)]
pub struct ZkProofForCreate {
    pub scoring_result_id: uuid::Uuid,
    pub proof_data: Vec<u8>,
    pub verification_key: Vec<u8>,
    pub verified: Option<bool>,
    pub blockchain_tx_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Fields)]
pub struct ZkProofForUpdate {
    pub verified: Option<bool>,
    pub blockchain_tx_hash: Option<String>,
}

#[derive(Debug, Clone, Deserialize, FilterNodes)]
pub struct ZkProofFilter {
    pub scoring_result_id: Option<OpValsValue>,
    pub verified: Option<OpValsValue>,
}

// Conversion implementations
impl From<ZkProofRecord> for responses::ZkProofResponse {
    fn from(record: ZkProofRecord) -> Self {
        Self {
            id: Id::new(record.id.to_string()),
            scoring_result_id: Id::new(record.scoring_result_id.to_string()),
            proof_data: String::from_utf8_lossy(&record.proof_data).to_string(),
            verification_key: String::from_utf8_lossy(&record.verification_key).to_string(),
            verified: record.verified,
            blockchain_tx_hash: record.blockchain_tx_hash,
            timestamp: record.timestamp,
        }
    }
}