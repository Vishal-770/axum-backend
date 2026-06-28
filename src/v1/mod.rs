pub mod auth;
pub mod session;
pub mod user;

use axum::Router;
use crate::database::db_state::AppState;

pub fn v1_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::routes::auth_routes(state.clone()))
        .merge(user::routes::user_routes(state.clone()))
        .nest("/sessions", session::routes::session_routes(state))
}
