pub mod auth_error;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use self::auth_error::AuthError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Auth(#[from] AuthError),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal server error")]
    InternalServer,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Auth(err) => err.into_response(),
            AppError::Database(err) => {
                // Log database errors in a real application
                println!("Database error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Internal database error" })),
                )
                    .into_response()
            }
            AppError::InternalServer => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            )
                .into_response(),
        }
    }
}
