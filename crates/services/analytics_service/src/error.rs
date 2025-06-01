use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize)]
pub enum Error {
    DeveloperNotFound(String),
    RepositoryNotFound(String),
    TeamNotFound(String),
    InvalidTimeRange,
    InvalidFilter(String),
    CalculationError(String),
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
            Error::RepositoryNotFound(id) => write!(f, "Repository not found: {}", id),
            Error::TeamNotFound(id) => write!(f, "Team not found: {}", id),
            Error::InvalidTimeRange => write!(f, "Invalid time range specified"),
            Error::InvalidFilter(msg) => write!(f, "Invalid filter: {}", msg),
            Error::CalculationError(msg) => write!(f, "Calculation error: {}", msg),
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
            Error::DeveloperNotFound(_) | Error::RepositoryNotFound(_) | Error::TeamNotFound(_) => {
                StatusCode::NOT_FOUND
            }
            Error::InvalidTimeRange | Error::InvalidFilter(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status_code, self.to_string()).into_response()
    }
}