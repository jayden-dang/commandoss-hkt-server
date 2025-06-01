use axum::{
  extract::{Json, Path, Query, State},
  http::{HeaderMap, StatusCode},
  response::Json as ResponseJson,
};
use jd_core::AppState;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

// Placeholder handlers for test compatibility
pub async fn handle_webhook(
  State(_app_state): State<AppState>,
  Json(_payload): Json<Value>,
) -> std::result::Result<ResponseJson<Value>, StatusCode> {
  let response = json!({
    "status": "processed",
    "message": "Webhook received"
  });
  Ok(ResponseJson(response))
}

pub async fn get_repository_info(
  State(_app_state): State<AppState>,
  Path((owner, repo)): Path<(String, String)>,
) -> std::result::Result<ResponseJson<Value>, StatusCode> {
  let response = json!({
    "owner": owner,
    "repo": repo,
    "full_name": format!("{}/{}", owner, repo),
    "description": "Repository information",
    "stars": 42,
    "forks": 10
  });
  Ok(ResponseJson(response))
}

pub async fn analyze_repository(
  State(_app_state): State<AppState>,
  Json(_payload): Json<Value>,
) -> std::result::Result<ResponseJson<Value>, StatusCode> {
  let response = json!({
    "status": "analyzing",
    "job_id": Uuid::new_v4(),
    "message": "Repository analysis started"
  });
  Ok(ResponseJson(response))
}

// Keep existing handlers below
use github_service::{
  AddRepositoryRequest, GitHubWebhookPayload, RepositoryDetailResponse, RepositoryHandler,
  RepositoryListParams, RepositoryListResponse, RepositoryResponse,
  UpdateRepositorySettingsRequest, WebhookHandler, WebhookResponse,
};

use crate::error::Error as ApiError;
type Result<T> = std::result::Result<T, ApiError>;

/// List repositories with optional filtering
pub async fn list_repositories(
  State(app_state): State<AppState>,
  Query(params): Query<RepositoryListParams>,
) -> Result<ResponseJson<RepositoryListResponse>> {
  // Create GitHub service components on demand
  let repository_handler = create_repository_handler(&app_state).map_err(|e| {
    error!("Failed to create GitHub repository handler: {}", e);
    ApiError::service_error("github", 500, Some(e.to_string()))
  })?;

  repository_handler
    .list_repositories(params)
    .await
    .map(ResponseJson)
    .map_err(|e| {
      error!("Failed to list repositories: {}", e);
      map_github_error(e)
    })
}

/// Add a new repository for monitoring
pub async fn add_repository(
  State(app_state): State<AppState>,
  Json(request): Json<AddRepositoryRequest>,
) -> Result<ResponseJson<RepositoryResponse>> {
  let repository_handler = create_repository_handler(&app_state).map_err(|e| {
    error!("Failed to create GitHub repository handler: {}", e);
    ApiError::service_error("github", 500, Some(e.to_string()))
  })?;

  repository_handler
    .add_repository(request)
    .await
    .map(ResponseJson)
    .map_err(|e| {
      error!("Failed to add repository: {}", e);
      map_github_error(e)
    })
}

/// Get detailed information about a repository
pub async fn get_repository(
  State(app_state): State<AppState>,
  Path(id): Path<Uuid>,
) -> Result<ResponseJson<RepositoryDetailResponse>> {
  let repository_handler = create_repository_handler(&app_state).map_err(|e| {
    error!("Failed to create GitHub repository handler: {}", e);
    ApiError::service_error("github", 500, Some(e.to_string()))
  })?;

  repository_handler
    .get_repository(id)
    .await
    .map(ResponseJson)
    .map_err(|e| {
      error!("Failed to get repository {}: {}", id, e);
      map_github_error(e)
    })
}

/// Update repository settings
pub async fn update_repository_settings(
  State(app_state): State<AppState>,
  Path(id): Path<Uuid>,
  Json(request): Json<UpdateRepositorySettingsRequest>,
) -> Result<ResponseJson<RepositoryResponse>> {
  let repository_handler = create_repository_handler(&app_state).map_err(|e| {
    error!("Failed to create GitHub repository handler: {}", e);
    ApiError::service_error("github", 500, Some(e.to_string()))
  })?;

  repository_handler
    .update_repository_settings(id, request)
    .await
    .map(ResponseJson)
    .map_err(|e| {
      error!("Failed to update repository {} settings: {}", id, e);
      map_github_error(e)
    })
}

