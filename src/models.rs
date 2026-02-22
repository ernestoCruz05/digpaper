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
    /// Job site address (optional)
    pub address: Option<String>,
    /// Client phone number (optional)
    pub client_phone: Option<String>,
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
    /// User notes/annotations for this document
    pub notes: Option<String>,
    /// Document status (DEFAULT, DOUBT, IN_PROGRESS, COMPLETED)
    pub status: String,
    /// Document category/room (e.g., Kitchen, Bathroom)
    pub category: Option<String>,
    /// Optional voice memo audio file path
    pub audio_path: Option<String>,
}

/// Document status enum for type-safe status handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocumentStatus {
    Default,
    Doubt,
    InProgress,
    Completed,
}

impl DocumentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DocumentStatus::Default => "DEFAULT",
            DocumentStatus::Doubt => "DOUBT",
            DocumentStatus::InProgress => "IN_PROGRESS",
            DocumentStatus::Completed => "COMPLETED",
        }
    }
}

impl std::fmt::Display for DocumentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Email routing rule
/// Routes emails from specific senders to specific projects
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct EmailRule {
    pub id: String,
    /// Email pattern to match (e.g., "client@example.com" or "*@company.com")
    pub sender_pattern: String,
    /// Target project ID (None = Inbox)
    pub project_id: Option<String>,
    /// Description for this rule
    pub description: Option<String>,
    /// Whether this rule is active
    pub active: bool,
    pub created_at: String,
}

/// Email attachment filter
/// Filters out unwanted attachments (logos, signatures, etc.)
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct EmailFilter {
    pub id: String,
    /// Pattern to match
    pub pattern: String,
    /// Type of filter: "filename", "extension", or "size_max"
    pub filter_type: String,
    /// Whether this filter is active
    pub active: bool,
    pub created_at: String,
}

/// Global User Profile
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct UserProfile {
    pub name: String,
    pub photo_url: String,
    pub updated_at: String,
}

// =============================================================================
// Request DTOs
// =============================================================================

/// Request payload for creating a new project
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    /// Name for the new project
    pub name: String,
    /// Job site address (optional)
    pub address: Option<String>,
    /// Client phone number (optional)
    pub client_phone: Option<String>,
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
    /// Optional category/room assignment at the time of move
    pub category: Option<String>,
}

/// Request payload for batch assigning documents to a project
#[derive(Debug, Deserialize)]
pub struct BatchAssignRequest {
    /// List of document IDs to assign
    pub document_ids: Vec<String>,
    /// Target project ID (None to move back to Inbox)
    pub project_id: Option<String>,
}

/// Request payload for updating project status
#[derive(Debug, Deserialize)]
pub struct UpdateProjectStatusRequest {
    /// New status: ACTIVE or ARCHIVED
    pub status: ProjectStatus,
}

/// Request payload for updating project details
#[derive(Debug, Deserialize)]
pub struct UpdateProjectDetailsRequest {
    pub address: Option<String>,
    pub client_phone: Option<String>,
}

/// Request payload for updating document notes
#[derive(Debug, Deserialize)]
pub struct UpdateDocumentNotesRequest {
    /// Notes text (can be empty to clear notes)
    pub notes: Option<String>,
}

/// Request payload for updating document status
#[derive(Debug, Deserialize)]
pub struct UpdateDocumentStatusRequest {
    pub status: DocumentStatus,
}

/// Request payload for updating document category (room)
#[derive(Debug, Deserialize)]
pub struct UpdateDocumentCategoryRequest {
    pub category: Option<String>,
}

/// Request payload for creating an email routing rule
#[derive(Debug, Deserialize)]
pub struct CreateEmailRuleRequest {
    /// Email pattern to match (e.g., "client@example.com")
    pub sender_pattern: String,
    /// Target project ID (None = Inbox)
    pub project_id: Option<String>,
    /// Description for this rule
    pub description: Option<String>,
}

/// Request payload for creating an email filter
#[derive(Debug, Deserialize)]
pub struct CreateEmailFilterRequest {
    /// Pattern to match
    pub pattern: String,
    /// Type of filter: "filename", "extension", or "size_max"
    pub filter_type: String,
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
    pub address: Option<String>,
    pub client_phone: Option<String>,
    pub created_at: String,
    pub document_count: i32,
}

