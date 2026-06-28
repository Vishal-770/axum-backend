use crate::database::db_state::AppState;
use super::super::dtos::SessionResponseDto;
use crate::errors::AppError;
use uuid::Uuid;

pub async fn get_all_sessions(
    user_id: Uuid,
    current_family_id: Uuid,
    state: AppState,
) -> Result<Vec<SessionResponseDto>, AppError> {
    let records = sqlx::query!(
        r#"
        SELECT 
            rt.family_id as "session_id!",
            rt.device_name,
            rt.ip_address,
            (SELECT MIN(rt2.created_at) FROM refresh_tokens rt2 WHERE rt2.family_id = rt.family_id) as "created_at!",
            rt.last_used_at as "last_seen_at!"
        FROM refresh_tokens rt
        WHERE rt.user_id = $1
          AND rt.revoked_at IS NULL
          AND rt.expires_at > NOW()
        ORDER BY rt.last_used_at DESC
        "#,
        user_id
    )
    .fetch_all(&state.db)
    .await?;

    let sessions = records
        .into_iter()
        .map(|rec| SessionResponseDto {
            session_id: rec.session_id,
            device_name: rec.device_name,
            ip_address: rec.ip_address,
            created_at: rec.created_at,
            last_seen_at: rec.last_seen_at,
            current: rec.session_id == current_family_id,
        })
        .collect();

    Ok(sessions)
}
