//! Document service module
//!
//! This module handles all document-related business logic including:
//! - File uploads with streaming to prevent memory bloat
//! - Document assignment to projects (Inbox workflow)
//! - Document retrieval and listing
//!
//! # Architecture Decision
//! File uploads are streamed directly to disk rather than buffered in memory.
//! This prevents memory exhaustion when handling large files and allows
//! the server to handle multiple concurrent uploads efficiently.

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::Document;
use axum::extract::multipart::Field;
use chrono::Local;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

/// Upload directory path
pub const UPLOADS_DIR: &str = "./uploads";

/// Document service handling all document-related business logic
pub struct DocumentService;

impl DocumentService {
    /// Process and save an uploaded file
    ///
    /// This function streams the file directly to disk to minimize memory usage.
    /// It generates a unique filename to prevent collisions.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `field` - Multipart form field containing the file
    ///
    /// # Returns
    /// The created document record
    pub async fn upload(pool: &DbPool, field: Field<'_>) -> AppResult<Document> {
        // Extract file metadata from the multipart field
        let raw_name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let content_type = field
            .content_type()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        // Determine file type category from MIME type
        let file_type = Self::categorize_mime_type(&content_type);

        // Generate date-based filename: YYYY-MM-DD_HH-MM-SS_xxxx.ext
        // The random suffix handles ties (multiple uploads in same second)
        let extension = Path::new(&raw_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("bin");

        let now = Local::now();
        let date_part = now.format("%Y-%m-%d_%H-%M-%S");
        let random_suffix = &Uuid::new_v4().to_string()[..4]; // First 4 chars of UUID
        let unique_filename = format!("{}_{}.{}", date_part, random_suffix, extension);
        let file_path = format!("{}/{}", UPLOADS_DIR, unique_filename);

        // Generate a readable display name if the original is generic (e.g., "image.jpg" from camera)
        let original_name = if Self::is_generic_filename(&raw_name) {
            // Use a friendly format: "Foto 28-12-2025 01:30"
            format!("Foto {}", now.format("%d-%m-%Y %H:%M"))
        } else {
            raw_name
        };

        // Ensure uploads directory exists
        tokio::fs::create_dir_all(UPLOADS_DIR).await?;

        // Stream file to disk
        // This is memory-efficient: we read chunks and write them immediately
        // rather than buffering the entire file in memory
        Self::stream_to_file(field, &file_path).await?;

        // Create document record in database (initially in Inbox with NULL project_id)
        let doc_id = Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT INTO documents (id, project_id, file_path, file_type, original_name, uploaded_at)
            VALUES (?, NULL, ?, ?, ?, datetime('now'))
            "#,
        )
        .bind(&doc_id)
        .bind(&unique_filename)
        .bind(&file_type)
        .bind(&original_name)
        .execute(pool)
        .await?;

        // Fetch and return the created document
        Self::get_by_id(pool, &doc_id).await
    }

    /// Stream multipart field data directly to a file
    ///
    /// This function reads the upload in chunks and writes directly to disk,
    /// preventing memory exhaustion for large files.
    async fn stream_to_file(mut field: Field<'_>, path: &str) -> AppResult<()> {
        let mut file = File::create(path).await?;

        // Read and write in chunks (Axum's multipart handles chunking internally)
        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to read upload chunk: {}", e)))?
        {
            file.write_all(&chunk).await?;
        }

        // Ensure all data is flushed to disk
        file.flush().await?;

        tracing::debug!("File written to: {}", path);
        Ok(())
    }

    /// Categorize MIME type into a simple file type category
    fn categorize_mime_type(mime: &str) -> String {
        if mime.starts_with("image/") {
            "image".to_string()
        } else if mime == "application/pdf" {
            "pdf".to_string()
        } else if mime.starts_with("video/") {
            "video".to_string()
        } else {
            "other".to_string()
        }
    }

    /// Check if a filename is generic (from camera or unknown source)
    fn is_generic_filename(name: &str) -> bool {
        let lower = name.to_lowercase();
        // Common generic names from mobile cameras and browsers
        lower == "image.jpg"
            || lower == "image.jpeg"
            || lower == "image.png"
            || lower == "photo.jpg"
            || lower == "photo.jpeg"
            || lower == "blob"
            || lower == "unknown"
            || lower.starts_with("img_")
            || lower.starts_with("dsc")
            || lower.starts_with("photo_")
    }

    /// Get a document by ID
    pub async fn get_by_id(pool: &DbPool, id: &str) -> AppResult<Document> {
        sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Document with id '{}' not found", id)))
    }

    /// List all documents in the Inbox (unassigned to any project)
    ///
    /// Documents in the Inbox are those with NULL project_id.
    /// They are waiting to be organized into projects.
    pub async fn list_inbox(pool: &DbPool) -> AppResult<Vec<Document>> {
        let docs = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE project_id IS NULL ORDER BY uploaded_at DESC",
        )
        .fetch_all(pool)
        .await?;

        Ok(docs)
    }

    /// List all documents assigned to a specific project
    pub async fn list_by_project(pool: &DbPool, project_id: &str) -> AppResult<Vec<Document>> {
        let docs = sqlx::query_as::<_, Document>(
            "SELECT * FROM documents WHERE project_id = ? ORDER BY uploaded_at DESC",
        )
        .bind(project_id)
        .fetch_all(pool)
        .await?;

        Ok(docs)
    }

    /// Assign a document to a project (or move back to Inbox if project_id is None)
    ///
    /// This implements the core Inbox workflow:
    /// 1. Documents are uploaded to Inbox (project_id = NULL)
    /// 2. User reviews and assigns documents to appropriate projects
    /// 3. Documents can be reassigned or moved back to Inbox
    pub async fn assign_to_project(
        pool: &DbPool,
        document_id: &str,
        project_id: Option<&str>,
    ) -> AppResult<Document> {
        // If assigning to a project, verify the project exists
        if let Some(pid) = project_id {
            let project_exists: Option<(i32,)> =
                sqlx::query_as("SELECT 1 FROM projects WHERE id = ?")
                    .bind(pid)
                    .fetch_optional(pool)
                    .await?;

            if project_exists.is_none() {
                return Err(AppError::NotFound(format!(
                    "Project with id '{}' not found",
                    pid
                )));
            }
        }

        // Update the document's project assignment
        let result = sqlx::query("UPDATE documents SET project_id = ? WHERE id = ?")
            .bind(project_id)
            .bind(document_id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Document with id '{}' not found",
                document_id
            )));
        }

        // Fetch and return the updated document
        Self::get_by_id(pool, document_id).await
    }
}
