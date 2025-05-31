use axum::{
  routing::{get, post},
  Router,
};
use jd_core::AppState;

// Import the static handler functions directly
use auth_service::application::handlers::auth_handler::AuthHandler;
use auth_service::infrastructure::database::{
  NonceRepositoryImpl, SignatureVerifierImpl, UserRepositoryImpl,
};

// Type alias for our concrete AuthHandler
type ConcreteAuthHandler =
  AuthHandler<NonceRepositoryImpl, UserRepositoryImpl, SignatureVerifierImpl>;

/// Creates authentication routes using auth_service handlers
pub fn auth_routes() -> Router<AppState> {
  Router::new()
    .route("/nonce", post(ConcreteAuthHandler::generate_nonce))
    .route("/login", post(ConcreteAuthHandler::verify_signature))
    .route("/refresh", post(ConcreteAuthHandler::refresh_token))
    .route("/me", get(ConcreteAuthHandler::get_current_user))
}
