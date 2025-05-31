use axum::{routing::post, Router};
use jd_core::AppState;

pub mod auth_endpoints;
pub mod unified_endpoints;

pub fn zkpersona_router() -> Router<AppState> {
  Router::new()
    .route("/generate-proof", post(unified_endpoints::generate_proof))
    .route("/verify", post(unified_endpoints::verify_proof))
    .nest("/auth", auth_endpoints::auth_routes())
}

pub fn zkpersona_protected_router() -> Router<AppState> {
  Router::new().route("/generate-proof", post(unified_endpoints::generate_proof))
}
