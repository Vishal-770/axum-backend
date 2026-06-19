use axum::{extract::State, http::{StatusCode, HeaderMap}, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use crate::database::db_state::AppState;
use crate::services::auth_services::refresh_service::refresh;
use crate::errors::{AppError, auth_error::AuthError};
use serde_json::json;

pub async fn refresh_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    // 1. Read refresh cookie
    let refresh_token = jar
        .get("refresh_token")
        .map(|cookie| cookie.value().to_string())
        .ok_or(AuthError::Unauthorized)?;

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

    // 2. Perform refresh
    let (new_access_token, new_refresh_token) = refresh(
        refresh_token,
        user_agent,
        ip_address,
        state,
    )
    .await?;

    // 3. Set the new access and refresh tokens in cookies
    let access_cookie = Cookie::build(("access_token", new_access_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();

    let refresh_cookie = Cookie::build(("refresh_token", new_refresh_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();

    let updated_jar = jar.add(access_cookie).add(refresh_cookie);

    // 4. Return success message
    Ok((
        StatusCode::OK,
        updated_jar,
        Json(json!({ "message": "Tokens refreshed successfully" })),
    ))
}
