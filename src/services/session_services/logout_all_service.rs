use crate::database::db_state::AppState;
use crate::errors::AppError;
use uuid::Uuid;

pub async fn logout_all_sessions(
    user_id: Uuid,
    state: AppState,
) -> Result<(), AppError> {
    sqlx::query!(
        "UPDATE refresh_tokens SET revoked_at = NOW() WHERE user_id = $1 AND revoked_at IS NULL",
        user_id
    )
    .execute(&state.db)
    .await?;

    Ok(())
}
