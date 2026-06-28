use super::handlers::me_handler;
use crate::database::db_state::AppState;
use crate::v1::auth::middleware::require_auth;
use crate::v1::auth::rate_limit::rate_limiter;
use axum::{Router, middleware, routing::get};

use axum::middleware::from_fn_with_state;

pub fn user_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/me", get(me_handler))
        .route_layer(from_fn_with_state(state, rate_limiter))
        .route_layer(middleware::from_fn(require_auth))
}
