//! Email processing service
//!
//! Handles processing of inbound emails from webhook services like Mailgun and SendGrid.
//! Extracts attachments and creates documents based on routing rules.

use axum::extract::Multipart;
use chrono::Utc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{Document, EmailFilter, EmailRule};
use crate::services::document_service::UPLOADS_DIR;

/// Result of processing an inbound email
#[allow(dead_code)]
pub struct EmailProcessingResult {
    pub sender: String,
    pub subject: String,
    pub documents_created: usize,
    pub documents_filtered: usize,
}

/// Service for handling email-related operations
pub struct EmailService;

impl EmailService {
    // =========================================================================
    // Email Rules CRUD
    // =========================================================================
    
    /// List all email routing rules
    pub async fn list_rules(pool: &DbPool) -> AppResult<Vec<EmailRule>> {
        let rules = sqlx::query_as::<_, EmailRule>(
            "SELECT * FROM email_rules ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await?;
        Ok(rules)
    }
    
    /// Create a new email routing rule
    pub async fn create_rule(
        pool: &DbPool,
        sender_pattern: &str,
        project_id: Option<&str>,
        description: Option<&str>,
    ) -> AppResult<EmailRule> {
        let id = Uuid::new_v4().to_string();
        
        sqlx::query(
            "INSERT INTO email_rules (id, sender_pattern, project_id, description) VALUES (?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(sender_pattern)
        .bind(project_id)
        .bind(description)
        .execute(pool)
        .await?;
        
        let rule = sqlx::query_as::<_, EmailRule>("SELECT * FROM email_rules WHERE id = ?")
            .bind(&id)
            .fetch_one(pool)
            .await?;
        
        Ok(rule)
    }
    
    /// Delete an email routing rule
    pub async fn delete_rule(pool: &DbPool, id: &str) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM email_rules WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Email rule '{}' not found", id)));
        }
        Ok(())
    }
    
    // =========================================================================
    // Email Filters CRUD
    // =========================================================================
    
