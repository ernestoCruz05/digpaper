//! Project service module
//!
//! This module contains business logic for project operations.
//! It abstracts database operations and provides a clean API for handlers.
//!
//! # Architecture Decision
//! The service layer separates business logic from HTTP handling,
//! making the code more testable and maintainable.

use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::models::{Project, ProjectStatus};
use uuid::Uuid;

/// Project service handling all project-related business logic
pub struct ProjectService;

impl ProjectService {
    /// Create a new project
    pub async fn create(
        pool: &DbPool,
        name: String,
        address: Option<String>,
        client_phone: Option<String>,
    ) -> AppResult<Project> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::BadRequest("Project name cannot be empty".into()));
        }

        let id = Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT INTO projects (id, name, status, address, client_phone, created_at)
            VALUES (?, ?, 'ACTIVE', ?, ?, datetime('now'))
            "#,
        )
        .bind(&id)
        .bind(&name)
        .bind(&address)
        .bind(&client_phone)
        .execute(pool)
        .await?;

        Self::get_by_id(pool, &id).await
    }

    /// Get a project by ID
    pub async fn get_by_id(pool: &DbPool, id: &str) -> AppResult<Project> {
        sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Project with id '{}' not found", id)))
    }

    /// List all projects, optionally filtered by status
    pub async fn list(pool: &DbPool, status_filter: Option<&str>) -> AppResult<Vec<Project>> {
        let projects = match status_filter {
            Some(status) => {
                let status = status.to_uppercase();
                if status != "ACTIVE" && status != "ARCHIVED" {
                    return Err(AppError::BadRequest(format!(
                        "Invalid status filter '{}'. Use 'active' or 'archived'",
                        status
                    )));
                }
                sqlx::query_as::<_, Project>(
                    "SELECT * FROM projects WHERE status = ? ORDER BY created_at DESC",
                )
                .bind(status)
                .fetch_all(pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, Project>("SELECT * FROM projects ORDER BY created_at DESC")
                    .fetch_all(pool)
                    .await?
            }
        };
        Ok(projects)
    }

    /// Update project status
    pub async fn update_status(
        pool: &DbPool,
        id: &str,
        status: ProjectStatus,
    ) -> AppResult<Project> {
        let result = sqlx::query("UPDATE projects SET status = ? WHERE id = ?")
            .bind(status.as_str())
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Project with id '{}' not found",
                id
            )));
        }

        Self::get_by_id(pool, id).await
    }

    /// Update project address and client phone
    pub async fn update_details(
        pool: &DbPool,
        id: &str,
        address: Option<String>,
        client_phone: Option<String>,
    ) -> AppResult<Project> {
        let result = sqlx::query("UPDATE projects SET address = ?, client_phone = ? WHERE id = ?")
            .bind(&address)
            .bind(&client_phone)
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "Project with id '{}' not found",
                id
            )));
        }

        Self::get_by_id(pool, id).await
    }

    /// Get the number of documents for a project
    pub async fn get_document_count(pool: &DbPool, project_id: &str) -> AppResult<i32> {
        let count: (i32,) =
            sqlx::query_as("SELECT COUNT(*) as count FROM documents WHERE project_id = ?")
                .bind(project_id)
                .fetch_one(pool)
                .await?;

        Ok(count.0)
    }
}
