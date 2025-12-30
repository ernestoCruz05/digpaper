//! Email processing service
//!
//! Handles processing of inbound emails from webhook services like Mailgun and SendGrid.
//! Extracts attachments and creates documents in the Inbox.

use axum::extract::Multipart;
use chrono::Utc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::Document;
use crate::services::document_service::UPLOADS_DIR;

/// Result of processing an inbound email
#[allow(dead_code)]
pub struct EmailProcessingResult {
    pub sender: String,
    pub subject: String,
    pub documents_created: usize,
}

/// Service for handling email-related operations
pub struct EmailService;

impl EmailService {
    /// Process an inbound email from a webhook service
    ///
    /// Parses the multipart form data to extract:
    /// - Email metadata (from, subject, body)
    /// - File attachments
    ///
    /// Each attachment is saved as a document in the Inbox with
    /// the email subject and sender added as notes.
    pub async fn process_inbound_email(
        pool: &DbPool,
        mut multipart: Multipart,
    ) -> AppResult<EmailProcessingResult> {
        let mut sender = String::new();
        let mut subject = String::new();
        let mut body = String::new();
        let mut documents_created = 0;

        // Collect attachments to process after we have metadata
        let mut attachments: Vec<(String, String, Vec<u8>)> = Vec::new();

        // Parse all multipart fields
        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to read multipart field: {}", e)))?
        {
            let field_name = field.name().unwrap_or("").to_string();

            match field_name.as_str() {
                // Mailgun format
                "from" => {
                    sender = field.text().await.unwrap_or_default();
                }
                "subject" => {
                    subject = field.text().await.unwrap_or_default();
                }
                "body-plain" | "text" | "body-html" => {
                    if body.is_empty() {
                        body = field.text().await.unwrap_or_default();
                    }
                }
                // Handle attachments - Mailgun uses attachment-1, attachment-2, etc.
                // SendGrid uses attachment1, attachment2, etc.
                name if name.starts_with("attachment") => {
                    if let Some(filename) = field.file_name().map(|s| s.to_string()) {
                        let content_type = field
                            .content_type()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "application/octet-stream".to_string());

                        let data = field
                            .bytes()
                            .await
                            .map_err(|e| AppError::BadRequest(format!("Failed to read attachment: {}", e)))?;

                        if !data.is_empty() {
                            attachments.push((filename, content_type, data.to_vec()));
                        }
                    }
                }
                // Some services send attachments with a numeric suffix
                name if name.parse::<i32>().is_ok() => {
                    if let Some(filename) = field.file_name().map(|s| s.to_string()) {
                        let content_type = field
                            .content_type()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "application/octet-stream".to_string());

                        let data = field
                            .bytes()
                            .await
                            .map_err(|e| AppError::BadRequest(format!("Failed to read attachment: {}", e)))?;

                        if !data.is_empty() {
                            attachments.push((filename, content_type, data.to_vec()));
                        }
                    }
                }
                _ => {
                    // Log unknown fields for debugging
                    tracing::debug!("Ignoring email field: {}", field_name);
                }
            }
        }

        // If no subject, use a default
        if subject.is_empty() {
            subject = "Email sem assunto".to_string();
        }

        // If no sender, use unknown
        if sender.is_empty() {
            sender = "unknown@email.com".to_string();
        }

        // Create notes from email metadata
        let notes = format!(
            "📧 Email de: {}\n📋 Assunto: {}",
            sender, subject
        );

        // Process each attachment
        for (filename, content_type, data) in attachments {
            match Self::save_attachment(pool, &filename, &content_type, &data, &notes).await {
                Ok(_) => {
                    documents_created += 1;
                    tracing::info!("Saved email attachment: {}", filename);
                }
                Err(e) => {
                    tracing::error!("Failed to save attachment '{}': {}", filename, e);
                }
            }
        }

        // If no attachments but we have body text, we could optionally save the email body
        // as a text file. For now, we just log this case.
        if documents_created == 0 && !body.is_empty() {
            tracing::info!("Email received with no attachments. Subject: {}", subject);
        }

        Ok(EmailProcessingResult {
            sender,
            subject,
            documents_created,
        })
    }

    /// Save an email attachment as a document
    async fn save_attachment(
        pool: &DbPool,
        original_name: &str,
        content_type: &str,
        data: &[u8],
        notes: &str,
    ) -> AppResult<Document> {
        // Generate unique filename with date prefix
        let extension = Self::get_extension(original_name, content_type);
        let date_prefix = Utc::now().format("%Y-%m-%d_%H-%M-%S");
        let short_uuid = &Uuid::new_v4().to_string()[..4];
        let safe_filename = format!("{}_{}.{}", date_prefix, short_uuid, extension);

        // Write file to disk
        let file_path = format!("{}/{}", UPLOADS_DIR, safe_filename);
        let mut file = File::create(&file_path)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create file: {}", e)))?;

        file.write_all(data)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to write file: {}", e)))?;

        file.flush().await?;

        // Determine file type
        let file_type = Self::categorize_content_type(content_type);

        // Create document record
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO documents (id, project_id, file_path, file_type, original_name, notes)
            VALUES (?, NULL, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&safe_filename)
        .bind(&file_type)
        .bind(original_name)
        .bind(notes)
        .execute(pool)
        .await?;

        // Fetch and return the created document
        let doc = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = ?")
            .bind(&id)
            .fetch_one(pool)
            .await?;

        Ok(doc)
    }

    /// Get file extension from filename or content type
    fn get_extension(filename: &str, content_type: &str) -> String {
        // Try to get extension from filename first
        if let Some(ext) = filename.rsplit('.').next() {
            if !ext.contains('/') && ext.len() <= 10 {
                return ext.to_lowercase();
            }
        }

        // Fall back to content type
        match content_type {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/gif" => "gif",
            "image/webp" => "webp",
            "application/pdf" => "pdf",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => "xlsx",
            "application/vnd.ms-excel" => "xls",
            "text/csv" => "csv",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => "docx",
            "application/msword" => "doc",
            "text/plain" => "txt",
            _ => "bin",
        }
        .to_string()
    }

    /// Categorize content type into simple file type
    fn categorize_content_type(content_type: &str) -> String {
        if content_type.starts_with("image/") {
            "image".to_string()
        } else if content_type == "application/pdf" {
            "pdf".to_string()
        } else if content_type.contains("spreadsheet") || content_type.contains("excel") || content_type == "text/csv" {
            "excel".to_string()
        } else if content_type.contains("word") || content_type.contains("document") {
            "word".to_string()
        } else if content_type.starts_with("video/") {
            "video".to_string()
        } else {
            "other".to_string()
        }
    }
}
