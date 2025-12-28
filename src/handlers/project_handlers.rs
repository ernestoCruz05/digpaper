//! Project handlers module
//!
//! HTTP handlers for project-related endpoints.
//! Handlers are thin wrappers that delegate to the service layer.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

use crate::db::DbPool;
use crate::error::AppResult;
use crate::models::{CreateProjectRequest, ListProjectsQuery, ProjectResponse, ProjectStatus, UpdateProjectStatusRequest};
use crate::services::ProjectService;

/// POST /projects - Create a new project
///
/// Creates a new project (Obra) with ACTIVE status.
/// The project name should be descriptive (e.g., "Obra Porto Seg Social").
///
/// # Request Body
/// ```json
/// { "name": "Obra Porto Seg Social" }
/// ```
///
/// # Response
/// Returns the created project with 201 Created status
pub async fn create_project(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateProjectRequest>,
) -> AppResult<(StatusCode, Json<ProjectResponse>)> {
    tracing::info!("Creating new project: {}", payload.name);

    let project = ProjectService::create(&pool, payload.name).await?;

    Ok((StatusCode::CREATED, Json(project.into())))
}

/// GET /projects - List all projects
///
/// Returns all projects, optionally filtered by status.
/// Use ?status=active for the mobile app dropdown (only ongoing works).
///
/// # Query Parameters
/// - `status` (optional): Filter by "active" or "archived"
///
/// # Response
/// Returns an array of projects
pub async fn list_projects(
    State(pool): State<DbPool>,
    Query(params): Query<ListProjectsQuery>,
) -> AppResult<Json<Vec<ProjectResponse>>> {
    tracing::debug!("Listing projects with filter: {:?}", params.status);

    let projects = ProjectService::list(&pool, params.status.as_deref()).await?;

    // Get document counts for each project
    let mut response = Vec::new();
    for p in projects {
        let count = ProjectService::get_document_count(&pool, &p.id).await.unwrap_or(0);
        response.push(ProjectResponse::from_project(p, count));
    }

    Ok(Json(response))
}

/// GET /projects/:id - Get a specific project
///
/// Returns details of a single project by ID.
///
/// # Path Parameters
/// - `id`: Project UUID
///
/// # Response
/// Returns the project or 404 if not found
pub async fn get_project(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
) -> AppResult<Json<ProjectResponse>> {
    tracing::debug!("Getting project: {}", id);

    let project = ProjectService::get_by_id(&pool, &id).await?;

    Ok(Json(project.into()))
}

/// PATCH /projects/:id/status - Update project status
///
/// Change a project's status between ACTIVE and ARCHIVED.
/// Use this to mark a project as complete/done.
///
/// # Path Parameters
/// - `id`: Project UUID
///
/// # Request Body
/// ```json
/// { "status": "ARCHIVED" }
/// ```
/// or
/// ```json
/// { "status": "ACTIVE" }
/// ```
///
/// # Response
/// Returns the updated project
pub async fn update_project_status(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateProjectStatusRequest>,
) -> AppResult<Json<ProjectResponse>> {
    tracing::info!("Updating project {} status to {:?}", id, payload.status);

    let project = ProjectService::update_status(&pool, &id, payload.status).await?;

    Ok(Json(project.into()))
}
