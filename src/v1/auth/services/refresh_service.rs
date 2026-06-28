use crate::v1::auth::claims::RefreshClaims;
use crate::v1::auth::jwt::{create_access_token, create_refresh_token};
use crate::database::db_state::AppState;
use crate::errors::AppError; use crate::v1::auth::errors::AuthError;
use jsonwebtoken::{DecodingKey, Validation, decode};
use sha2::{Digest, Sha256};

pub async fn refresh(
    refresh_token: String,
    user_agent: Option<String>,
    ip_address: Option<String>,
    state: AppState,
) -> Result<(String, String), AppError> {
    // 1. Verify JWT & extract jti / sub (user_id)
    let refresh_secret = std::env::var("JWT_REFRESH_SECRET")
        .expect("JWT_REFRESH_SECRET must be set");

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
    let received_hash = hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    // 3. Start transaction
    let mut tx = state.db.begin().await?;

    // 4. Look up the token record — do NOT filter on revoked_at here.
    //    We need to find revoked tokens too, so we can detect reuse.
    let record = sqlx::query!(
        r#"
        SELECT token_hash, device_name, expires_at, user_id, revoked_at, family_id
        FROM refresh_tokens
        WHERE id = $1
        "#,
        old_jti
    )
    .fetch_optional(&mut *tx)
    .await?;

    let record = match record {
        Some(rec) => rec,
        None => return Err(AuthError::Unauthorized.into()),
    };

    // 5. Cross-check JWT sub against the DB row (defense-in-depth)
    if record.user_id != user_id {
        return Err(AuthError::Unauthorized.into());
    }

    // 6. REUSE DETECTION — check this BEFORE expiry so that even an expired
    //    token that was already rotated triggers the alarm.
    //    A non-NULL revoked_at means this token was already consumed in a
    //    previous rotation or explicit logout. Seeing it again is a strong
    //    signal of theft or replay.
    if record.revoked_at.is_some() {
        eprintln!(
            "[SECURITY] Refresh token reuse detected — user_id={}, jti={}, family_id={}. \
             Revoking all sessions in this family.",
            record.user_id, old_jti, record.family_id
        );

        // Revoke every active token in the same family to kick all sessions
        // that descended from the potentially stolen token.
        sqlx::query!(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = NOW()
            WHERE family_id = $1
              AND revoked_at IS NULL
            "#,
            record.family_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        return Err(AuthError::Unauthorized.into());
    }

    // 7. Verify the token hash matches what we stored
    if record.token_hash != received_hash {
        return Err(AuthError::Unauthorized.into());
    }

    // 8. Verify the token has not expired
    if record.expires_at < chrono::Utc::now() {
        return Err(AuthError::Unauthorized.into());
    }

    // 9. Soft-revoke the old token (keep the row for the audit trail)
    sqlx::query!(
        "UPDATE refresh_tokens SET revoked_at = NOW() WHERE id = $1",
        old_jti
    )
    .execute(&mut *tx)
    .await?;

    // 10. Generate new access token and refresh token
    let access_secret = std::env::var("JWT_ACCESS_SECRET")
        .expect("JWT_ACCESS_SECRET must be set");

    let new_access_token =
        create_access_token(user_id, record.family_id, &access_secret).map_err(|_| AppError::InternalServer)?;
    let (new_refresh_token, new_jti, new_expires_at) =
        create_refresh_token(user_id, &refresh_secret).map_err(|_| AppError::InternalServer)?;

    // 11. Hash the new refresh token
    let mut hasher = Sha256::new();
    hasher.update(new_refresh_token.as_bytes());
    let new_token_hash = hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    // 12. Insert the new token, inheriting the same family_id so the lineage is
    //     tracked across all rotations from the original login.
    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens
            (id, user_id, token_hash, device_name, user_agent, ip_address, expires_at, family_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        new_jti,
        user_id,
        new_token_hash,
        record.device_name,
        user_agent,
        ip_address,
        new_expires_at,
        record.family_id // carry the family forward — same login lineage
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok((new_access_token, new_refresh_token))
}
