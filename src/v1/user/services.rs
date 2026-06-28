use crate::database::db_state::AppState;
use crate::errors::AppError; use crate::v1::auth::errors::AuthError;
use super::model::User;
use super::dtos::UserMeResponse;
use uuid::Uuid;

pub async fn get_me(user_id: Uuid, state: AppState) -> Result<UserMeResponse, AppError> {
    // Fetch the user from the database
    let user = sqlx::query_as!(
        User,
        "SELECT id, email, username, password, verified, created_at, updated_at FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(&state.db)
    .await?;

    let user = match user {
        Some(u) => u,
        None => return Err(AppError::Auth(AuthError::Unauthorized)),
    };

    Ok(UserMeResponse {
        id: user.id,
        email: user.email,
        username: user.username,
        verified: user.verified,
        created_at: user.created_at,
        updated_at: user.updated_at,
    })
}
