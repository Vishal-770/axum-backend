use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use rust_backend::{app::app_router, database::db_pool::connect_db};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

static TEST_MUTEX: std::sync::LazyLock<tokio::sync::Mutex<()>> =
    std::sync::LazyLock::new(|| tokio::sync::Mutex::new(()));

async fn setup_db() -> sqlx::PgPool {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    connect_db(&db_url).await
}

async fn setup_redis() -> redis::aio::MultiplexedConnection {
    dotenvy::dotenv().ok();
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_client = redis::Client::open(redis_url).expect("Invalid Redis URL");
    redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis")
}

// Extract cookies from Set-Cookie headers in a response
fn extract_cookies(headers: &axum::http::HeaderMap) -> (Option<String>, Option<String>) {
    let mut access_token = None;
    let mut refresh_token = None;

    for value in headers.get_all(header::SET_COOKIE) {
        if let Ok(cookie_str) = value.to_str() {
            if let Some(cookie) = cookie_str.split(';').next() {
                let parts: Vec<&str> = cookie.split('=').collect();
                if parts.len() == 2 {
                    let name = parts[0].trim();
                    let val = parts[1].trim().to_string();
                    if name == "access_token" && !val.is_empty() {
                        access_token = Some(val);
                    } else if name == "refresh_token" && !val.is_empty() {
                        refresh_token = Some(val);
                    }
                }
            }
        }
    }
    (access_token, refresh_token)
}

