use axum::{extract::State, response::IntoResponse, Extension, Json};
use crate::database::db_state::AppState;
use crate::errors::AppError;
use crate::v1::auth::middleware::ClaimsExtension;
use super::services::get_me;

pub async fn me_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<ClaimsExtension>,
) -> Result<impl IntoResponse, AppError> {
    let user_details = get_me(claims.user_id, state).await?;
    Ok(Json(user_details))
}
