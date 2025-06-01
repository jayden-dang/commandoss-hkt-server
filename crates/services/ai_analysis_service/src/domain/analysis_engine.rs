use crate::domain::analysis_models::{AnalysisRequest, AnalysisResult, AnalysisType, VulnerabilityFinding, SecurityRecommendation};
use crate::domain::llm_provider_trait::LLMProvider;
use crate::infrastructure::static_analyzer::SuiMoveStaticAnalyzer;
use crate::error::{Error, Result};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct AnalysisEngine {
    static_analyzer: SuiMoveStaticAnalyzer,
    llm_provider: Option<Arc<dyn LLMProvider>>,
}

impl AnalysisEngine {
    pub fn new() -> Self {
        Self {
            static_analyzer: SuiMoveStaticAnalyzer::new(),
            llm_provider: None,
        }
    }

    pub fn with_llm_provider(mut self, provider: Arc<dyn LLMProvider>) -> Self {
        self.llm_provider = Some(provider);
        self
    }

    pub async fn analyze_repository(&self, request: AnalysisRequest, file_contents: HashMap<String, String>) -> Result<Vec<AnalysisResult>> {
        let mut results = Vec::new();

        // Run static analysis first (always available)
        if request.analysis_types.contains(&AnalysisType::StaticAnalysis) || 
           request.analysis_types.contains(&AnalysisType::VulnerabilityDetection) {
            let static_result = self.static_analyzer.analyze(request.clone(), file_contents.clone()).await?;
            results.push(static_result);
        }

        // Run LLM analysis if provider is available
        if let Some(llm_provider) = &self.llm_provider {
            if request.analysis_types.contains(&AnalysisType::LLMReview) {
                let llm_result = self.run_llm_analysis(&request, &file_contents, llm_provider.clone()).await?;
                results.push(llm_result);
            }

            if request.analysis_types.contains(&AnalysisType::CodeQualityAssessment) {
                let quality_result = self.run_quality_analysis(&request, &file_contents, llm_provider.clone()).await?;
                results.push(quality_result);
            }
        }

        // If no results generated, run at least static analysis
        if results.is_empty() {
            let fallback_request = AnalysisRequest {
                analysis_types: vec![AnalysisType::StaticAnalysis],
                ..request
            };
            let static_result = self.static_analyzer.analyze(fallback_request, file_contents).await?;
            results.push(static_result);
        }

        Ok(results)
    }

    async fn run_llm_analysis(
        &self,
        request: &AnalysisRequest,
        file_contents: &HashMap<String, String>,
        llm_provider: Arc<dyn LLMProvider>,
    ) -> Result<AnalysisResult> {
        let start_time = std::time::Instant::now();
        let mut all_vulnerabilities = Vec::new();
        let mut all_recommendations = Vec::new();

        // Filter Move files
        let move_files: HashMap<String, String> = file_contents
            .iter()
            .filter(|(path, _)| path.ends_with(".move"))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        if move_files.is_empty() {
            return Err(Error::AnalysisFailed {
                message: "No Move files found for LLM analysis".to_string(),
            });
        }

        // Analyze each file with LLM
        for (file_path, content) in &move_files {
            // Skip very large files to avoid token limits
            if content.len() > 10000 {
                continue;
            }

            let analysis_response = llm_provider.detect_vulnerabilities(content, file_path).await?;
            
            // Generate recommendations for this file's vulnerabilities
            if !analysis_response.vulnerabilities.is_empty() {
                let recommendations = llm_provider
                    .generate_security_recommendations(content, &analysis_response.vulnerabilities)
                    .await?;
                all_recommendations.extend(recommendations);
            }
            
            all_vulnerabilities.extend(analysis_response.vulnerabilities);
        }

        let analysis_duration = start_time.elapsed();

        // Calculate scores based on LLM findings
        let (security_score, quality_score) = self.calculate_llm_scores(&all_vulnerabilities, &move_files).await;

        Ok(AnalysisResult {
            id: Uuid::new_v4(),
            repository_id: request.repository_id,
            commit_sha: request.commit_sha.clone(),
            analysis_type: AnalysisType::LLMReview,
            security_score,
            quality_score,
            vulnerabilities: all_vulnerabilities.clone(),
            recommendations: all_recommendations,
            analysis_duration_ms: analysis_duration.as_millis() as u64,
            analyzer_version: format!("{}-{}", llm_provider.get_provider_name(), llm_provider.get_model_name()),
            raw_results: json!({
                "files_analyzed": move_files.len(),
                "llm_provider": llm_provider.get_provider_name(),
                "llm_model": llm_provider.get_model_name(),
                "total_vulnerabilities": all_vulnerabilities.len(),
                "vulnerability_breakdown": self.get_vulnerability_breakdown(&all_vulnerabilities)
            }),
            created_at: Utc::now(),
        })
    }

