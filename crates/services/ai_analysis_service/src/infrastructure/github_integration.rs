use crate::application::use_cases::analysis_use_cases::AnalysisUseCases;
use crate::domain::analysis_models::AnalysisType;
use crate::error::{Error, Result};
use crate::models::requests::AnalyzeRepositoryRequest;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct GitHubIntegrationService {
    analysis_use_cases: Arc<AnalysisUseCases>,
    github_client: Option<Arc<dyn GitHubFileProvider>>,
}

#[async_trait::async_trait]
pub trait GitHubFileProvider: Send + Sync {
    async fn get_repository_files(&self, owner: &str, repo: &str, commit_sha: Option<&str>) -> Result<HashMap<String, String>>;
    async fn get_file_content(&self, owner: &str, repo: &str, file_path: &str, commit_sha: Option<&str>) -> Result<String>;
}

impl GitHubIntegrationService {
    pub fn new(analysis_use_cases: Arc<AnalysisUseCases>) -> Self {
        Self {
            analysis_use_cases,
            github_client: None,
        }
    }

    pub fn with_github_client(mut self, github_client: Arc<dyn GitHubFileProvider>) -> Self {
        self.github_client = Some(github_client);
        self
    }

    /// Automatically analyze a repository when it's added to the system
    pub async fn auto_analyze_repository(
        &self,
        repository_id: Uuid,
        owner: &str,
        repo_name: &str,
        commit_sha: Option<&str>,
    ) -> Result<crate::models::responses::AnalysisResponse> {
        info!("Starting auto-analysis for repository: {}/{}", owner, repo_name);

        // Get repository files
        let file_contents = if let Some(github_client) = &self.github_client {
            github_client.get_repository_files(owner, repo_name, commit_sha).await?
        } else {
            // Fallback: create empty map (analysis will fail gracefully)
            warn!("No GitHub client configured, using empty file contents for analysis");
            HashMap::new()
        };

        // Filter to only Move files for initial analysis
        let move_files: HashMap<String, String> = file_contents
            .into_iter()
            .filter(|(path, _)| path.ends_with(".move"))
            .collect();

        if move_files.is_empty() {
            return Err(Error::AnalysisFailed {
                message: "No Move files found in repository for analysis".to_string(),
            });
        }

        info!("Found {} Move files for analysis", move_files.len());

        // Create analysis request with default analysis types
        let analysis_request = AnalyzeRepositoryRequest {
            repository_id,
            commit_sha: commit_sha.unwrap_or("HEAD").to_string(),
            files_to_analyze: None, // Analyze all Move files
            analysis_types: vec![
                AnalysisType::StaticAnalysis,
                AnalysisType::VulnerabilityDetection,
            ],
            enable_llm_analysis: Some(true), // Enable LLM if available
        };

        // Run analysis
        let analysis_result = self.analysis_use_cases
            .analyze_repository(analysis_request, move_files)
            .await?;

        info!(
            "Auto-analysis completed for repository: {}/{} (ID: {}) - Security Score: {:.1}, {} vulnerabilities found",
            owner, repo_name, repository_id, analysis_result.security_score, analysis_result.vulnerabilities_found
        );

        Ok(analysis_result)
    }

