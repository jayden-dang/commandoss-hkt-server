use async_trait::async_trait;
use jd_domain::Id;
use jd_domain::zkpersona_domain::profile::BehaviorInput;
use crate::models::{requests::BehaviorQueryRequest, responses::{BehaviorInputResponse, BehaviorListResponse}};
use crate::Result;

#[async_trait]
pub trait BehaviorRepository: Send + Sync {
    async fn create_behavior_input(&self, input: BehaviorInput) -> Result<BehaviorInputResponse>;
    async fn get_behavior_input(&self, id: Id) -> Result<Option<BehaviorInputResponse>>;
    async fn list_behavior_inputs(&self, query: BehaviorQueryRequest) -> Result<BehaviorListResponse>;
    async fn mark_as_processed(&self, id: Id) -> Result<()>;
}