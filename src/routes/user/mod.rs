use axum::{middleware, routing::get, Router};
use crate::database::db_state::AppState;
use crate::handlers::user::me::me_handler;
use crate::middleware::auth::require_auth;

pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(me_handler))
        .route_layer(middleware::from_fn(require_auth))
}
