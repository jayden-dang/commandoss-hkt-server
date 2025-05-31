use serde::{Deserialize, Serialize};
use super::github_api_models::{GitHubRepository, GitHubUser};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHubWebhookPayload {
    pub action: Option<String>,
    pub repository: GitHubRepository,
    pub sender: GitHubUser,
    #[serde(flatten)]
    pub event_data: GitHubEventData,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum GitHubEventData {
    Push(PushEvent),
    PullRequest(PullRequestEvent),
    Release(ReleaseEvent),
    Other(serde_json::Value),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PushEvent {
    #[serde(rename = "ref")]
    pub ref_: String,
    pub before: String,
    pub after: String,
    pub commits: Vec<GitHubCommit>,
    pub head_commit: Option<GitHubCommit>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHubCommit {
    pub id: String,
    pub tree_id: String,
    pub distinct: bool,
    pub message: String,
    pub timestamp: String,
    pub url: String,
    pub author: GitHubCommitAuthor,
    pub committer: GitHubCommitAuthor,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHubCommitAuthor {
    pub name: String,
    pub email: String,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PullRequestEvent {
    pub action: String,
    pub number: u64,
    pub pull_request: GitHubPullRequest,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHubPullRequest {
    pub id: u64,
    pub node_id: String,
    pub html_url: String,
    pub number: u64,
    pub state: String,
    pub locked: bool,
    pub title: String,
    pub user: GitHubUser,
    pub body: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub merged_at: Option<String>,
    pub merge_commit_sha: Option<String>,
    pub head: GitHubPullRequestRef,
    pub base: GitHubPullRequestRef,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHubPullRequestRef {
    pub label: String,
    #[serde(rename = "ref")]
    pub ref_: String,
    pub sha: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReleaseEvent {
    pub action: String,
    pub release: GitHubRelease,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHubRelease {
    pub id: u64,
    pub node_id: String,
    pub tag_name: String,
    pub target_commitish: String,
    pub name: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: String,
    pub published_at: Option<String>,
    pub body: Option<String>,
}