use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AddRepositoryRequest {
    pub owner: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRepositorySettingsRequest {
    pub monitoring_enabled: Option<bool>,
    pub webhook_secret: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RepositoryListParams {
    pub language: Option<String>,
    pub security_score_min: Option<f64>,
    pub monitoring_enabled: Option<bool>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for RepositoryListParams {
    fn default() -> Self {
        Self {
            language: None,
            security_score_min: None,
            monitoring_enabled: None,
            search: None,
            limit: Some(20),
            offset: Some(0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RepositoryFilters {
    pub language: Option<String>,
    pub security_score_min: Option<f64>,
    pub monitoring_enabled: Option<bool>,
    pub search_term: Option<String>,
}