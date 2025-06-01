use crate::domain::analysis_engine::AnalysisEngine;
use crate::domain::analysis_models::{AnalysisRequest, AnalysisResult, AnalysisType};
use crate::domain::analysis_repository_trait::AnalysisRepository;
use crate::domain::llm_provider_trait::LLMProvider;
use crate::error::{Error, Result};
use crate::models::requests::{AnalyzeRepositoryRequest, AnalyzeCodeRequest, MarkVulnerabilityRequest, VulnerabilityAction};
use crate::models::responses::{AnalysisResponse, DetailedAnalysisResponse, CodeAnalysisResponse, AnalysisStatusResponse};
use crate::domain::analysis_repository_trait::VulnerabilityStatistics;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn, error};

pub struct AnalysisUseCases {
    analysis_engine: AnalysisEngine,
    analysis_repository: Arc<dyn AnalysisRepository>,
}

impl AnalysisUseCases {
    pub fn new(
        analysis_repository: Arc<dyn AnalysisRepository>,
        llm_provider: Option<Arc<dyn LLMProvider>>,
    ) -> Self {
        let mut analysis_engine = AnalysisEngine::new();
        if let Some(provider) = llm_provider {
            analysis_engine = analysis_engine.with_llm_provider(provider);
        }

        Self {
            analysis_engine,
            analysis_repository,
        }
    }

    pub async fn analyze_repository(
        &self,
        request: AnalyzeRepositoryRequest,
        file_contents: HashMap<String, String>,
    ) -> Result<AnalysisResponse> {
        info!("Starting repository analysis for repository: {}", request.repository_id);

        // Create internal analysis request
        let analysis_request = AnalysisRequest {
            repository_id: request.repository_id,
            commit_sha: request.commit_sha.clone(),
            files_to_analyze: request.files_to_analyze.unwrap_or_default(),
            analysis_types: if request.analysis_types.is_empty() {
                vec![AnalysisType::StaticAnalysis] // Default to static analysis
            } else {
                request.analysis_types
            },
        };

        // Filter to only Move files if no specific files requested
        let move_files: HashMap<String, String> = file_contents
            .into_iter()
            .filter(|(path, _)| path.ends_with(".move"))
            .collect();

        if move_files.is_empty() {
            return Err(Error::AnalysisFailed {
                message: "No Move files found in repository".to_string(),
            });
        }

        // Run analysis
        let analysis_results = self.analysis_engine
            .analyze_repository(analysis_request, move_files)
            .await?;

        if analysis_results.is_empty() {
            return Err(Error::AnalysisFailed {
                message: "No analysis results generated".to_string(),
            });
        }

        // Merge results if multiple analysis types were run
        let final_result = if analysis_results.len() == 1 {
            analysis_results.into_iter().next().unwrap()
        } else {
            self.analysis_engine.merge_analysis_results(analysis_results)?
        };

        // Save to database
        let analysis_id = self.analysis_repository
            .save_analysis_result(&final_result)
            .await?;

        info!("Analysis completed for repository: {} with ID: {}", request.repository_id, analysis_id);

        Ok(AnalysisResponse {
            analysis_id,
            repository_id: final_result.repository_id,
            commit_sha: final_result.commit_sha,
            security_score: final_result.security_score,
            quality_score: final_result.quality_score,
            vulnerabilities_found: final_result.vulnerabilities.len() as u32,
            critical_vulnerabilities: final_result.vulnerabilities
                .iter()
                .filter(|v| matches!(v.severity, crate::domain::analysis_models::Severity::Critical))
                .count() as u32,
            analysis_duration_ms: final_result.analysis_duration_ms,
            analysis_types_completed: vec![format!("{:?}", final_result.analysis_type)],
        })
    }

    pub async fn analyze_code(&self, request: AnalyzeCodeRequest) -> Result<CodeAnalysisResponse> {
        let start_time = std::time::Instant::now();

        // Create a temporary file contents map
        let mut file_contents = HashMap::new();
        file_contents.insert(request.file_path.clone(), request.code.clone());

        let analysis_request = AnalysisRequest {
            repository_id: uuid::Uuid::new_v4(), // Temporary ID for code analysis
            commit_sha: "temporary".to_string(),
            files_to_analyze: vec![request.file_path.clone()],
            analysis_types: if request.analysis_types.is_empty() {
                vec![AnalysisType::StaticAnalysis]
            } else {
                request.analysis_types
            },
        };

        let analysis_results = self.analysis_engine
            .analyze_repository(analysis_request, file_contents)
            .await?;

        let final_result = if analysis_results.len() == 1 {
            analysis_results.into_iter().next().unwrap()
        } else {
            self.analysis_engine.merge_analysis_results(analysis_results)?
        };

        let analysis_duration = start_time.elapsed();

        Ok(CodeAnalysisResponse {
            security_score: final_result.security_score,
            quality_score: final_result.quality_score,
            vulnerabilities: final_result.vulnerabilities,
            recommendations: final_result.recommendations,
            analysis_duration_ms: analysis_duration.as_millis() as u64,
        })
    }

