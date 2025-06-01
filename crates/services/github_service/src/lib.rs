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
    pub fn from_config(config: &jd_utils::config::Config) -> Result<Self> {
        let github_config = config.github.as_ref()
            .ok_or_else(|| Error::ConfigurationError("GitHub configuration not found".to_string()))?;
        
        let github_private_key = if let Some(key_path) = &github_config.private_key_path {
            std::fs::read_to_string(key_path).ok()
        } else {
            github_config.private_key.clone()
        };

        // Ensure we have either personal token or app credentials
        if github_config.token.is_none() && (github_config.app_id.is_none() || github_private_key.is_none()) {
            return Err(Error::ConfigurationError(
                "Either GITHUB.TOKEN or (GITHUB.APP_ID + GITHUB.PRIVATE_KEY) must be set".to_string()
            ));
        }

        Ok(Self {
            github_token: github_config.token.clone(),
            github_app_id: github_config.app_id,
            github_private_key,
            webhook_secret: github_config.webhook_secret.clone(),
            webhook_base_url: github_config.webhook_base_url
                .clone()
                .unwrap_or_else(|| "http://localhost:3000".to_string()),
            max_queue_size: github_config.max_queue_size.unwrap_or(1000),
            rate_limit_per_hour: github_config.rate_limit_per_hour.unwrap_or(5000),
        })
    }
    
    pub fn from_env() -> Result<Self> {
        let config = jd_utils::config::Config::from_env()
            .map_err(|e| Error::ConfigurationError(format!("Failed to load config: {}", e)))?;
        Self::from_config(&config)
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