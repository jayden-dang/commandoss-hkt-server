use crate::domain::{GitHubWebhookPayload, GitHubEventData, PushEvent, PullRequestEvent, ReleaseEvent, AnalysisJob, AnalysisType, AnalysisPriority, JobStatus};
use crate::error::{Error, Result};
use crate::infrastructure::{GitHubClient, GitHubFile, AnalysisQueueImpl, is_smart_contract_file};
use crate::models::WebhookResponse;
use axum::{
    extract::{State, Json, Query},
    http::{HeaderMap, StatusCode},
    response::Json as ResponseJson,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn, debug};
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
    ) -> Result<WebhookProcessingResponse, (StatusCode, Json<ErrorResponse>)> {
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

    pub async fn handle_github_webhook(
        &self,
        headers: HeaderMap,
        payload: GitHubWebhookPayload,
    ) -> Result<WebhookResponse> {
        // 1. Verify webhook signature
        let signature = headers
            .get("x-hub-signature-256")
            .and_then(|v| v.to_str().ok())
            .ok_or(Error::InvalidWebhookSignature)?;

        let payload_bytes = serde_json::to_vec(&payload)?;
        let is_valid = self.github_client
            .verify_webhook_signature(&payload_bytes, signature)
            ?;

        if !is_valid {
            return Err(Error::InvalidWebhookSignature);
        }

        // 2. Process webhook event based on event data
        let action = payload.action.as_deref().unwrap_or("unknown");
        
        match &payload.event_data {
            GitHubEventData::Push(push_event) => {
                self.handle_push_event(&payload, push_event).await?;
            },
            GitHubEventData::PullRequest(pr_event) => {
                self.handle_pull_request_event(&payload, pr_event).await?;
            },
            GitHubEventData::Release(release_event) => {
                self.handle_release_event(&payload, release_event).await?;
            },
            GitHubEventData::Other(_) => {
                info!("Ignoring webhook action: {}", action);
            }
        }

        Ok(WebhookResponse {
            status: "processed".to_string(),
            message: format!("Webhook {} processed successfully", action),
        })
    }

    async fn handle_push_event(
        &self,
        payload: &GitHubWebhookPayload,
        push_event: &PushEvent,
    ) -> Result<()> {
        info!(
            "Processing push event for repository {} with {} commits",
            payload.repository.full_name,
            push_event.commits.len()
        );

        for commit in &push_event.commits {
            // Check if commit contains smart contract files
            let smart_contract_files: Vec<_> = commit.added.iter()
                .chain(commit.modified.iter())
                .filter(|file| is_smart_contract_file(file))
                .collect();

            if !smart_contract_files.is_empty() {
                // Queue code analysis for this commit
                let analysis_job = AnalysisJob {
                    id: Uuid::new_v4(), // Will be overwritten by queue
                    repository_id: payload.repository.id,
                    commit_sha: commit.id.clone(),
                    files_to_analyze: smart_contract_files.iter().map(|s| s.to_string()).collect(),
                    analysis_type: AnalysisType::SmartContract,
                    priority: AnalysisPriority::Normal,
                    created_at: chrono::Utc::now(),
                    status: JobStatus::Queued,
                };

                let job_id = self.analysis_queue.enqueue(analysis_job).await?;

                info!(
                    "Queued analysis job {} for commit {} in repository {}",
                    job_id, commit.id, payload.repository.full_name
                );
            }
        }

        Ok(())
    }

    async fn handle_pull_request_event(
        &self,
        payload: &GitHubWebhookPayload,
        pr_event: &PullRequestEvent,
    ) -> Result<()> {
        match pr_event.action.as_str() {
            "opened" | "synchronize" | "reopened" => {
                info!(
                    "Processing pull request {} event for PR #{} in repository {}",
                    pr_event.action,
                    pr_event.pull_request.number,
                    payload.repository.full_name
                );

                // Queue analysis for the PR
                let analysis_job = AnalysisJob {
                    id: Uuid::new_v4(),
                    repository_id: payload.repository.id,
                    commit_sha: pr_event.pull_request.head.sha.clone(),
                    files_to_analyze: vec![], // Will be determined during analysis
                    analysis_type: AnalysisType::SecurityFocus,
                    priority: AnalysisPriority::High, // PRs get higher priority
                    created_at: chrono::Utc::now(),
                    status: JobStatus::Queued,
                };

                let job_id = self.analysis_queue.enqueue(analysis_job).await?;

                info!(
                    "Queued security analysis job {} for PR #{} in repository {}",
                    job_id, pr_event.pull_request.number, payload.repository.full_name
                );
            },
            _ => {
                info!("Ignoring pull request action: {}", pr_event.action);
            }
        }

        Ok(())
    }

    async fn handle_release_event(
        &self,
        payload: &GitHubWebhookPayload,
        release_event: &ReleaseEvent,
    ) -> Result<()> {
        if release_event.action == "published" {
            info!(
                "Processing release {} for repository {}",
                release_event.release.tag_name,
                payload.repository.full_name
            );

            // Queue comprehensive analysis for the release
            let analysis_job = AnalysisJob {
                id: Uuid::new_v4(),
                repository_id: payload.repository.id,
                commit_sha: release_event.release.target_commitish.clone(),
                files_to_analyze: vec![],
                analysis_type: AnalysisType::FullAnalysis,
                priority: AnalysisPriority::High,
                created_at: chrono::Utc::now(),
                status: JobStatus::Queued,
            };

            let job_id = self.analysis_queue.enqueue(analysis_job).await?;

            info!(
                "Queued full analysis job {} for release {} in repository {}",
                job_id, release_event.release.tag_name, payload.repository.full_name
            );
        }

        Ok(())
    }
}

// Axum handler function
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