    pub async fn get_analysis_status(&self, repository_id: uuid::Uuid) -> Result<AnalysisStatusResponse> {
        let latest_analysis = self.analysis_repository
            .get_latest_analysis_for_repository(repository_id)
            .await?;

        let vulnerability_statistics = self.analysis_repository
            .get_vulnerability_statistics(repository_id)
            .await?;

        let history = self.analysis_repository
            .get_analysis_history(repository_id, Some(100))
            .await?;

        let latest_summary = latest_analysis.map(|analysis| crate::models::responses::AnalysisSummary {
            analysis_id: analysis.id,
            commit_sha: analysis.commit_sha,
            analysis_type: format!("{:?}", analysis.analysis_type),
            security_score: analysis.security_score,
            quality_score: analysis.quality_score,
            vulnerabilities_count: analysis.vulnerabilities.len(),
            created_at: analysis.created_at,
        });

        Ok(AnalysisStatusResponse {
            repository_id,
            latest_analysis: latest_summary,
            total_analyses: history.len() as u32,
            vulnerability_statistics,
        })
    }

    pub async fn get_detailed_analysis(&self, analysis_id: uuid::Uuid) -> Result<DetailedAnalysisResponse> {
        let analysis_result = self.analysis_repository
            .get_analysis_result(analysis_id)
            .await?
            .ok_or_else(|| Error::AnalysisFailed {
                message: format!("Analysis {} not found", analysis_id),
            })?;

        let vulnerability_statistics = self.analysis_repository
            .get_vulnerability_statistics(analysis_result.repository_id)
            .await?;

        Ok(DetailedAnalysisResponse {
            analysis_result,
            vulnerability_statistics,
        })
    }

    pub async fn handle_vulnerability_action(&self, request: MarkVulnerabilityRequest) -> Result<()> {
        match request.action {
            VulnerabilityAction::MarkFalsePositive => {
                self.analysis_repository
                    .mark_vulnerability_as_false_positive(request.vulnerability_id)
                    .await?;
                info!("Marked vulnerability {} as false positive", request.vulnerability_id);
            }
            VulnerabilityAction::MarkFixed => {
                self.analysis_repository
                    .mark_vulnerability_as_fixed(request.vulnerability_id)
                    .await?;
                info!("Marked vulnerability {} as fixed", request.vulnerability_id);
            }
            VulnerabilityAction::Reopen => {
                // TODO: Implement reopen functionality
                warn!("Reopen functionality not yet implemented for vulnerability {}", request.vulnerability_id);
                return Err(Error::AnalysisFailed {
                    message: "Reopen functionality not yet implemented".to_string(),
                });
            }
        }

        Ok(())
    }

    pub async fn get_repository_vulnerabilities(&self, repository_id: uuid::Uuid) -> Result<crate::models::responses::VulnerabilityListResponse> {
        let vulnerabilities = self.analysis_repository
            .get_vulnerabilities_for_repository(repository_id)
            .await?;

        let statistics = self.analysis_repository
            .get_vulnerability_statistics(repository_id)
            .await?;

        Ok(crate::models::responses::VulnerabilityListResponse {
            repository_id,
            vulnerabilities,
            statistics,
        })
    }

    pub async fn get_analysis_history(&self, repository_id: uuid::Uuid, limit: Option<u32>) -> Result<crate::models::responses::AnalysisHistoryResponse> {
        let analyses = self.analysis_repository
            .get_analysis_history(repository_id, limit)
            .await?;

        let analysis_summaries = analyses
            .iter()
            .map(|analysis| crate::models::responses::AnalysisSummary {
                analysis_id: analysis.id,
                commit_sha: analysis.commit_sha.clone(),
                analysis_type: format!("{:?}", analysis.analysis_type),
                security_score: analysis.security_score,
                quality_score: analysis.quality_score,
                vulnerabilities_count: analysis.vulnerabilities.len(),
                created_at: analysis.created_at,
            })
            .collect();

        Ok(crate::models::responses::AnalysisHistoryResponse {
            repository_id,
            analyses: analysis_summaries,
            total_count: analyses.len(),
        })
    }
}