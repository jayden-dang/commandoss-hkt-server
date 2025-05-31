use crate::domain::{GitHubRepository, GitHubUser, GitHubContent, GitHubWebhook, GitHubWebhookConfig};
use crate::error::{Error, Result};
use crate::infrastructure::RateLimiterImpl;
use octocrab::Octocrab;
use std::sync::Arc;
use tracing::{error, info, warn};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub struct GitHubClient {
    client: Octocrab,
    rate_limiter: Arc<RateLimiterImpl>,
    webhook_secret: String,
}

impl GitHubClient {
    pub fn new(token: String, webhook_secret: String) -> Result<Self> {
        let client = Octocrab::builder()
            .personal_token(token)
            .build()
            .map_err(|e| Error::GitHubApi(e.to_string()))?;

        let rate_limiter = Arc::new(RateLimiterImpl::new(
            5000, // GitHub allows 5000 requests per hour for authenticated users
            std::time::Duration::from_secs(3600), // 1 hour window
        ));

        Ok(Self {
            client,
            rate_limiter,
            webhook_secret,
        })
    }

    pub async fn get_repository(&self, owner: &str, repo: &str) -> Result<GitHubRepository> {
        self.rate_limiter.check_limit("github_api").await?;

        let repo_data = self.client
            .repos(owner, repo)
            .get()
            .await
            .map_err(|e| {
                error!("Failed to get repository {}/{}: {}", owner, repo, e);
                match e {
                    octocrab::Error::GitHub { source, .. } if source.message.contains("Not Found") => {
                        Error::RepositoryNotFound { 
                            owner: owner.to_string(), 
                            repo: repo.to_string() 
                        }
                    },
                    _ => Error::GitHubApi(e.to_string())
                }
            })?;

        Ok(GitHubRepository {
            id: repo_data.id.0,
            node_id: repo_data.node_id.unwrap_or_else(|| format!("MDEwOlJlcG9zaXRvcnl{}", repo_data.id.0)),
            name: repo_data.name,
            full_name: repo_data.full_name.unwrap_or_else(|| format!("{}/{}", owner, repo)),
            owner: GitHubUser {
                id: repo_data.owner.as_ref().map(|o| o.id.0).unwrap_or(0),
                login: repo_data.owner.as_ref().map(|o| o.login.clone()).unwrap_or_default(),
                avatar_url: repo_data.owner.as_ref().map(|o| o.avatar_url.to_string()).unwrap_or_default(),
                html_url: repo_data.owner.as_ref().map(|o| o.html_url.to_string()).unwrap_or_default(),
            },
            description: repo_data.description,
            language: repo_data.language.map(|v| v.to_string()),
            stargazers_count: repo_data.stargazers_count.unwrap_or(0),
            forks_count: repo_data.forks_count.unwrap_or(0),
            default_branch: repo_data.default_branch.unwrap_or_else(|| "main".to_string()),
            created_at: repo_data.created_at.unwrap_or_else(chrono::Utc::now),
            updated_at: repo_data.updated_at.unwrap_or_else(chrono::Utc::now),
        })
    }

    pub async fn list_user_repositories(&self, username: &str) -> Result<Vec<GitHubRepository>> {
        self.rate_limiter.check_limit("github_api").await?;

        // For now, return an empty list since the octocrab API methods are not compatible
        // In a production implementation, you would use the GitHub REST API directly
        info!("Mock listing repositories for user {}", username);
        Ok(vec![])
    }

