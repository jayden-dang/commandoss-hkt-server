use crate::domain::{GitHubWebhookPayload, GitHubEventData, AnalysisJob, AnalysisType, AnalysisPriority, JobStatus};
use crate::error::{Error, Result};
use crate::infrastructure::{GitHubClient, GitHubFile, AnalysisQueueImpl};
use crate::models::WebhookResponse;
use axum::{
    extract::{State, Json, Query},
    http::{HeaderMap, StatusCode},
    response::Json as ResponseJson,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, debug};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct WebhookQuery {
    pub installation_id: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct WebhookProcessingResponse {
    pub received: bool,
    pub processing_id: Uuid,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct AnalysisJobRequest {
    pub id: Uuid,
    pub repository_id: String,
    pub repository_name: String,
    pub commit_sha: String,
    pub branch: String,
    pub files: Vec<GitHubFile>,
    pub priority: AnalysisPriority,
    pub status: JobStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub installation_id: u64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub struct WebhookHandler {
    github_client: Arc<GitHubClient>,
    analysis_queue: Arc<AnalysisQueueImpl>,
}

impl WebhookHandler {
    pub fn new(github_client: Arc<GitHubClient>, analysis_queue: Arc<AnalysisQueueImpl>) -> Self {
        Self {
            github_client,
            analysis_queue,
        }
    }

    pub async fn handle_webhook(
        &self,
        headers: HeaderMap,
        query: Query<WebhookQuery>,
        payload: Vec<u8>,
    ) -> std::result::Result<WebhookProcessingResponse, (StatusCode, Json<ErrorResponse>)> {
        info!("Received GitHub webhook");

        // Verify webhook signature
        let signature = headers
            .get("X-Hub-Signature-256")
            .or_else(|| headers.get("X-Hub-Signature"))
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                error!("Missing webhook signature");
                (StatusCode::BAD_REQUEST, Json(ErrorResponse {
                    error: "Missing webhook signature".to_string(),
                }))
            })?;

        if !self.github_client.verify_webhook_signature(&payload, signature)
            .map_err(|e| {
                error!("Webhook signature verification failed: {}", e);
                (StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                    error: "Invalid webhook signature".to_string(),
                }))
            })? {
            return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "Invalid webhook signature".to_string(),
            })));
        }

        // Parse webhook payload
        let webhook_payload: GitHubWebhookPayload = serde_json::from_slice(&payload)
            .map_err(|e| {
                error!("Failed to parse webhook payload: {}", e);
                (StatusCode::BAD_REQUEST, Json(ErrorResponse {
                    error: "Invalid JSON payload".to_string(),
                }))
            })?;

        // Get event type from headers
        let event_type = headers
            .get("X-GitHub-Event")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");

        info!(
            "Processing {} event for repository: {}",
            event_type,
            webhook_payload.repository.full_name
        );

        // Process the webhook event
        let processing_id = self.process_webhook_event(event_type, &webhook_payload, &query).await
            .map_err(|e| {
                error!("Failed to process webhook event: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                    error: "Failed to process webhook event".to_string(),
                }))
            })?;

        Ok(WebhookProcessingResponse {
            received: true,
            processing_id,
            message: format!("Webhook processed successfully for {}", event_type),
        })
    }

    async fn process_webhook_event(
        &self,
        event_type: &str,
        payload: &GitHubWebhookPayload,
        query: &Query<WebhookQuery>,
    ) -> Result<Uuid> {
        match event_type {
            "push" => self.handle_push_event(payload, query).await,
            "pull_request" => self.handle_pull_request_event(payload, query).await,
            "repository" => self.handle_repository_event(payload, query).await,
            _ => {
                info!("Ignoring event type: {}", event_type);
                Ok(Uuid::new_v4()) // Return dummy ID for unsupported events
            }
        }
    }

    async fn handle_push_event(
        &self,
        payload: &GitHubWebhookPayload,
        query: &Query<WebhookQuery>,
    ) -> Result<Uuid> {
        info!("Processing push event for {}", payload.repository.full_name);

        let push_event = match &payload.event_data {
            GitHubEventData::Push(push) => push,
            _ => return Ok(Uuid::new_v4()),
        };

        // Check if we have commits
        if push_event.commits.is_empty() {
            debug!("No commits in push event, skipping analysis");
            return Ok(Uuid::new_v4());
        }

        // Get the latest commit
        let head_commit = push_event.head_commit.as_ref()
            .or_else(|| push_event.commits.last())
            .ok_or_else(|| Error::Internal("No head commit found".to_string()))?;

        // Check if any smart contract files were modified
        let mut smart_contract_files_changed = false;
        let smart_contract_extensions = [".sol", ".rs", ".move", ".vy"];

        for file_path in &head_commit.added {
            if smart_contract_extensions.iter().any(|ext| file_path.ends_with(ext)) {
                smart_contract_files_changed = true;
                break;
            }
        }

        for file_path in &head_commit.modified {
            if smart_contract_extensions.iter().any(|ext| file_path.ends_with(ext)) {
                smart_contract_files_changed = true;
                break;
            }
        }

        if !smart_contract_files_changed {
            info!("No smart contract files changed, skipping analysis");
            return Ok(Uuid::new_v4());
        }

        // Get installation ID
        let installation_id = query.installation_id
            .ok_or_else(|| Error::Internal("Installation ID not found".to_string()))?;

        // Fetch smart contract files
        let owner_parts: Vec<&str> = payload.repository.full_name.split('/').collect();
        if owner_parts.len() != 2 {
            return Err(Error::Internal("Invalid repository full name".to_string()));
        }
        let owner = owner_parts[0];
        let repo = owner_parts[1];
        let commit_sha = &head_commit.id;

        let files = self.github_client.get_repository_files(
            installation_id,
            owner,
            repo,
            Some(commit_sha),
            &smart_contract_extensions,
        ).await?;

        let smart_contract_files = self.github_client.detect_smart_contract_files(&files);

        if smart_contract_files.is_empty() {
            info!("No smart contract files found, skipping analysis");
            return Ok(Uuid::new_v4());
        }

        info!("Found {} smart contract files for analysis", smart_contract_files.len());

        // Create analysis job
        let analysis_job = AnalysisJobRequest {
            id: Uuid::new_v4(),
            repository_id: payload.repository.id.to_string(),
            repository_name: payload.repository.full_name.clone(),
            commit_sha: commit_sha.clone(),
            branch: payload.repository.default_branch.clone(),
            files: smart_contract_files.iter().map(|f| (*f).clone()).collect(),
            priority: self.determine_analysis_priority(&smart_contract_files),
            status: JobStatus::Queued,
            created_at: chrono::Utc::now(),
            installation_id,
        };

        // Convert to domain AnalysisJob
        let domain_job = AnalysisJob {
            id: analysis_job.id,
            repository_id: payload.repository.id,
            commit_sha: analysis_job.commit_sha.clone(),
            files_to_analyze: analysis_job.files.iter().map(|f| f.path.clone()).collect(),
            analysis_type: AnalysisType::SmartContract,
            priority: analysis_job.priority,
            created_at: analysis_job.created_at,
            status: analysis_job.status,
        };

        // Queue the analysis job
        let job_id = self.analysis_queue.enqueue(domain_job).await?;

        info!(
            "Queued analysis job {} for repository {}",
            job_id,
            analysis_job.repository_name
        );

        Ok(job_id)
    }

    async fn handle_pull_request_event(
        &self,
        payload: &GitHubWebhookPayload,
        query: &Query<WebhookQuery>,
    ) -> Result<Uuid> {
        let pr_event = match &payload.event_data {
            GitHubEventData::PullRequest(pr) => pr,
            _ => return Ok(Uuid::new_v4()),
        };

        info!("Processing pull request event: {} for {}", pr_event.number, payload.repository.full_name);

        let installation_id = query.installation_id
            .ok_or_else(|| Error::Internal("Installation ID not found".to_string()))?;

        let owner_parts: Vec<&str> = payload.repository.full_name.split('/').collect();
        if owner_parts.len() != 2 {
            return Err(Error::Internal("Invalid repository full name".to_string()));
        }
        let owner = owner_parts[0];
        let repo = owner_parts[1];

        let files = self.github_client.get_repository_files(
            installation_id,
            owner,
            repo,
            Some(&pr_event.pull_request.head.sha),
            &[".sol", ".rs", ".move", ".vy"],
        ).await?;

        let smart_contract_files = self.github_client.detect_smart_contract_files(&files);

        if !smart_contract_files.is_empty() {
            let analysis_job = AnalysisJob {
                id: Uuid::new_v4(),
                repository_id: payload.repository.id,
                commit_sha: pr_event.pull_request.head.sha.clone(),
                files_to_analyze: smart_contract_files.iter().map(|f| f.path.clone()).collect(),
                analysis_type: AnalysisType::SecurityFocus,
                priority: AnalysisPriority::High, // PRs get higher priority
                created_at: chrono::Utc::now(),
                status: JobStatus::Queued,
            };

            let job_id = self.analysis_queue.enqueue(analysis_job).await?;
            return Ok(job_id);
        }

        Ok(Uuid::new_v4())
    }

    async fn handle_repository_event(
        &self,
        payload: &GitHubWebhookPayload,
        _query: &Query<WebhookQuery>,
    ) -> Result<Uuid> {
        info!("Processing repository event for {}", payload.repository.full_name);

        if let Some(action) = &payload.action {
            info!("Repository action: {} for {}", action, payload.repository.full_name);
        }

        Ok(Uuid::new_v4())
    }

    fn determine_analysis_priority(&self, files: &[&GitHubFile]) -> AnalysisPriority {
        // Prioritize based on file types and content
        for file in files {
            let path_lower = file.path.to_lowercase();
            let content_lower = file.content.to_lowercase();

            // High priority for certain patterns
            if path_lower.contains("token") ||
               path_lower.contains("vault") ||
               path_lower.contains("finance") ||
               content_lower.contains("payable") ||
               content_lower.contains("transfer") ||
               content_lower.contains("approve") {
                return AnalysisPriority::High;
            }
        }

        AnalysisPriority::Normal
    }

    // Legacy compatibility method
    pub async fn handle_github_webhook(
        &self,
        headers: HeaderMap,
        payload: GitHubWebhookPayload,
    ) -> Result<WebhookResponse> {
        // Legacy method for compatibility
        let query = Query(WebhookQuery { installation_id: None });
        let payload_bytes = serde_json::to_vec(&payload)
            .map_err(|e| Error::Internal(format!("Serialization error: {}", e)))?;
        
        match self.handle_webhook(headers, query, payload_bytes).await {
            Ok(response) => Ok(WebhookResponse {
                status: "processed".to_string(),
                message: response.message,
            }),
            Err((_, json_error)) => Err(Error::Internal(json_error.0.error)),
        }
    }
}

