use axum::{routing::post, Router};
mod sponsor_routes;
use jd_core::AppState;

pub fn sui_router() -> Router<AppState> {
  Router::new().merge(sponsor_routes::sponsor_router())
}
