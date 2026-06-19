use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};
use uuid::Uuid;

use crate::auth::claims::AccessClaims;
use crate::errors::{AppError, auth_error::AuthError};

#[derive(Clone, Copy, Debug)]
pub struct ClaimsExtension {
    pub user_id: Uuid,
    pub family_id: Uuid,
}

pub async fn require_auth(
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 1. Get the access token from the cookies
    let token = jar
        .get("access_token")
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| AppError::Auth(AuthError::Unauthorized))?;

    // 2. Fetch the access token secret from environment
    let access_secret = std::env::var("JWT_ACCESS_SECRET")
        .expect("JWT_ACCESS_SECRET must be set");

    // 3. Decode and validate the access token
    let token_data = decode::<AccessClaims>(
        &token,
        &DecodingKey::from_secret(access_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Auth(AuthError::Unauthorized))?;

    // 4. Inject the user_id and family_id into request extensions
    req.extensions_mut().insert(ClaimsExtension {
        user_id: token_data.claims.sub,
        family_id: token_data.claims.family_id,
    });

    // 5. Proceed to the next handler/middleware
    Ok(next.run(req).await)
}
