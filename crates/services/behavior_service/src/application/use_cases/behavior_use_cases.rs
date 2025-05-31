use jd_domain::Id;
use jd_domain::zkpersona_domain::profile::BehaviorInput;
use validator::Validate;

use crate::domain::behavior_repository_trait::BehaviorRepository;
use crate::models::{
    requests::{BehaviorInputRequest, BehaviorQueryRequest},
    responses::{BehaviorInputResponse, BehaviorListResponse},
};
use crate::{Error, Result};

pub struct BehaviorUseCases<R: BehaviorRepository> {
    repository: R,
}

impl<R: BehaviorRepository> BehaviorUseCases<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn create_behavior_input(&self, request: BehaviorInputRequest) -> Result<BehaviorInputResponse> {
        // Validate input
        request.validate().map_err(|e| Error::Validation(e.to_string()))?;
        
        // Validate input_data is not empty
        if request.input_data.is_null() {
            return Err(Error::InvalidInput("input_data cannot be null".to_string()));
        }
        
        let behavior_input = BehaviorInput {
            session_id: request.session_id,
            input_data: request.input_data,
        };
        
        self.repository.create_behavior_input(behavior_input).await
    }

    pub async fn get_behavior_input(&self, id: Id) -> Result<Option<BehaviorInputResponse>> {
        self.repository.get_behavior_input(id).await
    }

    pub async fn list_behavior_inputs(&self, request: BehaviorQueryRequest) -> Result<BehaviorListResponse> {
        self.repository.list_behavior_inputs(request).await
    }

    pub async fn mark_as_processed(&self, id: Id) -> Result<()> {
        self.repository.mark_as_processed(id).await
    }
}