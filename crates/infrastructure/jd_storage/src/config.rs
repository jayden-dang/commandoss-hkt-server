use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::time::Duration;
use tracing::{info, warn};

use crate::dbx::{Dbx, Result as DbxResult};

pub type Db = Pool<Postgres>;

/// Database configuration for ZK-Persona system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub database_url: String,
    
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    
    /// Minimum number of connections in the pool
    pub min_connections: u32,
    
    /// Connection timeout in seconds
    pub connect_timeout_secs: u64,
    
    /// Idle timeout in seconds
    pub idle_timeout_secs: Option<u64>,
    
    /// Maximum lifetime of a connection in seconds
    pub max_lifetime_secs: Option<u64>,
    
    /// Enable SSL
    pub ssl_mode: String,
    
    /// Run migrations on startup
    pub auto_migrate: bool,
    
    /// Enable transaction support
    pub enable_transactions: bool,
    
    /// Test connection on startup
    pub test_connection: bool,
    
    /// Connection retry attempts
    pub retry_attempts: u32,
    
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://localhost:5432/zkpersona".to_string(),
            max_connections: 50,
            min_connections: 5,
            connect_timeout_secs: 30,
            idle_timeout_secs: Some(600), // 10 minutes
            max_lifetime_secs: Some(3600), // 1 hour
            ssl_mode: "prefer".to_string(),
            auto_migrate: true,
            enable_transactions: true,
            test_connection: true,
            retry_attempts: 3,
            retry_delay_ms: 1000,
        }
    }
}

impl DatabaseConfig {
    /// Load configuration from environment variables using centralized config
    pub fn from_env() -> Result<Self, DatabaseConfigError> {
        let config = jd_utils::config::Config::from_env()
            .map_err(|e| DatabaseConfigError::InvalidConfig {
                field: "config".to_string(),
                error: format!("Failed to load configuration: {}", e),
            })?;

        Ok(Self {
            database_url: config.postgres.dsn,
            max_connections: config.postgres.max_conns,
            min_connections: config.postgres.min_conns.unwrap_or(5),
            connect_timeout_secs: config.postgres.connect_timeout_secs.unwrap_or(30),
            idle_timeout_secs: config.postgres.idle_timeout_secs,
            max_lifetime_secs: config.postgres.max_lifetime_secs,
            ssl_mode: config.postgres.ssl_mode.unwrap_or_else(|| "prefer".to_string()),
            auto_migrate: config.postgres.auto_migrate.unwrap_or(true),
            enable_transactions: config.postgres.enable_transactions.unwrap_or(true),
            test_connection: config.postgres.test_connection.unwrap_or(true),
            retry_attempts: config.postgres.retry_attempts.unwrap_or(3),
            retry_delay_ms: config.postgres.retry_delay_ms.unwrap_or(1000),
        })
    }

    /// Create from centralized config (for programmatic use)
    pub fn from_postgres_config(postgres: &jd_utils::config::Postgres) -> Self {
        Self {
            database_url: postgres.dsn.clone(),
            max_connections: postgres.max_conns,
            min_connections: postgres.min_conns.unwrap_or(5),
            connect_timeout_secs: postgres.connect_timeout_secs.unwrap_or(30),
            idle_timeout_secs: postgres.idle_timeout_secs,
            max_lifetime_secs: postgres.max_lifetime_secs,
            ssl_mode: postgres.ssl_mode.clone().unwrap_or_else(|| "prefer".to_string()),
            auto_migrate: postgres.auto_migrate.unwrap_or(true),
            enable_transactions: postgres.enable_transactions.unwrap_or(true),
            test_connection: postgres.test_connection.unwrap_or(true),
            retry_attempts: postgres.retry_attempts.unwrap_or(3),
            retry_delay_ms: postgres.retry_delay_ms.unwrap_or(1000),
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), DatabaseConfigError> {
        if self.database_url.is_empty() {
            return Err(DatabaseConfigError::InvalidConfig {
                field: "database_url".to_string(),
                error: "Database URL cannot be empty".to_string(),
            });
        }

        if self.max_connections == 0 {
            return Err(DatabaseConfigError::InvalidConfig {
                field: "max_connections".to_string(),
                error: "Max connections must be greater than 0".to_string(),
            });
        }

        if self.min_connections > self.max_connections {
            return Err(DatabaseConfigError::InvalidConfig {
                field: "min_connections".to_string(),
                error: "Min connections cannot be greater than max connections".to_string(),
            });
        }

        if self.connect_timeout_secs == 0 {
            return Err(DatabaseConfigError::InvalidConfig {
                field: "connect_timeout_secs".to_string(),
                error: "Connect timeout must be greater than 0".to_string(),
            });
        }

        Ok(())
    }
}

/// Database pool manager with connection pooling and health checks
pub struct DatabaseManager {
    config: DatabaseConfig,
    pool: Option<Db>,
}

impl DatabaseManager {
    /// Create a new database manager
    pub fn new(config: DatabaseConfig) -> Result<Self, DatabaseConfigError> {
        config.validate()?;
        Ok(Self {
            config,
            pool: None,
        })
    }

