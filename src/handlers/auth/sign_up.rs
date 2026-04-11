use axum::extract::State;
use axum::Json;
use axum::response::IntoResponse;
use crate::database::db_state::AppState;
use crate::dtos::auth_dtos::{SignUpDto, CreateUserResponse};
use crate::services::auth_services::sign_up_service::sign_up;
use crate::errors::{AppError, auth_error::AuthError};
use validator::Validate;
use axum::http::StatusCode;

pub async fn sign_up_handler(
    State(state): State<AppState>,
    Json(payload): Json<SignUpDto>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Validate the input
    payload.validate().map_err(|e| AuthError::Validation(e.to_string()))?;

    // 2. Call the service to create the user
    let user = sign_up(payload.email, payload.user_name, payload.password, state).await?;

    // 3. Map the result to a response DTO (excluding sensitive data)
    let response = CreateUserResponse {
        id: user.id,
        email: user.email,
        username: user.username,
        verified: user.verified,
    };

    Ok((StatusCode::CREATED, Json(response)))
}
