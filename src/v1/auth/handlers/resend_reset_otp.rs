use axum::{extract::State, response::IntoResponse, Json};
use crate::database::db_state::AppState;
use super::super::dtos::ResendResetOtpDto;
use super::super::services::resend_reset_otp_service::resend_reset_otp;
use crate::errors::AppError;
use super::super::errors::AuthError;
use validator::Validate;
use serde_json::json;

pub async fn resend_reset_otp_handler(
    State(state): State<AppState>,
    Json(payload): Json<ResendResetOtpDto>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Validate inputs
    payload.validate().map_err(|e| AuthError::Validation(e.to_string()))?;

    // 2. Call the resend reset OTP service
    resend_reset_otp(payload.email, payload.reset_token, state).await?;

    // 3. Return success response
    Ok(Json(json!({
        "message": "Verification code has been resent successfully."
    })))
}
