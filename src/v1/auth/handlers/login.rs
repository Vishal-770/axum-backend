use axum::{extract::State, http::{StatusCode, HeaderMap}, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use crate::database::db_state::AppState;
use super::super::dtos::{LoginDto, AuthResponse, CreateUserResponse};
use super::super::services::login_service::login;
use crate::errors::AppError; use crate::v1::auth::errors::AuthError;
use validator::Validate;

pub async fn login_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
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

    // 3. Set the access and refresh tokens in cookies
    let access_cookie = Cookie::build(("access_token", access_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();

    let refresh_cookie = Cookie::build(("refresh_token", refresh_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();

    let updated_jar = jar.add(access_cookie).add(refresh_cookie);

    // 4. Construct response DTO
    let response = AuthResponse {
        user: CreateUserResponse {
            id: user.id,
            email: user.email,
            username: user.username,
            verified: user.verified,
        },
    };

    Ok((StatusCode::OK, updated_jar, Json(response)))
}
