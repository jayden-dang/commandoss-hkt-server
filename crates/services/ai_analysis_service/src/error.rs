use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Analysis failed: {message}")]
    AnalysisFailed { message: String },

    #[error("LLM API error: {message}")]
    LLMApiError { message: String },

    #[error("Pattern matching error: {message}")]
    PatternMatchingError { message: String },

    #[error("File parsing error: {file_path} - {message}")]
    FileParsingError { file_path: String, message: String },

    #[error("Vulnerability scoring error: {message}")]
    VulnerabilityScoringError { message: String },

    #[error("Database error: {message}")]
    DatabaseError { message: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("External service error: {service} - {message}")]
    ExternalServiceError { service: String, message: String },

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::ExternalServiceError {
            service: "HTTP Client".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Internal(format!("JSON serialization error: {}", err))
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Self {
        Error::PatternMatchingError {
            message: err.to_string(),
        }
    }
}