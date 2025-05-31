pub mod config;
pub mod dbx;
pub mod repository;
pub mod utils;

// Re-export commonly used types
pub use config::{
    new_db_pool, new_db_pool_with_config, 
    DatabaseConfig, DatabaseManager, DatabaseStats, HealthStatus
};
pub use dbx::{Dbx, Error as DbxError, Result as DbxResult};

use sqlx::{Pool, Postgres};

pub type Db = Pool<Postgres>;
