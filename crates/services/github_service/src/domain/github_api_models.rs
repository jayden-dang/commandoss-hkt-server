use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepository {
    pub id: u64,
    pub node_id: String,
    pub name: String,
    pub full_name: String,
    pub owner: GitHubUser,
    pub description: Option<String>,
    pub language: Option<String>,
    pub stargazers_count: u32,
    pub forks_count: u32,
    pub default_branch: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: u64,
    pub login: String,
    pub avatar_url: String,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubContent {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub sha: String,
    pub size: Option<u64>,
    pub url: String,
    pub html_url: String,
    pub git_url: String,
    pub download_url: Option<String>,
    pub content: Option<String>,
    pub encoding: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubWebhook {
    pub id: u64,
    pub name: String,
    pub active: bool,
    pub events: Vec<String>,
    pub config: GitHubWebhookConfig,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub url: String,
    pub test_url: String,
    pub ping_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubWebhookConfig {
    pub url: String,
    pub content_type: String,
    pub secret: Option<String>,
    pub insecure_ssl: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebhookRequest {
    pub name: String,
    pub active: bool,
    pub events: Vec<String>,
    pub config: GitHubWebhookConfig,
}