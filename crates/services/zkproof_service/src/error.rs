use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid proof input: {0}")]
    InvalidInput(String),
    
    #[error("Proof generation failed: {0}")]
    ProofGeneration(String),
    
    #[error("Proof verification failed: {0}")]
    ProofVerification(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            Error::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            Error::ProofGeneration(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            Error::ProofVerification(msg) => (StatusCode::BAD_REQUEST, msg),
            Error::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
            Error::Serialization(_) => (StatusCode::BAD_REQUEST, "Invalid data format".to_string()),
            Error::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = Json(json!({
            "error": {
                "message": error_message,
                "type": "ZkProofServiceError"
            }
        }));

        (status, body).into_response()
    }
}