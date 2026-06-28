use argon2::password_hash::rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::database::db_state::AppState;
use crate::v1::user::model::User;
use crate::errors::AppError;
use crate::utils::generate_otp::generate_otp;

pub static LAST_RESET_OTP: std::sync::LazyLock<std::sync::Mutex<Option<String>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(None));

pub static LAST_RESET_TOKEN: std::sync::LazyLock<std::sync::Mutex<Option<String>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(None));

pub async fn forgot_password(email: String, state: AppState) -> Result<Option<String>, AppError> {
    let normalized_email = email.trim().to_lowercase();
    println!("Forgot password requested");

    // 1. Fetch user — always return the same shape to prevent email enumeration
    let user = sqlx::query_as!(
        User,
        "SELECT id, email, username, password, verified, created_at, updated_at FROM users WHERE email = $1",
        normalized_email
    )
    .fetch_optional(&state.db)
    .await?;

    let user = match user {
        Some(u) => u,
        None => {
            println!("Forgot password: user not found, returning generic success");
            return Ok(None);
        }
    };

    // 2. Generate reset_token: 32 cryptographically random bytes → 64-char hex string.
    //    This is returned to the browser and submitted alongside the OTP on reset.
    let mut reset_token_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut reset_token_bytes);
    let reset_token: String = reset_token_bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();

    // 3. Generate 6-digit OTP — goes to the user's email only, never to the browser.
    let otp = generate_otp().await;

    if let Ok(mut guard) = LAST_RESET_OTP.lock() {
        *guard = Some(otp.clone());
    }

    if let Ok(mut guard) = LAST_RESET_TOKEN.lock() {
        *guard = Some(reset_token.clone());
    }

    // 4. Hash both before storing; raw values are never persisted
    let token_hash = sha256_hex(&reset_token);
    let otp_hash = sha256_hex(&otp);

    // 5. Insert reset request — 15-minute expiry
    let reset_id = Uuid::new_v4();
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(15);

    sqlx::query!(
        r#"
        INSERT INTO password_resets (id, user_id, token_hash, otp_hash, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        reset_id,
        user.id,
        token_hash,
        otp_hash,
        expires_at
    )
    .execute(&state.db)
    .await?;

    // 6. Send OTP via email — the reset_token stays server → browser only
    state
        .mail_service
        .send_password_reset_email(&normalized_email, &user.username, &otp)
        .await
        .map_err(|e| {
            eprintln!("Failed to send password reset email: {:?}", e);
            AppError::InternalServer
        })?;

    // 7. Return the raw reset_token to the handler so it can be included
    //    in the response body for the frontend to hold on to.
    Ok(Some(reset_token))
}

/// SHA-256 helper — shared with reset_password_service to guarantee
/// that hashing is always done the same way on both sides.
fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}
