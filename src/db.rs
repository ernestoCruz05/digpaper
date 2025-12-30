//! Database module for DigPaper
//!
//! This module handles SQLite database initialization and connection pooling.
//! We use SQLx for compile-time verified queries and async database operations.
//!
//! # Architecture Decision
//! SQLite was chosen for its portability (single file), zero-configuration setup,
//! and excellent performance for self-hosted scenarios. The connection pool
//! ensures efficient resource utilization under concurrent load.

use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::time::Duration;

/// Type alias for the SQLite connection pool
pub type DbPool = Pool<Sqlite>;

/// Initialize the database connection pool and run migrations
///
/// # Arguments
/// * `database_url` - SQLite database URL (e.g., "sqlite:./digpaper.db?mode=rwc")
///
/// # Returns
/// A configured connection pool ready for use
///
/// # Panics
/// Panics if the database connection cannot be established or migrations fail
pub async fn init_db(database_url: &str) -> DbPool {
    tracing::info!("Initializing database connection pool...");

    // Configure the connection pool with sensible defaults for a self-hosted scenario
    // - max_connections: 5 is sufficient for moderate concurrent access
    // - acquire_timeout: Prevents indefinite waiting for connections
    // - idle_timeout: Releases unused connections to save resources
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(60))
        .connect(database_url)
        .await
        .expect("Failed to create database pool");

    // Run schema migrations to ensure tables exist
    run_migrations(&pool).await;

    tracing::info!("Database initialized successfully");
    pool
}

/// Execute database schema migrations
///
/// Creates the required tables if they don't exist.
/// Uses IF NOT EXISTS to make migrations idempotent.
async fn run_migrations(pool: &DbPool) {
    tracing::info!("Running database migrations...");

    // Projects table: Represents a work order or "Obra"
    // - id: UUID primary key for global uniqueness
    // - name: Human-readable project name (e.g., "Obra Porto Seg Social")
    // - status: Workflow state, either "ACTIVE" or "ARCHIVED"
    // - created_at: Timestamp for chronological ordering
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK(status IN ('ACTIVE', 'ARCHIVED')),
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create projects table");

    // Documents table: Represents uploaded files (photos, PDFs, etc.)
    // - id: UUID primary key
    // - project_id: Foreign key to projects, NULL means document is in "Inbox"
    // - file_path: Relative path within the uploads directory
    // - file_type: MIME type category (image, pdf, etc.)
    // - original_name: Original filename for display purposes
    // - uploaded_at: Upload timestamp
    //
    // Design Note: project_id is nullable to support the "Inbox" workflow where
    // documents are uploaded first and assigned to projects later.
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS documents (
            id TEXT PRIMARY KEY NOT NULL,
            project_id TEXT,
            file_path TEXT NOT NULL,
            file_type TEXT NOT NULL,
            original_name TEXT NOT NULL,
            uploaded_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE SET NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create documents table");

    // Create index for efficient inbox queries (documents without project)
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_documents_inbox 
        ON documents(project_id) WHERE project_id IS NULL
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create inbox index");

    // Create index for efficient project document lookups
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_documents_project 
        ON documents(project_id) WHERE project_id IS NOT NULL
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create project documents index");

    // Add notes column to documents table if it doesn't exist
    // SQLite doesn't have IF NOT EXISTS for ALTER TABLE, so we check pragmatically
    // PRAGMA table_info returns: (cid INTEGER, name TEXT, type TEXT, notnull INTEGER, dflt_value TEXT, pk INTEGER)
    let columns: Vec<(i32, String, String, i32, Option<String>, i32)> = 
        sqlx::query_as("PRAGMA table_info(documents)")
            .fetch_all(pool)
            .await
            .expect("Failed to query table info");
    
    let has_notes = columns.iter().any(|(_, name, _, _, _, _)| name == "notes");
    if !has_notes {
        sqlx::query("ALTER TABLE documents ADD COLUMN notes TEXT")
            .execute(pool)
            .await
            .expect("Failed to add notes column");
        tracing::info!("Added notes column to documents table");
    }

    // Email routing rules table
    // Allows routing emails from specific senders to specific projects
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS email_rules (
            id TEXT PRIMARY KEY NOT NULL,
            sender_pattern TEXT NOT NULL,
            project_id TEXT,
            description TEXT,
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create email_rules table");

    // Email filter patterns table
    // Stores patterns for files to ignore (logos, signatures, etc.)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS email_filters (
            id TEXT PRIMARY KEY NOT NULL,
            pattern TEXT NOT NULL,
            filter_type TEXT NOT NULL CHECK(filter_type IN ('filename', 'extension', 'size_max')),
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create email_filters table");

    // Insert default filters if table is empty
    let filter_count: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM email_filters")
        .fetch_one(pool)
        .await
        .expect("Failed to count filters");
    
    if filter_count.0 == 0 {
        // Default filters for common spam attachments
        let default_filters = vec![
            ("logo", "filename"),
            ("signature", "filename"),
            ("banner", "filename"),
            ("icon", "filename"),
            ("footer", "filename"),
            ("header", "filename"),
            ("facebook", "filename"),
            ("linkedin", "filename"),
            ("instagram", "filename"),
            ("twitter", "filename"),
            ("5000", "size_max"),  // Skip files smaller than 5KB (likely logos)
        ];
        
        for (pattern, filter_type) in default_filters {
            let id = uuid::Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO email_filters (id, pattern, filter_type) VALUES (?, ?, ?)")
                .bind(&id)
                .bind(pattern)
                .bind(filter_type)
                .execute(pool)
                .await
                .expect("Failed to insert default filter");
        }
        tracing::info!("Added default email filters");
    }

    tracing::info!("Migrations completed successfully");
}