    /// Analyze repository on webhook events (push, pull request)
    pub async fn analyze_on_webhook(
        &self,
        repository_id: Uuid,
        owner: &str,
        repo_name: &str,
        commit_sha: &str,
        changed_files: Vec<String>,
    ) -> Result<Option<crate::models::responses::AnalysisResponse>> {
        info!("Webhook triggered analysis for repository: {}/{} at commit: {}", owner, repo_name, commit_sha);

        // Filter to only Move files that were changed
        let move_files_changed: Vec<String> = changed_files
            .into_iter()
            .filter(|path| path.ends_with(".move"))
            .collect();

        if move_files_changed.is_empty() {
            info!("No Move files changed, skipping analysis");
            return Ok(None);
        }

        // Get content for changed Move files
        let mut file_contents = HashMap::new();
        if let Some(github_client) = &self.github_client {
            for file_path in &move_files_changed {
                match github_client.get_file_content(owner, repo_name, file_path, Some(commit_sha)).await {
                    Ok(content) => {
                        file_contents.insert(file_path.clone(), content);
                    }
                    Err(e) => {
                        warn!("Failed to get content for file {}: {}", file_path, e);
                        // Continue with other files
                    }
                }
            }
        }

        if file_contents.is_empty() {
            return Err(Error::AnalysisFailed {
                message: "Failed to retrieve content for changed Move files".to_string(),
            });
        }

        // Create incremental analysis request
        let analysis_request = AnalyzeRepositoryRequest {
            repository_id,
            commit_sha: commit_sha.to_string(),
            files_to_analyze: Some(move_files_changed),
            analysis_types: vec![
                AnalysisType::StaticAnalysis,
                AnalysisType::VulnerabilityDetection,
            ],
            enable_llm_analysis: Some(false), // Disable LLM for webhook analysis to save costs
        };

        let analysis_result = self.analysis_use_cases
            .analyze_repository(analysis_request, file_contents)
            .await?;

        info!(
            "Webhook analysis completed for repository: {}/{} - Security Score: {:.1}, {} vulnerabilities found",
            owner, repo_name, analysis_result.security_score, analysis_result.vulnerabilities_found
        );

        Ok(Some(analysis_result))
    }

    /// Get analysis status for a repository
    pub async fn get_repository_analysis_status(
        &self,
        repository_id: Uuid,
    ) -> Result<crate::models::responses::AnalysisStatusResponse> {
        self.analysis_use_cases.get_analysis_status(repository_id).await
    }

    /// Schedule periodic re-analysis of a repository
    pub async fn schedule_periodic_analysis(
        &self,
        repository_id: Uuid,
        owner: &str,
        repo_name: &str,
    ) -> Result<()> {
        info!("Scheduling periodic analysis for repository: {}/{}", owner, repo_name);
        
        // TODO: Implement actual scheduling logic (e.g., using a job queue)
        // For now, just trigger immediate analysis
        match self.auto_analyze_repository(repository_id, owner, repo_name, None).await {
            Ok(_) => {
                info!("Periodic analysis completed for repository: {}/{}", owner, repo_name);
                Ok(())
            }
            Err(e) => {
                error!("Periodic analysis failed for repository: {}/{} - {}", owner, repo_name, e);
                Err(e)
            }
        }
    }

    /// Update repository security score based on latest analysis
    pub async fn update_repository_security_score(
        &self,
        repository_id: Uuid,
    ) -> Result<f64> {
        let status = self.analysis_use_cases.get_analysis_status(repository_id).await?;
        
        if let Some(latest_analysis) = status.latest_analysis {
            info!(
                "Repository {} security score: {:.1}",
                repository_id, latest_analysis.security_score
            );
            Ok(latest_analysis.security_score)
        } else {
            warn!("No analysis found for repository {}", repository_id);
            Ok(0.0) // Default score if no analysis exists
        }
    }
}

// Mock implementation for testing
pub struct MockGitHubFileProvider;

#[async_trait::async_trait]
impl GitHubFileProvider for MockGitHubFileProvider {
    async fn get_repository_files(&self, _owner: &str, _repo: &str, _commit_sha: Option<&str>) -> Result<HashMap<String, String>> {
        // Return mock Move file for testing
        let mut files = HashMap::new();
        files.insert(
            "sources/test.move".to_string(),
            r#"
module test::example {
    use std::vector;
    
    public fun transfer(item: Item, recipient: address) {
        // Missing capability check - vulnerability
        transfer::public_transfer(item, recipient);
    }
    
    public fun unsafe_math(a: u64, b: u64): u64 {
        // Potential overflow - vulnerability
        a + b * 2
    }
}
"#.to_string(),
        );
        Ok(files)
    }

    async fn get_file_content(&self, _owner: &str, _repo: &str, file_path: &str, _commit_sha: Option<&str>) -> Result<String> {
        // Return mock content based on file path
        if file_path.ends_with(".move") {
            Ok(r#"
module test::example {
    public fun example() {
        // Example Move code
    }
}
"#.to_string())
        } else {
            Err(Error::AnalysisFailed {
                message: "File not found".to_string(),
            })
        }
    }
}