use axum::{routing::post, Router};
use jd_core::AppState;

mod unified_endpoints;

pub fn zkpersona_router() -> Router<AppState> {
    Router::new()
        .route("/generate-proof", post(unified_endpoints::generate_proof))
        .route("/verify", post(unified_endpoints::verify_proof))
}