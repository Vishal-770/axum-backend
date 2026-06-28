use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use crate::database::db_state::AppState;
use super::super::dtos::VerifyEmailDto;
use super::super::services::verify_email_service::verify_email;
use crate::errors::{AppError, auth_error::AuthError};
use validator::Validate;
use serde_json::json;

pub async fn verify_email_handler(
    State(state): State<AppState>,
    Json(payload): Json<VerifyEmailDto>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Validate the input
    payload.validate().map_err(|e| AuthError::Validation(e.to_string()))?;

    // 2. Call the service to verify email
    verify_email(payload.email, payload.otp, state).await?;

    // 3. Return success response
    Ok((StatusCode::OK, Json(json!({ "message": "Email verified successfully" }))))
}
