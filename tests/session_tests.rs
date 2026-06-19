use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use http_body_util::BodyExt;
use rust_backend::{app::app_router, database::db_pool::connect_db};
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

async fn setup_db() -> sqlx::PgPool {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    connect_db(&db_url).await
}

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

async fn create_verified_user(
    pool: &sqlx::PgPool,
    app: &axum::Router,
    email: &str,
    username: &str,
    password: &str,
) {
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
                .uri("/auth/sign-up")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&signup_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let otp_record = sqlx::query!("SELECT otp FROM email_otp WHERE email = $1 ORDER BY created_at DESC LIMIT 1", email)
        .fetch_one(pool)
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
                .uri("/auth/verify-email")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&verify_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_session_management_flow() {
    let pool = setup_db().await;
    let app = app_router(pool.clone());

    // Generate unique credentials for User A and User B
    let uuid_a = Uuid::new_v4().to_string();
    let email_a = format!("user_a_{}@example.com", uuid_a);
    let username_a = format!("usera_{}", &uuid_a[0..8]);

    let uuid_b = Uuid::new_v4().to_string();
    let email_b = format!("user_b_{}@example.com", uuid_b);
    let username_b = format!("userb_{}", &uuid_b[0..8]);

    let password = "Password123!";

    // Create verified users
    create_verified_user(&pool, &app, &email_a, &username_a, &password).await;
    create_verified_user(&pool, &app, &email_b, &username_b, &password).await;

    // 1. Log in User A - Session 1 (Laptop)
    let login_body_1 = serde_json::json!({
        "email": email_a,
        "password": password,
        "device_name": "Laptop"
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&login_body_1).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let (access_1, _refresh_1) = extract_cookies(response.headers());
    let access_1 = access_1.unwrap();

    // 2. Log in User A - Session 2 (Phone)
    let login_body_2 = serde_json::json!({
        "email": email_a,
        "password": password,
        "device_name": "Phone"
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&login_body_2).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let (access_2, _refresh_2) = extract_cookies(response.headers());
    let access_2 = access_2.unwrap();

    // 3. Log in User B
    let login_body_b = serde_json::json!({
        "email": email_b,
        "password": password,
        "device_name": "Tablet"
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_string(&login_body_b).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let (access_b, _refresh_b) = extract_cookies(response.headers());
    let access_b = access_b.unwrap();

    // 4. Retrieve sessions using User A - Session 1 (Laptop)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/sessions")
                .header(header::COOKIE, format!("access_token={}", access_1))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let sessions: Value = serde_json::from_slice(&body_bytes).unwrap();
    let sessions_array = sessions.as_array().expect("Expected array of sessions");
    assert_eq!(sessions_array.len(), 2);

    // Find session details
    let session_1 = sessions_array.iter().find(|s| s["device_name"] == "Laptop").unwrap();
    let session_2 = sessions_array.iter().find(|s| s["device_name"] == "Phone").unwrap();

    assert_eq!(session_1["current"].as_bool(), Some(true));
    assert_eq!(session_2["current"].as_bool(), Some(false));

    let session_1_id = session_1["session_id"].as_str().unwrap();
    let session_2_id = session_2["session_id"].as_str().unwrap();

    // 5. Retrieve current session details using User A - Session 1
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/sessions/current")
                .header(header::COOKIE, format!("access_token={}", access_1))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let current_session: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(current_session["session_id"].as_str().unwrap(), session_1_id);
    assert_eq!(current_session["device_name"].as_str().unwrap(), "Laptop");
    assert_eq!(current_session["current"].as_bool(), Some(true));

    // 6. Security Check: User B attempts to revoke User A's Session 2
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/sessions/{}", session_2_id))
                .header(header::COOKIE, format!("access_token={}", access_b))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // Should be unauthorized
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // 7. User A revokes their own Session 2 (Phone)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/sessions/{}", session_2_id))
                .header(header::COOKIE, format!("access_token={}", access_1))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify Session 2 is revoked (no longer returned in active sessions list)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/sessions")
                .header(header::COOKIE, format!("access_token={}", access_1))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let sessions: Value = serde_json::from_slice(&body_bytes).unwrap();
    let sessions_array = sessions.as_array().unwrap();
    assert_eq!(sessions_array.len(), 1);
    assert_eq!(sessions_array[0]["device_name"].as_str().unwrap(), "Laptop");

    // 8. Test GET /sessions using revoked Session 2 access token
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/sessions")
                .header(header::COOKIE, format!("access_token={}", access_2))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // It should STILL succeed because access token itself is not blacklisted/revoked (just refresh token is soft-revoked, access token lives until expiry which is standard in JWT).
    // Wait, the client is logged out when they try to refresh or if we blacklisted the access token. Since we don't have a blacklist of access tokens, this is standard behavior.
    assert_eq!(response.status(), StatusCode::OK);

    // 9. User A performs Logout All
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sessions/logout-all")
                .header(header::COOKIE, format!("access_token={}", access_1))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify all active sessions are revoked
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/sessions")
                .header(header::COOKIE, format!("access_token={}", access_1))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let sessions: Value = serde_json::from_slice(&body_bytes).unwrap();
    let sessions_array = sessions.as_array().unwrap();
    assert_eq!(sessions_array.len(), 0);
}
