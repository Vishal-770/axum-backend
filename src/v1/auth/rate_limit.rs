use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use http_body_util::BodyExt;
use serde_json::Value;

use crate::database::db_state::AppState;
use crate::errors::AppError;
use super::errors::AuthError;
use super::claims::RefreshClaims;
use super::middleware::ClaimsExtension;

#[derive(Clone, Copy, Debug)]
pub enum RateLimitKeyType {
    Ip,
    Email,
    IpAndEmail,
    RefreshTokenId,
    EmailAndIp,
    ResetToken,
    User,
}

// Global Rate Limiter: limits every client IP to 100 requests per 1-second window (100 RPS)
pub async fn global_rate_limiter(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let headers = req.headers();

    // Extract client IP address
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "127.0.0.1".to_string());

    let key = format!("rl:global:{}", ip);
    let now = chrono::Utc::now().timestamp_millis();
    let member = format!("{}:{}", now, uuid::Uuid::new_v4());
    let window_ms = 1000; // 1 second
    let limit = 100;      // 100 RPS

    let mut conn = state.redis.clone();

    let script = redis::Script::new(r#"
        local key = KEYS[1]
        local now = tonumber(ARGV[1])
        local window = tonumber(ARGV[2])
        local limit = tonumber(ARGV[3])
        local member = ARGV[4]

        -- Remove old entries outside of the sliding window
        redis.call('ZREMRANGEBYSCORE', key, '-inf', now - window)

        -- Get current active request count
        local current_requests = redis.call('ZCARD', key)

        if current_requests < limit then
            -- Add current request timestamp
            redis.call('ZADD', key, now, member)
            -- Set TTL so the key expires once the window closes
            redis.call('EXPIRE', key, math.ceil(window / 1000))
            return 1
        else
            return 0
        end
    "#);

    let allowed: i32 = match script
        .key(key)
        .arg(now)
        .arg(window_ms)
        .arg(limit)
        .arg(member)
        .invoke_async(&mut conn)
        .await {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Redis global rate limiter error: {:?}", e);
                return AppError::InternalServer.into_response();
            }
        };

    if allowed == 0 {
        return AuthError::TooManyRequests.into_response();
    }

    next.run(req).await
}

