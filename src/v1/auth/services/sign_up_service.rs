use crate::database::db_state::AppState;
use crate::v1::user::model::User;
use crate::errors::{AppError, auth_error::AuthError};
use crate::utils::generate_otp::generate_otp;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use uuid::Uuid;

pub async fn sign_up(
    email: String,
    user_name: String,
    password: String,
    state: AppState,
) -> Result<User, AppError> {
    let normalized_email = email.trim().to_lowercase();
    println!("Sign up request received");

    // 1. Start a transaction
    let mut tx = state.db.begin().await?;

    // 2. Check for existing user (use explicit column list instead of SELECT *)
    let existing_user = sqlx::query_as!(
        User,
        "SELECT id, email, username, password, verified, created_at, updated_at FROM users WHERE email = $1",
        normalized_email
    )
    .fetch_optional(&mut *tx)
    .await?;

    let (user, otp) = match existing_user {
        Some(user) => {
            if user.verified {
                // User is already verified, cannot claim
                return Err(
                    AuthError::Conflict("User with this email already exists".to_string()).into(),
                );
            } else {
                // User exists but is NOT verified - allow claim (overwrite)
                println!("Unverified user found — overwriting account (user_id={})", user.id);

                // Hash the password only when we know we need to perform write operations
                let salt = SaltString::generate(&mut OsRng);
                let hashed_password = Argon2::default()
                    .hash_password(password.as_bytes(), &salt)
                    .map_err(|_| AppError::InternalServer)?
                    .to_string();

                let updated_user = sqlx::query_as!(
                    User,
                    r#"
                    UPDATE users 
                    SET username = $1, password = $2, updated_at = NOW()
                    WHERE email = $3
                    RETURNING id, email, username, password, verified, created_at, updated_at
                    "#,
                    user_name,
                    hashed_password,
                    normalized_email
                )
                .fetch_one(&mut *tx)
                .await?;

                // Generate and update existing OTP
                let otp = generate_otp().await;
                let expires_at = chrono::Utc::now().naive_utc() + chrono::Duration::minutes(10);
                println!("OTP issued for existing unverified user");

                let rows_affected = sqlx::query!(
                    "UPDATE email_otp SET otp = $1, expires_at = $2, created_at = NOW() WHERE email = $3",
                    otp,
                    expires_at,
                    normalized_email
                )
                .execute(&mut *tx)
                .await?
                .rows_affected();

                if rows_affected == 0 {
                    // Fallback to insert if no record exists yet
                    sqlx::query!(
                        "INSERT INTO email_otp (email, otp, expires_at) VALUES ($1, $2, $3)",
                        normalized_email,
                        otp,
                        expires_at
                    )
                    .execute(&mut *tx)
                    .await?;
                }

                (updated_user, otp)
            }
        }
        None => {
            // No existing user - create new one
            // Hash the password only when we know we need to perform write operations
            let salt = SaltString::generate(&mut OsRng);
            let hashed_password = Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .map_err(|_| AppError::InternalServer)?
                .to_string();

            let new_user = sqlx::query_as!(
                User,
                r#"
                INSERT INTO users (id, email, username, password)
                VALUES ($1, $2, $3, $4)
                RETURNING id, email, username, password, verified, created_at, updated_at
                "#,
                Uuid::new_v4(),
                normalized_email,
                user_name,
                hashed_password
            )
            .fetch_one(&mut *tx)
            .await?;

            // Generate and insert new OTP
            let otp = generate_otp().await;
            let expires_at = chrono::Utc::now().naive_utc() + chrono::Duration::minutes(10);
            println!("OTP issued for new user");

            let rows_affected = sqlx::query!(
                "UPDATE email_otp SET otp = $1, expires_at = $2, created_at = NOW() WHERE email = $3",
                otp,
                expires_at,
                normalized_email
            )
            .execute(&mut *tx)
            .await?
            .rows_affected();

            if rows_affected == 0 {
                sqlx::query!(
                    "INSERT INTO email_otp (email, otp, expires_at) VALUES ($1, $2, $3)",
                    normalized_email,
                    otp,
                    expires_at
                )
                .execute(&mut *tx)
                .await?;
            }

            (new_user, otp)
        }
    };

    tx.commit().await?;

    // 4. Send verification email
    state
        .mail_service
        .send_otp_email(&normalized_email, &user.username, &otp)
        .await
        .map_err(|e| {
            eprintln!("Failed to send verification email: {:?}", e);
            AppError::InternalServer
        })?;

    Ok(user)
}
