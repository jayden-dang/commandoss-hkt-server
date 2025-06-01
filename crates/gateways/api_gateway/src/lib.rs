use axum::{middleware as axum_middleware, Router, response::Json};
use jd_core::AppState;
use serde_json::json;

mod analytics;
mod developers;
mod error;
mod github;
mod log;
pub mod middleware;
mod patches;
mod routes_rpc;
mod sui;
mod vulnerabilities;
mod zkpersona;

pub type Result<T> = std::result::Result<T, error::Error>;

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "zkpersona-api",
        "version": "1.0.0"
    }))
}

pub fn v1_routes(app_state: AppState) -> Router {
  let mm = app_state.mm.as_ref().clone();

  // Create protected routes that require authentication
  let protected_zkpersona_routes =
    zkpersona::zkpersona_protected_router().route_layer(axum_middleware::from_fn_with_state(
      app_state.clone(),
      middleware::mw_user_auth::mw_ctx_require_user_auth,
    ));

  // Create public routes
  let public_zkpersona_routes = Router::new()
    .route("/verify", axum::routing::post(zkpersona::unified_endpoints::verify_proof))
    .nest("/auth", zkpersona::auth_endpoints::auth_routes());

  Router::new()
    .nest(
      "/api/v1",
      Router::<AppState>::new()
        .route("/health", axum::routing::get(health_check))
        .nest("/analytics", analytics::analytics_router())
        .nest("/vulnerabilities", vulnerabilities::vulnerability_router())
        .nest("/patches", patches::patch_router())
        .nest("/developers", developers::developer_router())
        .nest(
          "/zkpersona",
          Router::new()
            .merge(protected_zkpersona_routes)
            .merge(public_zkpersona_routes),
        )
        .nest("/sui", sui::sui_router())
        .nest("/github", github::github_router()),
    )
    .nest("/api", routes_rpc::routes(mm))
    .with_state(app_state)
}
