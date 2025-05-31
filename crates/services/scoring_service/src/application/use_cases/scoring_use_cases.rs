use jd_domain::Id;
use jd_domain::zkpersona_domain::profile::ScoringResult;

use crate::domain::scoring_repository_trait::ScoringRepository;
use crate::domain::scoring_model::HardcodedScoringModel;
use crate::models::{
    requests::{ScoringRequest, ScoringQueryRequest},
    responses::{ScoringResponse, ScoringListResponse},
};
use crate::Result;

pub struct ScoringUseCases<R: ScoringRepository> {
    repository: R,
    model: HardcodedScoringModel,
}

impl<R: ScoringRepository> ScoringUseCases<R> {
    pub fn new(repository: R) -> Self {
        Self { 
            repository,
            model: HardcodedScoringModel::new(),
        }
    }

    pub async fn calculate_score(&self, request: ScoringRequest, behavior_data: serde_json::Value) -> Result<ScoringResponse> {
        // Use the hardcoded model to calculate score
        let score = self.model.calculate_score(&behavior_data).await?;
        
        let model_version = request.model_version
            .unwrap_or_else(|| self.model.version().to_string());
        
        let scoring_result = ScoringResult {
            behavior_input_id: request.behavior_input_id,
            score,
            model_version,
        };
        
        self.repository.create_scoring_result(scoring_result).await
    }

    pub async fn get_scoring_result(&self, id: Id) -> Result<Option<ScoringResponse>> {
        self.repository.get_scoring_result(id).await
    }

    pub async fn get_scoring_by_behavior_id(&self, behavior_input_id: Id) -> Result<Option<ScoringResponse>> {
        self.repository.get_scoring_by_behavior_id(behavior_input_id).await
    }

    pub async fn list_scoring_results(&self, request: ScoringQueryRequest) -> Result<ScoringListResponse> {
        self.repository.list_scoring_results(request).await
    }

    pub fn get_model_info(&self) -> serde_json::Value {
        self.model.get_model_metadata()
    }
}