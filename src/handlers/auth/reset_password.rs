use axum::{extract::State, response::IntoResponse, Json};
use crate::database::db_state::AppState;
use crate::dtos::auth_dtos::ResetPasswordDto;
use crate::services::auth_services::reset_password_service::reset_password;
use crate::errors::{AppError, auth_error::AuthError};
use validator::Validate;
use serde_json::json;

pub async fn reset_password_handler(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordDto>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Validate inputs
    payload.validate().map_err(|e| AuthError::Validation(e.to_string()))?;

    // 2. Call the reset password service
    reset_password(payload.reset_token, payload.new_password, state).await?;

    // 3. Return success response
    Ok(Json(json!({
        "message": "Password has been reset successfully."
    })))
}
