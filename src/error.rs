//! Error handling module for DigPaper
//!
//! This module defines a unified error type that can be converted into HTTP responses.
//! Using `thiserror` for ergonomic error definition and implementing `IntoResponse`
//! for seamless integration with Axum handlers.
//!
//! # Architecture Decision
//! A centralized error type allows consistent error responses across all endpoints
//! and simplifies error propagation using the `?` operator.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// Unified application error type
///
/// Each variant maps to a specific HTTP status code and provides
/// context about what went wrong.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Resource not found (404)
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Invalid request data (400)
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Database operation failed (500)
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// File I/O operation failed (500)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Internal server error (500)
    #[error("Internal error: {0}")]
    Internal(String),
}

/// JSON response body for error messages
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Map error variants to appropriate HTTP status codes
        let (status, error_type) = match &self {
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database_error"),
            AppError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "io_error"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };

        // Log the error for debugging (internal errors get more detail)
        match &self {
            AppError::Database(e) => tracing::error!("Database error: {:?}", e),
            AppError::Io(e) => tracing::error!("IO error: {:?}", e),
            AppError::Internal(e) => tracing::error!("Internal error: {}", e),
            _ => tracing::warn!("Request error: {}", self),
        }

        let body = ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
        };

        (status, Json(body)).into_response()
    }
}

/// Type alias for handler results
pub type AppResult<T> = Result<T, AppError>;