/// Handle GitHub webhook events
pub async fn handle_github_webhook(
  State(app_state): State<AppState>,
  headers: HeaderMap,
  Json(payload): Json<GitHubWebhookPayload>,
) -> Result<ResponseJson<WebhookResponse>> {
  let webhook_handler = create_webhook_handler(&app_state).map_err(|e| {
    error!("Failed to create GitHub webhook handler: {}", e);
    ApiError::service_error("github", 500, Some(e.to_string()))
  })?;

  webhook_handler
    .handle_github_webhook(headers, payload)
    .await
    .map(ResponseJson)
    .map_err(|e| {
      error!("Webhook processing error: {}", e);
      map_github_error(e)
    })
}

// Helper functions to create GitHub service handlers
fn create_repository_handler(
  app_state: &AppState,
) -> std::result::Result<RepositoryHandler, Box<dyn std::error::Error + Send + Sync>> {
  use github_service::{GitHubServiceConfig, GitHubServiceFactory};

  let github_config = GitHubServiceConfig::from_env()?;

  let github_client = Arc::new(GitHubServiceFactory::create_client(&github_config)?);
  let analysis_queue = Arc::new(GitHubServiceFactory::create_analysis_queue(&github_config));

  let repository_repo =
    Arc::new(jd_storage::repository::developer_repositories::GitHubRepositoryRepository::new(
      app_state.mm().dbx().clone(),
    ));

  Ok(RepositoryHandler::new(
    github_client,
    analysis_queue,
    repository_repo,
    github_config.webhook_base_url,
  ))
}

fn create_webhook_handler(
  _app_state: &AppState,
) -> std::result::Result<WebhookHandler, Box<dyn std::error::Error + Send + Sync>> {
  use github_service::{GitHubServiceConfig, GitHubServiceFactory};

  let github_config = GitHubServiceConfig::from_env()?;

  let github_client = Arc::new(GitHubServiceFactory::create_client(&github_config)?);
  let analysis_queue = Arc::new(GitHubServiceFactory::create_analysis_queue(&github_config));

  Ok(WebhookHandler::new(github_client, analysis_queue))
}

// Helper function to map GitHub service errors to API gateway errors
fn map_github_error(error: github_service::Error) -> ApiError {
  match error {
    github_service::Error::RepositoryNotFound { owner, repo } => ApiError::RouteNotFound {
      path: format!("/repositories/{}/{}", owner, repo),
      method: "GET".to_string(),
    },
    github_service::Error::NoSmartContractsFound => ApiError::InvalidRequestFormat {
      message: "Repository does not contain any smart contracts".to_string(),
    },
    github_service::Error::GitHubApi(msg) => ApiError::service_error("github", 502, Some(msg)),
    github_service::Error::InvalidWebhookSignature => {
      ApiError::ApiKeyAuthFailed { reason: "Invalid webhook signature".to_string() }
    }
    github_service::Error::WebhookPayloadError(msg) => {
      ApiError::InvalidRequestFormat { message: format!("Invalid webhook payload: {}", msg) }
    }
    github_service::Error::RateLimitExceeded { retry_after_seconds: _ } => {
      ApiError::RateLimitExceeded {
        client_id: "github_api".to_string(),
        limit: 5000,
        window: "hour".to_string(),
      }
    }
    github_service::Error::JobNotFound(id) => {
      ApiError::RouteNotFound { path: format!("/jobs/{}", id), method: "GET".to_string() }
    }
    github_service::Error::QueueFull => ApiError::service_unavailable("github_queue"),
    github_service::Error::AuthenticationError(msg) => ApiError::ApiKeyAuthFailed { reason: msg },
    github_service::Error::ConfigurationError(msg) => {
      ApiError::GatewayConfig { config_key: format!("github_service: {}", msg) }
    }
    github_service::Error::Database(err) => {
      ApiError::service_error("database", 500, Some(err.to_string()))
    }
    github_service::Error::Serialization(err) => {
      ApiError::InvalidRequestFormat { message: format!("Serialization error: {}", err) }
    }
    github_service::Error::HttpRequest(err) => {
      ApiError::service_error("github_api", 502, Some(err.to_string()))
    }
    github_service::Error::Octocrab(err) => {
      ApiError::service_error("github_api", 502, Some(err.to_string()))
    }
    github_service::Error::Internal(msg) => ApiError::service_error("github", 500, Some(msg)),
  }
}