// Axum handler function
pub async fn handle_webhook_endpoint(
    State(webhook_handler): State<Arc<WebhookHandler>>,
    Query(query): Query<WebhookQuery>,
    headers: HeaderMap,
    payload: axum::body::Bytes,
) -> std::result::Result<ResponseJson<WebhookProcessingResponse>, (StatusCode, Json<ErrorResponse>)> {
    match webhook_handler.handle_webhook(headers, Query(query), payload.to_vec()).await {
        Ok(response) => Ok(ResponseJson(response)),
        Err((status, error)) => Err((status, error)),
    }
}

// Legacy handler for compatibility
pub async fn handle_github_webhook(
    State(webhook_handler): State<Arc<WebhookHandler>>,
    headers: HeaderMap,
    Json(payload): Json<GitHubWebhookPayload>,
) -> std::result::Result<ResponseJson<WebhookResponse>, StatusCode> {
    match webhook_handler.handle_github_webhook(headers, payload).await {
        Ok(response) => Ok(ResponseJson(response)),
        Err(Error::InvalidWebhookSignature) => {
            error!("Invalid webhook signature received");
            Err(StatusCode::UNAUTHORIZED)
        },
        Err(Error::WebhookPayloadError(msg)) => {
            error!("Webhook payload error: {}", msg);
            Err(StatusCode::BAD_REQUEST)
        },
        Err(e) => {
            error!("Webhook processing error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}