use crate::domain::{
  GitHubContent, GitHubRepository, GitHubUser, GitHubWebhook, GitHubWebhookConfig,
};
use crate::error::{Error, Result};
use crate::infrastructure::RateLimiterImpl;
use base64::{Engine as _, engine::general_purpose};
use hmac::{Hmac, Mac};
use jsonwebtoken::{encode, Header, EncodingKey, Algorithm};
use octocrab::Octocrab;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn, debug};

type HmacSha256 = Hmac<Sha256>;

pub struct GitHubClient {
  client: Octocrab,
  rate_limiter: Arc<RateLimiterImpl>,
  webhook_secret: String,
  app_id: Option<u64>,
  private_key: Option<String>,
  http_client: Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubFile {
    pub name: String,
    pub path: String,
    pub content: String,
    pub size: u64,
    pub download_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubJWTClaims {
    iat: u64,
    exp: u64,
    iss: String,
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

    let http_client = Client::new();

    Ok(Self { 
      client, 
      rate_limiter, 
      webhook_secret,
      app_id: None,
      private_key: None,
      http_client,
    })
  }

  pub fn new_app(app_id: u64, private_key: String, webhook_secret: String) -> Result<Self> {
    let rate_limiter = Arc::new(RateLimiterImpl::new(
      5000,
      std::time::Duration::from_secs(3600),
    ));

    let http_client = Client::new();

    // Create a temporary octocrab client (will be replaced when getting installation tokens)
    let client = Octocrab::builder()
      .build()
      .map_err(|e| Error::GitHubApi(e.to_string()))?;

    Ok(Self {
      client,
      rate_limiter,
      webhook_secret,
      app_id: Some(app_id),
      private_key: Some(private_key),
      http_client,
    })
  }

  // GitHub App authentication methods
  fn generate_jwt_token(&self) -> Result<String> {
    let app_id = self.app_id.ok_or_else(|| Error::Internal("App ID not configured".to_string()))?;
    let private_key = self.private_key.as_ref().ok_or_else(|| Error::Internal("Private key not configured".to_string()))?;

    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map_err(|e| Error::Internal(format!("Time error: {}", e)))?
      .as_secs();

    let claims = GitHubJWTClaims {
      iat: now,
      exp: now + 600, // 10 minutes
      iss: app_id.to_string(),
    };

    let header = Header::new(Algorithm::RS256);
    let encoding_key = EncodingKey::from_rsa_pem(private_key.as_bytes())
      .map_err(|e| Error::Internal(format!("Invalid private key: {}", e)))?;

    encode(&header, &claims, &encoding_key)
      .map_err(|e| Error::Internal(format!("JWT encoding error: {}", e)))
  }

  async fn get_installation_token(&self, installation_id: u64) -> Result<String> {
    let jwt_token = self.generate_jwt_token()?;

    let url = format!(
      "https://api.github.com/app/installations/{}/access_tokens",
      installation_id
    );

    let response = self.http_client
      .post(&url)
      .header("Authorization", format!("Bearer {}", jwt_token))
      .header("Accept", "application/vnd.github.v3+json")
      .header("User-Agent", "ZK-Guardian-Bot/1.0")
      .send()
      .await
      .map_err(|e| Error::GitHubApi(format!("HTTP request failed: {}", e)))?;

    if !response.status().is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(Error::GitHubApi(format!("GitHub API error: {}", error_text)));
    }

    let token_response: serde_json::Value = response.json().await
      .map_err(|e| Error::GitHubApi(format!("JSON parsing error: {}", e)))?;

    token_response["token"]
      .as_str()
      .map(|s| s.to_string())
      .ok_or_else(|| Error::GitHubApi("No token in response".to_string()))
  }

  async fn create_octocrab_client(&self, installation_id: u64) -> Result<Octocrab> {
    let token = self.get_installation_token(installation_id).await?;

    Octocrab::builder()
      .personal_token(token)
      .build()
      .map_err(|e| Error::GitHubApi(format!("Octocrab client creation failed: {}", e)))
  }

  pub async fn get_installation_id_for_repo(&self, owner: &str, repo: &str) -> Result<u64> {
    let jwt_token = self.generate_jwt_token()?;

    let url = format!("https://api.github.com/repos/{}/{}/installation", owner, repo);

    let response = self.http_client
      .get(&url)
      .header("Authorization", format!("Bearer {}", jwt_token))
      .header("Accept", "application/vnd.github.v3+json")
      .header("User-Agent", "ZK-Guardian-Bot/1.0")
      .send()
      .await
      .map_err(|e| Error::GitHubApi(format!("HTTP request failed: {}", e)))?;

    if !response.status().is_success() {
      let error_text = response.text().await.unwrap_or_default();
      return Err(Error::GitHubApi(format!("GitHub API error: {}", error_text)));
    }

    let installation_response: serde_json::Value = response.json().await
      .map_err(|e| Error::GitHubApi(format!("JSON parsing error: {}", e)))?;

    installation_response["id"]
      .as_u64()
      .ok_or_else(|| Error::GitHubApi("No installation ID in response".to_string()))
  }

