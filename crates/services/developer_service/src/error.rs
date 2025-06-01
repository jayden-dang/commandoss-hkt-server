use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize)]
pub enum Error {
    DeveloperNotFound(String),
    DeveloperAlreadyExists(String),
    InvalidSkill(String),
    InvalidVerification(String),
    InsufficientReputation(String),
    CollaboratorNotFound(String),
    NetworkError(String),
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
            Error::DeveloperNotFound(id) => write!(f, "Developer not found: {}", id),
            Error::DeveloperAlreadyExists(id) => write!(f, "Developer already exists: {}", id),
            Error::InvalidSkill(skill) => write!(f, "Invalid skill: {}", skill),
            Error::InvalidVerification(msg) => write!(f, "Invalid verification: {}", msg),
            Error::InsufficientReputation(msg) => write!(f, "Insufficient reputation: {}", msg),
            Error::CollaboratorNotFound(id) => write!(f, "Collaborator not found: {}", id),
            Error::NetworkError(msg) => write!(f, "Network error: {}", msg),
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
            Error::DeveloperNotFound(_) | Error::CollaboratorNotFound(_) => StatusCode::NOT_FOUND,
            Error::DeveloperAlreadyExists(_) => StatusCode::CONFLICT,
            Error::InvalidSkill(_) | Error::InvalidVerification(_) => StatusCode::BAD_REQUEST,
            Error::InsufficientReputation(_) => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status_code, self.to_string()).into_response()
    }
}