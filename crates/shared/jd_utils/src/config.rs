use serde::Deserialize;

use crate::error::Error;

#[derive(Deserialize)]
pub struct WebConfig {
  pub addr: String,
}

#[derive(Deserialize)]
pub struct Postgres {
  pub dsn: String,
  pub max_conns: u32,
  pub min_conns: Option<u32>,
  pub connect_timeout_secs: Option<u64>,
  pub idle_timeout_secs: Option<u64>,
  pub max_lifetime_secs: Option<u64>,
  pub ssl_mode: Option<String>,
  pub auto_migrate: Option<bool>,
  pub enable_transactions: Option<bool>,
  pub test_connection: Option<bool>,
  pub retry_attempts: Option<u32>,
  pub retry_delay_ms: Option<u64>,
}

#[derive(Deserialize)]
pub struct Redis {
  pub addr: String,
}

#[derive(Deserialize)]
pub struct SuiConfig {
  pub env: String,
  pub sponsor_address: Option<String>,
  pub sponsor_private_key: Option<String>,
  pub max_gas_budget: Option<u64>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GitHubConfig {
  pub token: Option<String>,
  pub app_id: Option<u64>,
  pub client_id: Option<String>,
  pub client_secret: Option<String>,
  pub private_key_path: Option<String>,
  pub private_key: Option<String>,
  pub webhook_secret: String,
  pub webhook_base_url: Option<String>,
  pub max_queue_size: Option<usize>,
  pub rate_limit_per_hour: Option<u32>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ZkProofConfig {
  pub timeout_secs: Option<u64>,
  pub max_retries: Option<u32>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct BehaviorAnalysisConfig {
  pub batch_size: Option<usize>,
  pub timeout_secs: Option<u64>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ScoringConfig {
  pub model_version: Option<String>,
  pub confidence_threshold: Option<f64>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ReputationConfig {
  pub update_interval_secs: Option<u64>,
  pub min_score: Option<f64>,
  pub max_score: Option<f64>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct MetricsConfig {
  pub enable: Option<bool>,
  pub port: Option<u16>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DevelopmentConfig {
  pub dev_mode: Option<bool>,
  pub debug_queries: Option<bool>,
  pub rust_log: Option<String>,
}

#[derive(Deserialize)]
pub struct Config {
  pub web: WebConfig,
  pub postgres: Postgres,
  pub redis: Redis,
  pub sui: SuiConfig,
  pub github: Option<GitHubConfig>,
  pub zk_proof: Option<ZkProofConfig>,
  pub behavior_analysis: Option<BehaviorAnalysisConfig>,
  pub scoring: Option<ScoringConfig>,
  pub reputation: Option<ReputationConfig>,
  pub metrics: Option<MetricsConfig>,
  pub development: Option<DevelopmentConfig>,
  #[serde(rename = "auth_jwt_secret")]
  pub auth_jwt_secret: String,
}

impl Config {
  pub fn from_env() -> crate::Result<Config> {
    config::Config::builder()
      .add_source(config::Environment::default())
      .build()
      .map_err(Error::Config)?
      .try_deserialize::<Config>()
      .map_err(Error::Config)
  }
}
