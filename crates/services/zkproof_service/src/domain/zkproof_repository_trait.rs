use async_trait::async_trait;
use jd_domain::Id;
use jd_domain::zkpersona_domain::profile::ZkProof;
use crate::models::{requests::ProofQueryRequest, responses::{ZkProofResponse, ZkProofListResponse}};
use crate::Result;

#[async_trait]
pub trait ZkProofRepository: Send + Sync {
    async fn create_zkproof(&self, proof: ZkProof) -> Result<ZkProofResponse>;
    async fn get_zkproof(&self, id: Id) -> Result<Option<ZkProofResponse>>;
    async fn get_zkproof_by_scoring_id(&self, scoring_result_id: Id) -> Result<Option<ZkProofResponse>>;
    async fn list_zkproofs(&self, query: ProofQueryRequest) -> Result<ZkProofListResponse>;
    async fn mark_as_verified(&self, id: Id) -> Result<()>;
    async fn update_blockchain_tx(&self, id: Id, tx_hash: String) -> Result<()>;
}