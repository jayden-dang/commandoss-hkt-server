pub mod application;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod models;

pub use error::{Error, Result};

// Re-export main components
pub use application::handlers::analysis_handler::AnalysisHandler;
pub use application::use_cases::analysis_use_cases::AnalysisUseCases;
pub use domain::analysis_engine::AnalysisEngine;
pub use domain::vulnerability_patterns::VulnerabilityPatterns;
pub use infrastructure::static_analyzer::SuiMoveStaticAnalyzer;
pub use infrastructure::llm_client::LLMClient;