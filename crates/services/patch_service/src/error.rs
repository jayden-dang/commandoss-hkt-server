use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize)]
pub enum Error {
    PatchNotFound(String),
    VulnerabilityNotFound(String),
    DeveloperNotFound(String),
    InvalidPatchState(String),
    InvalidVote(String),
    AlreadyVoted(String),
    InsufficientReputation(String),
    PatchGenerationFailed(String),
    ValidationFailed(String),
    GithubIntegrationError(String),
    DatabaseError(String),
    ServiceError(String),
    CtxError(String),
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        Error::DatabaseError(e.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::ServiceError(e.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::PatchNotFound(id) => write!(f, "Patch not found: {}", id),
            Error::VulnerabilityNotFound(id) => write!(f, "Vulnerability not found: {}", id),
            Error::DeveloperNotFound(id) => write!(f, "Developer not found: {}", id),
            Error::InvalidPatchState(msg) => write!(f, "Invalid patch state: {}", msg),
            Error::InvalidVote(msg) => write!(f, "Invalid vote: {}", msg),
            Error::AlreadyVoted(msg) => write!(f, "Already voted: {}", msg),
            Error::InsufficientReputation(msg) => write!(f, "Insufficient reputation: {}", msg),
            Error::PatchGenerationFailed(msg) => write!(f, "Patch generation failed: {}", msg),
            Error::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            Error::GithubIntegrationError(msg) => write!(f, "GitHub integration error: {}", msg),
            Error::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            Error::ServiceError(msg) => write!(f, "Service error: {}", msg),
            Error::CtxError(msg) => write!(f, "Context error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status_code = match &self {
            Error::PatchNotFound(_) | Error::VulnerabilityNotFound(_) | Error::DeveloperNotFound(_) => {
                StatusCode::NOT_FOUND
            }
            Error::InvalidPatchState(_) | Error::InvalidVote(_) | Error::AlreadyVoted(_) => {
                StatusCode::BAD_REQUEST
            }
            Error::InsufficientReputation(_) => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status_code, self.to_string()).into_response()
    }
}