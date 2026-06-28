#[derive(Clone, Debug)]
pub struct AuthConfig {
    pub jwt_access_secret: String,
    pub jwt_refresh_secret: String,
}

impl AuthConfig {
    pub fn from_env() -> Self {
        let jwt_access_secret = std::env::var("JWT_ACCESS_SECRET")
            .expect("JWT_ACCESS_SECRET must be set");
        let jwt_refresh_secret = std::env::var("JWT_REFRESH_SECRET")
            .expect("JWT_REFRESH_SECRET must be set");
        Self {
            jwt_access_secret,
            jwt_refresh_secret,
        }
    }
}
