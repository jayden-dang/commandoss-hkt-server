use crate::domain::analysis_models::{AnalysisResult, AnalysisRequest, AnalysisType, VulnerabilityFinding};
use crate::domain::vulnerability_patterns::VulnerabilityPatterns;
use crate::error::{Error, Result};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

pub struct SuiMoveStaticAnalyzer {
    patterns: VulnerabilityPatterns,
    version: String,
}

impl SuiMoveStaticAnalyzer {
    pub fn new() -> Self {
        Self {
            patterns: VulnerabilityPatterns::new(),
            version: "1.0.0".to_string(),
        }
    }

    pub async fn analyze(&self, request: AnalysisRequest, file_contents: HashMap<String, String>) -> Result<AnalysisResult> {
        let start_time = std::time::Instant::now();
        let mut all_vulnerabilities = Vec::new();

        // Filter only Move files
        let move_files: HashMap<String, String> = file_contents
            .into_iter()
            .filter(|(path, _)| path.ends_with(".move"))
            .collect();

        if move_files.is_empty() {
            return Err(Error::AnalysisFailed {
                message: "No Move files found for analysis".to_string(),
            });
        }

        // Analyze each Move file
        for (file_path, content) in &move_files {
            let vulnerabilities = self.analyze_file(file_path, content).await?;
            all_vulnerabilities.extend(vulnerabilities);
        }

        let analysis_duration = start_time.elapsed();
        
        // Calculate scores based on findings
        let (security_score, quality_score) = self.calculate_scores(&all_vulnerabilities, &move_files);

        Ok(AnalysisResult {
            id: Uuid::new_v4(),
            repository_id: request.repository_id,
            commit_sha: request.commit_sha,
            analysis_type: AnalysisType::StaticAnalysis,
            security_score,
            quality_score,
            vulnerabilities: all_vulnerabilities.clone(),
            recommendations: self.generate_recommendations(&all_vulnerabilities),
            analysis_duration_ms: analysis_duration.as_millis() as u64,
            analyzer_version: self.version.clone(),
            raw_results: json!({
                "files_analyzed": move_files.len(),
                "total_vulnerabilities": all_vulnerabilities.len(),
                "vulnerability_breakdown": self.get_vulnerability_breakdown(&all_vulnerabilities)
            }),
            created_at: Utc::now(),
        })
    }

    async fn analyze_file(&self, file_path: &str, content: &str) -> Result<Vec<VulnerabilityFinding>> {
        // Basic file validation
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Check if it's a valid Move file
        if !self.is_valid_move_file(content) {
            return Err(Error::FileParsingError {
                file_path: file_path.to_string(),
                message: "Invalid Move file format".to_string(),
            });
        }

        // Apply vulnerability patterns
        let mut findings = self.patterns.scan_code(file_path, content)?;

        // Apply additional Move-specific checks
        findings.extend(self.check_move_specific_patterns(file_path, content)?);

        // Apply confidence scoring based on context
        self.adjust_confidence_scores(&mut findings, content);

        Ok(findings)
    }

    fn is_valid_move_file(&self, content: &str) -> bool {
        // Basic Move file validation
        content.contains("module") || content.contains("script") || content.contains("use ")
    }

    fn check_move_specific_patterns(&self, file_path: &str, content: &str) -> Result<Vec<VulnerabilityFinding>> {
        let mut findings = Vec::new();

        // Check for missing module documentation
        if !content.contains("///") && content.contains("module") {
            findings.push(VulnerabilityFinding {
                id: Uuid::new_v4(),
                vulnerability_type: crate::domain::analysis_models::VulnerabilityType::Other("Documentation".to_string()),
                severity: crate::domain::analysis_models::Severity::Low,
                confidence_score: 90.0,
                file_path: file_path.to_string(),
                line_number: Some(1),
                code_snippet: None,
                description: "Module lacks proper documentation".to_string(),
                recommendation: "Add module documentation using /// comments".to_string(),
                cve_id: None,
                is_false_positive: false,
            });
        }

        // Check for friend declarations (potential access control issues)
        if content.contains("friend") {
            for (line_number, line) in content.lines().enumerate() {
                if line.trim().starts_with("friend") {
                    findings.push(VulnerabilityFinding {
                        id: Uuid::new_v4(),
                        vulnerability_type: crate::domain::analysis_models::VulnerabilityType::AccessControl,
                        severity: crate::domain::analysis_models::Severity::Medium,
                        confidence_score: 70.0,
                        file_path: file_path.to_string(),
                        line_number: Some((line_number + 1) as u32),
                        code_snippet: Some(line.trim().to_string()),
                        description: "Friend declaration may introduce unexpected access".to_string(),
                        recommendation: "Review friend module access and ensure it's necessary".to_string(),
                        cve_id: None,
                        is_false_positive: false,
                    });
                }
            }
        }

        // Check for test functions in non-test modules
        if !file_path.contains("test") && content.contains("#[test]") {
            findings.push(VulnerabilityFinding {
                id: Uuid::new_v4(),
                vulnerability_type: crate::domain::analysis_models::VulnerabilityType::Other("Test in Production".to_string()),
                severity: crate::domain::analysis_models::Severity::Medium,
                confidence_score: 85.0,
                file_path: file_path.to_string(),
                line_number: None,
                code_snippet: None,
                description: "Test functions found in production module".to_string(),
                recommendation: "Move test functions to separate test modules".to_string(),
                cve_id: None,
                is_false_positive: false,
            });
        }

        Ok(findings)
    }

