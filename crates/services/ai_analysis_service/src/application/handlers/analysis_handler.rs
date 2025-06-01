use crate::application::use_cases::analysis_use_cases::AnalysisUseCases;
use crate::error::{Error, Result};
use crate::models::requests::{AnalyzeRepositoryRequest, AnalyzeCodeRequest, MarkVulnerabilityRequest, GetAnalysisHistoryRequest};
use crate::models::responses::{AnalysisResponse, DetailedAnalysisResponse, CodeAnalysisResponse, AnalysisStatusResponse, VulnerabilityListResponse, AnalysisHistoryResponse};
use axum::{
    extract::{Path, Query, State, Json},
    http::StatusCode,
    response::Json as ResponseJson,
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

pub struct AnalysisHandler {
    analysis_use_cases: Arc<AnalysisUseCases>,
}

impl AnalysisHandler {
    pub fn new(analysis_use_cases: Arc<AnalysisUseCases>) -> Self {
        Self { analysis_use_cases }
    }

    pub async fn analyze_repository(
        &self,
        request: AnalyzeRepositoryRequest,
        file_contents: HashMap<String, String>,
    ) -> Result<AnalysisResponse> {
        info!("Handling repository analysis request for: {}", request.repository_id);
        
        self.analysis_use_cases
            .analyze_repository(request, file_contents)
            .await
    }

    pub async fn analyze_code(&self, request: AnalyzeCodeRequest) -> Result<CodeAnalysisResponse> {
        info!("Handling code analysis request for file: {}", request.file_path);
        
        self.analysis_use_cases.analyze_code(request).await
    }

    pub async fn get_analysis_status(&self, repository_id: Uuid) -> Result<AnalysisStatusResponse> {
        self.analysis_use_cases.get_analysis_status(repository_id).await
    }

    pub async fn get_detailed_analysis(&self, analysis_id: Uuid) -> Result<DetailedAnalysisResponse> {
        self.analysis_use_cases.get_detailed_analysis(analysis_id).await
    }

    pub async fn handle_vulnerability_action(&self, request: MarkVulnerabilityRequest) -> Result<()> {
        self.analysis_use_cases.handle_vulnerability_action(request).await
    }

    pub async fn get_repository_vulnerabilities(&self, repository_id: Uuid) -> Result<VulnerabilityListResponse> {
        self.analysis_use_cases.get_repository_vulnerabilities(repository_id).await
    }

    pub async fn get_analysis_history(&self, repository_id: Uuid, limit: Option<u32>) -> Result<AnalysisHistoryResponse> {
        self.analysis_use_cases.get_analysis_history(repository_id, limit).await
    }
}

// Axum handler functions
pub async fn analyze_repository(
    State(handler): State<Arc<AnalysisHandler>>,
    Json(request): Json<AnalyzeRepositoryRequest>,
) -> std::result::Result<ResponseJson<AnalysisResponse>, StatusCode> {
    // Note: In real implementation, file_contents would be fetched from Git
    let file_contents = HashMap::new(); // Placeholder
    
    match handler.analyze_repository(request, file_contents).await {
        Ok(response) => Ok(ResponseJson(response)),
        Err(Error::AnalysisFailed { .. }) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        Err(e) => {
            error!("Failed to analyze repository: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn analyze_code(
    State(handler): State<Arc<AnalysisHandler>>,
    Json(request): Json<AnalyzeCodeRequest>,
) -> std::result::Result<ResponseJson<CodeAnalysisResponse>, StatusCode> {
    match handler.analyze_code(request).await {
        Ok(response) => Ok(ResponseJson(response)),
        Err(Error::AnalysisFailed { .. }) => Err(StatusCode::UNPROCESSABLE_ENTITY),
        Err(e) => {
            error!("Failed to analyze code: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_analysis_status(
    State(handler): State<Arc<AnalysisHandler>>,
    Path(repository_id): Path<Uuid>,
) -> std::result::Result<ResponseJson<AnalysisStatusResponse>, StatusCode> {
    match handler.get_analysis_status(repository_id).await {
        Ok(response) => Ok(ResponseJson(response)),
        Err(e) => {
            error!("Failed to get analysis status for repository {}: {}", repository_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_detailed_analysis(
    State(handler): State<Arc<AnalysisHandler>>,
    Path(analysis_id): Path<Uuid>,
) -> std::result::Result<ResponseJson<DetailedAnalysisResponse>, StatusCode> {
    match handler.get_detailed_analysis(analysis_id).await {
        Ok(response) => Ok(ResponseJson(response)),
        Err(Error::AnalysisFailed { .. }) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get detailed analysis {}: {}", analysis_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn mark_vulnerability(
    State(handler): State<Arc<AnalysisHandler>>,
    Json(request): Json<MarkVulnerabilityRequest>,
) -> std::result::Result<StatusCode, StatusCode> {
    match handler.handle_vulnerability_action(request).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(Error::AnalysisFailed { .. }) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to handle vulnerability action: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_repository_vulnerabilities(
    State(handler): State<Arc<AnalysisHandler>>,
    Path(repository_id): Path<Uuid>,
) -> std::result::Result<ResponseJson<VulnerabilityListResponse>, StatusCode> {
    match handler.get_repository_vulnerabilities(repository_id).await {
        Ok(response) => Ok(ResponseJson(response)),
        Err(e) => {
            error!("Failed to get vulnerabilities for repository {}: {}", repository_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_analysis_history(
    State(handler): State<Arc<AnalysisHandler>>,
    Path(repository_id): Path<Uuid>,
    Query(params): Query<GetAnalysisHistoryRequest>,
) -> std::result::Result<ResponseJson<AnalysisHistoryResponse>, StatusCode> {
    match handler.get_analysis_history(repository_id, params.limit).await {
        Ok(response) => Ok(ResponseJson(response)),
        Err(e) => {
            error!("Failed to get analysis history for repository {}: {}", repository_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}