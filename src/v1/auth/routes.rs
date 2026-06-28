use axum::Router;
use axum::routing::post;
use crate::database::db_state::AppState;
use super::handlers::{
    forgot_password::forgot_password_handler,
    login::login_handler,
    logout::logout_handler,
    refresh::refresh_handler,
    reset_password::reset_password_handler,
    sign_up::sign_up_handler,
    verify_email::verify_email_handler,
    resend_otp::resend_otp_handler,
    resend_reset_otp::resend_reset_otp_handler,
};

use axum::middleware::from_fn_with_state;

pub fn auth_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/logout", post(logout_handler))
        .route("/sign-up", post(sign_up_handler))
        .route("/forgot-password", post(forgot_password_handler))
        .route("/reset-password", post(reset_password_handler))
        .route("/verify-email", post(verify_email_handler))
        .route("/refresh", post(refresh_handler))
        .route("/resend-otp", post(resend_otp_handler))
        .route("/resend-reset-otp", post(resend_reset_otp_handler))
        .route_layer(from_fn_with_state(state, super::rate_limit::rate_limiter))
}
