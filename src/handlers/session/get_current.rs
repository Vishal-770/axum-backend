use axum::{extract::State, response::IntoResponse, Extension, Json};
use crate::database::db_state::AppState;
use crate::errors::AppError;
use crate::middleware::auth::ClaimsExtension;
use crate::services::session_services::get_current_service::get_current_session;

pub async fn get_current_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<ClaimsExtension>,
) -> Result<impl IntoResponse, AppError> {
    let session = get_current_session(claims.user_id, claims.family_id, state).await?;
    Ok(Json(session))
}