    async fn run_quality_analysis(
        &self,
        request: &AnalysisRequest,
        file_contents: &HashMap<String, String>,
        llm_provider: Arc<dyn LLMProvider>,
    ) -> Result<AnalysisResult> {
        let start_time = std::time::Instant::now();

        let move_files: HashMap<String, String> = file_contents
            .iter()
            .filter(|(path, _)| path.ends_with(".move"))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        if move_files.is_empty() {
            return Err(Error::AnalysisFailed {
                message: "No Move files found for quality analysis".to_string(),
            });
        }

        let mut total_quality_score = 0.0;
        let mut analyzed_files = 0;

        // Analyze quality for each file
        for (_, content) in &move_files {
            // Skip very large files
            if content.len() > 8000 {
                continue;
            }

            match llm_provider.assess_code_quality(content).await {
                Ok(score) => {
                    total_quality_score += score;
                    analyzed_files += 1;
                }
                Err(_) => {
                    // Continue with other files if one fails
                    continue;
                }
            }
        }

        let analysis_duration = start_time.elapsed();

        let average_quality = if analyzed_files > 0 {
            total_quality_score / analyzed_files as f64
        } else {
            50.0 // Default score if no files could be analyzed
        };

        // Generate quality-based recommendations
        let quality_recommendations = self.generate_quality_recommendations(average_quality);

        Ok(AnalysisResult {
            id: Uuid::new_v4(),
            repository_id: request.repository_id,
            commit_sha: request.commit_sha.clone(),
            analysis_type: AnalysisType::CodeQualityAssessment,
            security_score: average_quality, // Use quality as security for this analysis type
            quality_score: average_quality,
            vulnerabilities: Vec::new(), // Quality analysis doesn't find vulnerabilities
            recommendations: quality_recommendations,
            analysis_duration_ms: analysis_duration.as_millis() as u64,
            analyzer_version: format!("{}-{}-quality", llm_provider.get_provider_name(), llm_provider.get_model_name()),
            raw_results: json!({
                "files_analyzed": analyzed_files,
                "average_quality_score": average_quality,
                "llm_provider": llm_provider.get_provider_name(),
                "llm_model": llm_provider.get_model_name()
            }),
            created_at: Utc::now(),
        })
    }

    async fn calculate_llm_scores(&self, vulnerabilities: &[VulnerabilityFinding], _files: &HashMap<String, String>) -> (f64, f64) {
        if vulnerabilities.is_empty() {
            return (90.0, 85.0); // High scores for clean code
        }

        // Calculate security score based on LLM-found vulnerabilities
        let mut security_penalty = 0.0;
        for vuln in vulnerabilities {
            if vuln.is_false_positive {
                continue;
            }
            
            let severity_penalty = match vuln.severity {
                crate::domain::analysis_models::Severity::Critical => 30.0,
                crate::domain::analysis_models::Severity::High => 20.0,
                crate::domain::analysis_models::Severity::Medium => 10.0,
                crate::domain::analysis_models::Severity::Low => 5.0,
            };
            
            // Weight by confidence score
            security_penalty += severity_penalty * (vuln.confidence_score / 100.0);
        }

        let security_score = (100.0 - security_penalty).max(0.0);
        
        // Quality score is generally lower than security score for LLM analysis
        // as LLM might catch more nuanced issues
        let quality_score = (security_score * 0.9).max(0.0);

        (security_score.min(100.0), quality_score.min(100.0))
    }

