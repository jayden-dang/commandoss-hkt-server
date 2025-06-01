use crate::domain::analysis_models::{AnalysisResult, VulnerabilityFinding};
use crate::error::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VulnerabilityStatistics {
    pub total_vulnerabilities: i64,
    pub critical_count: i64,
    pub high_count: i64,
    pub medium_count: i64,
    pub low_count: i64,
    pub false_positive_count: i64,
    pub fixed_count: i64,
}

#[async_trait]
pub trait AnalysisRepository: Send + Sync {
    async fn save_analysis_result(&self, result: &AnalysisResult) -> Result<Uuid>;
    
    async fn get_analysis_result(&self, id: Uuid) -> Result<Option<AnalysisResult>>;
    
    async fn get_latest_analysis_for_repository(&self, repository_id: Uuid) -> Result<Option<AnalysisResult>>;
    
    async fn get_analysis_history(&self, repository_id: Uuid, limit: Option<u32>) -> Result<Vec<AnalysisResult>>;
    
    async fn save_vulnerability(&self, vulnerability: &VulnerabilityFinding, analysis_id: Uuid) -> Result<Uuid>;
    
    async fn get_vulnerabilities_for_analysis(&self, analysis_id: Uuid) -> Result<Vec<VulnerabilityFinding>>;
    
    async fn get_vulnerabilities_for_repository(&self, repository_id: Uuid) -> Result<Vec<VulnerabilityFinding>>;
    
    async fn mark_vulnerability_as_false_positive(&self, vulnerability_id: Uuid) -> Result<()>;
    
    async fn mark_vulnerability_as_fixed(&self, vulnerability_id: Uuid) -> Result<()>;
    
    async fn get_vulnerability_statistics(&self, repository_id: Uuid) -> Result<VulnerabilityStatistics>;
}