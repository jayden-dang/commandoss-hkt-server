use async_trait::async_trait;
use jd_core::AppState;
use uuid::Uuid;
use sqlx::Row;

use crate::domain::{AuthUser, UserRepository};
use crate::error::{Error, Result};

pub struct ZkPersonaUserRepositoryImpl {
  state: AppState,
}

impl ZkPersonaUserRepositoryImpl {
  pub fn new(state: AppState) -> Self {
    Self { state }
  }
}

#[async_trait]
impl UserRepository for ZkPersonaUserRepositoryImpl {
  async fn create_user(&self, user: &AuthUser) -> Result<()> {
    // Since the zkpersona users table uses UUID as primary key, we need to generate one
    let user_id = Uuid::new_v4();
    
    // Create user in zkpersona users table with auth fields
    sqlx::query(
      r#"
      INSERT INTO users (id, wallet_address, public_key, last_login, login_count, status, ctime, mtime)
      VALUES ($1, $2, $3, $4, $5, 'active', NOW(), NOW())
      "#
    )
    .bind(user_id)
    .bind(&user.address)
    .bind(&user.public_key)
    .bind(user.last_login)
    .bind(user.login_count)
    .execute(self.state.mm().dbx().db())
    .await
    .map_err(|e| Error::database_error(&e.to_string()))?;

    Ok(())
  }

  async fn get_user(&self, address: &str) -> Result<Option<AuthUser>> {
    let result = sqlx::query(
      r#"
      SELECT wallet_address, public_key, ctime as created_at, last_login, login_count
      FROM users
      WHERE wallet_address = $1
      "#
    )
    .bind(address)
    .fetch_optional(self.state.mm().dbx().db())
    .await
    .map_err(|e| Error::database_error(&e.to_string()))?;

    match result {
      Some(row) => {
        let user = AuthUser {
          address: row.get("wallet_address"),
          public_key: row.get("public_key"),
          created_at: row.get("created_at"),
          last_login: row.get("last_login"),
          login_count: row.get("login_count"),
        };
        Ok(Some(user))
      }
      None => Ok(None),
    }
  }

  async fn update_user(&self, user: &AuthUser) -> Result<()> {
    let rows_affected = sqlx::query(
      r#"
      UPDATE users
      SET public_key = $1, last_login = $2, login_count = $3, mtime = NOW()
      WHERE wallet_address = $4
      "#
    )
    .bind(&user.public_key)
    .bind(user.last_login)
    .bind(user.login_count)
    .bind(&user.address)
    .execute(self.state.mm().dbx().db())
    .await
    .map_err(|e| Error::database_error(&e.to_string()))?
    .rows_affected();

    if rows_affected == 0 {
      return Err(Error::database_error("User not found for update"));
    }

    Ok(())
  }
}