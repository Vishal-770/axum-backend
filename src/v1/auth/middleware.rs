use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};
use uuid::Uuid;

use super::claims::AccessClaims;
use super::errors::AuthError;
use crate::errors::AppError;
use crate::database::db_state::AppState;

#[derive(Clone, Copy, Debug)]
pub struct ClaimsExtension {
    pub user_id: Uuid,
    pub family_id: Uuid,
}

pub async fn require_auth(
    State(state): State<AppState>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // 1. Get the access token from the cookies
    let token = jar
        .get("access_token")
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| AppError::Auth(AuthError::Unauthorized))?;

    // 2. Read the access token secret from AppState (cached at boot, no env mutex)
    let access_secret = &state.config.jwt_access_secret;

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