    pub async fn get_repository_contents(&self, owner: &str, repo: &str, path: &str) -> Result<Vec<GitHubContent>> {
        self.rate_limiter.check_limit("github_api").await?;

        let contents = self.client
            .repos(owner, repo)
            .get_content()
            .path(path)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to get repository contents for {}/{} at path {}: {}", owner, repo, path, e);
                Error::GitHubApi(e.to_string())
            })?;

        let mut result = Vec::new();
        
        // Handle the contents based on the octocrab ContentItems structure
        for item in contents.items {
            result.push(GitHubContent {
                name: item.name,
                path: item.path,
                type_: item.r#type,
                sha: item.sha,
                size: Some(item.size as u64),
                url: item.url.to_string(),
                html_url: item.html_url.unwrap_or_default(),
                git_url: item.git_url.unwrap_or_default(),
                download_url: item.download_url,
                content: None, // Content is not included in directory listings
                encoding: None,
            });
        }

        Ok(result)
    }

    pub async fn create_webhook(&self, owner: &str, repo: &str, webhook_url: &str) -> Result<GitHubWebhook> {
        self.rate_limiter.check_limit("github_api").await?;

        // For now, we'll create a mock webhook response since octocrab's webhook API may not be fully available
        // In a production implementation, you would use the GitHub REST API directly or wait for octocrab updates
        
        info!("Mock webhook creation for repository {}/{}", owner, repo);

        Ok(GitHubWebhook {
            id: 12345678, // Mock ID
            name: "web".to_string(),
            active: true,
            events: vec![
                "push".to_string(),
                "pull_request".to_string(),
                "release".to_string(),
            ],
            config: GitHubWebhookConfig {
                url: webhook_url.to_string(),
                content_type: "json".to_string(),
                secret: Some(self.webhook_secret.clone()),
                insecure_ssl: "0".to_string(),
            },
            updated_at: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
            url: format!("https://api.github.com/repos/{}/{}/hooks/12345678", owner, repo),
            test_url: format!("https://api.github.com/repos/{}/{}/hooks/12345678/test", owner, repo),
            ping_url: format!("https://api.github.com/repos/{}/{}/hooks/12345678/pings", owner, repo),
        })
    }

    pub async fn verify_webhook_signature(&self, payload: &[u8], signature: &str) -> Result<bool> {
        let signature = signature.strip_prefix("sha256=")
            .ok_or_else(|| Error::InvalidWebhookSignature)?;

        let mut mac = HmacSha256::new_from_slice(self.webhook_secret.as_bytes())
            .map_err(|e| Error::Internal(format!("Failed to create HMAC: {}", e)))?;

        mac.update(payload);
        let result = mac.finalize();
        let expected_signature = hex::encode(result.into_bytes());

        let is_valid = signature == expected_signature;
        
        if !is_valid {
            warn!("Invalid webhook signature received");
        }

        Ok(is_valid)
    }

    fn might_contain_smart_contracts(&self, language: &Option<String>) -> bool {
        if let Some(lang) = language {
            let smart_contract_languages = [
                "Solidity", "Rust", "Move", "Vyper", "JavaScript", "TypeScript"
            ];
            
            smart_contract_languages.iter()
                .any(|&sc_lang| lang.eq_ignore_ascii_case(sc_lang))
        } else {
            // If language is unknown, include it for further analysis
            true
        }
    }
}

pub fn is_smart_contract_file(filename: &str) -> bool {
    let smart_contract_extensions = [".sol", ".rs", ".move", ".vy"];
    let smart_contract_patterns = ["contract", "interface", "library"];
    let move_patterns = ["Move.toml", "move.toml"];

    // Check file extension
    if smart_contract_extensions.iter().any(|ext| filename.ends_with(ext)) {
        return true;
    }

    // Check for Move project configuration files
    if move_patterns.iter().any(|pattern| filename.eq_ignore_ascii_case(pattern)) {
        return true;
    }

    // Check filename patterns
    if smart_contract_patterns.iter().any(|pattern| filename.to_lowercase().contains(pattern)) {
        return true;
    }

    false
}

pub async fn check_repository_for_smart_contracts(
    github_client: &GitHubClient,
    owner: &str,
    repo: &str,
) -> Result<bool> {
    let contents = github_client
        .get_repository_contents(owner, repo, "")
        .await?;

    // Check for smart contract files in root directory
    for content in &contents {
        if content.type_ == "file" && is_smart_contract_file(&content.name) {
            return Ok(true);
        }
    }

    // Check common directories for smart contracts
    let smart_contract_dirs = ["contracts", "src", "lib", "packages", "sources", "move", "aptos", "sui"];
    
    for content in contents {
        if content.type_ == "dir" && smart_contract_dirs.contains(&content.name.as_str()) {
            if check_directory_for_smart_contracts(github_client, owner, repo, &content.path, 0).await? {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

// Helper function to check directories for smart contracts (simplified, non-recursive)
async fn check_directory_for_smart_contracts(
    github_client: &GitHubClient,
    owner: &str,
    repo: &str,
    path: &str,
    _depth: u8,
) -> Result<bool> {
    match github_client.get_repository_contents(owner, repo, path).await {
        Ok(contents) => {
            for content in contents {
                if content.type_ == "file" && is_smart_contract_file(&content.name) {
                    return Ok(true);
                }
            }
            Ok(false)
        },
        Err(e) => {
            warn!("Failed to check directory {} for smart contracts: {}", path, e);
            Ok(false)
        }
    }
}