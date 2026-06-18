use crate::auth::claims::{AccessClaims, RefreshClaims};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use uuid::Uuid;

pub fn create_access_token(
    user_id: Uuid,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();

    encode(
        &Header::default(),
        &AccessClaims {
            sub: user_id,
            iat: now.timestamp() as usize,
            exp: (now + Duration::minutes(15)).timestamp() as usize,
        },
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}
pub fn create_refresh_token(
    user_id: Uuid,
    secret: &str,
) -> Result<(String, Uuid), jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let jti = Uuid::new_v4();
    let token = encode(
        &Header::default(),
        &RefreshClaims {
            sub: user_id,
            iat: now.timestamp() as usize,
            exp: (now + Duration::days(7)).timestamp() as usize,
            jti: jti,
        },
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok((token, jti))
}
