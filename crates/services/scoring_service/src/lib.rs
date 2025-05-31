pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod models;

mod error;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

use jd_core::base::DMC;
use application::handlers::scoring_handler::ScoringHandler;
use infrastructure::scoring_repository_impl::ScoringRepositoryImpl;
use jd_core::AppState;

pub struct ScoringService {
    handler: ScoringHandler<ScoringRepositoryImpl>,
}

impl ScoringService {
    pub async fn new(state: AppState) -> Self {
        let repository = ScoringRepositoryImpl::new(state);
        let handler = ScoringHandler::new(repository);
        
        Self { handler }
    }

    pub fn handler(&self) -> &ScoringHandler<ScoringRepositoryImpl> {
        &self.handler
    }
}

pub struct ScoringResultDmc;

impl DMC for ScoringResultDmc {
    const SCHEMA: &'static str = "public";
    const TABLE: &'static str = "scoring_results";
    const ID: &'static str = "id";
    const ENUM_COLUMNS: &'static [&'static str] = &[];
}