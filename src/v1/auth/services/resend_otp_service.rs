use crate::database::db_state::AppState;
use crate::errors::AppError;
use crate::v1::auth::errors::AuthError;
use crate::utils::generate_otp::generate_otp;

pub async fn resend_otp(email: String, state: AppState) -> Result<(), AppError> {
    let normalized_email = email.trim().to_lowercase();

    // 1. Start a transaction
    let mut tx = state.db.begin().await?;

    // 2. Verify that the user exists and is unverified, fetch username
    let user_record = sqlx::query!(
        "SELECT username, verified FROM users WHERE email = $1",
        normalized_email
    )
    .fetch_optional(&mut *tx)
    .await?;

    let user = match user_record {
        Some(record) => record,
        None => return Err(AuthError::Unauthorized.into()), // User doesn't exist
    };

    if user.verified {
        return Err(AuthError::Conflict("Email is already verified".to_string()).into());
    }

    // 3. Retrieve the active unverified OTP record
    let otp_record = sqlx::query!(
        "SELECT resend_count, last_sent_at FROM email_otp WHERE email = $1 AND used_at IS NULL ORDER BY created_at DESC LIMIT 1",
        normalized_email
    )
    .fetch_optional(&mut *tx)
    .await?;

    let otp_record = match otp_record {
        Some(record) => record,
        None => return Err(AuthError::InvalidCode.into()), // No active OTP session
    };

    // 4. Check restrictions
    // A. 60-Second Cooldown
    let now = chrono::Utc::now().naive_utc();
    let time_elapsed = now.signed_duration_since(otp_record.last_sent_at);
    if time_elapsed.num_seconds() < 60 {
        return Err(AuthError::Validation("Please wait 60 seconds before resending OTP".to_string()).into());
    }

    // B. Max 3 Resends
    if otp_record.resend_count >= 3 {
        return Err(AuthError::Validation("Resend limit reached. Please register again.".to_string()).into());
    }

    // 5. Generate new OTP and update constraints
    let new_otp = generate_otp().await;
    let expires_at = chrono::Utc::now().naive_utc() + chrono::Duration::minutes(10);

    sqlx::query!(
        "UPDATE email_otp SET otp = $1, expires_at = $2, resend_count = resend_count + 1, last_sent_at = NOW() WHERE email = $3 AND used_at IS NULL",
        new_otp,
        expires_at,
        normalized_email
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // 6. Asynchronously send OTP email
    state
        .mail_service
        .send_otp_email(&normalized_email, &user.username, &new_otp)
        .await
        .map_err(|e| {
            eprintln!("Failed to send OTP resend email: {:?}", e);
            AppError::InternalServer
        })?;

    Ok(())
}