impl ProjectResponse {
    pub fn from_project(p: Project, document_count: i32) -> Self {
        Self {
            id: p.id,
            name: p.name,
            status: p.status,
            address: p.address,
            client_phone: p.client_phone,
            created_at: p.created_at,
            document_count,
        }
    }
}

impl From<Project> for ProjectResponse {
    fn from(p: Project) -> Self {
        Self {
            id: p.id,
            name: p.name,
            status: p.status,
            address: p.address,
            client_phone: p.client_phone,
            created_at: p.created_at,
            document_count: 0,
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
    /// User notes/annotations
    pub notes: Option<String>,
    /// Current workflow status
    pub status: String,
    /// Category/Room
    pub category: Option<String>,
    /// Voice memo file path
    pub audio_path: Option<String>,
    /// Voice memo URL
    pub audio_url: Option<String>,
}

impl DocumentResponse {
    /// Create response from document entity, generating a relative file URL
    pub fn from_document(doc: Document) -> Self {
        let file_url = format!("/files/{}", doc.file_path);
        let audio_url = doc
            .audio_path
            .as_ref()
            .map(|path| format!("/files/{}", path));
        Self {
            id: doc.id,
            project_id: doc.project_id,
            file_path: doc.file_path,
            file_type: doc.file_type,
            original_name: doc.original_name,
            uploaded_at: doc.uploaded_at,
            file_url,
            notes: doc.notes,
            status: doc.status,
            category: doc.category,
            audio_path: doc.audio_path,
            audio_url,
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

// =============================================================================
// Forum Entities
// =============================================================================

/// Forum message entity
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ForumMessage {
    pub id: String,
    pub project_id: Option<String>,
    pub parent_id: Option<String>,
    pub message_type: String,
    pub content: Option<String>,
    pub document_id: Option<String>,
    pub audio_path: Option<String>,
    pub author_name: String,
    pub created_at: String,
}

/// Task item entity (checklist item within a TASK_LIST message)
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct TaskItem {
    pub id: String,
    pub message_id: String,
    pub text: String,
    pub completed: bool,
    pub completed_by: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

// =============================================================================
// Forum Request DTOs
// =============================================================================

/// Request for creating a text or task_list forum message
#[derive(Debug, Deserialize)]
pub struct CreateForumMessageRequest {
    pub message_type: String,
    pub content: Option<String>,
    pub author_name: Option<String>,
    /// For TASK_LIST type: list of task item texts
    pub items: Option<Vec<String>>,
}

/// Request for replying to a forum message
#[derive(Debug, Deserialize)]
pub struct CreateReplyRequest {
    pub content: String,
    pub author_name: Option<String>,
}

/// Request for toggling a task item
#[derive(Debug, Deserialize)]
pub struct ToggleTaskItemRequest {
    pub completed_by: Option<String>,
}

/// Request for updating a forum message
#[derive(Debug, Deserialize)]
pub struct UpdateForumMessageRequest {
    /// Message text filtering
    pub content_filter: Option<String>,
}

/// Request to update a user's profile photo
#[derive(Debug, Deserialize)]
pub struct UpdateProfilePhotoRequest {
    pub photo_url: String,
}

// =============================================================================
// Forum Response DTOs
// =============================================================================

/// Response for a forum message
#[derive(Debug, Serialize)]
pub struct ForumMessageResponse {
    pub id: String,
    pub project_id: Option<String>,
    pub parent_id: Option<String>,
    pub message_type: String,
    pub content: Option<String>,
    pub document_id: Option<String>,
    pub audio_path: Option<String>,
    pub audio_url: Option<String>,
    pub author_name: String,
    pub created_at: String,
    /// Number of replies (comments) this message has
    pub reply_count: i32,
    /// For PHOTO messages, include the document info
    pub document: Option<DocumentResponse>,
    /// For TASK_LIST messages, include the items
    pub items: Option<Vec<TaskItemResponse>>,
}

/// Response for a task item
#[derive(Debug, Clone, Serialize)]
pub struct TaskItemResponse {
    pub id: String,
    pub text: String,
    pub completed: bool,
    pub completed_by: Option<String>,
    pub completed_at: Option<String>,
}

impl From<TaskItem> for TaskItemResponse {
    fn from(item: TaskItem) -> Self {
        Self {
            id: item.id,
            text: item.text,
            completed: item.completed,
            completed_by: item.completed_by,
            completed_at: item.completed_at,
        }
    }
}
