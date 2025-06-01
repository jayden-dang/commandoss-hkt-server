use axum::{
    routing::{get, post, put},
    Router,
};
use std::sync::Arc;

// Re-export handlers from ai_analysis_service
pub use ai_analysis_service::application::handlers::analysis_handler::{
    analyze_code, analyze_repository, get_analysis_history, get_analysis_status,
    get_detailed_analysis, get_repository_vulnerabilities, mark_vulnerability,
};
use ai_analysis_service::application::handlers::analysis_handler::AnalysisHandler;

pub fn analysis_routes(analysis_handler: Arc<AnalysisHandler>) -> Router {
    Router::new()
        // Repository analysis routes
        .route("/repositories/analyze", post(analyze_repository))
        .route("/repositories/:id/analysis/status", get(get_analysis_status))
        .route("/repositories/:id/vulnerabilities", get(get_repository_vulnerabilities))
        .route("/repositories/:id/analysis/history", get(get_analysis_history))
        
        // Analysis detail routes
        .route("/analysis/:id", get(get_detailed_analysis))
        
        // Code analysis routes (for direct code analysis)
        .route("/code/analyze", post(analyze_code))
        
        // Vulnerability management routes
        .route("/vulnerabilities/mark", put(mark_vulnerability))
        
        .with_state(analysis_handler)
}

// Integration helper for setting up the AI analysis service
pub mod integration {
    use ai_analysis_service::{
        application::use_cases::analysis_use_cases::AnalysisUseCases,
        infrastructure::{
            analysis_repository_impl::AnalysisRepositoryImpl,
            github_integration::GitHubIntegrationService,
            llm_client::LLMClient,
        },
    };
    use jd_core::AppState;
    use std::sync::Arc;

    pub struct AiAnalysisServiceConfig {
        pub app_state: AppState,
        pub openai_api_key: Option<String>,
        pub anthropic_api_key: Option<String>,
        pub enable_llm_analysis: bool,
    }

    pub fn setup_ai_analysis_service(config: AiAnalysisServiceConfig) -> (
        Arc<super::AnalysisHandler>,
        Arc<GitHubIntegrationService>,
    ) {
        // Setup repository
        let analysis_repository = Arc::new(AnalysisRepositoryImpl::new(config.app_state));

        // Setup LLM provider if configured
        let llm_provider = if config.enable_llm_analysis {
            if let Some(api_key) = config.openai_api_key {
                Some(Arc::new(LLMClient::new_openai(api_key)) as Arc<dyn ai_analysis_service::domain::llm_provider_trait::LLMProvider>)
            } else if let Some(api_key) = config.anthropic_api_key {
                Some(Arc::new(LLMClient::new_anthropic(api_key)) as Arc<dyn ai_analysis_service::domain::llm_provider_trait::LLMProvider>)
            } else {
                None
            }
        } else {
            None
        };

        // Setup use cases
        let analysis_use_cases = Arc::new(AnalysisUseCases::new(
            analysis_repository,
            llm_provider,
        ));

        // Setup handlers
        let analysis_handler = Arc::new(super::AnalysisHandler::new(analysis_use_cases.clone()));

        // Setup GitHub integration
        let github_integration = Arc::new(GitHubIntegrationService::new(analysis_use_cases));

        (analysis_handler, github_integration)
    }
}