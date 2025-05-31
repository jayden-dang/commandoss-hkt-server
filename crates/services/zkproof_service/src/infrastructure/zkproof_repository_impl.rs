use async_trait::async_trait;
use jd_core::{AppState, base};
use jd_domain::Id;
use jd_domain::zkpersona_domain::profile::ZkProof;

use crate::{
    ZkProofDmc,
    domain::zkproof_repository_trait::ZkProofRepository,
    models::{
        requests::ProofQueryRequest,
        responses::{ZkProofResponse, ZkProofListResponse},
        ZkProofRecord, ZkProofForCreate, ZkProofForUpdate, ZkProofFilter,
    },
    Result,
};

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
        let scoring_result_uuid = proof.scoring_result_id.to_uuid();
            
        let create_req = ZkProofForCreate {
            scoring_result_id: scoring_result_uuid,
            proof_data: proof.proof_data,
            verification_key: proof.verification_key,
            verified: Some(proof.verified),
            blockchain_tx_hash: proof.blockchain_tx_hash,
        };
        
        let record = base::rest::create::<ZkProofDmc, _, ZkProofRecord>(
            &self.app_state.mm, 
            create_req
        ).await?;
            
        Ok(ZkProofResponse::from(record))
    }

    async fn get_zkproof(&self, id: Id) -> Result<Option<ZkProofResponse>> {
        let id_uuid = id.to_uuid();
            
        match base::rest::get_by_id::<ZkProofDmc, ZkProofRecord>(&self.app_state.mm, id_uuid).await {
            Ok(record) => Ok(Some(ZkProofResponse::from(record))),
            Err(_) => Ok(None),
        }
    }

    async fn get_zkproof_by_scoring_id(&self, scoring_result_id: Id) -> Result<Option<ZkProofResponse>> {
        let scoring_result_uuid = scoring_result_id.to_uuid();
            
        let filter = ZkProofFilter {
            scoring_result_id: Some(vec![modql::filter::OpValValue::Eq(serde_json::Value::String(scoring_result_uuid.to_string()))].into()),
            verified: None,
        };
        
        // Use first to get the most recent result (ordered by timestamp DESC by default)
        let list_options = modql::filter::ListOptions {
            limit: Some(1),
            offset: None,
            order_bys: Some("!timestamp".into()), // ! prefix for descending
        };
        
        match base::rest::first::<ZkProofDmc, _, ZkProofRecord>(&self.app_state.mm, Some(filter), Some(list_options)).await {
            Ok(Some(record)) => Ok(Some(ZkProofResponse::from(record))),
            Ok(None) => Ok(None),
            Err(_) => Ok(None),
        }
    }

    async fn list_zkproofs(&self, query_req: ProofQueryRequest) -> Result<ZkProofListResponse> {
        let limit = query_req.limit.unwrap_or(50).min(100);
        let offset = query_req.offset.unwrap_or(0);
        
        let filter = match (&query_req.scoring_result_id, query_req.verified) {
            (Some(scoring_result_id), Some(verified)) => {
                let scoring_result_uuid = scoring_result_id.to_uuid();
                Some(ZkProofFilter {
                    scoring_result_id: Some(vec![modql::filter::OpValValue::Eq(serde_json::Value::String(scoring_result_uuid.to_string()))].into()),
                    verified: Some(vec![modql::filter::OpValValue::Eq(serde_json::Value::Bool(verified))].into()),
                })
            },
            (Some(scoring_result_id), None) => {
                let scoring_result_uuid = scoring_result_id.to_uuid();
                Some(ZkProofFilter {
                    scoring_result_id: Some(vec![modql::filter::OpValValue::Eq(serde_json::Value::String(scoring_result_uuid.to_string()))].into()),
                    verified: None,
                })
            },
            (None, Some(verified)) => {
                Some(ZkProofFilter {
                    scoring_result_id: None,
                    verified: Some(vec![modql::filter::OpValValue::Eq(serde_json::Value::Bool(verified))].into()),
                })
            },
            (None, None) => None,
        };
        
        let list_options = modql::filter::ListOptions {
            limit: Some(limit as i64),
            offset: Some(offset as i64),
            order_bys: Some("!timestamp".into()), // ! prefix for descending
        };
        
        let (records, meta) = base::rest::list::<ZkProofDmc, _, ZkProofRecord>(
            &self.app_state.mm, 
            filter, 
            Some(list_options)
        ).await?;
        
        let items: Vec<ZkProofResponse> = records
            .into_iter()
            .map(ZkProofResponse::from)
            .collect();
        
        Ok(ZkProofListResponse {
            items,
            total: meta.total_items(),
            limit,
            offset,
        })
    }

    async fn mark_as_verified(&self, id: Id) -> Result<()> {
        let id_uuid = id.to_uuid();
            
        let update_req = ZkProofForUpdate {
            verified: Some(true),
            blockchain_tx_hash: None,
        };
        
        base::rest::update::<ZkProofDmc, _>(&self.app_state.mm, id_uuid, update_req)
            .await?;
            
        Ok(())
    }

    async fn update_blockchain_tx(&self, id: Id, tx_hash: String) -> Result<()> {
        let id_uuid = id.to_uuid();
            
        let update_req = ZkProofForUpdate {
            verified: None,
            blockchain_tx_hash: Some(tx_hash),
        };
        
        base::rest::update::<ZkProofDmc, _>(&self.app_state.mm, id_uuid, update_req)
            .await?;
            
        Ok(())
    }
}