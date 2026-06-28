use axum::{extract::State, response::IntoResponse, Extension, Json};
use crate::database::db_state::AppState;
use crate::errors::AppError;
use crate::v1::auth::middleware::ClaimsExtension;
use super::super::services::get_all_service::get_all_sessions;

pub async fn get_all_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<ClaimsExtension>,
) -> Result<impl IntoResponse, AppError> {
    let sessions = get_all_sessions(claims.user_id, claims.family_id, state).await?;
    Ok(Json(sessions))
}
