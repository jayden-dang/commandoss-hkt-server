use axum::{
  body::Body,
  extract::{Request, State},
  http::{HeaderMap, StatusCode},
  middleware::Next,
  response::Response,
};
use jd_core::{ctx::Ctx, AppState};
use jd_domain::Id;
use jd_storage::{config::{DatabaseConfig, DatabaseManager}};
use serde_json::json;
use std::sync::Arc;
use tower_cookies::{Cookie, Cookies};
use tracing::{error, info};
use auth_service::infrastructure::database::UserRepositoryImpl;

pub const AUTH_TOKEN: &str = "auth-token";

/// Middleware for user authentication
/// Extracts user ID from auth token and adds it to the request context
pub async fn mw_ctx_require_user_auth(
  State(app_state): State<AppState>,
  cookies: Cookies,
  mut req: Request<Body>,
  next: Next,
) -> Result<Response, StatusCode> {
  info!(">>> {:<12} - mw_ctx_require_user_auth", "MIDDLEWARE");

  // Get user ID from token
  let user_id = get_user_id_from_token(&cookies, &app_state).await?;

  // For now, use a simple hash of the UUID as i64
  // In production, you might want to store a mapping
  let user_id_i64 = user_id.to_string().chars().take(15).fold(1i64, |acc, c| {
    acc.wrapping_add(c as i64).wrapping_mul(31)
  }).abs();

  // Create context with user ID
  let ctx = Ctx::new(user_id_i64).map_err(|e| {
    error!("Failed to create context: {}", e);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  // Add context to request extensions
  req.extensions_mut().insert(ctx);

  Ok(next.run(req).await)
}

/// Optional authentication middleware
/// Similar to require auth but doesn't fail if no auth token
pub async fn mw_ctx_optional_user_auth(
  State(app_state): State<AppState>,
  cookies: Cookies,
  mut req: Request<Body>,
  next: Next,
) -> Result<Response, StatusCode> {
  info!(">>> {:<12} - mw_ctx_optional_user_auth", "MIDDLEWARE");

  // Try to get user ID from token, but don't fail if not present
  if let Ok(user_id) = get_user_id_from_token(&cookies, &app_state).await {
    // For now, use a simple hash of the UUID as i64
    let user_id_i64 = user_id.to_string().chars().take(15).fold(1i64, |acc, c| {
      acc.wrapping_add(c as i64).wrapping_mul(31)
    }).abs();
    
    // Create context with user ID
    if let Ok(ctx) = Ctx::new(user_id_i64) {
      req.extensions_mut().insert(ctx);
    }
  }

  Ok(next.run(req).await)
}

/// Extract user ID from authentication token
async fn get_user_id_from_token(
  cookies: &Cookies,
  app_state: &AppState,
) -> Result<Id, StatusCode> {
  // Get auth token from cookies
  let auth_token = cookies
    .get(AUTH_TOKEN)
    .map(|c| c.value().to_string())
    .ok_or_else(|| {
      error!("No auth token found in cookies");
      StatusCode::UNAUTHORIZED
    })?;

  // Parse token to get user ID
  // For now, we'll use a simple implementation where the token IS the user ID
  // In production, this should be a JWT or similar secure token
  let user_id = Id::from_str(&auth_token).map_err(|e| {
    error!("Invalid auth token format: {}", e);
    StatusCode::UNAUTHORIZED
  })?;

  // Verify user exists in database
  let db_config = DatabaseConfig::from_env().map_err(|e| {
    error!("Failed to load database config: {}", e);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let mut db_manager = DatabaseManager::new(db_config).map_err(|e| {
    error!("Failed to create database manager: {}", e);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  db_manager.initialize().await.map_err(|e| {
    error!("Failed to initialize database: {}", e);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let dbx = db_manager.dbx().map_err(|e| {
    error!("Failed to get database connection: {}", e);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  // TODO: Implement proper user validation with auth_service
  // For now, we skip database validation and assume valid users

  info!("User authenticated: {}", user_id);
  Ok(user_id)
}

/// Set auth token cookie after successful authentication
pub fn set_auth_cookie(cookies: &Cookies, user_id: &Id) {
  let cookie = Cookie::build((AUTH_TOKEN, user_id.to_string()))
    .path("/")
    .http_only(true)
    .secure(true)
    .same_site(tower_cookies::cookie::SameSite::Strict)
    .build();
  
  cookies.add(cookie);
}

/// Remove auth token cookie for logout
pub fn remove_auth_cookie(cookies: &Cookies) {
  cookies.remove(Cookie::named(AUTH_TOKEN));
}