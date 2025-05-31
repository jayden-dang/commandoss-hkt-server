use axum::{
  Router,
  routing::get,
  response::Json,
};
use jd_core::AppState;
use serde_json::{json, Value};

// Sponsor operations - simplified to working methods only
pub fn sponsor_router() -> Router<AppState> {
  Router::new()
    .route("/health", get(health_check))
    .route("/test-connection", get(test_connection))
}

async fn health_check() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

async fn test_connection() -> Json<Value> {
    Json(json!({"connection": "ok"}))
}