    fn adjust_confidence_scores(&self, findings: &mut Vec<VulnerabilityFinding>, content: &str) {
        for finding in findings {
            // Increase confidence if similar patterns are found multiple times
            let pattern_count = content.matches(&finding.description).count();
            if pattern_count > 1 {
                finding.confidence_score = (finding.confidence_score * 1.2).min(95.0);
            }

            // Adjust based on file type and context
            if content.contains("entry fun") && finding.vulnerability_type == crate::domain::analysis_models::VulnerabilityType::AccessControl {
                finding.confidence_score = (finding.confidence_score * 1.3).min(95.0);
            }

            // Lower confidence for files with extensive validation
            if content.matches("assert!").count() > 5 {
                finding.confidence_score *= 0.9;
            }
        }
    }

    fn calculate_scores(&self, vulnerabilities: &[VulnerabilityFinding], files: &HashMap<String, String>) -> (f64, f64) {
        if vulnerabilities.is_empty() {
            return (95.0, 90.0); // High scores for clean code
        }

        let total_lines: usize = files.values().map(|content| content.lines().count()).sum();
        let vulnerability_density = vulnerabilities.len() as f64 / total_lines.max(1) as f64;

        // Calculate security score (lower with more severe vulnerabilities)
        let mut security_penalty = 0.0;
        for vuln in vulnerabilities {
            if vuln.is_false_positive {
                continue;
            }
            
            let severity_penalty = match vuln.severity {
                crate::domain::analysis_models::Severity::Critical => 25.0,
                crate::domain::analysis_models::Severity::High => 15.0,
                crate::domain::analysis_models::Severity::Medium => 8.0,
                crate::domain::analysis_models::Severity::Low => 3.0,
            };
            
            security_penalty += severity_penalty * (vuln.confidence_score / 100.0);
        }

        let security_score = (100.0 - security_penalty - (vulnerability_density * 100.0)).max(0.0);

        // Calculate quality score (based on code patterns and documentation)
        let mut quality_score = 80.0;
        
        // Bonus for documentation
        let total_content = files.values().map(|s| s.as_str()).collect::<Vec<_>>().join("\n");
        if total_content.matches("///").count() > 0 {
            quality_score += 10.0;
        }
        
        // Penalty for vulnerability density
        quality_score -= vulnerability_density * 50.0;
        
        // Bonus for assertion usage (defensive programming)
        let assertion_ratio = total_content.matches("assert!").count() as f64 / total_lines.max(1) as f64;
        quality_score += assertion_ratio * 20.0;

        (security_score.max(0.0).min(100.0), quality_score.max(0.0).min(100.0))
    }

    fn generate_recommendations(&self, vulnerabilities: &[VulnerabilityFinding]) -> Vec<crate::domain::analysis_models::SecurityRecommendation> {
        use crate::domain::analysis_models::{SecurityRecommendation, RecommendationCategory, Priority, CodeExample};
        
        let mut recommendations = Vec::new();

        // Group vulnerabilities by type for targeted recommendations
        let mut vuln_counts = HashMap::new();
        for vuln in vulnerabilities {
            if !vuln.is_false_positive {
                *vuln_counts.entry(&vuln.vulnerability_type).or_insert(0) += 1;
            }
        }

        // Generate recommendations based on vulnerability patterns
        for (vuln_type, count) in vuln_counts {
            if count > 0 {
                let rec = match vuln_type {
                    crate::domain::analysis_models::VulnerabilityType::UnauthorizedAccess => {
                        SecurityRecommendation {
                            id: Uuid::new_v4(),
                            category: RecommendationCategory::AccessControl,
                            title: "Implement Proper Access Control".to_string(),
                            description: "Add capability-based access control to restrict function execution".to_string(),
                            priority: Priority::High,
                            code_examples: vec![
                                CodeExample {
                                    title: "Add Capability Parameter".to_string(),
                                    before: Some("public fun transfer(item: Item, recipient: address)".to_string()),
                                    after: "public fun transfer(cap: &AdminCap, item: Item, recipient: address)".to_string(),
                                    explanation: "Require capability object to authorize transfers".to_string(),
                                }
                            ],
                        }
                    },
                    crate::domain::analysis_models::VulnerabilityType::InsufficientValidation => {
                        SecurityRecommendation {
                            id: Uuid::new_v4(),
                            category: RecommendationCategory::InputValidation,
                            title: "Add Input Validation".to_string(),
                            description: "Implement comprehensive input validation for all public functions".to_string(),
                            priority: Priority::Medium,
                            code_examples: vec![
                                CodeExample {
                                    title: "Add Assertion Checks".to_string(),
                                    before: Some("public fun set_price(price: u64)".to_string()),
                                    after: "public fun set_price(price: u64) {\n    assert!(price > 0, EInvalidPrice);\n    // function body\n}".to_string(),
                                    explanation: "Validate inputs before processing".to_string(),
                                }
                            ],
                        }
                    },
                    _ => continue,
                };
                recommendations.push(rec);
            }
        }

        // Add general recommendations
        if vulnerabilities.len() > 5 {
            recommendations.push(SecurityRecommendation {
                id: Uuid::new_v4(),
                category: RecommendationCategory::Testing,
                title: "Increase Test Coverage".to_string(),
                description: "High vulnerability count suggests need for more comprehensive testing".to_string(),
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
}