use crate::domain::{AnalysisJob, AnalysisType, AnalysisPriority, JobStatus};
use crate::error::{Error, Result};
use crate::infrastructure::{GitHubClient, AnalysisQueueImpl, check_repository_for_smart_contracts};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

pub struct GitHubUseCase {
    github_client: Arc<GitHubClient>,
    analysis_queue: Arc<AnalysisQueueImpl>,
}

impl GitHubUseCase {
    pub fn new(
        github_client: Arc<GitHubClient>,
        analysis_queue: Arc<AnalysisQueueImpl>,
    ) -> Self {
        Self {
            github_client,
            analysis_queue,
        }
    }

    pub async fn discover_repositories(&self, username: &str) -> Result<Vec<String>> {
        info!("Discovering repositories for user: {}", username);

        let repositories = self.github_client
            .list_user_repositories(username)
            .await?;

        let mut smart_contract_repos = Vec::new();

        for repo in repositories {
            let owner_login = &repo.owner.login;
            let repo_name = &repo.name;

            match check_repository_for_smart_contracts(&self.github_client, owner_login, repo_name).await {
                Ok(true) => {
                    smart_contract_repos.push(repo.full_name);
                },
                Ok(false) => {
                    // Skip repositories without smart contracts
                },
                Err(e) => {
                    warn!("Failed to check repository {} for smart contracts: {}", repo.full_name, e);
                }
            }
        }

        info!("Found {} repositories with smart contracts for user {}", smart_contract_repos.len(), username);
        Ok(smart_contract_repos)
    }

    pub async fn queue_repository_analysis(&self, repository_id: u64, analysis_type: AnalysisType) -> Result<Uuid> {
        let analysis_job = AnalysisJob {
            id: Uuid::new_v4(),
            repository_id,
            commit_sha: "HEAD".to_string(), // Will be resolved during analysis
            files_to_analyze: vec![],
            analysis_type,
            priority: match analysis_type {
                AnalysisType::InitialScan => AnalysisPriority::High,
                AnalysisType::SecurityFocus => AnalysisPriority::Critical,
                AnalysisType::SmartContract => AnalysisPriority::Normal,
                AnalysisType::FullAnalysis => AnalysisPriority::High,
            },
            created_at: chrono::Utc::now(),
            status: JobStatus::Queued,
        };

        let job_id = self.analysis_queue.enqueue(analysis_job).await?;
        info!("Queued analysis job {} for repository {} with type {:?}", job_id, repository_id, analysis_type);
        
        Ok(job_id)
    }

    pub async fn get_analysis_status(&self, job_id: Uuid) -> Result<Option<JobStatus>> {
        Ok(self.analysis_queue.get_job_status(job_id).await)
    }

    pub async fn cancel_analysis(&self, job_id: Uuid) -> Result<()> {
        self.analysis_queue.cancel_job(job_id).await
    }

    pub async fn get_queue_metrics(&self) -> Result<crate::domain::QueueStatus> {
        Ok(self.analysis_queue.get_queue_status().await)
    }

    pub async fn validate_repository_access(&self, owner: &str, repo: &str) -> Result<bool> {
        match self.github_client.get_repository(owner, repo).await {
            Ok(_) => Ok(true),
            Err(Error::RepositoryNotFound { .. }) => Ok(false),
            Err(Error::GitHubApi(msg)) if msg.contains("403") => Ok(false), // Forbidden
            Err(e) => Err(e),
        }
    }

    pub async fn sync_repository_metadata(&self, owner: &str, repo: &str) -> Result<crate::domain::GitHubRepository> {
        let github_repo = self.github_client.get_repository(owner, repo).await?;
        
        info!("Synced metadata for repository {}/{}", owner, repo);
        Ok(github_repo)
    }
}