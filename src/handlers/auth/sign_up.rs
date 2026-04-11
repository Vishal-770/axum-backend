use crate::database::db_state::AppState;
use crate::dtos::auth_dtos::SignUpDto;
use crate::services::auth_services::sign_up_service::sign_up;
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;

pub async fn sign_up_handler(
    State(state): State<AppState>,
    Json(payload): Json<SignUpDto>,
) -> impl IntoResponse {
    let _result = sign_up(payload.email, payload.user_name, payload.password, state)
        .await
        .unwrap();

    return "Need to implement".to_string();
}
