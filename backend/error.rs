// backend/error.rs
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use diesel::r2d2;
use diesel::result::Error as DieselError;
use serde::Serializer;
use std::fmt;

#[derive(Debug)]
pub enum ApiError {
    Database(DieselError),
    DatabaseConnection(String),
    Validation(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Database(e) => write!(f, "Database error: {}", e),
            ApiError::DatabaseConnection(e) => write!(f, "Database connection error: {}", e),
            ApiError::Validation(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl From<DieselError> for ApiError {
    fn from(err: DieselError) -> Self {
        ApiError::Database(err)
    }
}

impl From<r2d2::Error> for ApiError {
    fn from(err: r2d2::Error) -> Self {
        ApiError::DatabaseConnection(err.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::Database(e) => match e {
                DieselError::NotFound => (
                    StatusCode::NOT_FOUND,
                    serde_json::json!({
                        "error": "User not found",
                        "details": e.to_string()
                    }),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    serde_json::json!({
                        "error": "Database error",
                        "details": e.to_string()
                    }),
                ),
            },
            ApiError::DatabaseConnection(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::json!({
                    "error": "Database connection error",
                    "details": e
                }),
            ),
            ApiError::Validation(e) => (
                StatusCode::BAD_REQUEST,
                serde_json::json!({
                    "error": "Validation error",
                    "details": e
                }),
            ),
        };
        (status, axum::Json(error_message)).into_response()
    }
}