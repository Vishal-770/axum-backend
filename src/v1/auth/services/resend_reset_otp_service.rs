use sha2::{Digest, Sha256};
use crate::database::db_state::AppState;
use crate::errors::AppError;
use crate::v1::auth::errors::AuthError;
use crate::utils::generate_otp::generate_otp;

pub async fn resend_reset_otp(
    email: String,
    reset_token: String,
    state: AppState,
) -> Result<(), AppError> {
    let normalized_email = email.trim().to_lowercase();

    // 1. Start a transaction
    let mut tx = state.db.begin().await?;

    // 2. Fetch the user - if they don't exist, return generic success to protect email enumeration
    let user_record = sqlx::query!(
        "SELECT id, username FROM users WHERE email = $1",
        normalized_email
    )
    .fetch_optional(&mut *tx)
    .await?;

    let user = match user_record {
        Some(record) => record,
        None => {
            println!("Resend reset OTP: user not found, returning generic success");
            return Ok(());
        }
    };

    // 3. Hash the submitted reset_token to find the database record
    let mut hasher = Sha256::new();
    hasher.update(reset_token.as_bytes());
    let token_hash = hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    // 4. Retrieve the active unverified password reset session for the user
    let reset_record = sqlx::query!(
        "SELECT id, resend_count, last_sent_at FROM password_resets WHERE token_hash = $1 AND user_id = $2 AND used_at IS NULL AND expires_at > NOW() ORDER BY created_at DESC LIMIT 1",
        token_hash,
        user.id
    )
    .fetch_optional(&mut *tx)
    .await?;

    let reset_record = match reset_record {
        Some(record) => record,
        None => return Err(AuthError::InvalidCode.into()), // No active reset session matches
    };

    // 5. Check restrictions
    // A. 60-Second Cooldown
    let now = chrono::Utc::now();
    let time_elapsed = now.signed_duration_since(reset_record.last_sent_at);
    if time_elapsed.num_seconds() < 60 {
        return Err(AuthError::Validation("Please wait 60 seconds before resending OTP".to_string()).into());
    }

    // B. Max 3 Resends
    if reset_record.resend_count >= 3 {
        return Err(AuthError::Validation("Resend limit reached. Please request forgot-password again.".to_string()).into());
    }

    // 6. Generate a new OTP and update constraints
    let new_otp = generate_otp().await;
    let mut otp_hasher = Sha256::new();
    otp_hasher.update(new_otp.as_bytes());
    let new_otp_hash = otp_hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    let new_expires_at = chrono::Utc::now() + chrono::Duration::minutes(15);

    sqlx::query!(
        "UPDATE password_resets SET otp_hash = $1, expires_at = $2, resend_count = resend_count + 1, last_sent_at = NOW() WHERE id = $3 AND used_at IS NULL",
        new_otp_hash,
        new_expires_at,
        reset_record.id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // 7. Asynchronously send OTP email
    state
        .mail_service
        .send_password_reset_email(&normalized_email, &user.username, &new_otp)
        .await
        .map_err(|e| {
            eprintln!("Failed to send reset OTP resend email: {:?}", e);
            AppError::InternalServer
        })?;

    Ok(())
}
