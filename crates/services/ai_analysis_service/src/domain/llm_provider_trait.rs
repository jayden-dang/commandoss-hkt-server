use crate::domain::analysis_models::{VulnerabilityFinding, SecurityRecommendation};
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    pub prompt: String,
    pub code_context: String,
    pub max_tokens: u32,
    pub temperature: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub usage: TokenUsage,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisResponse {
    pub vulnerabilities: Vec<VulnerabilityFinding>,
    pub recommendations: Vec<SecurityRecommendation>,
    pub summary: String,
    pub confidence: f64,
}

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn analyze_code(&self, request: LLMRequest) -> Result<LLMResponse>;
    
    async fn detect_vulnerabilities(&self, code: &str, file_path: &str) -> Result<CodeAnalysisResponse>;
    
    async fn generate_security_recommendations(&self, code: &str, vulnerabilities: &[VulnerabilityFinding]) -> Result<Vec<SecurityRecommendation>>;
    
    async fn assess_code_quality(&self, code: &str) -> Result<f64>;
    
    fn get_provider_name(&self) -> &str;
    
    fn get_model_name(&self) -> &str;
}