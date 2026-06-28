use crate::v1::auth::jwt::{create_access_token, create_refresh_token};
use crate::database::db_state::AppState;
use crate::v1::user::model::User;
use crate::errors::AppError; use crate::v1::auth::errors::AuthError;
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};

pub async fn login(
    email: String,
    password: String,
    device_name: Option<String>,
    user_agent: Option<String>,
    ip_address: Option<String>,
    state: AppState,
) -> Result<(User, String, String), AppError> {
    let normalized_email = email.trim().to_lowercase();
    println!("Login attempt");

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

    // 3. Verify password with Argon2id
    let parsed_hash =
        PasswordHash::new(&user.password).map_err(|_| AppError::InternalServer)?;
    let is_valid = Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();
    if !is_valid {
        return Err(AuthError::Unauthorized.into());
    }

    // 4. Generate JWT tokens
    let access_secret = std::env::var("JWT_ACCESS_SECRET")
        .expect("JWT_ACCESS_SECRET must be set");
    let refresh_secret = std::env::var("JWT_REFRESH_SECRET")
        .expect("JWT_REFRESH_SECRET must be set");

    // Each new login starts a fresh token family used for reuse detection
    let family_id = uuid::Uuid::new_v4();

    let access_token =
        create_access_token(user.id, family_id, &access_secret).map_err(|_| AppError::InternalServer)?;
    let (refresh_token, jti, expires_at) =
        create_refresh_token(user.id, &refresh_secret).map_err(|_| AppError::InternalServer)?;

    // Calculate SHA-256 hash of the refresh token
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(refresh_token.as_bytes());
    let hash_result = hasher.finalize();
    let token_hash = hash_result
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    sqlx::query!(
        "INSERT INTO refresh_tokens (id, user_id, token_hash, device_name, user_agent, ip_address, expires_at, family_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        jti,
        user.id,
        token_hash,
        device_name,
        user_agent,
        ip_address,
        expires_at,
        family_id
    )
    .execute(&state.db)
    .await?;

    Ok((user, access_token, refresh_token))
}
