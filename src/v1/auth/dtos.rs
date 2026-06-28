use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct LoginDto {
    #[validate(email)]
    pub email: String,
    pub password: String,
    pub device_name: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct SignUpDto {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
    #[validate(length(min = 3, message = "Username must be at least 3 characters long"))]
    pub user_name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateUserResponse {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub verified: bool,
}

#[derive(Deserialize, Validate)]
pub struct VerifyEmailDto {
    #[validate(email)]
    pub email: String,
    #[validate(length(equal = 6, message = "OTP must be exactly 6 digits"))]
    pub otp: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: CreateUserResponse,
}

#[derive(Deserialize, Validate)]
pub struct ForgotPasswordDto {
    #[validate(email)]
    pub email: String,
}

#[derive(Deserialize, Validate)]
pub struct ResetPasswordDto {
    #[validate(length(min = 1, message = "Reset token is required"))]
    pub reset_token: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResendOtpDto {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResendResetOtpDto {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 1, message = "Reset token is required"))]
    pub reset_token: String,
}
