use crate::database::db_state::AppState;
use crate::errors::AppError; use crate::v1::auth::errors::AuthError;

pub async fn verify_email(email: String, otp: String, state: AppState) -> Result<(), AppError> {
    let normalized_email = email.trim().to_lowercase();

    // 1. Start a transaction
    let mut tx = state.db.begin().await?;

    // 2. Fetch the latest OTP entry for this email
    let otp_record = sqlx::query!(
        "SELECT otp, expires_at FROM email_otp WHERE email = $1 ORDER BY created_at DESC LIMIT 1",
        normalized_email
    )
    .fetch_optional(&mut *tx)
    .await?;

    let otp_record = match otp_record {
        Some(record) => record,
        None => return Err(AuthError::InvalidCode.into()),
    };

    // 3. Verify OTP code
    if otp_record.otp != otp {
        return Err(AuthError::InvalidCode.into());
    }

    // 4. Verify expiration
    if otp_record.expires_at < chrono::Utc::now().naive_utc() {
        return Err(AuthError::InvalidCode.into());
    }

    // 5. Update user to be verified
    let rows_affected = sqlx::query!(
        "UPDATE users SET verified = true, updated_at = NOW() WHERE email = $1",
        normalized_email
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        // User not found
        return Err(AuthError::Unauthorized.into());
    }

    // 6. Delete OTP entry (or entries) for this email
    sqlx::query!("DELETE FROM email_otp WHERE email = $1", normalized_email)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}
