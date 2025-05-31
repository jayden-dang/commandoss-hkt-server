use thiserror::Error;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("GitHub API error: {0}")]
    GitHubApi(String),
    
    #[error("Repository not found: {owner}/{repo}")]
    RepositoryNotFound { owner: String, repo: String },
    
    #[error("Invalid webhook signature")]
    InvalidWebhookSignature,
    
    #[error("Webhook payload parsing error: {0}")]
    WebhookPayloadError(String),
    
    #[error("Rate limit exceeded, retry after {retry_after_seconds} seconds")]
    RateLimitExceeded { retry_after_seconds: u64 },
    
    #[error("No smart contracts found in repository")]
    NoSmartContractsFound,
    
    #[error("Job not found: {0}")]
    JobNotFound(Uuid),
    
    #[error("Queue is full")]
    QueueFull,
    
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("HTTP request error: {0}")]
    HttpRequest(#[from] reqwest::Error),
    
    #[error("Octocrab error: {0}")]
    Octocrab(#[from] octocrab::Error),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Internal(err.to_string())
    }
}