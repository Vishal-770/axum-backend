
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

use crate::v1::auth::rate_limit::global_rate_limiter;
use axum::middleware::from_fn_with_state;

pub fn app_router(pool: PgPool, redis_conn: redis::aio::MultiplexedConnection) -> Router {
    let mail_service = MailService::new();
    let auth_config = crate::config::auth_config::AuthConfig::from_env();
    let state = AppState {
        db: pool,
        mail_service,
        redis: redis_conn,
        config: auth_config,
    };
    Router::new()
        .route("/", get(root_handler))
        .nest("/v1", v1_routes(state.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(from_fn_with_state(state.clone(), global_rate_limiter))
        .with_state(state)
}