    /// List all email attachment filters
    pub async fn list_filters(pool: &DbPool) -> AppResult<Vec<EmailFilter>> {
        let filters = sqlx::query_as::<_, EmailFilter>(
            "SELECT * FROM email_filters ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await?;
        Ok(filters)
    }
    
    /// Create a new email attachment filter
    pub async fn create_filter(
        pool: &DbPool,
        pattern: &str,
        filter_type: &str,
    ) -> AppResult<EmailFilter> {
        // Validate filter type
        if !["filename", "extension", "size_max"].contains(&filter_type) {
            return Err(AppError::BadRequest(
                "filter_type must be 'filename', 'extension', or 'size_max'".into()
            ));
        }
        
        let id = Uuid::new_v4().to_string();
        
        sqlx::query(
            "INSERT INTO email_filters (id, pattern, filter_type) VALUES (?, ?, ?)"
        )
        .bind(&id)
        .bind(pattern)
        .bind(filter_type)
        .execute(pool)
        .await?;
        
        let filter = sqlx::query_as::<_, EmailFilter>("SELECT * FROM email_filters WHERE id = ?")
            .bind(&id)
            .fetch_one(pool)
            .await?;
        
        Ok(filter)
    }
    
    /// Delete an email attachment filter
    pub async fn delete_filter(pool: &DbPool, id: &str) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM email_filters WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Email filter '{}' not found", id)));
        }
        Ok(())
    }
    
    // =========================================================================
    // Email Processing
    // =========================================================================
    
    /// Find matching routing rule for a sender
    async fn find_matching_rule(pool: &DbPool, sender: &str) -> AppResult<Option<EmailRule>> {
        let rules = sqlx::query_as::<_, EmailRule>(
            "SELECT * FROM email_rules WHERE active = 1"
        )
        .fetch_all(pool)
        .await?;
        
        let sender_lower = sender.to_lowercase();
        
        for rule in rules {
            let pattern = rule.sender_pattern.to_lowercase();
            
            // Support wildcard patterns like "*@domain.com"
            if pattern.starts_with('*') {
                let suffix = &pattern[1..];
                if sender_lower.ends_with(suffix) {
                    return Ok(Some(rule));
                }
            } else if sender_lower.contains(&pattern) {
                return Ok(Some(rule));
            }
        }
        
        Ok(None)
    }
    
    /// Check if an attachment should be filtered out
    async fn should_filter_attachment(
        pool: &DbPool,
        filename: &str,
        size: usize,
    ) -> AppResult<bool> {
        let filters = sqlx::query_as::<_, EmailFilter>(
            "SELECT * FROM email_filters WHERE active = 1"
        )
        .fetch_all(pool)
        .await?;
        
        let filename_lower = filename.to_lowercase();
        
        for filter in filters {
            match filter.filter_type.as_str() {
                "filename" => {
                    // Check if filename contains the pattern
                    if filename_lower.contains(&filter.pattern.to_lowercase()) {
                        tracing::debug!("Filtering '{}': matches filename pattern '{}'", filename, filter.pattern);
                        return Ok(true);
                    }
                }
                "extension" => {
                    // Check file extension
                    let pattern = filter.pattern.to_lowercase();
                    let pattern = if pattern.starts_with('.') { pattern } else { format!(".{}", pattern) };
                    if filename_lower.ends_with(&pattern) {
                        tracing::debug!("Filtering '{}': matches extension '{}'", filename, filter.pattern);
                        return Ok(true);
                    }
                }
                "size_max" => {
                    // Filter files smaller than the threshold (likely logos/icons)
                    if let Ok(max_size) = filter.pattern.parse::<usize>() {
                        if size < max_size {
                            tracing::debug!("Filtering '{}': size {} < {} bytes", filename, size, max_size);
                            return Ok(true);
                        }
                    }
                }
                _ => {}
            }
        }
        
        Ok(false)
    }

    /// Process an inbound email from a webhook service
    ///
    /// Parses the multipart form data to extract:
    /// - Email metadata (from, subject, body)
    /// - File attachments
    ///
    /// Attachments are filtered and routed based on configured rules.
    pub async fn process_inbound_email(
        pool: &DbPool,
        mut multipart: Multipart,
    ) -> AppResult<EmailProcessingResult> {
        let mut sender = String::new();
        let mut subject = String::new();
        let mut body = String::new();
        let mut documents_created = 0;
        let mut documents_filtered = 0;

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

        // Find matching routing rule for this sender
        let target_project_id = match Self::find_matching_rule(pool, &sender).await? {
            Some(rule) => {
                tracing::info!(
                    "Email from '{}' matches rule '{}' -> project {:?}",
                    sender,
                    rule.sender_pattern,
                    rule.project_id
                );
                rule.project_id
            }
            None => {
                tracing::debug!("No routing rule matches sender '{}', going to Inbox", sender);
                None
            }
        };

        // Process each attachment
        for (filename, content_type, data) in attachments {
            // Check if this attachment should be filtered out
            if Self::should_filter_attachment(pool, &filename, data.len()).await? {
                documents_filtered += 1;
                tracing::info!("Filtered out attachment: {} ({} bytes)", filename, data.len());
                continue;
            }
            
            match Self::save_attachment(pool, &filename, &content_type, &data, &notes, target_project_id.as_deref()).await {
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
            tracing::info!("Email received with no attachments (or all filtered). Subject: {}", subject);
        }

        Ok(EmailProcessingResult {
            sender,
            subject,
            documents_created,
            documents_filtered,
        })
    }

    /// Save an email attachment as a document
    async fn save_attachment(
        pool: &DbPool,
        original_name: &str,
        content_type: &str,
        data: &[u8],
        notes: &str,
        project_id: Option<&str>,
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

        // Create document record (with project_id if routing rule matched)
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO documents (id, project_id, file_path, file_type, original_name, notes)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(project_id)
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
