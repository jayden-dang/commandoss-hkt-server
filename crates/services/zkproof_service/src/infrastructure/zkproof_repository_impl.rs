use async_trait::async_trait;
use jd_core::AppState;
use jd_domain::Id;
use jd_domain::zkpersona_domain::profile::ZkProof;
use jd_utils::time::now_utc;
use sqlx::Row;

use crate::domain::zkproof_repository_trait::ZkProofRepository;
use crate::models::{
    requests::ProofQueryRequest,
    responses::{ZkProofResponse, ZkProofListResponse},
};
use crate::Result;

#[derive(Clone)]
pub struct ZkProofRepositoryImpl {
    app_state: AppState,
}

impl ZkProofRepositoryImpl {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }
}

#[async_trait]
impl ZkProofRepository for ZkProofRepositoryImpl {
    async fn create_zkproof(&self, proof: ZkProof) -> Result<ZkProofResponse> {
        let id = Id::generate();
        let timestamp = now_utc();
        
        let query = r#"
            INSERT INTO zkml_proofs (id, scoring_result_id, proof_data, verification_key, verified, blockchain_tx_hash, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, scoring_result_id, proof_data, verification_key, verified, blockchain_tx_hash, timestamp
        "#;
        
        let row = sqlx::query(query)
            .bind(id.value())
            .bind(proof.scoring_result_id.value())
            .bind(&proof.proof_data)
            .bind(&proof.verification_key)
            .bind(proof.verified)
            .bind(&proof.blockchain_tx_hash)
            .bind(timestamp)
            .fetch_one(self.app_state.mm.dbx().db())
            .await?;
            
        Ok(ZkProofResponse {
            id: Id::new(row.get::<uuid::Uuid, _>("id").to_string()),
            scoring_result_id: Id::new(row.get::<uuid::Uuid, _>("scoring_result_id").to_string()),
            proof_data: String::from_utf8_lossy(&row.get::<Vec<u8>, _>("proof_data")).to_string(),
            verification_key: String::from_utf8_lossy(&row.get::<Vec<u8>, _>("verification_key")).to_string(),
            verified: row.get("verified"),
            blockchain_tx_hash: row.get("blockchain_tx_hash"),
            timestamp: row.get("timestamp"),
        })
    }

    async fn get_zkproof(&self, id: Id) -> Result<Option<ZkProofResponse>> {
        let query = r#"
            SELECT id, scoring_result_id, proof_data, verification_key, verified, blockchain_tx_hash, timestamp
            FROM zkml_proofs
            WHERE id = $1
        "#;
        
        let row = sqlx::query(query)
            .bind(id.value())
            .fetch_optional(self.app_state.mm.dbx().db())
            .await?;
            
        Ok(row.map(|r| ZkProofResponse {
            id: Id::new(r.get::<uuid::Uuid, _>("id").to_string()),
            scoring_result_id: Id::new(r.get::<uuid::Uuid, _>("scoring_result_id").to_string()),
            proof_data: String::from_utf8_lossy(&r.get::<Vec<u8>, _>("proof_data")).to_string(),
            verification_key: String::from_utf8_lossy(&r.get::<Vec<u8>, _>("verification_key")).to_string(),
            verified: r.get("verified"),
            blockchain_tx_hash: r.get("blockchain_tx_hash"),
            timestamp: r.get("timestamp"),
        }))
    }

    async fn get_zkproof_by_scoring_id(&self, scoring_result_id: Id) -> Result<Option<ZkProofResponse>> {
        let query = r#"
            SELECT id, scoring_result_id, proof_data, verification_key, verified, blockchain_tx_hash, timestamp
            FROM zkml_proofs
            WHERE scoring_result_id = $1
            ORDER BY timestamp DESC
            LIMIT 1
        "#;
        
        let row = sqlx::query(query)
            .bind(scoring_result_id.value())
            .fetch_optional(self.app_state.mm.dbx().db())
            .await?;
            
        Ok(row.map(|r| ZkProofResponse {
            id: Id::new(r.get::<uuid::Uuid, _>("id").to_string()),
            scoring_result_id: Id::new(r.get::<uuid::Uuid, _>("scoring_result_id").to_string()),
            proof_data: String::from_utf8_lossy(&r.get::<Vec<u8>, _>("proof_data")).to_string(),
            verification_key: String::from_utf8_lossy(&r.get::<Vec<u8>, _>("verification_key")).to_string(),
            verified: r.get("verified"),
            blockchain_tx_hash: r.get("blockchain_tx_hash"),
            timestamp: r.get("timestamp"),
        }))
    }

    async fn list_zkproofs(&self, query_req: ProofQueryRequest) -> Result<ZkProofListResponse> {
        let limit = query_req.limit.unwrap_or(50).min(100);
        let offset = query_req.offset.unwrap_or(0);
        
        // Build query and execute based on filters
        let (rows, total) = match (&query_req.scoring_result_id, query_req.verified) {
            (Some(scoring_result_id), Some(verified)) => {
                // Both filters
                let query = r#"
                    SELECT id, scoring_result_id, proof_data, verification_key, verified, blockchain_tx_hash, timestamp
                    FROM zkml_proofs
                    WHERE scoring_result_id = $1 AND verified = $2
                    ORDER BY timestamp DESC
                    LIMIT $3 OFFSET $4
                "#;
                let rows = sqlx::query(query)
                    .bind(scoring_result_id.value())
                    .bind(verified)
                    .bind(limit as i64)
                    .bind(offset as i64)
                    .fetch_all(self.app_state.mm.dbx().db())
                    .await?;

                let count_row = sqlx::query(
                    "SELECT COUNT(*) as count FROM zkml_proofs WHERE scoring_result_id = $1 AND verified = $2"
                )
                .bind(scoring_result_id.value())
                .bind(verified)
                .fetch_one(self.app_state.mm.dbx().db())
                .await?;
                let total: i64 = count_row.get("count");
                (rows, total)
            },
            (Some(scoring_result_id), None) => {
                // Only scoring_result_id filter
                let query = r#"
                    SELECT id, scoring_result_id, proof_data, verification_key, verified, blockchain_tx_hash, timestamp
                    FROM zkml_proofs
                    WHERE scoring_result_id = $1
                    ORDER BY timestamp DESC
                    LIMIT $2 OFFSET $3
                "#;
                let rows = sqlx::query(query)
                    .bind(scoring_result_id.value())
                    .bind(limit as i64)
                    .bind(offset as i64)
                    .fetch_all(self.app_state.mm.dbx().db())
                    .await?;

                let count_row = sqlx::query("SELECT COUNT(*) as count FROM zkml_proofs WHERE scoring_result_id = $1")
                    .bind(scoring_result_id.value())
                    .fetch_one(self.app_state.mm.dbx().db())
                    .await?;
                let total: i64 = count_row.get("count");
                (rows, total)
            },
            (None, Some(verified)) => {
                // Only verified filter
                let query = r#"
                    SELECT id, scoring_result_id, proof_data, verification_key, verified, blockchain_tx_hash, timestamp
                    FROM zkml_proofs
                    WHERE verified = $1
                    ORDER BY timestamp DESC
                    LIMIT $2 OFFSET $3
                "#;
                let rows = sqlx::query(query)
                    .bind(verified)
                    .bind(limit as i64)
                    .bind(offset as i64)
                    .fetch_all(self.app_state.mm.dbx().db())
                    .await?;

                let count_row = sqlx::query("SELECT COUNT(*) as count FROM zkml_proofs WHERE verified = $1")
                    .bind(verified)
                    .fetch_one(self.app_state.mm.dbx().db())
                    .await?;
                let total: i64 = count_row.get("count");
                (rows, total)
            },
            (None, None) => {
                // No filters
                let query = r#"
                    SELECT id, scoring_result_id, proof_data, verification_key, verified, blockchain_tx_hash, timestamp
                    FROM zkml_proofs
                    ORDER BY timestamp DESC
                    LIMIT $1 OFFSET $2
                "#;
                let rows = sqlx::query(query)
                    .bind(limit as i64)
                    .bind(offset as i64)
                    .fetch_all(self.app_state.mm.dbx().db())
                    .await?;

                let count_row = sqlx::query("SELECT COUNT(*) as count FROM zkml_proofs")
                    .fetch_one(self.app_state.mm.dbx().db())
                    .await?;
                let total: i64 = count_row.get("count");
                (rows, total)
            }
        };
        
        let items: Vec<ZkProofResponse> = rows
            .into_iter()
            .map(|r| ZkProofResponse {
                id: Id::new(r.get::<uuid::Uuid, _>("id").to_string()),
                scoring_result_id: Id::new(r.get::<uuid::Uuid, _>("scoring_result_id").to_string()),
                proof_data: String::from_utf8_lossy(&r.get::<Vec<u8>, _>("proof_data")).to_string(),
                verification_key: String::from_utf8_lossy(&r.get::<Vec<u8>, _>("verification_key")).to_string(),
                verified: r.get("verified"),
                blockchain_tx_hash: r.get("blockchain_tx_hash"),
                timestamp: r.get("timestamp"),
            })
            .collect();
        
        Ok(ZkProofListResponse {
            items,
            total: total as u64,
            limit,
            offset,
        })
    }

    async fn mark_as_verified(&self, id: Id) -> Result<()> {
        let query = r#"
            UPDATE zkml_proofs
            SET verified = true
            WHERE id = $1
        "#;
        
        sqlx::query(query)
            .bind(id.value())
            .execute(self.app_state.mm.dbx().db())
            .await?;
            
        Ok(())
    }

    async fn update_blockchain_tx(&self, id: Id, tx_hash: String) -> Result<()> {
        let query = r#"
            UPDATE zkml_proofs
            SET blockchain_tx_hash = $1
            WHERE id = $2
        "#;
        
        sqlx::query(query)
            .bind(&tx_hash)
            .bind(id.value())
            .execute(self.app_state.mm.dbx().db())
            .await?;
            
        Ok(())
    }
}