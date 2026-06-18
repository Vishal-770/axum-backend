
use axum::{Router};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

use crate::database::db_state::AppState;
use crate::routes::auth::auth_routes;
use crate::routes::root::root_route;

use crate::config::mail_config::MailService;

pub fn app_router(pool: PgPool) -> Router {
    let mail_service = MailService::new();
    let state = AppState { db: pool, mail_service };
    Router::new()
        .merge(root_route())
        .nest("/auth", auth_routes())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}



