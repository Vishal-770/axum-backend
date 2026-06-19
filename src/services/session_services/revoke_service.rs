use crate::database::db_state::AppState;
use crate::errors::{AppError, auth_error::AuthError};
use uuid::Uuid;

pub async fn revoke_session(
    user_id: Uuid,
    target_family_id: Uuid,
    state: AppState,
) -> Result<(), AppError> {
    // 1. Verify ownership of the target family_id session
    let family_owner = sqlx::query!(
        "SELECT user_id FROM refresh_tokens WHERE family_id = $1 LIMIT 1",
        target_family_id
    )
    .fetch_optional(&state.db)
    .await?;

    if let Some(record) = family_owner {
        if record.user_id != user_id {
            // User is trying to access a session they do not own!
            return Err(AppError::Auth(AuthError::Unauthorized));
        }
    } else {
        // Session not found
        return Err(AppError::Auth(AuthError::Unauthorized));
    }

    // 2. Invalidate all active tokens in the family
    sqlx::query!(
        "UPDATE refresh_tokens SET revoked_at = NOW() WHERE family_id = $1 AND user_id = $2 AND revoked_at IS NULL",
        target_family_id,
        user_id
    )
    .execute(&state.db)
    .await?;

    Ok(())
}
