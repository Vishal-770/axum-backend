use crate::database::db_state::AppState;
use crate::database::models::user_model::User;
use crate::errors::{AppError, auth_error::AuthError};
use uuid::Uuid;
use bcrypt::{hash, DEFAULT_COST};

pub async fn sign_up(
    email: String,
    user_name: String,
    password: String,
    state: AppState,
) -> Result<User, AppError> {
    let normalized_email = email.trim().to_lowercase();
    println!("Sign up request for email: {}", normalized_email);

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

    let user = match existing_user {
        Some(user) => {
            if user.verified {
                // User is already verified, cannot claim
                return Err(AuthError::Conflict("User with this email already exists".to_string()).into());
            } else {
                // User exists but is NOT verified - allow claim (overwrite)
                println!("Unverified user found for email: {}. Overwriting account.", normalized_email);
                
                // Hash the password only when we know we need to perform write operations
                let hashed_password = hash(password, DEFAULT_COST)
                    .map_err(|_| AppError::InternalServer)?;

                sqlx::query_as!(
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
                .await?
            }
        }
        None => {
            // No existing user - create new one
            // Hash the password only when we know we need to perform write operations
            let hashed_password = hash(password, DEFAULT_COST)
                .map_err(|_| AppError::InternalServer)?;

            sqlx::query_as!(
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
            .await?
        }
    };

    tx.commit().await?;

    Ok(user)
}
