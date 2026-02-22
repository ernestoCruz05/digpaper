//! Document handlers module
//!
//! HTTP handlers for document-related endpoints including file uploads,
//! inbox management, and project assignment.

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    Json,
};

use crate::db::DbPool;
use crate::error::AppResult;
use crate::models::{
    AssignDocumentRequest, BatchAssignRequest, DocumentResponse, UpdateDocumentCategoryRequest,
    UpdateDocumentNotesRequest, UpdateDocumentStatusRequest, UploadResponse,
};
use crate::services::DocumentService;

/// POST /upload - Upload a new document
///
/// Handles multipart/form-data file uploads. Files are streamed directly
/// to disk to prevent memory exhaustion with large files.
///
/// The uploaded document is placed in the Inbox (project_id = NULL)
/// until manually assigned to a project.
///
/// # Request
/// Multipart form with a file field
///
/// # Response
/// Returns the created document record with 201 Created status
pub async fn upload_document(
    State(pool): State<DbPool>,
    multipart: Multipart,
) -> AppResult<(StatusCode, Json<UploadResponse>)> {
    tracing::info!("Processing file upload");

    // Pass the entire multipart stream to the service to extract both the file and optional audio
    let doc = DocumentService::upload(&pool, multipart).await?;

    let response = UploadResponse {
        id: doc.id,
        file_path: doc.file_path.clone(),
        file_type: doc.file_type,
        original_name: doc.original_name,
        file_url: format!("/files/{}", doc.file_path),
    };

    tracing::info!("File uploaded successfully: {}", response.file_path);
    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /documents/inbox - List all documents in the Inbox
///
/// Returns all documents that are not assigned to any project.
/// These are waiting to be organized by office staff.
///
/// # Response
/// Returns an array of documents
pub async fn list_inbox(State(pool): State<DbPool>) -> AppResult<Json<Vec<DocumentResponse>>> {
    tracing::debug!("Listing inbox documents");

    let docs = DocumentService::list_inbox(&pool).await?;

    let response: Vec<DocumentResponse> = docs
        .into_iter()
        .map(|d| DocumentResponse::from_document(d))
        .collect();

    Ok(Json(response))
}

/// PATCH /documents/:id/assign - Assign a document to a project
///
/// Moves a document from the Inbox to a specific project, or moves
/// a document back to the Inbox by setting project_id to null.
///
/// This is the core workflow for organizing uploaded documents:
/// 1. Workshop employees upload photos (goes to Inbox)
/// 2. Office staff reviews and assigns to appropriate projects
///
/// # Path Parameters
/// - `id`: Document UUID
///
/// # Request Body
/// ```json
/// { "project_id": "uuid-of-target-project" }
/// ```
/// Or to move back to inbox:
/// ```json
/// { "project_id": null }
/// ```
///
/// # Response
/// Returns the updated document
pub async fn assign_document(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    Json(payload): Json<AssignDocumentRequest>,
) -> AppResult<Json<DocumentResponse>> {
    tracing::info!(
        "Assigning document {} to project {:?}",
        id,
        payload.project_id
    );

    let doc = DocumentService::assign_to_project(
        &pool,
        &id,
        payload.project_id.as_deref(),
        payload.category.as_deref(),
    )
    .await?;

    Ok(Json(DocumentResponse::from_document(doc)))
}

/// GET /projects/:id/documents - List all documents for a project
///
/// Returns all documents assigned to a specific project.
///
/// # Path Parameters
/// - `id`: Project UUID
///
/// # Response
/// Returns an array of documents
pub async fn list_project_documents(
    State(pool): State<DbPool>,
    Path(project_id): Path<String>,
) -> AppResult<Json<Vec<DocumentResponse>>> {
    tracing::debug!("Listing documents for project: {}", project_id);

    let docs = DocumentService::list_by_project(&pool, &project_id).await?;

    let response: Vec<DocumentResponse> = docs
        .into_iter()
        .map(|d| DocumentResponse::from_document(d))
        .collect();

    Ok(Json(response))
}

/// GET /documents/:id - Get a specific document by ID
///
/// Used for deep linking and direct sharing.
pub async fn get_document(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> AppResult<Json<DocumentResponse>> {
    tracing::debug!("Fetching document by ID: {}", id);

    let doc = DocumentService::get_by_id(&pool, &id).await?;
    Ok(Json(DocumentResponse::from_document(doc)))
}

/// DELETE /documents/:id - Delete a document
///
/// Permanently deletes a document and its associated file from disk.
///
/// # Path Parameters
/// - `id`: Document UUID
///
/// # Response
/// Returns 204 No Content on success
pub async fn delete_document(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    tracing::info!("Deleting document: {}", id);

    DocumentService::delete(&pool, &id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// PATCH /documents/:id/notes - Update document notes
///
/// Adds or updates notes for a document.
///
/// # Path Parameters
/// - `id`: Document UUID
///
/// # Request Body
/// ```json
/// { "notes": "Some annotation text" }
/// ```
///
/// # Response
/// Returns the updated document
pub async fn update_document_notes(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateDocumentNotesRequest>,
) -> AppResult<Json<DocumentResponse>> {
    tracing::info!("Updating notes for document: {}", id);

    let doc = DocumentService::update_notes(&pool, &id, payload.notes.as_deref()).await?;

    Ok(Json(DocumentResponse::from_document(doc)))
}

/// PATCH /documents/batch-assign - Batch assign documents to a project
///
/// Moves multiple documents from the Inbox to a specific project.
///
/// # Request Body
/// ```json
/// {
///     "document_ids": ["uuid-1", "uuid-2"],
///     "project_id": "uuid-of-target-project"
/// }
/// ```
///
/// # Response
/// Returns an array of updated documents
pub async fn batch_assign_documents(
    State(pool): State<DbPool>,
    Json(payload): Json<BatchAssignRequest>,
) -> AppResult<Json<Vec<DocumentResponse>>> {
    tracing::info!(
        "Batch assigning {} documents to project {:?}",
        payload.document_ids.len(),
        payload.project_id
    );

    let docs = DocumentService::batch_assign_to_project(
        &pool,
        &payload.document_ids,
        payload.project_id.as_deref(),
    )
    .await?;

    let response: Vec<DocumentResponse> = docs
        .into_iter()
        .map(|d| DocumentResponse::from_document(d))
        .collect();

    Ok(Json(response))
}

/// PATCH /documents/:id/status - Update document status
///
/// Updates the status of a document (e.g. from Default to Doubt)
///
/// # Path Parameters
/// - `id`: Document UUID
///
/// # Request Body
/// ```json
/// { "status": "DOUBT" }
/// ```
pub async fn update_document_status(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateDocumentStatusRequest>,
) -> AppResult<Json<DocumentResponse>> {
    tracing::info!(
        "Updating status for document: {} to {:?}",
        id,
        payload.status
    );

    let doc = DocumentService::update_status(&pool, &id, payload.status.as_str()).await?;

    Ok(Json(DocumentResponse::from_document(doc)))
}

/// PATCH /documents/:id/category - Update document category
///
/// Assigns a room/category to a document.
///
/// # Path Parameters
/// - `id`: Document UUID
///
/// # Request Body
/// ```json
/// { "category": "Kitchen" }
/// ```
pub async fn update_document_category(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateDocumentCategoryRequest>,
) -> AppResult<Json<DocumentResponse>> {
    tracing::info!("Updating category for document: {}", id);

    let doc = DocumentService::update_category(&pool, &id, payload.category.as_deref()).await?;

    Ok(Json(DocumentResponse::from_document(doc)))
}
