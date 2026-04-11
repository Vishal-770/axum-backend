
use axum::{Router};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

use crate::database::db_state::AppState;
use crate::routes::auth::auth_routes;
use crate::routes::root::root_route;

pub fn app_router(pool: PgPool) -> Router {
    let state = AppState { db: pool };
    Router::new()
        .merge(root_route())
        .nest("/auth", auth_routes())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}