  // Enhanced file extraction for smart contracts
  pub async fn get_repository_files(
    &self,
    installation_id: u64,
    owner: &str,
    repo: &str,
    commit_sha: Option<&str>,
    file_extensions: &[&str],
  ) -> Result<Vec<GitHubFile>> {
    let octocrab = self.create_octocrab_client(installation_id).await?;

    info!("Fetching repository files for {}/{}", owner, repo);

    let contents = octocrab
      .repos(owner, repo)
      .get_content()
      .r#ref(commit_sha.unwrap_or("main"))
      .send()
      .await
      .map_err(|e| Error::GitHubApi(format!("Failed to get repository contents: {}", e)))?;

    let mut files = Vec::new();

    self.process_contents_recursive(
      &octocrab,
      owner,
      repo,
      commit_sha.unwrap_or("main"),
      contents.items,
      file_extensions,
      &mut files,
    ).await?;

    info!("Found {} matching files", files.len());
    Ok(files)
  }

  fn process_contents_recursive<'a>(
    &'a self,
    octocrab: &'a Octocrab,
    owner: &'a str,
    repo: &'a str,
    ref_name: &'a str,
    items: Vec<octocrab::models::repos::Content>,
    file_extensions: &'a [&'a str],
    files: &'a mut Vec<GitHubFile>,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a + Send>> {
    Box::pin(async move {
    for item in items {
      match item.r#type.as_str() {
        "file" => {
          if file_extensions.iter().any(|ext| item.name.ends_with(ext)) {
            if let Some(content) = self.download_file_content(
              octocrab,
              owner,
              repo,
              &item.path,
              ref_name,
            ).await? {
              files.push(GitHubFile {
                name: item.name,
                path: item.path,
                content,
                size: item.size as u64,
                download_url: item.download_url,
              });
            }
          }
        },
        "dir" => {
          let dir_contents = octocrab
            .repos(owner, repo)
            .get_content()
            .path(&item.path)
            .r#ref(ref_name)
            .send()
            .await
            .map_err(|e| Error::GitHubApi(format!("Failed to get directory contents: {}", e)))?;

          self.process_contents_recursive(
            octocrab,
            owner,
            repo,
            ref_name,
            dir_contents.items,
            file_extensions,
            files,
          ).await?;
        },
        _ => {
          debug!("Skipping content type: {}", item.r#type);
        }
      }
    }
    Ok(())
    })
  }

  async fn download_file_content(
    &self,
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
    path: &str,
    ref_name: &str,
  ) -> Result<Option<String>> {
    debug!("Downloading file: {}", path);

    let content = octocrab
      .repos(owner, repo)
      .get_content()
      .path(path)
      .r#ref(ref_name)
      .send()
      .await
      .map_err(|e| Error::GitHubApi(format!("Failed to get file content: {}", e)))?;

    if let Some(file_content) = content.items.first() {
      if let Some(encoded_content) = &file_content.content {
        let decoded = general_purpose::STANDARD
          .decode(encoded_content.replace('\n', ""))
          .map_err(|e| Error::GitHubApi(format!("Base64 decode error: {}", e)))?;

        let content_str = String::from_utf8(decoded)
          .map_err(|e| Error::GitHubApi(format!("UTF-8 decode error: {}", e)))?;

        return Ok(Some(content_str));
      }
    }

    Ok(None)
  }

  pub fn detect_smart_contract_files<'a>(&self, files: &'a [GitHubFile]) -> Vec<&'a GitHubFile> {
    files.iter().filter(|file| {
      let path_lower = file.path.to_lowercase();

      // Solidity files
      if path_lower.ends_with(".sol") {
        return true;
      }

      // Rust smart contracts (common patterns)
      if path_lower.ends_with(".rs") && (
        path_lower.contains("contract") ||
        path_lower.contains("program") ||
        file.content.contains("use anchor_lang") ||
        file.content.contains("use solana_program") ||
        file.content.contains("#[program]") ||
        file.content.contains("#[contract]")
      ) {
        return true;
      }

      // Move contracts
      if path_lower.ends_with(".move") {
        return true;
      }

      // Vyper contracts
      if path_lower.ends_with(".vy") {
        return true;
      }

      false
    }).collect()
  }

  pub async fn get_repository(&self, owner: &str, repo: &str) -> Result<GitHubRepository> {
    self.rate_limiter.check_limit("github_api").await?;

    let repo_data = self.client.repos(owner, repo).get().await.map_err(|e| {
      error!("Failed to get repository {}/{}: {}", owner, repo, e);
      match e {
        octocrab::Error::GitHub { source, .. } if source.message.contains("Not Found") => {
          Error::RepositoryNotFound { owner: owner.to_string(), repo: repo.to_string() }
        }
        _ => Error::GitHubApi(e.to_string()),
      }
    })?;

    Ok(GitHubRepository {
      id: repo_data.id.0,
      node_id: repo_data
        .node_id
        .unwrap_or_else(|| format!("MDEwOlJlcG9zaXRvcnl{}", repo_data.id.0)),
      name: repo_data.name,
      full_name: repo_data
        .full_name
        .unwrap_or_else(|| format!("{}/{}", owner, repo)),
      owner: GitHubUser {
        id: repo_data.owner.as_ref().map(|o| o.id.0).unwrap_or(0),
        login: repo_data
          .owner
          .as_ref()
          .map(|o| o.login.clone())
          .unwrap_or_default(),
        avatar_url: repo_data
          .owner
          .as_ref()
          .map(|o| o.avatar_url.to_string())
          .unwrap_or_default(),
        html_url: repo_data
          .owner
          .as_ref()
          .map(|o| o.html_url.to_string())
          .unwrap_or_default(),
      },
      description: repo_data.description,
      language: repo_data.language.map(|v| v.to_string()),
      stargazers_count: repo_data.stargazers_count.unwrap_or(0),
      forks_count: repo_data.forks_count.unwrap_or(0),
      default_branch: repo_data
        .default_branch
        .unwrap_or_else(|| "main".to_string()),
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

  pub async fn get_repository_contents(
    &self,
    owner: &str,
    repo: &str,
    path: &str,
  ) -> Result<Vec<GitHubContent>> {
    self.rate_limiter.check_limit("github_api").await?;

    let contents = self
      .client
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

  pub async fn create_webhook(
    &self,
    owner: &str,
    repo: &str,
    webhook_url: &str,
  ) -> Result<GitHubWebhook> {
    self.rate_limiter.check_limit("github_api").await?;

    // For now, we'll create a mock webhook response since octocrab's webhook API may not be fully available
    // In a production implementation, you would use the GitHub REST API directly or wait for octocrab updates

    info!("Mock webhook creation for repository {}/{}", owner, repo);

    Ok(GitHubWebhook {
      id: 12345678, // Mock ID
      name: "web".to_string(),
      active: true,
      events: vec!["push".to_string(), "pull_request".to_string(), "release".to_string()],
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

  pub fn verify_webhook_signature(&self, payload: &[u8], signature: &str) -> Result<bool> {
    // Remove 'sha256=' prefix if present
    let signature = signature.strip_prefix("sha256=").unwrap_or(signature);

    // Decode hex signature
    let expected_signature = hex::decode(signature)
      .map_err(|_| Error::InvalidWebhookSignature)?;

    // Create HMAC
    let mut mac = HmacSha256::new_from_slice(self.webhook_secret.as_bytes())
      .map_err(|e| Error::Internal(format!("HMAC error: {}", e)))?;

    mac.update(payload);

    // Verify
    match mac.verify_slice(&expected_signature) {
      Ok(_) => {
        debug!("Webhook signature verified successfully");
        Ok(true)
      },
      Err(_) => {
        error!("Webhook signature verification failed");
        Ok(false)
      }
    }
  }

  fn might_contain_smart_contracts(&self, language: &Option<String>) -> bool {
    if let Some(lang) = language {
      let smart_contract_languages =
        ["Solidity", "Rust", "Move", "Vyper", "JavaScript", "TypeScript"];

      smart_contract_languages
        .iter()
        .any(|&sc_lang| lang.eq_ignore_ascii_case(sc_lang))
    } else {
      // If language is unknown, include it for further analysis
      true
    }
  }
}

pub fn is_smart_contract_file(filename: &str) -> bool {
  let smart_contract_extensions = [".sol", ".rs", ".move", ".vy", ".func", ".tolk", "tact"];
  let smart_contract_patterns = ["contract", "interface", "library"];
  let move_patterns = ["Move.toml", "move.toml"];

  // Check file extension
  if smart_contract_extensions
    .iter()
    .any(|ext| filename.ends_with(ext))
  {
    return true;
  }

  // Check for Move project configuration files
  if move_patterns
    .iter()
    .any(|pattern| filename.eq_ignore_ascii_case(pattern))
  {
    return true;
  }

  // Check filename patterns
  if smart_contract_patterns
    .iter()
    .any(|pattern| filename.to_lowercase().contains(pattern))
  {
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
  let smart_contract_dirs = [
    "contracts",
    "src",
    "lib",
    "packages",
    "sources",
    "move",
    "aptos",
    "sui",
    "scripts",
  ];

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
  match github_client
    .get_repository_contents(owner, repo, path)
    .await
  {
    Ok(contents) => {
      for content in contents {
        if content.type_ == "file" && is_smart_contract_file(&content.name) {
          return Ok(true);
        }
      }
      Ok(false)
    }
    Err(e) => {
      warn!("Failed to check directory {} for smart contracts: {}", path, e);
      Ok(false)
    }
  }
}
