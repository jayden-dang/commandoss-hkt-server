use axum::{
  routing::{get, post, put},
  Router,
};
use jd_core::AppState;

mod github_routes;

pub use github_routes::*;

pub fn github_router() -> Router<AppState> {
  Router::new()
    // Test endpoints
    .route("/webhook", post(handle_webhook))
    .route("/repository/{owner}/{repo}", get(get_repository_info))
    .route("/analyze", post(analyze_repository))
    // Original endpoints
    .route("/repositories", get(list_repositories).post(add_repository))
    .route("/repositories/{id}", get(get_repository))
    .route("/repositories/{id}/settings", put(update_repository_settings))
    .route("/webhooks/github", post(handle_github_webhook))
}
