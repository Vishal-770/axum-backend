use axum::Router;
use axum::routing::post;
use crate::handlers::auth::{
    forgot_password::forgot_password_handler,
    login::login_handler,
    sign_up::sign_up_handler,
    logout::logout_handler
};
use crate::database::db_state::AppState;


pub fn auth_routes() -> Router<AppState> {
    let auth_router = Router::new()
        .route("/login", post(login_handler))
        .route("/logout", post(logout_handler))
        .route("/sign-up", post(sign_up_handler))
        .route("/forgot-password", post(forgot_password_handler));

    auth_router
}



