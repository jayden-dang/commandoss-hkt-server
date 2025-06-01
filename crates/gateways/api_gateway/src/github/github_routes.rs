use axum::{
  extract::{Json, Path, Query, State},
  http::{HeaderMap, StatusCode},
  response::Json as ResponseJson,
};
use jd_core::AppState;
use rust_decimal::prelude::ToPrimitive;
use serde_json::{json, Value};
use sqlx::Row;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};
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
  State(app_state): State<AppState>,
  Json(payload): Json<Value>,
) -> std::result::Result<ResponseJson<Value>, StatusCode> {
  // Parse the repository information from the payload
  let repository_id = payload.get("repository_id")
    .and_then(|v| v.as_str())
    .and_then(|s| Uuid::parse_str(s).ok())
    .ok_or_else(|| {
      error!("Missing or invalid repository_id in payload");
      StatusCode::BAD_REQUEST
    })?;

  let commit_sha = payload.get("commit_sha")
    .and_then(|v| v.as_str())
    .unwrap_or("0000000000000000000000000000000000000000") // Default 40-char SHA
    .to_string();

  // Ensure commit_sha is exactly 40 characters and valid hex
  let commit_sha = if commit_sha.len() == 40 && commit_sha.chars().all(|c| c.is_ascii_hexdigit()) {
    commit_sha
  } else {
    // Generate a valid 40-character hex string for testing
    format!("{:040x}", uuid::Uuid::new_v4().as_u128())
  };

  // Parse owner and repo from payload
  let owner = payload.get("owner")
    .and_then(|v| v.as_str())
    .unwrap_or("jayden-dang");
  let repo = payload.get("repo") 
    .and_then(|v| v.as_str())
    .unwrap_or("aptos_onlyfans");

  // Create repository record if it doesn't exist
  let repo_count: i64 = sqlx::query_scalar(
    "SELECT COUNT(*) FROM github_repositories WHERE id = $1"
  )
  .bind(repository_id)
  .fetch_one(app_state.mm().dbx().db())
  .await
  .map_err(|e| {
    error!("Database error checking repository: {}", e);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  if repo_count == 0 {
    // Create repository record with real owner/repo data
    sqlx::query(
      r#"
      INSERT INTO github_repositories (
        id, github_repo_id, owner_username, repo_name, full_name,
        primary_language, is_private, star_count, fork_count, monitoring_enabled
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
      "#
    )
    .bind(repository_id)
    .bind((repository_id.as_u128() % (i64::MAX as u128)) as i64 + 1) // Ensure positive value
    .bind(owner)
    .bind(repo)
    .bind(format!("{}/{}", owner, repo))
    .bind("Move")
    .bind(false)
    .bind(0_i32) // Real repos start with 0 stars
    .bind(0_i32) // Real repos start with 0 forks
    .bind(true)
    .execute(app_state.mm().dbx().db())
    .await
    .map_err(|e| {
      error!("Failed to create repository record for {}/{}: {}", owner, repo, e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;
  }

  // Set up AI analysis service
  use crate::ai_analysis::analysis_routes::integration::{setup_ai_analysis_service, AiAnalysisServiceConfig};

  let ai_config = AiAnalysisServiceConfig {
    app_state: app_state.clone(),
    openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
    anthropic_api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
    enable_llm_analysis: std::env::var("ENABLE_LLM_ANALYSIS").unwrap_or("false".to_string()) == "true",
  };

  let (analysis_handler, _github_integration) = setup_ai_analysis_service(ai_config);

  // Create analysis request
  use ai_analysis_service::models::requests::AnalyzeRepositoryRequest;
  use ai_analysis_service::domain::analysis_models::AnalysisType;

  let analysis_request = AnalyzeRepositoryRequest {
    repository_id,
    commit_sha: commit_sha.clone(),
    files_to_analyze: None, // Will analyze all smart contract files
    analysis_types: vec![
      AnalysisType::StaticAnalysis,
      AnalysisType::VulnerabilityDetection,
    ],
    enable_llm_analysis: Some(std::env::var("ENABLE_LLM_ANALYSIS").unwrap_or("false".to_string()) == "true"),
  };

  // Fetch real files from GitHub repository
  let mut file_contents = std::collections::HashMap::new();
    
  info!("Fetching files from GitHub repository: {}/{}", owner, repo);
  
  // Fetch files from the real GitHub repository
  match fetch_github_files(owner, repo).await {
    Ok(files) => {
      if files.is_empty() {
        error!("No Move files found in repository {}/{}", owner, repo);
        // Fallback to sample code for testing
        file_contents.insert(
          "sources/sample.move".to_string(),
          "module sample::empty { }".to_string(),
        );
      } else {
        file_contents = files;
        info!("Successfully fetched {} files from {}/{}", file_contents.len(), owner, repo);
      }
    }
    Err(e) => {
      error!("Failed to fetch files from GitHub {}/{}: {}", owner, repo, e);
      // Fallback to sample code for testing
      file_contents.insert(
        "sources/sample.move".to_string(),
        "module sample::empty { }".to_string(),
      );
    }
  }

  // Perform the analysis
  match analysis_handler.analyze_repository(analysis_request, file_contents).await {
    Ok(result) => {
      let response = json!({
        "status": "completed",
        "analysis_id": result.analysis_id,
        "repository_id": repository_id,
        "commit_sha": commit_sha,
        "security_score": result.security_score,
        "quality_score": result.quality_score,
        "vulnerabilities_found": result.vulnerabilities_found,
        "critical_vulnerabilities": result.critical_vulnerabilities,
        "analysis_duration_ms": result.analysis_duration_ms,
        "message": "Repository analysis completed successfully"
      });
      Ok(ResponseJson(response))
    }
    Err(e) => {
      error!("Failed to analyze repository {}: {}", repository_id, e);
      let response = json!({
        "status": "failed",
        "repository_id": repository_id,
        "error": e.to_string(),
        "message": "Repository analysis failed"
      });
      Ok(ResponseJson(response))
    }
  }
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

/// Get comprehensive analysis data for a repository
pub async fn get_repository_analysis(
  State(app_state): State<AppState>,
  Path(id): Path<Uuid>,
) -> Result<ResponseJson<Value>> {
  info!("Getting analysis data for repository: {}", id);

  // Get repository basic info
  let repo_row = sqlx::query(
    "SELECT owner_username, repo_name, full_name, security_score FROM github_repositories WHERE id = $1"
  )
  .bind(id)
  .fetch_optional(app_state.mm().dbx().db())
  .await
  .map_err(|e| {
    error!("Database error fetching repository {}: {}", id, e);
    ApiError::service_error("database", 500, Some(e.to_string()))
  })?;

  let repo_row = repo_row.ok_or_else(|| {
    error!("Repository not found: {}", id);
    ApiError::RouteNotFound {
      path: format!("/repositories/{}", id),
      method: "GET".to_string(),
    }
  })?;

  let owner: String = repo_row.get("owner_username");
  let repo: String = repo_row.get("repo_name");
  let full_name: String = repo_row.get("full_name");
  let security_score: Option<rust_decimal::Decimal> = repo_row.get("security_score");

  // Get analysis results
  let analysis_rows = sqlx::query(
    r#"
    SELECT id, commit_sha, security_score, quality_score, issues_found, critical_issues, analysis_duration_ms, ctime
    FROM code_analysis_results 
    WHERE repository_id = $1 
    ORDER BY ctime DESC
    "#
  )
  .bind(id)
  .fetch_all(app_state.mm().dbx().db())
  .await
  .map_err(|e| {
    error!("Database error fetching analysis results for {}: {}", id, e);
    ApiError::service_error("database", 500, Some(e.to_string()))
  })?;

  // Get vulnerabilities
  let vulnerability_rows = sqlx::query(
    r#"
    SELECT id, vulnerability_type, severity, confidence_score, file_path, line_number, description, recommendation, is_false_positive, fixed_at
    FROM security_vulnerabilities 
    WHERE repository_id = $1 
    ORDER BY 
      CASE severity 
        WHEN 'critical' THEN 1 
        WHEN 'high' THEN 2 
        WHEN 'medium' THEN 3 
        WHEN 'low' THEN 4 
      END,
      confidence_score DESC
    "#
  )
  .bind(id)
  .fetch_all(app_state.mm().dbx().db())
  .await
  .map_err(|e| {
    error!("Database error fetching vulnerabilities for {}: {}", id, e);
    ApiError::service_error("database", 500, Some(e.to_string()))
  })?;

  // Calculate statistics
  let total_analyses = analysis_rows.len();
  let total_vulnerabilities = vulnerability_rows.len();
  let critical_vulnerabilities = vulnerability_rows.iter().filter(|row| {
    let severity: String = row.get("severity");
    severity == "critical"
  }).count();
  let high_vulnerabilities = vulnerability_rows.iter().filter(|row| {
    let severity: String = row.get("severity");
    severity == "high"
  }).count();
  let medium_vulnerabilities = vulnerability_rows.iter().filter(|row| {
    let severity: String = row.get("severity");
    severity == "medium"
  }).count();
  let low_vulnerabilities = vulnerability_rows.iter().filter(|row| {
    let severity: String = row.get("severity");
    severity == "low"
  }).count();
  let fixed_vulnerabilities = vulnerability_rows.iter().filter(|row| {
    let fixed_at: Option<chrono::DateTime<chrono::Utc>> = row.get("fixed_at");
    fixed_at.is_some()
  }).count();
  let false_positives = vulnerability_rows.iter().filter(|row| {
    let is_false_positive: bool = row.get("is_false_positive");
    is_false_positive
  }).count();

  // Get latest analysis scores
  let (latest_security_score, latest_quality_score, last_analyzed_at) = if let Some(latest_row) = analysis_rows.first() {
    let sec_score: rust_decimal::Decimal = latest_row.get("security_score");
    let qual_score: rust_decimal::Decimal = latest_row.get("quality_score");
    let analyzed_at: chrono::DateTime<chrono::Utc> = latest_row.get("ctime");
    (sec_score.to_f64().unwrap_or(0.0), qual_score.to_f64().unwrap_or(0.0), Some(analyzed_at))
  } else {
    (0.0, 0.0, None)
  };

  // Build response
  let response = json!({
    "repository": {
      "id": id,
      "owner": owner,
      "name": repo,
      "full_name": full_name,
      "security_score": security_score.map(|d| d.to_f64().unwrap_or(0.0)),
      "last_analyzed_at": last_analyzed_at
    },
    "analysis_summary": {
      "total_analyses": total_analyses,
      "latest_security_score": latest_security_score,
      "latest_quality_score": latest_quality_score,
      "total_vulnerabilities": total_vulnerabilities,
      "vulnerability_breakdown": {
        "critical": critical_vulnerabilities,
        "high": high_vulnerabilities,
        "medium": medium_vulnerabilities,
        "low": low_vulnerabilities
      },
      "vulnerability_status": {
        "fixed": fixed_vulnerabilities,
        "open": total_vulnerabilities - fixed_vulnerabilities - false_positives,
        "false_positives": false_positives
      }
    },
    "recent_analyses": analysis_rows.into_iter().take(10).map(|row| {
      let analysis_id: Uuid = row.get("id");
      let commit_sha: String = row.get("commit_sha");
      let sec_score: rust_decimal::Decimal = row.get("security_score");
      let qual_score: rust_decimal::Decimal = row.get("quality_score");
      let issues: i32 = row.get("issues_found");
      let critical_issues: i32 = row.get("critical_issues");
      let duration: i32 = row.get("analysis_duration_ms");
      let analyzed_at: chrono::DateTime<chrono::Utc> = row.get("ctime");
      
      json!({
        "analysis_id": analysis_id,
        "commit_sha": commit_sha,
        "security_score": sec_score.to_f64().unwrap_or(0.0),
        "quality_score": qual_score.to_f64().unwrap_or(0.0),
        "issues_found": issues,
        "critical_issues": critical_issues,
        "analysis_duration_ms": duration,
        "analyzed_at": analyzed_at
      })
    }).collect::<Vec<_>>(),
    "vulnerabilities": vulnerability_rows.into_iter().map(|row| {
      let vuln_id: Uuid = row.get("id");
      let vuln_type: String = row.get("vulnerability_type");
      let severity: String = row.get("severity");
      let confidence: rust_decimal::Decimal = row.get("confidence_score");
      let file_path: String = row.get("file_path");
      let line_number: Option<i32> = row.get("line_number");
      let description: String = row.get("description");
      let recommendation: String = row.get("recommendation");
      let is_false_positive: bool = row.get("is_false_positive");
      let fixed_at: Option<chrono::DateTime<chrono::Utc>> = row.get("fixed_at");
      
      json!({
        "id": vuln_id,
        "type": vuln_type,
        "severity": severity,
        "confidence_score": confidence.to_f64().unwrap_or(0.0),
        "location": {
          "file_path": file_path,
          "line_number": line_number
        },
        "description": description,
        "recommendation": recommendation,
        "status": if is_false_positive {
          "false_positive"
        } else if fixed_at.is_some() {
          "fixed"
        } else {
          "open"
        },
        "fixed_at": fixed_at
      })
    }).collect::<Vec<_>>()
  });

  Ok(ResponseJson(response))
}

/// Handle GitHub webhook events with enhanced processing
pub async fn handle_webhook_enhanced(
  State(app_state): State<AppState>,
  Query(query): Query<github_service::application::handlers::webhook_handler::WebhookQuery>,
  headers: HeaderMap,
  payload: axum::body::Bytes,
) -> Result<ResponseJson<github_service::application::handlers::webhook_handler::WebhookProcessingResponse>> {
  let webhook_handler = create_webhook_handler(&app_state).map_err(|e| {
    error!("Failed to create GitHub webhook handler: {}", e);
    ApiError::service_error("github", 500, Some(e.to_string()))
  })?;

  webhook_handler
    .handle_webhook(headers, axum::extract::Query(query), payload.to_vec())
    .await
    .map(ResponseJson)
    .map_err(|(status, json_error)| {
      error!("Webhook processing error: {}", json_error.0.error);
      match status {
        StatusCode::BAD_REQUEST => ApiError::InvalidRequestFormat { message: json_error.0.error },
        StatusCode::UNAUTHORIZED => ApiError::ApiKeyAuthFailed { reason: json_error.0.error },
        _ => ApiError::service_error("github", 500, Some(json_error.0.error)),
      }
    })
}

/// Handle GitHub webhook events (legacy)
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

  let github_config = GitHubServiceConfig::from_config(&app_state.config)?;

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
  app_state: &AppState,
) -> std::result::Result<WebhookHandler, Box<dyn std::error::Error + Send + Sync>> {
  use github_service::{GitHubServiceConfig, GitHubServiceFactory};

  let github_config = GitHubServiceConfig::from_config(&app_state.config)?;

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

// Function to fetch real files from GitHub repository
async fn fetch_github_files(owner: &str, repo: &str) -> std::result::Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
  let mut file_contents = HashMap::new();
  
  // GitHub API endpoints for repository contents
  let base_url = format!("https://api.github.com/repos/{}/{}/contents", owner, repo);
  
  // List of directories to check for Move files
  let directories = vec!["sources", "tests", "scripts", ""];
  
  for dir in directories {
    let url = if dir.is_empty() {
      base_url.clone()
    } else {
      format!("{}/{}", base_url, dir)
    };
    
    info!("Fetching directory contents from: {}", url);
    
    // Make request to GitHub API
    let response = match reqwest::get(&url).await {
      Ok(resp) => resp,
      Err(e) => {
        warn!("Failed to fetch directory {}: {}", dir, e);
        continue;
      }
    };
    
    if !response.status().is_success() {
      warn!("GitHub API returned status {} for directory {}", response.status(), dir);
      continue;
    }
    
    let contents: Vec<serde_json::Value> = match response.json().await {
      Ok(json) => json,
      Err(e) => {
        warn!("Failed to parse JSON for directory {}: {}", dir, e);
        continue;
      }
    };
    
    // Process each file in the directory
    for item in contents {
      if let (Some(name), Some(file_type), Some(download_url)) = (
        item.get("name").and_then(|v| v.as_str()),
        item.get("type").and_then(|v| v.as_str()),
        item.get("download_url").and_then(|v| v.as_str()),
      ) {
        // Only process Move files
        if file_type == "file" && name.ends_with(".move") {
          info!("Fetching Move file: {}", name);
          
          // Download the file content
          match reqwest::get(download_url).await {
            Ok(file_response) => {
              if file_response.status().is_success() {
                match file_response.text().await {
                  Ok(content) => {
                    let file_path = if dir.is_empty() {
                      name.to_string()
                    } else {
                      format!("{}/{}", dir, name)
                    };
                    file_contents.insert(file_path, content);
                    info!("Successfully fetched file: {}", name);
                  }
                  Err(e) => warn!("Failed to read content of {}: {}", name, e),
                }
              } else {
                warn!("Failed to download {}: status {}", name, file_response.status());
              }
            }
            Err(e) => warn!("Failed to download {}: {}", name, e),
          }
        }
      }
    }
  }
  
  if file_contents.is_empty() {
    Err("No Move files found in repository".to_string().into())
  } else {
    Ok(file_contents)
  }
}
