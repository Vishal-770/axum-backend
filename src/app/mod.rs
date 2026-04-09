use axum::{routing::get, Router};
use crate::handlers::root_handler::root_handler;

pub fn appRouter() -> Router {
    let app = Router::new().route("/", get(root_handler));
    app
}

