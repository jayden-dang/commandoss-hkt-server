use axum::response::IntoResponse;
use axum::Router;
use axum::{extract::State, routing::post, Json};
use jd_core::ctx::Ctx;
use jd_core::{AppState, ModelManager};
use serde_json::{json, Value};

/// Simple RPC handler that routes to the appropriate function
pub async fn rpc_handler(
  State(app_state): State<AppState>,
  Json(rpc_req): Json<Value>,
) -> impl IntoResponse {
  // Create a default context for now
  let ctx = Ctx::new(0).unwrap_or_else(|_| Ctx::root_ctx());
  // Extract method and params from the request
  let method = rpc_req.get("method").and_then(|v| v.as_str()).unwrap_or("");

  let params = rpc_req.get("params").cloned().unwrap_or(json!({}));

  let id = rpc_req.get("id").cloned();

  // Route to the appropriate handler (placeholder for future ZK-Persona methods)
  let result: Result<Value, jd_core::Error> = match method {
    _ => Err(jd_core::Error::RpcError(format!("Unknown method: {}", method))),
  };

  // Build the response
  match result {
    Ok(data) => Json(json!({
        "jsonrpc": "2.0",
        "result": data,
        "id": id
    })),
    Err(e) => Json(json!({
        "jsonrpc": "2.0",
        "error": {
            "code": -32603,
            "message": e.to_string()
        },
        "id": id
    })),
  }
}

/// Build the Axum router for '/api/rpc'
pub fn routes(_mm: ModelManager) -> Router<AppState> {
  Router::new().route("/rpc", post(rpc_handler))
}
