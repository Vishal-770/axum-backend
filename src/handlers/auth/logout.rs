use std::result;
use axum::Json;
use axum::response::IntoResponse;
use crate::dtos::auth_dtos::LogoutDto;
use crate::services::auth_services::logout_service::logout;

pub async fn logout_handler(Json(payload):Json<LogoutDto>) -> impl IntoResponse{
    let result=logout(payload.refresh_token).await.unwrap();
    return "Need to implement".to_string();
}