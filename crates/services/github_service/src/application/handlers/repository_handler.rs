use crate::domain::{AnalysisJob, AnalysisType, AnalysisPriority, JobStatus};
use crate::error::{Error, Result};
use crate::infrastructure::{GitHubClient, AnalysisQueueImpl, check_repository_for_smart_contracts};
use crate::models::{
    AddRepositoryRequest, UpdateRepositorySettingsRequest, RepositoryListParams,
    RepositoryResponse, RepositoryListResponse, RepositoryDetailResponse, RepositoryFilters,
    VulnerabilitySummary,
};
use axum::{
    extract::{Path, Query, State, Json},
    http::StatusCode,
    response::Json as ResponseJson,
};
use jd_domain::zkpersona_domain::developer_models::{GitHubRepositoryForCreate, GitHubRepositoryForUpdate};
use jd_storage::repository::{developer_repositories::GitHubRepositoryRepository, Repository};
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct RepositoryHandler {
    github_client: Arc<GitHubClient>,
    analysis_queue: Arc<AnalysisQueueImpl>,
    repository_repo: Arc<GitHubRepositoryRepository>,
    webhook_base_url: String,
}

impl RepositoryHandler {
    pub fn new(
        github_client: Arc<GitHubClient>,
        analysis_queue: Arc<AnalysisQueueImpl>,
        repository_repo: Arc<GitHubRepositoryRepository>,
        webhook_base_url: String,
    ) -> Self {
        Self {
            github_client,
            analysis_queue,
            repository_repo,
            webhook_base_url,
        }
    }

    pub async fn list_repositories(
        &self,
        params: RepositoryListParams,
    ) -> Result<RepositoryListResponse> {
        let _filters = RepositoryFilters {
            language: params.language,
            security_score_min: params.security_score_min,
            monitoring_enabled: params.monitoring_enabled,
            search_term: params.search,
        };

        let limit = params.limit.unwrap_or(20);
        let offset = params.offset.unwrap_or(0);

        // TODO: Implement filtering logic in repository  
        let repositories = self.repository_repo
            .find_all()
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;

        let total_count = repositories.len() as i64;

        Ok(RepositoryListResponse {
            repositories,
            total_count,
            limit,
            offset,
        })
    }

    pub async fn add_repository(
        &self,
        request: AddRepositoryRequest,
    ) -> Result<RepositoryResponse> {
        info!("Adding repository {}/{}", request.owner, request.name);

        // 1. Fetch repository from GitHub
        let github_repo = self.github_client
            .get_repository(&request.owner, &request.name)
            .await?;

        // 2. Check if repository contains smart contracts
        let has_smart_contracts = check_repository_for_smart_contracts(
            &self.github_client,
            &request.owner,
            &request.name
        ).await?;

        if !has_smart_contracts {
            return Err(Error::NoSmartContractsFound);
        }

        // 3. Create repository record in database
        let new_repo = GitHubRepositoryForCreate {
            github_repo_id: github_repo.id as i64,
            owner_username: github_repo.owner.login.clone(),
            repo_name: github_repo.name.clone(),
            description: github_repo.description.clone(),
            primary_language: github_repo.language.clone(),
            is_private: false, // Assume public repositories for now
            star_count: Some(github_repo.stargazers_count as i32),
            fork_count: Some(github_repo.forks_count as i32),
            webhook_secret: None,
            monitoring_enabled: Some(true),
        };

        let repository = self.repository_repo
            .create(new_repo)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;

        // 4. Setup webhook for repository
        let webhook_url = format!("{}/api/v1/webhooks/github", self.webhook_base_url);
        let _webhook = self.github_client
            .create_webhook(&request.owner, &request.name, &webhook_url)
            .await
            .map_err(|e| {
                warn!("Failed to create webhook for {}/{}: {}", request.owner, request.name, e);
                e
            })?;

        // 5. Queue initial security analysis
        let analysis_job = AnalysisJob {
            id: Uuid::new_v4(),
            repository_id: github_repo.id,
            commit_sha: github_repo.default_branch.clone(),
            files_to_analyze: vec![], // Will scan all smart contract files
            analysis_type: AnalysisType::InitialScan,
            priority: AnalysisPriority::High,
            created_at: chrono::Utc::now(),
            status: JobStatus::Queued,
        };

        let job_id = self.analysis_queue.enqueue(analysis_job).await?;

        info!(
            "Successfully added repository {} with initial analysis job {}",
            repository.full_name, job_id
        );

        Ok(RepositoryResponse {
            repository,
            webhook_configured: true,
            initial_scan_queued: true,
        })
    }

