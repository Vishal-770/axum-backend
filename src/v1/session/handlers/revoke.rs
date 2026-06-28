use axum::{extract::{Path, State}, response::IntoResponse, Extension, Json};
use crate::database::db_state::AppState;
use crate::errors::AppError;
use crate::v1::auth::middleware::ClaimsExtension;
use super::super::services::revoke_service::revoke_session;
use uuid::Uuid;
use serde_json::json;

pub async fn revoke_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<ClaimsExtension>,
    Path(family_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    revoke_session(claims.user_id, family_id, state).await?;
    Ok(Json(json!({ "message": "Session revoked successfully" })))
}
