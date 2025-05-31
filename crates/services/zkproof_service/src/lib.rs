pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod models;

mod error;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

use application::handlers::zkproof_handler::ZkProofHandler;
use infrastructure::zkproof_repository_impl::ZkProofRepositoryImpl;
use jd_core::AppState;

pub struct ZkProofService {
    handler: ZkProofHandler<ZkProofRepositoryImpl>,
}

impl ZkProofService {
    pub async fn new(state: AppState) -> Self {
        let repository = ZkProofRepositoryImpl::new(state);
        let handler = ZkProofHandler::new(repository);
        
        Self { handler }
    }

    pub fn handler(&self) -> &ZkProofHandler<ZkProofRepositoryImpl> {
        &self.handler
    }
}