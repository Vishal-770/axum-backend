use axum::{extract::State, http::{StatusCode, HeaderMap}, response::IntoResponse, Json};
use crate::database::db_state::AppState;
use crate::dtos::auth_dtos::{LoginDto, AuthResponse, CreateUserResponse};
use crate::services::auth_services::login_service::login;
use crate::errors::{AppError, auth_error::AuthError};
use validator::Validate;

pub async fn login_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LoginDto>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Validate inputs
    payload.validate().map_err(|e| AuthError::Validation(e.to_string()))?;

    // Extract user agent and client IP address
    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
        });

    // 2. Perform login
    let (user, access_token, refresh_token) = login(
        payload.email,
        payload.password,
        payload.device_name,
        user_agent,
        ip_address,
        state,
    ).await?;

    // 3. Construct response DTO
    let response = AuthResponse {
        access_token,
        refresh_token,
        user: CreateUserResponse {
            id: user.id,
            email: user.email,
            username: user.username,
            verified: user.verified,
        },
    };

    Ok((StatusCode::OK, Json(response)))
}



