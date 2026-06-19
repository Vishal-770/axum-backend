use axum::{middleware, routing::{get, post, delete}, Router};
use crate::database::db_state::AppState;
use crate::handlers::session::{
    get_all::get_all_handler,
    get_current::get_current_handler,
    revoke::revoke_handler,
    logout_all::logout_all_handler,
};
use crate::middleware::auth::require_auth;

pub fn session_routes() -> Router<AppState> {
    Router::new()
        .route("/sessions", get(get_all_handler))
        .route("/sessions/current", get(get_current_handler))
        .route("/sessions/{family_id}", delete(revoke_handler))
        .route("/sessions/logout-all", post(logout_all_handler))
        .route_layer(middleware::from_fn(require_auth))
}
