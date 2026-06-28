pub mod auth;
pub mod session;
pub mod user;

use axum::Router;
use crate::database::db_state::AppState;

pub fn v1_routes() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::routes::auth_routes())
        .merge(user::routes::user_routes())
        .nest("/sessions", session::routes::session_routes())
}
