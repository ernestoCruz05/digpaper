//! Data models for DigPaper
//!
//! This module defines the core domain entities and DTOs (Data Transfer Objects)
//! used throughout the application. Models are separated into:
//! - Domain entities (stored in database)
//! - Request DTOs (received from clients)
//! - Response DTOs (sent to clients)
//!
//! # Architecture Decision
//! Separating DTOs from domain entities allows for flexibility in API evolution
//! without affecting the database schema, and vice versa.

use serde::{Deserialize, Serialize};

// =============================================================================
// Domain Entities
// =============================================================================

/// Project entity representing a work order ("Obra")
///
/// Projects are the primary organizational unit. Each project groups
/// related documents (sketches, cut-lists, photos) together.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Project {
    /// Unique identifier (UUID v4)
    pub id: String,
    /// Human-readable project name (e.g., "Obra Porto Seg Social")
    pub name: String,
    /// Workflow status: "ACTIVE" for ongoing, "ARCHIVED" for completed
    pub status: String,
    /// Timestamp when the project was created
    pub created_at: String,
}

/// Project status enum for type-safe status handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ProjectStatus {
    Active,
    Archived,
}

impl ProjectStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectStatus::Active => "ACTIVE",
            ProjectStatus::Archived => "ARCHIVED",
        }
    }
}

impl std::fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Document entity representing an uploaded file
///
/// Documents can be in two states:
/// - Inbox: project_id is None (unassigned, waiting for organization)
/// - Assigned: project_id points to a specific project
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Document {
    /// Unique identifier (UUID v4)
    pub id: String,
    /// Associated project ID, None if in Inbox
    pub project_id: Option<String>,
    /// Relative path to the file in uploads directory
    pub file_path: String,
    /// File type category (image, pdf, etc.)
    pub file_type: String,
    /// Original filename as uploaded by user
    pub original_name: String,
    /// Timestamp when the document was uploaded
    pub uploaded_at: String,
}

// =============================================================================
// Request DTOs
// =============================================================================

/// Request payload for creating a new project
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    /// Name for the new project
    pub name: String,
}

/// Query parameters for listing projects
#[derive(Debug, Deserialize)]
pub struct ListProjectsQuery {
    /// Optional status filter (active/archived)
    pub status: Option<String>,
}

/// Request payload for assigning a document to a project
#[derive(Debug, Deserialize)]
pub struct AssignDocumentRequest {
    /// Target project ID (None to move back to Inbox)
    pub project_id: Option<String>,
}

/// Request payload for updating project status
#[derive(Debug, Deserialize)]
pub struct UpdateProjectStatusRequest {
    /// New status: ACTIVE or ARCHIVED
    pub status: ProjectStatus,
}

// =============================================================================
// Response DTOs
// =============================================================================

/// Response for successful project creation
#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub status: String,
    pub created_at: String,
}

impl From<Project> for ProjectResponse {
    fn from(p: Project) -> Self {
        Self {
            id: p.id,
            name: p.name,
            status: p.status,
            created_at: p.created_at,
        }
    }
}

/// Response for document operations
#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: String,
    pub project_id: Option<String>,
    pub file_path: String,
    pub file_type: String,
    pub original_name: String,
    pub uploaded_at: String,
    /// Full URL to access the file
    pub file_url: String,
}

impl DocumentResponse {
    /// Create response from document entity, generating the file URL
    pub fn from_document(doc: Document, base_url: &str) -> Self {
        let file_url = format!("{}/files/{}", base_url, doc.file_path);
        Self {
            id: doc.id,
            project_id: doc.project_id,
            file_path: doc.file_path,
            file_type: doc.file_type,
            original_name: doc.original_name,
            uploaded_at: doc.uploaded_at,
            file_url,
        }
    }
}

/// Response for file upload operations
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub id: String,
    pub file_path: String,
    pub file_type: String,
    pub original_name: String,
    pub file_url: String,
}

/// Generic success message response
/// Reserved for future use in delete/archive operations
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}
