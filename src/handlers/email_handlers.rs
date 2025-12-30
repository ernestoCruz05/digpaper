//! Email webhook handlers module
//!
//! HTTP handlers for receiving inbound emails from services like Mailgun or SendGrid.
//! Emails with attachments will have their attachments saved as documents in the Inbox.

use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;

use crate::db::DbPool;
use crate::error::AppResult;
use crate::services::EmailService;

/// Response for email webhook processing
#[derive(Debug, Serialize)]
pub struct EmailWebhookResponse {
    pub success: bool,
    pub message: String,
    pub documents_created: usize,
}

/// POST /api/email/inbound - Receive inbound email webhook
///
/// This endpoint receives emails forwarded from Mailgun, SendGrid, or similar services.
/// It extracts attachments and creates documents in the Inbox.
///
/// # Mailgun Format
/// Mailgun sends a multipart/form-data POST with fields like:
/// - `from`: Sender email address
/// - `subject`: Email subject line
/// - `body-plain`: Plain text body
/// - `attachment-1`, `attachment-2`, etc.: File attachments
///
/// # SendGrid Format
/// SendGrid uses similar multipart format with:
/// - `from`: Sender email
/// - `subject`: Subject line  
/// - `text`: Plain text body
/// - `attachmentX`: File attachments
///
/// # Response
/// Returns the number of documents created from attachments
pub async fn receive_inbound_email(
    State(pool): State<DbPool>,
    multipart: Multipart,
) -> AppResult<(StatusCode, Json<EmailWebhookResponse>)> {
    tracing::info!("Received inbound email webhook");

    let result = EmailService::process_inbound_email(&pool, multipart).await?;

    let response = EmailWebhookResponse {
        success: true,
        message: format!(
            "Email processed successfully. {} document(s) created from attachments.",
            result.documents_created
        ),
        documents_created: result.documents_created,
    };

    tracing::info!(
        "Email from '{}' processed: {} attachments saved",
        result.sender,
        result.documents_created
    );

    Ok((StatusCode::OK, Json(response)))
}

/// GET /api/email/status - Check email webhook configuration
///
/// Returns the status of the email webhook endpoint.
/// Useful for testing that the endpoint is accessible.
pub async fn email_webhook_status() -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "status": "active",
        "endpoint": "/api/email/inbound",
        "supported_services": ["mailgun", "sendgrid"],
        "note": "Configure your email service to forward emails to this endpoint"
    })))
}
