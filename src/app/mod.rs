
use axum::Router;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

use crate::database::db_state::AppState;
use crate::v1::v1_routes;
use crate::config::mail_config::MailService;

// Simple root route mapping for health-check / entrypoint
use axum::routing::get;
async fn root_handler() -> &'static str {
    "Welcome to the Axum Backend API!"
}

pub fn app_router(pool: PgPool, redis_conn: redis::aio::MultiplexedConnection) -> Router {
    let mail_service = MailService::new();
    let state = AppState { db: pool, mail_service, redis: redis_conn };
    Router::new()
        .route("/", get(root_handler))
        .nest("/v1", v1_routes())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}



