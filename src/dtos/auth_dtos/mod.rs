use serde::Deserialize;
#[derive(Deserialize)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}
#[derive(Deserialize)]
pub struct SignUpDto {
    pub email: String,
    pub password: String,
    pub user_name: String,
}

#[derive(Deserialize)]
pub struct LoginDto {
    pub refresh_token: String,
}
