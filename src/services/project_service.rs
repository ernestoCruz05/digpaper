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
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `name` - Name for the new project
    ///
    /// # Returns
    /// The newly created project
    pub async fn create(pool: &DbPool, name: String) -> AppResult<Project> {
        // Validate input
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::BadRequest("Project name cannot be empty".into()));
        }

        // Generate a new UUID for the project
        let id = Uuid::new_v4().to_string();

        // Insert the project with ACTIVE status
        sqlx::query(
            r#"
            INSERT INTO projects (id, name, status, created_at)
            VALUES (?, ?, 'ACTIVE', datetime('now'))
            "#,
        )
        .bind(&id)
        .bind(&name)
        .execute(pool)
        .await?;

        // Fetch and return the created project
        Self::get_by_id(pool, &id).await
    }

    /// Get a project by ID
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Project UUID
    ///
    /// # Returns
    /// The project if found
    pub async fn get_by_id(pool: &DbPool, id: &str) -> AppResult<Project> {
        sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Project with id '{}' not found", id)))
    }

    /// List all projects, optionally filtered by status
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `status_filter` - Optional status filter (ACTIVE/ARCHIVED)
    ///
    /// # Returns
    /// List of projects matching the filter
    pub async fn list(pool: &DbPool, status_filter: Option<&str>) -> AppResult<Vec<Project>> {
        let projects = match status_filter {
            Some(status) => {
                // Normalize status to uppercase for case-insensitive matching
                let status = status.to_uppercase();
                
                // Validate status value
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
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `id` - Project UUID
    /// * `status` - New status (ACTIVE or ARCHIVED)
    ///
    /// # Returns
    /// The updated project
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

    /// Get the number of documents for a project
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `project_id` - Project UUID
    ///
    /// # Returns
    /// Count of documents in the project
    pub async fn get_document_count(pool: &DbPool, project_id: &str) -> AppResult<i32> {
        let count: (i32,) = sqlx::query_as(
            "SELECT COUNT(*) as count FROM documents WHERE project_id = ?",
        )
        .bind(project_id)
        .fetch_one(pool)
        .await?;

        Ok(count.0)
    }
}
