

use crate::database::db_state::AppState;
use crate::database::models::user_model::User;
use crate::errors::{AppError, auth_error::AuthError};
use crate::auth::jwt::{create_access_token, create_refresh_token};
use bcrypt::verify;

pub async fn login(
    email: String,
    password: String,
    device_name: Option<String>,
    user_agent: Option<String>,
    ip_address: Option<String>,
    state: AppState,
) -> Result<(User, String, String), AppError> {
    let normalized_email = email.trim().to_lowercase();
    println!("Login attempt for email: {}", normalized_email);

    // 1. Fetch user by email
    let user = sqlx::query_as!(
        User,
        "SELECT id, email, username, password, verified, created_at, updated_at FROM users WHERE email = $1",
        normalized_email
    )
    .fetch_optional(&state.db)
    .await?;

    let user = match user {
        Some(user) => user,
        None => return Err(AuthError::Unauthorized.into()),
    };

    // 2. Ensure user is verified before logging in
    if !user.verified {
        return Err(AuthError::Unauthorized.into());
    }

    // 3. Verify password
    let is_valid = verify(password, &user.password).map_err(|_| AppError::InternalServer)?;
    if !is_valid {
        return Err(AuthError::Unauthorized.into());
    }

    // 4. Generate JWT tokens
    let access_secret = std::env::var("JWT_ACCESS_SECRET").unwrap_or_else(|_| "default_jwt_access_secret_key_1234567890".to_string());
    let refresh_secret = std::env::var("JWT_REFRESH_SECRET").unwrap_or_else(|_| "default_jwt_refresh_secret_key_0987654321".to_string());

    let access_token = create_access_token(user.id, &access_secret).map_err(|_| AppError::InternalServer)?;
    let (refresh_token, jti) = create_refresh_token(user.id, &refresh_secret).map_err(|_| AppError::InternalServer)?;

    // Calculate SHA-256 hash of the refresh token
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(refresh_token.as_bytes());
    let hash_result = hasher.finalize();
    let token_hash = hash_result.iter().map(|b| format!("{:02x}", b)).collect::<String>();

    let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

    sqlx::query!(
        "INSERT INTO refresh_tokens (id, user_id, token_hash, device_name, user_agent, ip_address, expires_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        jti,
        user.id,
        token_hash,
        device_name,
        user_agent,
        ip_address,
        expires_at
    )
    .execute(&state.db)
    .await?;

    Ok((user, access_token, refresh_token))
}
