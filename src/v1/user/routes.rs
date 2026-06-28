use axum::{middleware, routing::get, Router};
use crate::database::db_state::AppState;
use crate::v1::auth::middleware::require_auth;
use super::handlers::me_handler;

pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(me_handler))
        .route_layer(middleware::from_fn(require_auth))
}