// Route-Specific Rate Limiter: applies limits based on endpoint parameters
pub async fn rate_limiter(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    // 1. Determine rate limit configuration for the path/method
    let (limit, window_ms, key_type) = match (method.as_str(), path.as_str()) {
        ("POST", "/v1/auth/sign-up") => (5, 60 * 1000, RateLimitKeyType::Ip),
        ("POST", "/v1/auth/verify-email") => (10, 10 * 60 * 1000, RateLimitKeyType::Email),
        ("POST", "/v1/auth/login") => (5, 60 * 1000, RateLimitKeyType::IpAndEmail),
        ("POST", "/v1/auth/refresh") => (30, 60 * 1000, RateLimitKeyType::RefreshTokenId),
        ("POST", "/v1/auth/forgot-password") => (3, 15 * 60 * 1000, RateLimitKeyType::EmailAndIp),
        ("POST", "/v1/auth/reset-password") => (5, 15 * 60 * 1000, RateLimitKeyType::ResetToken),
        ("POST", "/v1/auth/logout") => (30, 60 * 1000, RateLimitKeyType::User),
        ("GET", "/v1/sessions") => (60, 60 * 1000, RateLimitKeyType::User),
        ("GET", "/v1/sessions/current") => (120, 60 * 1000, RateLimitKeyType::User),
        ("POST", "/v1/sessions/logout-all") => (5, 5 * 60 * 1000, RateLimitKeyType::User),
        ("GET", "/v1/me") => (120, 60 * 1000, RateLimitKeyType::User),
        ("DELETE", p) if p.starts_with("/v1/sessions/") => (20, 60 * 1000, RateLimitKeyType::User),
        _ => {
            // No rate limit for unspecified routes
            return next.run(req).await;
        }
    };

    // 2. Buffer the request body conditionally
    let (parts, body) = req.into_parts();

    let (next_body, body_json, is_json_extracted) = match key_type {
        RateLimitKeyType::Email | RateLimitKeyType::IpAndEmail | RateLimitKeyType::EmailAndIp | RateLimitKeyType::ResetToken => {
            // Buffer the body
            let collected = match body.collect().await {
                Ok(c) => c,
                Err(_) => return AppError::InternalServer.into_response(),
            };
            let req_body_bytes = collected.to_bytes();
            let json = serde_json::from_slice(&req_body_bytes).unwrap_or(Value::Null);
            (axum::body::Body::from(req_body_bytes), json, true)
        }
        _ => (body, Value::Null, false),
    };
    let reconstructed_req = Request::from_parts(parts, next_body);

    // 3. Extract IP address
    let ip = reconstructed_req.headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            reconstructed_req.headers()
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "127.0.0.1".to_string());

    // 4. Extract Email
    let email = if is_json_extracted {
        body_json.get("email")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_lowercase())
            .unwrap_or_default()
    } else {
        String::new()
    };

    // 5. Extract Reset Token
    let reset_token = if is_json_extracted {
        body_json.get("reset_token")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    } else {
        String::new()
    };

    // 6. Extract Refresh Token ID
    let refresh_token_id = match key_type {
        RateLimitKeyType::RefreshTokenId => {
            let jar = CookieJar::from_headers(reconstructed_req.headers());
            jar.get("refresh_token")
                .map(|cookie| cookie.value().to_string())
                .and_then(|token| {
                    let secret = std::env::var("JWT_REFRESH_SECRET").ok()?;
                    let token_data = jsonwebtoken::decode::<RefreshClaims>(
                        &token,
                        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
                        &jsonwebtoken::Validation::default(),
                    ).ok()?;
                    Some(token_data.claims.jti.to_string())
                })
                .unwrap_or_else(|| "anonymous_refresh".to_string())
        }
        _ => String::new(),
    };

    // 7. Extract User ID (expect ClaimsExtension from require_auth middleware)
    let user_id = match key_type {
        RateLimitKeyType::User => {
            reconstructed_req.extensions()
                .get::<ClaimsExtension>()
                .map(|c| c.user_id.to_string())
                .unwrap_or_else(|| "anonymous_user".to_string())
        }
        _ => String::new(),
    };

    // 8. Construct Redis Rate Limiting Key
    let key = match key_type {
        RateLimitKeyType::Ip => format!("rl:signup:{}", ip),
        RateLimitKeyType::Email => format!("rl:verifyemail:{}", email),
        RateLimitKeyType::IpAndEmail => format!("rl:login:{}:{}", ip, email),
        RateLimitKeyType::RefreshTokenId => format!("rl:refresh:{}", refresh_token_id),
        RateLimitKeyType::EmailAndIp => format!("rl:forgotpassword:{}:{}", email, ip),
        RateLimitKeyType::ResetToken => format!("rl:resetpassword:{}", reset_token),
        RateLimitKeyType::User => match path.as_str() {
            "/v1/sessions" => format!("rl:sessions:{}", user_id),
            "/v1/sessions/current" => format!("rl:currentsession:{}", user_id),
            "/v1/sessions/logout-all" => format!("rl:logoutall:{}", user_id),
            "/v1/me" => format!("rl:me:{}", user_id),
            "/v1/auth/logout" => format!("rl:logout:{}", user_id),
            _ if path.starts_with("/v1/sessions/") => format!("rl:revokesession:{}", user_id),
            _ => format!("rl:generic:{}", user_id),
        }
    };

    // 9. Execute atomic sliding window check using Redis Lua script
    let now = chrono::Utc::now().timestamp_millis();
    let member = format!("{}:{}", now, uuid::Uuid::new_v4());

    let mut conn = state.redis.clone();

    let script = redis::Script::new(r#"
        local key = KEYS[1]
        local now = tonumber(ARGV[1])
        local window = tonumber(ARGV[2])
        local limit = tonumber(ARGV[3])
        local member = ARGV[4]

        -- Remove old entries outside of the sliding window
        redis.call('ZREMRANGEBYSCORE', key, '-inf', now - window)

        -- Get current active request count
        local current_requests = redis.call('ZCARD', key)

        if current_requests < limit then
            -- Add current request timestamp
            redis.call('ZADD', key, now, member)
            -- Set TTL so the key expires once the window closes
            redis.call('EXPIRE', key, math.ceil(window / 1000))
            return 1
        else
            return 0
        end
    "#);

    let allowed: i32 = match script
        .key(key)
        .arg(now)
        .arg(window_ms)
        .arg(limit)
        .arg(member)
        .invoke_async(&mut conn)
        .await {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Redis rate limiter error: {:?}", e);
                return AppError::InternalServer.into_response();
            }
        };

    if allowed == 0 {
        return AuthError::TooManyRequests.into_response();
    }

    next.run(reconstructed_req).await
}
