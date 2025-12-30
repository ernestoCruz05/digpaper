//! Email webhook handlers module
//!
//! HTTP handlers for receiving inbound emails from services like Mailgun or SendGrid.
//! Emails with attachments will have their attachments saved as documents in the Inbox.

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;

use crate::db::DbPool;
use crate::error::AppResult;
use crate::models::{CreateEmailFilterRequest, CreateEmailRuleRequest, EmailFilter, EmailRule};
use crate::services::EmailService;

/// Response for email webhook processing
#[derive(Debug, Serialize)]
pub struct EmailWebhookResponse {
    pub success: bool,
    pub message: String,
    pub documents_created: usize,
    pub documents_filtered: usize,
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
            "Email processed: {} document(s) created, {} filtered out.",
            result.documents_created,
            result.documents_filtered
        ),
        documents_created: result.documents_created,
        documents_filtered: result.documents_filtered,
    };

    tracing::info!(
        "Email from '{}' processed: {} attachments saved, {} filtered",
        result.sender,
        result.documents_created,
        result.documents_filtered
    );

    Ok((StatusCode::OK, Json(response)))
}

/// GET /api/email/status - Check email webhook configuration
///
/// Returns the status of the email webhook endpoint.
/// Useful for testing that the endpoint is accessible.
pub async fn email_webhook_status(
    State(pool): State<DbPool>,
) -> AppResult<Json<serde_json::Value>> {
    let rules = EmailService::list_rules(&pool).await?;
    let filters = EmailService::list_filters(&pool).await?;
    
    Ok(Json(serde_json::json!({
        "status": "active",
        "endpoint": "/api/email/inbound",
        "supported_services": ["mailgun", "sendgrid"],
        "rules_count": rules.len(),
        "filters_count": filters.len(),
        "note": "Configure your email service to forward emails to this endpoint"
    })))
}

// =============================================================================
// Email Rules CRUD
// =============================================================================

/// GET /api/email/rules - List all email routing rules
pub async fn list_email_rules(
    State(pool): State<DbPool>,
) -> AppResult<Json<Vec<EmailRule>>> {
    let rules = EmailService::list_rules(&pool).await?;
    Ok(Json(rules))
}

/// POST /api/email/rules - Create a new email routing rule
pub async fn create_email_rule(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateEmailRuleRequest>,
) -> AppResult<(StatusCode, Json<EmailRule>)> {
    let rule = EmailService::create_rule(
        &pool,
        &payload.sender_pattern,
        payload.project_id.as_deref(),
        payload.description.as_deref(),
    ).await?;
    Ok((StatusCode::CREATED, Json(rule)))
}

/// DELETE /api/email/rules/:id - Delete an email routing rule
pub async fn delete_email_rule(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    EmailService::delete_rule(&pool, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// =============================================================================
// Email Filters CRUD
// =============================================================================

/// GET /api/email/filters - List all email attachment filters
pub async fn list_email_filters(
    State(pool): State<DbPool>,
) -> AppResult<Json<Vec<EmailFilter>>> {
    let filters = EmailService::list_filters(&pool).await?;
    Ok(Json(filters))
}

/// POST /api/email/filters - Create a new email attachment filter
pub async fn create_email_filter(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateEmailFilterRequest>,
) -> AppResult<(StatusCode, Json<EmailFilter>)> {
    let filter = EmailService::create_filter(
        &pool,
        &payload.pattern,
        &payload.filter_type,
    ).await?;
    Ok((StatusCode::CREATED, Json(filter)))
}

/// DELETE /api/email/filters/:id - Delete an email attachment filter
pub async fn delete_email_filter(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    EmailService::delete_filter(&pool, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