#[tokio::test]
async fn test_full_cookie_auth_flow() {
    let pool = setup_db().await;
    let redis_conn = setup_redis().await;
    let app = app_router(pool.clone(), redis_conn);

    // Generate unique email and username to avoid conflict
    let test_uuid = Uuid::new_v4().to_string();
    let email = format!("test_{}@example.com", test_uuid);
    let username = format!("user_{}", &test_uuid[0..8]);
    let password = "Password123!";

    // 1. Sign up the user
    let signup_body = serde_json::json!({
        "email": email,
        "password": password,
        "user_name": username
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/sign-up")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&signup_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // 2. Fetch the verification OTP from database
    let otp_record = sqlx::query!(
        "SELECT otp FROM email_otp WHERE email = $1 ORDER BY created_at DESC LIMIT 1",
        email
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch OTP from database");

    // 3. Verify the email
    let verify_body = serde_json::json!({
        "email": email,
        "otp": otp_record.otp
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/verify-email")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&verify_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 4. Log in and verify cookies are set
    let login_body = serde_json::json!({
        "email": email,
        "password": password
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&login_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify cookies are in the headers
    let (access_cookie, refresh_cookie) = extract_cookies(response.headers());
    assert!(access_cookie.is_some(), "Access cookie missing");
    assert!(refresh_cookie.is_some(), "Refresh cookie missing");

    let access_token = access_cookie.unwrap();
    let refresh_token = refresh_cookie.unwrap();

    // Verify response body has user details and no tokens
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body_json.get("user").is_some());
    assert!(body_json.get("access_token").is_none());
    assert!(body_json.get("refresh_token").is_none());

    // 4b. Test GET /me with valid access token cookie
    let me_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/me")
                .header(header::COOKIE, format!("access_token={}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(me_response.status(), StatusCode::OK);

    let me_body_bytes = me_response.into_body().collect().await.unwrap().to_bytes();
    let me_body_json: Value = serde_json::from_slice(&me_body_bytes).unwrap();
    assert_eq!(me_body_json.get("email").unwrap().as_str().unwrap(), email);
    assert_eq!(
        me_body_json.get("username").unwrap().as_str().unwrap(),
        username
    );
    assert!(
        me_body_json.get("password").is_none(),
        "Password should be omitted"
    );

    // Test GET /me with missing access token cookie - should fail
    let me_fail_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(me_fail_response.status(), StatusCode::UNAUTHORIZED);

    // 5. Refresh using the refresh cookie
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/refresh")
                .header(header::COOKIE, format!("refresh_token={}", refresh_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let (new_access_cookie, new_refresh_cookie) = extract_cookies(response.headers());
    assert!(new_access_cookie.is_some(), "New access cookie missing");
    assert!(new_refresh_cookie.is_some(), "New refresh cookie missing");

    let _new_access_token = new_access_cookie.unwrap();
    let new_refresh_token = new_refresh_cookie.unwrap();

    assert_ne!(refresh_token, new_refresh_token);

    // Verify refresh body has success message and no tokens
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(
        body_json.get("message").unwrap().as_str().unwrap(),
        "Tokens refreshed successfully"
    );
    assert!(body_json.get("access_token").is_none());

    // 6. Test Replay Protection: attempt to refresh using old refresh token again
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/refresh")
                .header(header::COOKIE, format!("refresh_token={}", refresh_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // 7. Logout using the new refresh token
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/logout")
                .header(
                    header::COOKIE,
                    format!("refresh_token={}", new_refresh_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify cookies are cleared (Set-Cookie headers should have empty values or max-age/expiry in past)
    let mut cleared_access = false;
    let mut cleared_refresh = false;

    for value in response.headers().get_all(header::SET_COOKIE) {
        if let Ok(cookie_str) = value.to_str() {
            println!("Set-Cookie header value: {}", cookie_str);
            if cookie_str.contains("access_token=")
                && (cookie_str.contains("max-age=0")
                    || cookie_str.contains("expires=")
                    || cookie_str.contains("Max-Age=0"))
            {
                cleared_access = true;
            }
            if cookie_str.contains("refresh_token=")
                && (cookie_str.contains("max-age=0")
                    || cookie_str.contains("expires=")
                    || cookie_str.contains("Max-Age=0"))
            {
                cleared_refresh = true;
            }
        }
    }
    assert!(cleared_access, "Access token cookie was not cleared");
    assert!(cleared_refresh, "Refresh token cookie was not cleared");

    // 8. Try to refresh with the logged out token - should fail
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/refresh")
                .header(
                    header::COOKIE,
                    format!("refresh_token={}", new_refresh_token),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_password_reset_flow() {
    let _guard = TEST_MUTEX.lock().await;
    let pool = setup_db().await;
    let redis_conn = setup_redis().await;
    let app = app_router(pool.clone(), redis_conn);

    // Generate unique email and username to avoid conflict
    let test_uuid = Uuid::new_v4().to_string();
    let email = format!("reset_{}@example.com", test_uuid);
    let username = format!("reset_{}", &test_uuid[0..8]);
    let password = "OldPassword123!";
    let new_password = "NewSuperPassword321!";

    // 1. Sign up the user
    let signup_body = serde_json::json!({
        "email": email,
        "password": password,
        "user_name": username
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/sign-up")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&signup_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // 2. Fetch the verification OTP and verify email
    let otp_record = sqlx::query!(
        "SELECT otp FROM email_otp WHERE email = $1 ORDER BY created_at DESC LIMIT 1",
        email
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch OTP from database");

    let verify_body = serde_json::json!({
        "email": email,
        "otp": otp_record.otp
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/verify-email")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&verify_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 3. Request Forgot Password
    let forgot_body = serde_json::json!({
        "email": email
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/forgot-password")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&forgot_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify generic success message
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(
        body_json.get("message").unwrap().as_str().unwrap(),
        "If the email is registered, a password reset code has been sent."
    );

    // 4. Retrieve raw OTP code and reset token from the static test hooks
    let (_otp, reset_token) = {
        let otp_guard =
            rust_backend::v1::auth::services::forgot_password_service::LAST_RESET_OTP
                .lock()
                .unwrap();
        let token_guard =
            rust_backend::v1::auth::services::forgot_password_service::LAST_RESET_TOKEN
                .lock()
                .unwrap();
        (
            otp_guard
                .as_ref()
                .expect("Forgot password service did not capture the generated OTP")
                .clone(),
            token_guard
                .as_ref()
                .expect("Forgot password service did not capture the generated token")
                .clone(),
        )
    };

    // 5. Try Reset Password with invalid token
    let reset_invalid_otp = serde_json::json!({
        "reset_token": "invalid_token",
        "new_password": new_password
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/reset-password")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&reset_invalid_otp).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST); // InvalidCode is 400

    // 6. Try Reset Password with short password validation failure
    let reset_short_pwd = serde_json::json!({
        "reset_token": reset_token,
        "new_password": "123"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/reset-password")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&reset_short_pwd).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // 7. Successful Reset Password
    let reset_valid = serde_json::json!({
        "reset_token": reset_token,
        "new_password": new_password
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/reset-password")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&reset_valid).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(
        body_json.get("message").unwrap().as_str().unwrap(),
        "Password has been reset successfully."
    );

    // 8. Try to login with old password - should fail
    let login_old = serde_json::json!({
        "email": email,
        "password": password
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&login_old).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // 9. Login with new password - should succeed
    let login_new = serde_json::json!({
        "email": email,
        "password": new_password
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&login_new).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 10. Replay Protection: Try to use the same reset OTP again - should fail
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/reset-password")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&reset_valid).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // 11. Test User Enumeration vulnerability on Forgot Password: request for non-existent email
    let forgot_non_existent = serde_json::json!({
        "email": "nonexistent_user_email_12345@example.com"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/forgot-password")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_string(&forgot_non_existent).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_otp_resend_cooldown_and_limit_flow() {
    let pool = setup_db().await;
    let redis_conn = setup_redis().await;
    let app = app_router(pool.clone(), redis_conn);

    let test_uuid = Uuid::new_v4().to_string();
    let email = format!("resend_{}@example.com", test_uuid);
    let username = format!("user_{}", &test_uuid[0..8]);
    let password = "Password123!";

    // 1. Sign up the user
    let signup_body = serde_json::json!({
        "email": email,
        "password": password,
        "user_name": username
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/sign-up")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&signup_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // 2. Try to resend OTP immediately (should fail with 60s cooldown limit)
    let resend_body = serde_json::json!({
        "email": email
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/resend-otp")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&resend_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // 3. Bypass cooldown by manually updating the `last_sent_at` in the database to be 61 seconds ago
    let past_time = chrono::Utc::now().naive_utc() - chrono::Duration::seconds(65);
    sqlx::query!(
        "UPDATE email_otp SET last_sent_at = $1 WHERE email = $2 AND used_at IS NULL",
        past_time,
        email
    )
    .execute(&pool)
    .await
    .unwrap();

    // 4. Try resend #1 (should succeed)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/resend-otp")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&resend_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 5. Bypass cooldown again to do resend #2 (should succeed)
    sqlx::query!(
        "UPDATE email_otp SET last_sent_at = $1 WHERE email = $2 AND used_at IS NULL",
        past_time,
        email
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/resend-otp")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&resend_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 6. Bypass cooldown again to do resend #3 (should succeed)
    sqlx::query!(
        "UPDATE email_otp SET last_sent_at = $1 WHERE email = $2 AND used_at IS NULL",
        past_time,
        email
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/resend-otp")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&resend_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 7. Bypass cooldown again to try resend #4 (should fail due to max resend limit)
    sqlx::query!(
        "UPDATE email_otp SET last_sent_at = $1 WHERE email = $2 AND used_at IS NULL",
        past_time,
        email
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/resend-otp")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&resend_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Verify response body states resend limit reached
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    let error_msg = body_json.get("error").unwrap().as_str().unwrap();
    assert!(error_msg.contains("Resend limit reached"));
}

#[tokio::test]
async fn test_password_reset_otp_resend_flow() {
    let _guard = TEST_MUTEX.lock().await;
    let pool = setup_db().await;
    let redis_conn = setup_redis().await;
    let app = app_router(pool.clone(), redis_conn);

    let test_uuid = Uuid::new_v4().to_string();
    let email = format!("reset_resend_{}@example.com", test_uuid);
    let username = format!("user_{}", &test_uuid[0..8]);
    let password = "Password123!";

    // 1. Sign up and verify the user (needed because forgot-password requires a verified user in production)
    let signup_body = serde_json::json!({
        "email": email,
        "password": password,
        "user_name": username
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/sign-up")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&signup_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let otp_record = sqlx::query!(
        "SELECT otp FROM email_otp WHERE email = $1 ORDER BY created_at DESC LIMIT 1",
        email
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let verify_body = serde_json::json!({
        "email": email,
        "otp": otp_record.otp
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/verify-email")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&verify_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 2. Trigger forgot-password to get reset token
    let forgot_body = serde_json::json!({
        "email": email
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/forgot-password")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&forgot_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let reset_token = {
        let token_guard =
            rust_backend::v1::auth::services::forgot_password_service::LAST_RESET_TOKEN
                .lock()
                .unwrap();
        token_guard
            .as_ref()
            .expect("Forgot password service did not capture the generated token")
            .clone()
    };

    // 3. Try to resend OTP immediately (should fail with 60s cooldown limit)
    let resend_body = serde_json::json!({
        "email": email,
        "reset_token": reset_token
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/resend-reset-otp")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&resend_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // 4. Bypass cooldown by manually updating the `last_sent_at` in the database to be 65 seconds ago
    let past_time = chrono::Utc::now() - chrono::Duration::seconds(65);
    sqlx::query!(
        "UPDATE password_resets SET last_sent_at = $1 WHERE used_at IS NULL",
        past_time
    )
    .execute(&pool)
    .await
    .unwrap();

    // 5. Try resend #1 (should succeed)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/resend-reset-otp")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&resend_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 6. Bypass cooldown again to do resend #2 (should succeed)
    sqlx::query!(
        "UPDATE password_resets SET last_sent_at = $1 WHERE used_at IS NULL",
        past_time
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/resend-reset-otp")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&resend_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 7. Bypass cooldown again to do resend #3 (should succeed)
    sqlx::query!(
        "UPDATE password_resets SET last_sent_at = $1 WHERE used_at IS NULL",
        past_time
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/resend-reset-otp")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&resend_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 8. Bypass cooldown again to try resend #4 (should fail due to max resend limit)
    sqlx::query!(
        "UPDATE password_resets SET last_sent_at = $1 WHERE used_at IS NULL",
        past_time
    )
    .execute(&pool)
    .await
    .unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/auth/resend-reset-otp")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&resend_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Verify response body states resend limit reached
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    let error_msg = body_json.get("error").unwrap().as_str().unwrap();
    assert!(error_msg.contains("Resend limit reached"));
}
