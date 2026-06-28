use crate::auth::claims::RefreshClaims;
use crate::database::db_state::AppState;
use crate::errors::AppError;
use jsonwebtoken::{DecodingKey, Validation, decode};

pub async fn logout(refresh_token: String, state: AppState) -> Result<(), AppError> {
    let refresh_secret = std::env::var("JWT_REFRESH_SECRET")
        .expect("JWT_REFRESH_SECRET must be set");

    // Verify signature but ignore expiration — we want to revoke the DB row
    // even if the token expired before the user explicitly logged out.
    let mut validation = Validation::default();
    validation.validate_exp = false;

    let jti = match decode::<RefreshClaims>(
        &refresh_token,
        &DecodingKey::from_secret(refresh_secret.as_bytes()),
        &validation,
    ) {
        Ok(data) => data.claims.jti,
        Err(_) => return Ok(()), // forged / malformed token → nothing to revoke
    };

    // 2. Revoke by primary key (jti = id) — direct indexed lookup, no hashing needed.
    //    AND revoked_at IS NULL means we skip rows that are already revoked,
    //    keeping the query a no-op on repeat calls (idempotent).
    sqlx::query!(
        "UPDATE refresh_tokens SET revoked_at = NOW() WHERE id = $1 AND revoked_at IS NULL",
        jti
    )
    .execute(&state.db)
    .await?;

    Ok(())
}
