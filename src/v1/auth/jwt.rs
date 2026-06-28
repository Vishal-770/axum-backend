use super::claims::{AccessClaims, RefreshClaims};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use uuid::Uuid;

pub fn create_access_token(
    user_id: Uuid,
    family_id: Uuid,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();

    encode(
        &Header::default(),
        &AccessClaims {
            sub: user_id,
            iat: now.timestamp() as usize,
            exp: (now + Duration::minutes(15)).timestamp() as usize,
            family_id,
        },
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn create_refresh_token(
    user_id: Uuid,
    secret: &str,
) -> Result<(String, Uuid, chrono::DateTime<chrono::Utc>), jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let jti = Uuid::new_v4();
    let expires_at = now + Duration::days(7);
    let token = encode(
        &Header::default(),
        &RefreshClaims {
            sub: user_id,
            iat: now.timestamp() as usize,
            exp: expires_at.timestamp() as usize,
            jti,
        },
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok((token, jti, expires_at))
}
