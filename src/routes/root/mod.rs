use axum::Router;
use axum::routing::get;
use crate::handlers::root_handler::root_handler;
use crate::database::db_state::AppState;

pub fn root_route () -> Router<AppState> {
    let root_router = Router::new().route("/", get(root_handler));
    root_router
}

