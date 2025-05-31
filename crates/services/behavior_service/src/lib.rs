pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod models;

mod error;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

use application::handlers::behavior_handler::BehaviorHandler;
use infrastructure::behavior_repository_impl::BehaviorRepositoryImpl;
use jd_core::AppState;

pub struct BehaviorService {
    handler: BehaviorHandler<BehaviorRepositoryImpl>,
}

impl BehaviorService {
    pub async fn new(state: AppState) -> Self {
        let repository = BehaviorRepositoryImpl::new(state);
        let handler = BehaviorHandler::new(repository);
        
        Self { handler }
    }

    pub fn handler(&self) -> &BehaviorHandler<BehaviorRepositoryImpl> {
        &self.handler
    }
}