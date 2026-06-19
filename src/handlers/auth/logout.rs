use axum::{extract::State, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use crate::database::db_state::AppState;
use crate::services::auth_services::logout_service::logout;
use crate::errors::AppError;
use serde_json::json;

pub async fn logout_handler(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    // 1. If the refresh token cookie exists, call the logout service to delete from DB
    if let Some(cookie) = jar.get("refresh_token") {
        let refresh_token = cookie.value().to_string();
        let _ = logout(refresh_token, state).await;
    }

    // 2. Remove access and refresh cookies from the browser by adding explicit removal cookies
    let mut access_cookie = Cookie::build(("access_token", ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();
    access_cookie.make_removal();

    let mut refresh_cookie = Cookie::build(("refresh_token", ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();
    refresh_cookie.make_removal();

    let updated_jar = jar.add(access_cookie).add(refresh_cookie);

    // 3. Return success response with updated jar
    Ok((
        updated_jar,
        Json(json!({ "message": "Logged out successfully" })),
    ))
}