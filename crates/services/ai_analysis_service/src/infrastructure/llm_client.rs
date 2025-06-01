use crate::domain::analysis_models::{VulnerabilityFinding, SecurityRecommendation, VulnerabilityType, Severity, RecommendationCategory, Priority, CodeExample};
use crate::domain::llm_provider_trait::{LLMProvider, LLMRequest, LLMResponse, TokenUsage, CodeAnalysisResponse};
use crate::error::{Error, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

pub struct LLMClient {
    client: Client,
    provider: LLMProviderType,
    api_key: String,
    base_url: String,
    model: String,
}

#[derive(Debug, Clone)]
pub enum LLMProviderType {
    OpenAI,
    Anthropic,
    Local,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
    model: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

impl LLMClient {
    pub fn new_openai(api_key: String) -> Self {
        Self {
            client: Client::new(),
            provider: LLMProviderType::OpenAI,
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4".to_string(),
        }
    }

    pub fn new_anthropic(api_key: String) -> Self {
        Self {
            client: Client::new(),
            provider: LLMProviderType::Anthropic,
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
            model: "claude-3-sonnet-20240229".to_string(),
        }
    }

    pub fn new_local(base_url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            provider: LLMProviderType::Local,
            api_key: String::new(),
            base_url,
            model,
        }
    }

    fn create_vulnerability_detection_prompt(&self, code: &str, file_path: &str) -> String {
        format!(
            r#"Analyze the following Sui Move smart contract code for security vulnerabilities.

File: {}

Code:
```move
{}
```

Please provide a detailed security analysis focusing on:

1. **Access Control Issues**: Missing capability checks, unauthorized function access
2. **Resource Management**: Improper handling of Sui objects, balance operations
3. **Integer Overflow/Underflow**: Arithmetic operations without bounds checking
4. **Logic Errors**: Flawed business logic, incorrect state transitions
5. **Timestamp Dependencies**: Critical logic relying on timestamps
6. **Input Validation**: Missing or insufficient parameter validation
7. **Reentrancy-like Issues**: Functions that could be called recursively with harmful effects

For each vulnerability found, provide:
- Vulnerability type and severity (Critical/High/Medium/Low)
- Line number(s) affected
- Code snippet showing the issue
- Detailed explanation of the risk
- Specific remediation recommendations
- Confidence level (0-100)

Respond in JSON format:
{{
  "vulnerabilities": [
    {{
      "type": "vulnerability_type",
      "severity": "severity_level",
      "line_number": number,
      "code_snippet": "code",
      "description": "detailed_description",
      "recommendation": "specific_fix",
      "confidence": number
    }}
  ],
  "summary": "overall_assessment",
  "overall_confidence": number
}}"#,
            file_path, code
        )
    }

    fn create_security_recommendations_prompt(&self, code: &str, vulnerabilities: &[VulnerabilityFinding]) -> String {
        let vuln_summary = vulnerabilities
            .iter()
            .map(|v| format!("- {} ({}): {}", v.vulnerability_type, v.severity, v.description))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"Based on the following Sui Move code and identified vulnerabilities, provide comprehensive security recommendations:

Code:
```move
{}
```

Identified Vulnerabilities:
{}

Please provide:
1. **Prioritized Recommendations**: Most critical fixes first
2. **Code Examples**: Before/after code snippets for each recommendation
3. **Best Practices**: General security patterns for Sui Move
4. **Testing Strategies**: How to verify the fixes

Respond in JSON format:
{{
  "recommendations": [
    {{
      "category": "category_name",
      "title": "recommendation_title",
      "description": "detailed_description",
      "priority": "Critical/High/Medium/Low",
      "code_examples": [
        {{
          "title": "example_title",
          "before": "vulnerable_code",
          "after": "secure_code",
          "explanation": "why_this_fixes_issue"
        }}
      ]
    }}
  ]
}}"#,
            code, vuln_summary
        )
    }

    fn create_code_quality_prompt(&self, code: &str) -> String {
        format!(
            r#"Assess the code quality of this Sui Move smart contract on a scale of 0-100:

```move
{}
```

Consider:
- Code structure and organization
- Documentation quality
- Error handling
- Input validation
- Gas efficiency
- Readability and maintainability
- Following Sui Move best practices

Provide a score (0-100) and brief explanation.

Respond in JSON format:
{{
  "quality_score": number,
  "explanation": "detailed_assessment"
}}"#,
            code
        )
    }

    async fn call_openai_api(&self, prompt: &str, max_tokens: u32, temperature: f64) -> Result<LLMResponse> {
        let request_body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a security expert specializing in Sui Move smart contract analysis."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "max_tokens": max_tokens,
            "temperature": temperature
        });

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::LLMApiError {
                message: format!("OpenAI API error: {}", error_text),
            });
        }

        let openai_response: OpenAIResponse = response.json().await?;
        
        let first_choice = openai_response.choices.into_iter().next()
            .ok_or_else(|| Error::LLMApiError {
                message: "No choices in OpenAI response".to_string(),
            })?;

        Ok(LLMResponse {
            content: first_choice.message.content,
            usage: TokenUsage {
                prompt_tokens: openai_response.usage.prompt_tokens,
                completion_tokens: openai_response.usage.completion_tokens,
                total_tokens: openai_response.usage.total_tokens,
            },
            model: openai_response.model,
        })
    }

    fn parse_vulnerability_response(&self, content: &str, file_path: &str) -> Result<CodeAnalysisResponse> {
        #[derive(Deserialize)]
        struct VulnResponse {
            vulnerabilities: Vec<RawVulnerability>,
            summary: String,
            overall_confidence: f64,
        }

        #[derive(Deserialize)]
        struct RawVulnerability {
            #[serde(rename = "type")]
            vuln_type: String,
            severity: String,
            line_number: Option<u32>,
            code_snippet: Option<String>,
            description: String,
            recommendation: String,
            confidence: f64,
        }

        let parsed: VulnResponse = serde_json::from_str(content)
            .map_err(|e| Error::LLMApiError {
                message: format!("Failed to parse vulnerability response: {}", e),
            })?;

        let vulnerabilities = parsed
            .vulnerabilities
            .into_iter()
            .map(|raw| VulnerabilityFinding {
                id: Uuid::new_v4(),
                vulnerability_type: self.parse_vulnerability_type(&raw.vuln_type),
                severity: self.parse_severity(&raw.severity),
                confidence_score: raw.confidence,
                file_path: file_path.to_string(),
                line_number: raw.line_number,
                code_snippet: raw.code_snippet,
                description: raw.description,
                recommendation: raw.recommendation,
                cve_id: None,
                is_false_positive: false,
            })
            .collect();

        Ok(CodeAnalysisResponse {
            vulnerabilities,
            recommendations: Vec::new(), // Will be filled separately
            summary: parsed.summary,
            confidence: parsed.overall_confidence,
        })
    }

    fn parse_vulnerability_type(&self, type_str: &str) -> VulnerabilityType {
        match type_str.to_lowercase().as_str() {
            "access_control" | "unauthorized_access" => VulnerabilityType::UnauthorizedAccess,
            "resource_exhaustion" => VulnerabilityType::ResourceExhaustion,
            "integer_overflow" | "overflow" => VulnerabilityType::IntegerOverflow,
            "logic_error" => VulnerabilityType::LogicError,
            "timestamp_dependence" => VulnerabilityType::TimestampDependence,
            "insufficient_validation" | "input_validation" => VulnerabilityType::InsufficientValidation,
            _ => VulnerabilityType::Other(type_str.to_string()),
        }
    }

    fn parse_severity(&self, severity_str: &str) -> Severity {
        match severity_str.to_lowercase().as_str() {
            "critical" => Severity::Critical,
            "high" => Severity::High,
            "medium" => Severity::Medium,
            "low" => Severity::Low,
            _ => Severity::Medium,
        }
    }

    fn parse_recommendations_response(&self, content: &str) -> Result<Vec<SecurityRecommendation>> {
        #[derive(Deserialize)]
        struct RecommendationsResponse {
            recommendations: Vec<RawRecommendation>,
        }

        #[derive(Deserialize)]
        struct RawRecommendation {
            category: String,
            title: String,
            description: String,
            priority: String,
            code_examples: Vec<RawCodeExample>,
        }

        #[derive(Deserialize)]
        struct RawCodeExample {
            title: String,
            before: Option<String>,
            after: String,
            explanation: String,
        }

        let parsed: RecommendationsResponse = serde_json::from_str(content)
            .map_err(|e| Error::LLMApiError {
                message: format!("Failed to parse recommendations response: {}", e),
            })?;

        Ok(parsed
            .recommendations
            .into_iter()
            .map(|raw| SecurityRecommendation {
                id: Uuid::new_v4(),
                category: self.parse_recommendation_category(&raw.category),
                title: raw.title,
                description: raw.description,
                priority: self.parse_priority(&raw.priority),
                code_examples: raw
                    .code_examples
                    .into_iter()
                    .map(|ex| CodeExample {
                        title: ex.title,
                        before: ex.before,
                        after: ex.after,
                        explanation: ex.explanation,
                    })
                    .collect(),
            })
            .collect())
    }

    fn parse_recommendation_category(&self, category_str: &str) -> RecommendationCategory {
        match category_str.to_lowercase().as_str() {
            "access_control" => RecommendationCategory::AccessControl,
            "input_validation" => RecommendationCategory::InputValidation,
            "error_handling" => RecommendationCategory::ErrorHandling,
            "gas_optimization" => RecommendationCategory::GasOptimization,
            "code_structure" => RecommendationCategory::CodeStructure,
            "testing" => RecommendationCategory::Testing,
            _ => RecommendationCategory::CodeStructure,
        }
    }

    fn parse_priority(&self, priority_str: &str) -> Priority {
        match priority_str.to_lowercase().as_str() {
            "critical" => Priority::Critical,
            "high" => Priority::High,
            "medium" => Priority::Medium,
            "low" => Priority::Low,
            _ => Priority::Medium,
        }
    }
}

