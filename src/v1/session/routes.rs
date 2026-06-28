use axum::{routing::{get, post, delete}, Router};
use crate::database::db_state::AppState;
use crate::v1::auth::middleware::require_auth;
use crate::v1::auth::rate_limit::rate_limiter;
use super::handlers::{
    get_all::get_all_handler,
    get_current::get_current_handler,
    logout_all::logout_all_handler,
    revoke::revoke_handler,
};

use axum::middleware::from_fn_with_state;

pub fn session_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_all_handler))
        .route("/current", get(get_current_handler))
        .route("/{family_id}", delete(revoke_handler))
        .route("/logout-all", post(logout_all_handler))
        .route_layer(from_fn_with_state(state.clone(), rate_limiter))
        .route_layer(from_fn_with_state(state, require_auth))
}
