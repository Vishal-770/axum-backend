use axum::{extract::State, response::IntoResponse, Json};
use crate::database::db_state::AppState;
use super::super::dtos::ResendOtpDto;
use super::super::services::resend_otp_service::resend_otp;
use crate::errors::AppError;
use super::super::errors::AuthError;
use validator::Validate;
use serde_json::json;

pub async fn resend_otp_handler(
    State(state): State<AppState>,
    Json(payload): Json<ResendOtpDto>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Validate inputs
    payload.validate().map_err(|e| AuthError::Validation(e.to_string()))?;

    // 2. Call the resend OTP service
    resend_otp(payload.email, state).await?;

    // 3. Return success response
    Ok(Json(json!({
        "message": "Verification code has been resent successfully."
    })))
}
