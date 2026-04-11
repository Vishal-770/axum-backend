use axum::Json;
use axum::response::IntoResponse;
use crate::dtos::auth_dtos::SignUpDto;
use crate::services::auth_services::sign_up_service::sign_up;

pub async fn sign_up_handler(Json(payload):Json<SignUpDto>) ->impl IntoResponse {
    let _result=sign_up(payload.email,payload.user_name,payload.password).await.unwrap();

    return "Need to implement".to_string();
}

