//! API Key Authentication Middleware
//!
//! Simple authentication using a shared API key.
//! The key is set via APP_API_KEY environment variable.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

/// Middleware to validate API key from X-API-Key header
pub async fn api_key_auth(request: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    // Get the expected API key from environment
    let expected_key = match std::env::var("APP_API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => {
            // No API key configured - allow all requests (development mode)
            tracing::warn!("APP_API_KEY not set - API is unprotected!");
            return Ok(next.run(request).await);
        }
    };

    // Check for API key in header
    let provided_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());

    match provided_key {
        Some(key) if key == expected_key => Ok(next.run(request).await),
        Some(_) => {
            tracing::warn!("Invalid API key provided");
            Err(StatusCode::UNAUTHORIZED)
        }
        None => {
            tracing::debug!("No API key provided");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}
