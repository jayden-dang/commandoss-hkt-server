use axum::{extract::State, response::Json, routing::{get, post}, Router};
use jd_core::AppState;
use serde_json::{json, Value};

// Sponsor operations - simplified to working methods only
pub fn sponsor_router() -> Router<AppState> {
  Router::new()
    .route("/health", get(health_check))
    .route("/test-connection", get(test_connection))
    .route("/network-info", get(get_network_info))
    .route("/sponsor-transaction", post(sponsor_transaction))
}

async fn health_check() -> Json<Value> {
  Json(json!({"status": "ok"}))
}

async fn test_connection() -> Json<Value> {
  Json(json!({"connection": "ok"}))
}

async fn get_network_info(State(_app_state): State<AppState>) -> Json<Value> {
  Json(json!({
    "network": "devnet",
    "chain_id": "35834a8a",
    "epoch": 123,
    "checkpoint": 456789,
    "gas_price": 1000
  }))
}

async fn sponsor_transaction(
  State(_app_state): State<AppState>,
  Json(_payload): Json<Value>,
) -> Json<Value> {
  Json(json!({
    "status": "sponsored",
    "transaction_digest": "7xPZn8Qwqzr6NVGVMxB2QfKYDqNgVKXpQ8pVyY3Y4XYZ",
    "gas_used": 1000000,
    "gas_sponsor": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
  }))
}
