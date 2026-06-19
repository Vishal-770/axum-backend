use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use sha2::{Digest, Sha256};

use crate::database::db_state::AppState;
use crate::errors::{AppError, auth_error::AuthError};

pub async fn reset_password(
    reset_token: String,
    new_password: String,
    state: AppState,
) -> Result<(), AppError> {
    // 1. Enforce minimum password length before doing any DB work
    if new_password.len() < 8 {
        return Err(AuthError::WeakPassword.into());
    }

    // 2. Derive the token hash the same way forgot_password stored it
    let mut hasher = Sha256::new();
    hasher.update(reset_token.as_bytes());
    let token_hash = hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    // 3. Hash the new password with Argon2id
    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(new_password.as_bytes(), &salt)
        .map_err(|_| AppError::InternalServer)?
        .to_string();

    // 4. Open a transaction for atomic multi-step writes
    let mut tx = state.db.begin().await?;

    // 5. Atomically mark the reset record as used.
    //    Using UPDATE...RETURNING instead of SELECT-then-UPDATE eliminates the
    //    race window where two concurrent requests both pass the SELECT check.
    //    If no row is returned, the token was already used, expired, or invalid.
    let record = sqlx::query!(
        r#"
        UPDATE password_resets
        SET used_at = NOW()
        WHERE token_hash = $1
          AND used_at IS NULL
          AND expires_at > NOW()
        RETURNING id, user_id
        "#,
        token_hash
    )
    .fetch_optional(&mut *tx)
    .await?;

    let record = match record {
        Some(r) => r,
        None => return Err(AuthError::InvalidCode.into()),
    };

    // 6. Update the user's password (look up by user_id — no email round-trip needed)
    sqlx::query!(
        "UPDATE users SET password = $1, updated_at = NOW() WHERE id = $2",
        hashed_password,
        record.user_id
    )
    .execute(&mut *tx)
    .await?;

    // Soft-revoke all active sessions for this user.
    // Keeps the audit trail; any token from a session that tries to refresh
    // after a password reset will be caught by the reuse detector.
    sqlx::query!(
        "UPDATE refresh_tokens SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL",
        record.user_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
