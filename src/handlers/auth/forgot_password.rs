use axum::{extract::State, response::IntoResponse, Json};
use crate::database::db_state::AppState;
use crate::dtos::auth_dtos::ForgotPasswordDto;
use crate::services::auth_services::forgot_password_service::forgot_password;
use crate::errors::{AppError, auth_error::AuthError};
use validator::Validate;
use serde_json::json;

pub async fn forgot_password_handler(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordDto>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Validate input
    payload.validate().map_err(|e| AuthError::Validation(e.to_string()))?;

    // 2. Call the forgot password service
    forgot_password(payload.email, state).await?;

    // 3. Return success response (generic message to prevent email enumeration)
    Ok(Json(json!({
        "message": "If the email is registered, a password reset code has been sent."
    })))
}
