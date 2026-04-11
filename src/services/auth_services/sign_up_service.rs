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
    println!("Sign up request for email: {}", email);

    // 1. Hash the password upfront
    let hashed_password = hash(password, DEFAULT_COST)
        .map_err(|_| AppError::InternalServer)?;

    // 2. Start a transaction
    let mut tx = state.db.begin().await?;

    // 3. Check for existing user
    let existing_user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE email = $1",
        email
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
                println!("Unverified user found for email: {}. Overwriting account.", email);
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
                    email
                )
                .fetch_one(&mut *tx)
                .await?
            }
        }
        None => {
            // No existing user - create new one
            sqlx::query_as!(
                User,
                r#"
                INSERT INTO users (id, email, username, password)
                VALUES ($1, $2, $3, $4)
                RETURNING id, email, username, password, verified, created_at, updated_at
                "#,
                Uuid::new_v4(),
                email,
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