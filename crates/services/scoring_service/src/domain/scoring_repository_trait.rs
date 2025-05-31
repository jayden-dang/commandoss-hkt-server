use async_trait::async_trait;
use jd_domain::Id;
use jd_domain::zkpersona_domain::profile::ScoringResult;
use crate::models::{requests::ScoringQueryRequest, responses::{ScoringResponse, ScoringListResponse}};
use crate::Result;

#[async_trait]
pub trait ScoringRepository: Send + Sync {
    async fn create_scoring_result(&self, result: ScoringResult) -> Result<ScoringResponse>;
    async fn get_scoring_result(&self, id: Id) -> Result<Option<ScoringResponse>>;
    async fn get_scoring_by_behavior_id(&self, behavior_input_id: Id) -> Result<Option<ScoringResponse>>;
    async fn list_scoring_results(&self, query: ScoringQueryRequest) -> Result<ScoringListResponse>;
}