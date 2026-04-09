use axum::Router;
use axum::routing::get;
use crate::handlers::root_handler::root_handler;

pub fn root_route () ->Router{
    let root_router= Router::new().route("/", get(root_handler));
    root_router
}
