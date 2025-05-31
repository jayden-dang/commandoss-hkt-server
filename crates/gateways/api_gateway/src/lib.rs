use axum::{middleware as axum_middleware, Router};
use jd_core::AppState;

mod error;
mod github;
mod log;
pub mod middleware;
mod routes_rpc;
mod sui;
mod zkpersona;

pub type Result<T> = std::result::Result<T, error::Error>;

pub fn v1_routes(app_state: AppState) -> Router {
  let mm = app_state.mm.as_ref().clone();
  
  // Create protected routes that require authentication
  let protected_zkpersona_routes = zkpersona::zkpersona_protected_router()
    .route_layer(axum_middleware::from_fn_with_state(
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
      Router::new()
        .nest("/zkpersona", 
          Router::new()
            .merge(protected_zkpersona_routes)
            .merge(public_zkpersona_routes)
        )
        .nest("/sui", sui::sui_router())
        .nest("/github", github::github_router()),
    )
    .nest("/api", routes_rpc::routes(mm))
    .with_state(app_state)
}
