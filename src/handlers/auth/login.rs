use axum::Json;
use axum::response::IntoResponse;
use crate::dtos::auth_dtos::LoginDto;
use crate::services::auth_services::login_service::login;

pub async fn login_handler(Json(payload): Json<LoginDto>) -> impl IntoResponse{
     let result=login(payload.password,payload.email).await.unwrap();
 
     return "Need to implement".to_string();
}


