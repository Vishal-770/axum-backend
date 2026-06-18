use crate::database::db_state::AppState;
use crate::errors::{AppError, auth_error::AuthError};
use crate::auth::jwt::{create_access_token, create_refresh_token};
use crate::auth::claims::RefreshClaims;
use jsonwebtoken::{decode, DecodingKey, Validation};
use sha2::{Sha256, Digest};

pub async fn refresh(
    refresh_token: String,
    user_agent: Option<String>,
    ip_address: Option<String>,
    state: AppState,
) -> Result<(String, String), AppError> {
    // 1. Verify JWT & Extract jti / sub (user_id)
    let refresh_secret = std::env::var("JWT_REFRESH_SECRET")
        .unwrap_or_else(|_| "default_jwt_refresh_secret_key_0987654321".to_string());

    let token_data = decode::<RefreshClaims>(
        &refresh_token,
        &DecodingKey::from_secret(refresh_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AuthError::Unauthorized)?;

    let claims = token_data.claims;
    let old_jti = claims.jti;
    let user_id = claims.sub;

    // 2. Compute SHA-256 hash of the received refresh token
    let mut hasher = Sha256::new();
    hasher.update(refresh_token.as_bytes());
    let hash_result = hasher.finalize();
    let received_hash = hash_result.iter().map(|b| format!("{:02x}", b)).collect::<String>();

    // 3. Start transaction
    let mut tx = state.db.begin().await?;

    // 4. Find jti in DB
    let record = sqlx::query!(
        "SELECT token_hash, device_name, expires_at FROM refresh_tokens WHERE id = $1",
        old_jti
    )
    .fetch_optional(&mut *tx)
    .await?;

    let record = match record {
        Some(rec) => rec,
        None => return Err(AuthError::Unauthorized.into()),
    };

    // Verify token hash matches
    if record.token_hash != received_hash {
        return Err(AuthError::Unauthorized.into());
    }

    // Verify token is not expired
    if record.expires_at < chrono::Utc::now() {
        return Err(AuthError::Unauthorized.into());
    }

    // 5. Delete old jti
    sqlx::query!(
        "DELETE FROM refresh_tokens WHERE id = $1",
        old_jti
    )
    .execute(&mut *tx)
    .await?;

    // 6. Generate new access token and refresh token
    let access_secret = std::env::var("JWT_ACCESS_SECRET")
        .unwrap_or_else(|_| "default_jwt_access_secret_key_1234567890".to_string());

    let new_access_token = create_access_token(user_id, &access_secret)
        .map_err(|_| AppError::InternalServer)?;
    let (new_refresh_token, new_jti) = create_refresh_token(user_id, &refresh_secret)
        .map_err(|_| AppError::InternalServer)?;

    // 7. Calculate new SHA-256 hash
    let mut hasher = Sha256::new();
    hasher.update(new_refresh_token.as_bytes());
    let hash_result = hasher.finalize();
    let new_token_hash = hash_result.iter().map(|b| format!("{:02x}", b)).collect::<String>();

    let new_expires_at = chrono::Utc::now() + chrono::Duration::days(7);

    // 8. Insert new jti/refresh token record
    sqlx::query!(
        "INSERT INTO refresh_tokens (id, user_id, token_hash, device_name, user_agent, ip_address, expires_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        new_jti,
        user_id,
        new_token_hash,
        record.device_name,
        user_agent,
        ip_address,
        new_expires_at
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok((new_access_token, new_refresh_token))
}
