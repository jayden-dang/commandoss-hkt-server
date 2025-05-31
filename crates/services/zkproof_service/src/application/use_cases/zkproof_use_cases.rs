use jd_domain::Id;
use jd_domain::zkpersona_domain::profile::ZkProof;

use crate::domain::zkproof_repository_trait::ZkProofRepository;
use crate::domain::mock_proof_generator::MockProofGenerator;
use crate::models::{
    requests::{GenerateProofRequest, VerifyProofRequest, ProofQueryRequest},
    responses::{GenerateProofResponse, VerifyProofResponse, ZkProofResponse, ZkProofListResponse},
};
use crate::{Error, Result};

pub struct ZkProofUseCases<R: ZkProofRepository> {
    repository: R,
    proof_generator: MockProofGenerator,
}

impl<R: ZkProofRepository> ZkProofUseCases<R> {
    pub fn new(repository: R) -> Self {
        Self { 
            repository,
            proof_generator: MockProofGenerator::new(),
        }
    }

    pub async fn generate_proof(&self, request: GenerateProofRequest, scoring_result_id: Id, score: f64) -> Result<GenerateProofResponse> {
        // Generate ZK proof using mock generator
        let (proof_data, verification_key, public_signals) = self.proof_generator
            .generate_proof(score, &request.behavior_input)
            .await?;
        
        // Create ZK proof record
        let zk_proof = ZkProof {
            scoring_result_id: scoring_result_id.clone(),
            proof_data: proof_data.as_bytes().to_vec(),
            verification_key: verification_key.as_bytes().to_vec(),
            verified: false,
            blockchain_tx_hash: None,
        };
        
        let stored_proof = self.repository.create_zkproof(zk_proof).await?;
        
        // Create behavior input record (this would typically be done by behavior service)
        let behavior_input_id = Id::generate(); // This should come from actual behavior service
        
        Ok(GenerateProofResponse {
            proof_id: stored_proof.id,
            behavior_input_id,
            scoring_result_id,
            score,
            proof_data,
            verification_key,
            public_signals,
            timestamp: stored_proof.timestamp,
        })
    }

    pub async fn verify_proof(&self, request: VerifyProofRequest) -> Result<VerifyProofResponse> {
        // Verify the proof using mock generator
        let is_valid = self.proof_generator
            .verify_proof(&request.proof_data, &request.verification_key, &request.public_signals)
            .await?;
        
        let mut proof_id = None;
        
        // If valid, try to find and mark the proof as verified
        if is_valid {
            // Try to find the proof by matching proof_data
            // This is a simplified approach - in practice you'd have better indexing
            let query = ProofQueryRequest {
                scoring_result_id: None,
                verified: Some(false),
                limit: Some(100),
                offset: Some(0),
            };
            
            let proofs = self.repository.list_zkproofs(query).await?;
            
            for proof in proofs.items {
                if proof.proof_data == request.proof_data {
                    let proof_id_clone = proof.id.clone();
                    self.repository.mark_as_verified(proof.id).await?;
                    proof_id = Some(proof_id_clone);
                    break;
                }
            }
        }
        
        Ok(VerifyProofResponse {
            valid: is_valid,
            proof_id,
            timestamp: jd_utils::time::now_utc(),
        })
    }

    pub async fn get_zkproof(&self, id: Id) -> Result<Option<ZkProofResponse>> {
        self.repository.get_zkproof(id).await
    }

    pub async fn get_zkproof_by_scoring_id(&self, scoring_result_id: Id) -> Result<Option<ZkProofResponse>> {
        self.repository.get_zkproof_by_scoring_id(scoring_result_id).await
    }

    pub async fn list_zkproofs(&self, request: ProofQueryRequest) -> Result<ZkProofListResponse> {
        self.repository.list_zkproofs(request).await
    }

    pub async fn update_blockchain_tx(&self, proof_id: Id, tx_hash: String) -> Result<()> {
        self.repository.update_blockchain_tx(proof_id, tx_hash).await
    }

    pub fn get_proof_generator_info(&self) -> serde_json::Value {
        self.proof_generator.get_proof_metadata()
    }
}