#[async_trait]
impl LLMProvider for LLMClient {
    async fn analyze_code(&self, request: LLMRequest) -> Result<LLMResponse> {
        match self.provider {
            LLMProviderType::OpenAI => {
                self.call_openai_api(&request.prompt, request.max_tokens, request.temperature).await
            }
            LLMProviderType::Anthropic => {
                // TODO: Implement Anthropic API
                Err(Error::LLMApiError {
                    message: "Anthropic provider not implemented yet".to_string(),
                })
            }
            LLMProviderType::Local => {
                // TODO: Implement local model API
                Err(Error::LLMApiError {
                    message: "Local provider not implemented yet".to_string(),
                })
            }
        }
    }

    async fn detect_vulnerabilities(&self, code: &str, file_path: &str) -> Result<CodeAnalysisResponse> {
        let prompt = self.create_vulnerability_detection_prompt(code, file_path);
        let request = LLMRequest {
            prompt,
            code_context: code.to_string(),
            max_tokens: 2000,
            temperature: 0.1,
        };

        let response = self.analyze_code(request).await?;
        self.parse_vulnerability_response(&response.content, file_path)
    }

    async fn generate_security_recommendations(&self, code: &str, vulnerabilities: &[VulnerabilityFinding]) -> Result<Vec<SecurityRecommendation>> {
        let prompt = self.create_security_recommendations_prompt(code, vulnerabilities);
        let request = LLMRequest {
            prompt,
            code_context: code.to_string(),
            max_tokens: 1500,
            temperature: 0.1,
        };

        let response = self.analyze_code(request).await?;
        self.parse_recommendations_response(&response.content)
    }

    async fn assess_code_quality(&self, code: &str) -> Result<f64> {
        let prompt = self.create_code_quality_prompt(code);
        let request = LLMRequest {
            prompt,
            code_context: code.to_string(),
            max_tokens: 500,
            temperature: 0.1,
        };

        let response = self.analyze_code(request).await?;
        
        #[derive(Deserialize)]
        struct QualityResponse {
            quality_score: f64,
        }

        let parsed: QualityResponse = serde_json::from_str(&response.content)
            .map_err(|e| Error::LLMApiError {
                message: format!("Failed to parse quality response: {}", e),
            })?;

        Ok(parsed.quality_score.max(0.0).min(100.0))
    }

    fn get_provider_name(&self) -> &str {
        match self.provider {
            LLMProviderType::OpenAI => "OpenAI",
            LLMProviderType::Anthropic => "Anthropic",
            LLMProviderType::Local => "Local",
        }
    }

    fn get_model_name(&self) -> &str {
        &self.model
    }
}