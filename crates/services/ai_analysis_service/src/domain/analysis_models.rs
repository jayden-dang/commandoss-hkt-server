use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub repository_id: Uuid,
    pub commit_sha: String,
    pub files_to_analyze: Vec<String>,
    pub analysis_types: Vec<AnalysisType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnalysisType {
    StaticAnalysis,
    LLMReview,
    VulnerabilityDetection,
    CodeQualityAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub commit_sha: String,
    pub analysis_type: AnalysisType,
    pub security_score: f64,
    pub quality_score: f64,
    pub vulnerabilities: Vec<VulnerabilityFinding>,
    pub recommendations: Vec<SecurityRecommendation>,
    pub analysis_duration_ms: u64,
    pub analyzer_version: String,
    pub raw_results: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityFinding {
    pub id: Uuid,
    pub vulnerability_type: VulnerabilityType,
    pub severity: Severity,
    pub confidence_score: f64,
    pub file_path: String,
    pub line_number: Option<u32>,
    pub code_snippet: Option<String>,
    pub description: String,
    pub recommendation: String,
    pub cve_id: Option<String>,
    pub is_false_positive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VulnerabilityType {
    // Sui Move specific vulnerabilities
    UnauthorizedAccess,
    ResourceExhaustion,
    IntegerOverflow,
    LogicError,
    TimestampDependence,
    ReentrancyLike,
    InsufficientValidation,
    
    // General smart contract vulnerabilities
    AccessControl,
    Other(String),
}

impl std::fmt::Display for VulnerabilityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VulnerabilityType::UnauthorizedAccess => write!(f, "Unauthorized Access"),
            VulnerabilityType::ResourceExhaustion => write!(f, "Resource Exhaustion"),
            VulnerabilityType::IntegerOverflow => write!(f, "Integer Overflow"),
            VulnerabilityType::LogicError => write!(f, "Logic Error"),
            VulnerabilityType::TimestampDependence => write!(f, "Timestamp Dependence"),
            VulnerabilityType::ReentrancyLike => write!(f, "Reentrancy-like"),
            VulnerabilityType::InsufficientValidation => write!(f, "Insufficient Validation"),
            VulnerabilityType::AccessControl => write!(f, "Access Control"),
            VulnerabilityType::Other(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "Critical"),
            Severity::High => write!(f, "High"),
            Severity::Medium => write!(f, "Medium"),
            Severity::Low => write!(f, "Low"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRecommendation {
    pub id: Uuid,
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub code_examples: Vec<CodeExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    AccessControl,
    InputValidation,
    ErrorHandling,
    GasOptimization,
    CodeStructure,
    Testing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    pub title: String,
    pub before: Option<String>,
    pub after: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticAnalysisConfig {
    pub enabled_patterns: Vec<String>,
    pub severity_thresholds: SeverityThresholds,
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityThresholds {
    pub critical_min_confidence: f64,
    pub high_min_confidence: f64,
    pub medium_min_confidence: f64,
    pub low_min_confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMAnalysisConfig {
    pub provider: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f64,
    pub prompts: LLMPrompts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMPrompts {
    pub vulnerability_detection: String,
    pub code_review: String,
    pub security_assessment: String,
}