    pub async fn get_repository(&self, id: Uuid) -> Result<RepositoryDetailResponse> {
        let repository = self.repository_repo
            .find_by_id(id.into())
            .await
            .map_err(|e| Error::Internal(e.to_string()))?
            .ok_or_else(|| Error::RepositoryNotFound { 
                owner: "unknown".to_string(), 
                repo: id.to_string() 
            })?;

        // TODO: Get latest security analysis from analysis service
        let latest_analysis = None; // Placeholder

        // TODO: Get vulnerability summary from vulnerability service
        let vulnerability_summary = VulnerabilitySummary {
            total_vulnerabilities: 0,
            critical_count: 0,
            high_count: 0,
            medium_count: 0,
            low_count: 0,
        };

        // TODO: Get security trends
        let security_trends = vec![];

        Ok(RepositoryDetailResponse {
            repository,
            latest_analysis,
            vulnerability_summary,
            security_trends,
        })
    }

    pub async fn update_repository_settings(
        &self,
        id: Uuid,
        request: UpdateRepositorySettingsRequest,
    ) -> Result<RepositoryResponse> {
        let _update_dto = GitHubRepositoryForUpdate {
            description: None,
            primary_language: None,
            is_private: None,
            star_count: None,
            fork_count: None,
            security_score: None,
            last_analyzed_at: None,
            webhook_secret: request.webhook_secret,
            monitoring_enabled: request.monitoring_enabled,
        };

        // For now, we'll fetch the existing repository and create a new one with updates
        // In a real implementation, we'd have an update method
        let existing = self.repository_repo
            .find_by_id(id.into())
            .await
            .map_err(|e| Error::Internal(e.to_string()))?
            .ok_or_else(|| Error::RepositoryNotFound { 
                owner: "unknown".to_string(), 
                repo: id.to_string() 
            })?;

        // TODO: Implement proper update method
        let repository = existing;

        Ok(RepositoryResponse {
            repository,
            webhook_configured: true,
            initial_scan_queued: false,
        })
    }
}

// Axum handler functions
pub async fn list_repositories(
    State(handler): State<Arc<RepositoryHandler>>,
    Query(params): Query<RepositoryListParams>,
) -> std::result::Result<ResponseJson<RepositoryListResponse>, StatusCode> {
    match handler.list_repositories(params).await {
        Ok(response) => Ok(ResponseJson(response)),
        Err(e) => {
            error!("Failed to list repositories: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn add_repository(
    State(handler): State<Arc<RepositoryHandler>>,
    Json(request): Json<AddRepositoryRequest>,
) -> std::result::Result<ResponseJson<RepositoryResponse>, StatusCode> {
    match handler.add_repository(request).await {
        Ok(response) => Ok(ResponseJson(response)),
        Err(Error::RepositoryNotFound { .. }) => Err(StatusCode::NOT_FOUND),
        Err(Error::NoSmartContractsFound) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        Err(Error::GitHubApi(_)) => Err(StatusCode::BAD_GATEWAY),
        Err(e) => {
            error!("Failed to add repository: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_repository(
    State(handler): State<Arc<RepositoryHandler>>,
    Path(id): Path<Uuid>,
) -> std::result::Result<ResponseJson<RepositoryDetailResponse>, StatusCode> {
    match handler.get_repository(id).await {
        Ok(response) => Ok(ResponseJson(response)),
        Err(Error::RepositoryNotFound { .. }) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get repository {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_repository_settings(
    State(handler): State<Arc<RepositoryHandler>>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateRepositorySettingsRequest>,
) -> std::result::Result<ResponseJson<RepositoryResponse>, StatusCode> {
    match handler.update_repository_settings(id, request).await {
        Ok(response) => Ok(ResponseJson(response)),
        Err(Error::RepositoryNotFound { .. }) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to update repository {} settings: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}