    /// Initialize the database pool with retry logic
    pub async fn initialize(&mut self) -> Result<(), DatabaseConfigError> {
        info!("Initializing database connection pool...");
        
        let mut last_error = None;
        
        for attempt in 1..=self.config.retry_attempts {
            match self.create_pool().await {
                Ok(pool) => {
                    self.pool = Some(pool);
                    info!("Database connection pool initialized successfully");
                    
                    if self.config.test_connection {
                        self.test_connection().await?;
                    }
                    
                    if self.config.auto_migrate {
                        self.run_migrations().await?;
                    }
                    
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.config.retry_attempts {
                        warn!(
                            "Database connection attempt {} failed, retrying in {}ms...", 
                            attempt, 
                            self.config.retry_delay_ms
                        );
                        tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
                    }
                }
            }
        }
        
        Err(DatabaseConfigError::ConnectionFailed {
            attempts: self.config.retry_attempts,
            last_error: last_error.unwrap(),
        })
    }

    /// Create the database pool
    async fn create_pool(&self) -> Result<Db, sqlx::Error> {
        let mut options = PgPoolOptions::new()
            .max_connections(self.config.max_connections)
            .min_connections(self.config.min_connections)
            .acquire_timeout(Duration::from_secs(self.config.connect_timeout_secs));

        if let Some(idle_timeout) = self.config.idle_timeout_secs {
            options = options.idle_timeout(Duration::from_secs(idle_timeout));
        }

        if let Some(max_lifetime) = self.config.max_lifetime_secs {
            options = options.max_lifetime(Duration::from_secs(max_lifetime));
        }

        options.connect(&self.config.database_url).await
    }

    /// Test database connection
    async fn test_connection(&self) -> Result<(), DatabaseConfigError> {
        let pool = self.pool.as_ref().ok_or(DatabaseConfigError::PoolNotInitialized)?;
        
        sqlx::query("SELECT 1")
            .execute(pool)
            .await
            .map_err(DatabaseConfigError::ConnectionTest)?;
            
        info!("Database connection test successful");
        Ok(())
    }

    /// Run database migrations
    async fn run_migrations(&self) -> Result<(), DatabaseConfigError> {
        let pool = self.pool.as_ref().ok_or(DatabaseConfigError::PoolNotInitialized)?;
        
        info!("Running database migrations...");
        sqlx::migrate!("../../../sql")
            .run(pool)
            .await
            .map_err(DatabaseConfigError::Migration)?;
            
        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Get the database pool
    pub fn pool(&self) -> Result<&Db, DatabaseConfigError> {
        self.pool.as_ref().ok_or(DatabaseConfigError::PoolNotInitialized)
    }

    /// Create a new Dbx instance
    pub fn dbx(&self) -> DbxResult<Dbx> {
        let pool = self.pool()
            .map_err(|_e| crate::dbx::Error::Sqlx(sqlx::Error::PoolClosed))?;
        Dbx::new(pool.clone(), self.config.enable_transactions)
    }

    /// Get database pool statistics
    pub fn pool_stats(&self) -> Option<DatabaseStats> {
        self.pool.as_ref().map(|pool| DatabaseStats {
            size: pool.size(),
            idle: pool.num_idle(),
            max_connections: self.config.max_connections,
        })
    }

    /// Check database health
    pub async fn health_check(&self) -> HealthStatus {
        match self.pool.as_ref() {
            Some(pool) => {
                match sqlx::query("SELECT 1").execute(pool).await {
                    Ok(_) => HealthStatus::Healthy,
                    Err(e) => HealthStatus::Unhealthy(e.to_string()),
                }
            }
            None => HealthStatus::Uninitialized,
        }
    }

    /// Close the database pool
    pub async fn close(&mut self) {
        if let Some(pool) = self.pool.take() {
            pool.close().await;
            info!("Database connection pool closed");
        }
    }
}

/// Database pool statistics
#[derive(Debug, Clone, Serialize)]
pub struct DatabaseStats {
    pub size: u32,
    pub idle: usize,
    pub max_connections: u32,
}

/// Database health status
#[derive(Debug, Clone, Serialize)]
pub enum HealthStatus {
    Healthy,
    Unhealthy(String),
    Uninitialized,
}

/// Database configuration errors
#[derive(Debug, thiserror::Error)]
pub enum DatabaseConfigError {
    #[error("Database URL is missing. Set DATABASE_URL or POSTGRES_URL environment variable")]
    MissingDatabaseUrl,

    #[error("Invalid configuration for field '{field}': {error}")]
    InvalidConfig { field: String, error: String },

    #[error("Database pool is not initialized")]
    PoolNotInitialized,

    #[error("Connection failed after {attempts} attempts: {last_error}")]
    ConnectionFailed {
        attempts: u32,
        last_error: sqlx::Error,
    },

    #[error("Connection test failed: {0}")]
    ConnectionTest(sqlx::Error),

    #[error("Migration failed: {0}")]
    Migration(sqlx::migrate::MigrateError),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Helper function to create a new database pool with default configuration
pub async fn new_db_pool() -> Result<Db, DatabaseConfigError> {
    let config = DatabaseConfig::from_env()?;
    let mut manager = DatabaseManager::new(config)?;
    manager.initialize().await?;
    Ok(manager.pool()?.clone())
}

/// Helper function to create a new database pool with custom configuration
pub async fn new_db_pool_with_config(config: DatabaseConfig) -> Result<Db, DatabaseConfigError> {
    let mut manager = DatabaseManager::new(config)?;
    manager.initialize().await?;
    Ok(manager.pool()?.clone())
}