use axum::{extract::State, response::IntoResponse, Extension, Json};
use crate::database::db_state::AppState;
use crate::errors::AppError;
use crate::middleware::auth::ClaimsExtension;
use super::super::services::logout_all_service::logout_all_sessions;
use serde_json::json;

pub async fn logout_all_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<ClaimsExtension>,
) -> Result<impl IntoResponse, AppError> {
    logout_all_sessions(claims.user_id, state).await?;
    Ok(Json(json!({ "message": "All sessions revoked successfully" })))
}
