use axum::{Router};
use tower_http::trace::TraceLayer;
use crate::routes::auth::auth_routes;
use crate::routes::root::root_route;

pub fn app_router() -> Router {
    let app = Router::new().
        merge(root_route()).
        nest("/auth",auth_routes()
            .layer(TraceLayer::new_for_http())
        );
    app
}

