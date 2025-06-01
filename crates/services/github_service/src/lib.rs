pub mod application;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod models;

pub use application::*;
pub use domain::*;
pub use error::*;
pub use infrastructure::*;
pub use models::*;

// Re-export key types for easier usage
pub use crate::application::handlers::{WebhookHandler, RepositoryHandler};
pub use crate::application::use_cases::GitHubUseCase;
pub use crate::infrastructure::{GitHubClient, GitHubFile, AnalysisQueueImpl, RateLimiterImpl};
pub use crate::domain::{AnalysisJob, AnalysisType, AnalysisPriority, JobStatus, QueueStatus};
pub use crate::models::{
    AddRepositoryRequest, UpdateRepositorySettingsRequest, RepositoryListParams,
    RepositoryResponse, RepositoryListResponse, RepositoryDetailResponse, WebhookResponse,
};

// Configuration structure for the GitHub service
#[derive(Debug, Clone)]
pub struct GitHubServiceConfig {
    pub github_token: Option<String>,
    pub github_app_id: Option<u64>,
    pub github_private_key: Option<String>,
    pub webhook_secret: String,
    pub webhook_base_url: String,
    pub max_queue_size: usize,
    pub rate_limit_per_hour: u32,
}

impl GitHubServiceConfig {
    pub fn from_env() -> Result<Self> {
        let github_token = std::env::var("GITHUB_TOKEN").ok();
        let github_app_id = std::env::var("GITHUB_APP_ID")
            .ok()
            .and_then(|s| s.parse().ok());
        let github_private_key = if let Ok(key_path) = std::env::var("GITHUB_PRIVATE_KEY_PATH") {
            std::fs::read_to_string(key_path).ok()
        } else {
            std::env::var("GITHUB_PRIVATE_KEY").ok()
        };

        // Ensure we have either personal token or app credentials
        if github_token.is_none() && (github_app_id.is_none() || github_private_key.is_none()) {
            return Err(Error::ConfigurationError(
                "Either GITHUB_TOKEN or (GITHUB_APP_ID + GITHUB_PRIVATE_KEY) must be set".to_string()
            ));
        }

        Ok(Self {
            github_token,
            github_app_id,
            github_private_key,
            webhook_secret: std::env::var("GITHUB_WEBHOOK_SECRET")
                .map_err(|_| Error::ConfigurationError("GITHUB_WEBHOOK_SECRET not set".to_string()))?,
            webhook_base_url: std::env::var("WEBHOOK_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            max_queue_size: std::env::var("GITHUB_MAX_QUEUE_SIZE")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            rate_limit_per_hour: std::env::var("GITHUB_RATE_LIMIT_PER_HOUR")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .unwrap_or(5000),
        })
    }
}

// Service factory for creating all GitHub service components
pub struct GitHubServiceFactory;

impl GitHubServiceFactory {
    pub fn create_client(config: &GitHubServiceConfig) -> Result<GitHubClient> {
        if let (Some(app_id), Some(private_key)) = (config.github_app_id, &config.github_private_key) {
            // Use GitHub App authentication
            GitHubClient::new_app(app_id, private_key.clone(), config.webhook_secret.clone())
        } else if let Some(token) = &config.github_token {
            // Use personal token authentication
            GitHubClient::new(token.clone(), config.webhook_secret.clone())
        } else {
            Err(Error::ConfigurationError("No valid GitHub authentication method configured".to_string()))
        }
    }

    pub fn create_analysis_queue(config: &GitHubServiceConfig) -> AnalysisQueueImpl {
        AnalysisQueueImpl::new(config.max_queue_size)
    }

    pub fn create_rate_limiter(config: &GitHubServiceConfig) -> RateLimiterImpl {
        RateLimiterImpl::new(
            config.rate_limit_per_hour,
            std::time::Duration::from_secs(3600),
        )
    }
}