    fn generate_quality_recommendations(&self, quality_score: f64) -> Vec<SecurityRecommendation> {
        use crate::domain::analysis_models::{RecommendationCategory, Priority, CodeExample};

        let mut recommendations = Vec::new();

        if quality_score < 70.0 {
            recommendations.push(SecurityRecommendation {
                id: Uuid::new_v4(),
                category: RecommendationCategory::CodeStructure,
                title: "Improve Code Quality".to_string(),
                description: "Code quality score is below recommended threshold. Focus on code organization, documentation, and best practices.".to_string(),
                priority: Priority::Medium,
                code_examples: vec![
                    CodeExample {
                        title: "Add Function Documentation".to_string(),
                        before: Some("public fun transfer(item: Item) { ... }".to_string()),
                        after: "/// Transfers an item to a new owner\n/// \n/// # Arguments\n/// * `item` - The item to transfer\npublic fun transfer(item: Item) { ... }".to_string(),
                        explanation: "Document public functions with clear descriptions and parameter explanations".to_string(),
                    }
                ],
            });
        }

        if quality_score < 50.0 {
            recommendations.push(SecurityRecommendation {
                id: Uuid::new_v4(),
                category: RecommendationCategory::Testing,
                title: "Increase Test Coverage".to_string(),
                description: "Low quality score indicates potential need for more comprehensive testing.".to_string(),
                priority: Priority::High,
                code_examples: vec![],
            });
        }

        recommendations
    }

    fn get_vulnerability_breakdown(&self, vulnerabilities: &[VulnerabilityFinding]) -> serde_json::Value {
        let mut breakdown = HashMap::new();
        
        for vuln in vulnerabilities {
            if !vuln.is_false_positive {
                let severity_key = format!("{:?}", vuln.severity).to_lowercase();
                *breakdown.entry(severity_key).or_insert(0) += 1;
            }
        }

        json!(breakdown)
    }

    pub fn merge_analysis_results(&self, results: Vec<AnalysisResult>) -> Result<AnalysisResult> {
        if results.is_empty() {
            return Err(Error::AnalysisFailed {
                message: "No analysis results to merge".to_string(),
            });
        }

        if results.len() == 1 {
            return Ok(results.into_iter().next().unwrap());
        }

        let first_result = &results[0];
        let mut merged_vulnerabilities = Vec::new();
        let mut merged_recommendations = Vec::new();
        let mut total_duration = 0;

        // Combine all vulnerabilities and recommendations
        for result in &results {
            merged_vulnerabilities.extend(result.vulnerabilities.clone());
            merged_recommendations.extend(result.recommendations.clone());
            total_duration += result.analysis_duration_ms;
        }

        // Deduplicate vulnerabilities by file_path + line_number + vulnerability_type
        merged_vulnerabilities.sort_by(|a, b| {
            (a.file_path.as_str(), a.line_number, &a.vulnerability_type)
                .cmp(&(b.file_path.as_str(), b.line_number, &b.vulnerability_type))
        });
        merged_vulnerabilities.dedup_by(|a, b| {
            a.file_path == b.file_path 
                && a.line_number == b.line_number 
                && std::mem::discriminant(&a.vulnerability_type) == std::mem::discriminant(&b.vulnerability_type)
        });

        // Calculate merged scores (weighted average)
        let total_results = results.len() as f64;
        let merged_security_score = results.iter().map(|r| r.security_score).sum::<f64>() / total_results;
        let merged_quality_score = results.iter().map(|r| r.quality_score).sum::<f64>() / total_results;

        Ok(AnalysisResult {
            id: Uuid::new_v4(),
            repository_id: first_result.repository_id,
            commit_sha: first_result.commit_sha.clone(),
            analysis_type: AnalysisType::StaticAnalysis, // Use static as default for merged
            security_score: merged_security_score,
            quality_score: merged_quality_score,
            vulnerabilities: merged_vulnerabilities.clone(),
            recommendations: merged_recommendations,
            analysis_duration_ms: total_duration,
            analyzer_version: "merged-analysis-1.0.0".to_string(),
            raw_results: json!({
                "merged_from": results.len(),
                "analysis_types": results.iter().map(|r| format!("{:?}", r.analysis_type)).collect::<Vec<_>>(),
                "total_vulnerabilities": merged_vulnerabilities.len(),
                "vulnerability_breakdown": self.get_vulnerability_breakdown(&merged_vulnerabilities)
            }),
            created_at: Utc::now(),
        })
    }
}