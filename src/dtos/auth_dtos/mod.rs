use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct LoginDto {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Validate)]
pub struct SignUpDto {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters long"))]
    pub password: String,
    #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
    pub user_name: String,
}

#[derive(Deserialize)]
pub struct LogoutDto {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct CreateUserResponse {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub verified: bool,
}
