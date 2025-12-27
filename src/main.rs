//! DigPaper - Document Management System Backend
//!
//! A high-performance REST API for managing documents and projects
//! in a carpentry business. Designed to be self-hosted and portable.
//!
//! # Architecture Overview
//!
//! The application follows a layered architecture:
//! - **Handlers**: HTTP request/response handling (thin layer)
//! - **Services**: Business logic and orchestration
//! - **Models**: Domain entities and DTOs
//! - **DB**: Database connection and migrations
//!
//! # Key Features
//!
//! - **Projects (Obras)**: Organize work into logical units
//! - **Inbox Workflow**: Upload first, organize later
//! - **Streaming Uploads**: Memory-efficient file handling
//! - **Static File Serving**: Direct access to uploaded files
//!
//! # API Endpoints
//!
//! - `POST /projects` - Create a new project
//! - `GET /projects` - List projects (optional ?status=active filter)
//! - `GET /projects/:id` - Get project details
//! - `GET /projects/:id/documents` - List project documents
//! - `POST /upload` - Upload a file (goes to Inbox)
//! - `GET /documents/inbox` - List unassigned documents
//! - `PATCH /documents/:id/assign` - Assign document to project
//! - `GET /files/:filename` - Serve uploaded files

mod db;
mod error;
mod handlers;
mod models;
mod services;

use axum::{
    routing::{get, patch, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::handlers::{
    assign_document, create_project, get_project, list_inbox, list_project_documents,
    list_projects, update_project_status, upload_document,
};
use crate::services::document_service::UPLOADS_DIR;

/// Database URL for SQLite
/// The `?mode=rwc` flag creates the database if it doesn't exist
const DATABASE_URL: &str = "sqlite:./digpaper.db?mode=rwc";

/// Server bind address
const SERVER_ADDR: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber for structured logging
    // This integrates with Tower's TraceLayer for request logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "digpaper=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting DigPaper Document Management System");

    // Initialize database connection pool and run migrations
    let pool = db::init_db(DATABASE_URL).await;

    // Ensure uploads directory exists
    tokio::fs::create_dir_all(UPLOADS_DIR)
        .await
        .expect("Failed to create uploads directory");

    // Configure CORS for cross-origin requests
    // This is necessary for the Tauri desktop app and mobile app
    //
    // Security Note: In production, replace `Any` with specific origins:
    // .allow_origin(["http://localhost:1420".parse().unwrap(), ...])
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build the application router with all endpoints
    let app = Router::new()
        // Project endpoints
        .route("/projects", post(create_project).get(list_projects))
        .route("/projects/:id", get(get_project))
        .route("/projects/:id/documents", get(list_project_documents))
        .route("/projects/:id/status", patch(update_project_status))
        // Document endpoints
        .route("/upload", post(upload_document))
        .route("/documents/inbox", get(list_inbox))
        .route("/documents/:id/assign", patch(assign_document))
        // Static file serving for uploaded documents
        // This allows apps to directly fetch images/PDFs via GET /files/:filename
        .nest_service("/files", ServeDir::new(UPLOADS_DIR))
        // Add shared state (database pool)
        .with_state(pool)
        // Add middleware layers
        .layer(TraceLayer::new_for_http()) // Request/response logging
        .layer(cors); // CORS headers

    // Parse server address
    let addr: SocketAddr = SERVER_ADDR
        .parse()
        .expect("Invalid server address configuration");

    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("API Documentation:");
    tracing::info!("  POST   /projects              - Create project");
    tracing::info!("  GET    /projects              - List projects (?status=active)");
    tracing::info!("  GET    /projects/:id          - Get project");
    tracing::info!("  GET    /projects/:id/documents - List project documents");
    tracing::info!("  POST   /upload                - Upload file (multipart)");
    tracing::info!("  GET    /documents/inbox       - List inbox documents");
    tracing::info!("  PATCH  /documents/:id/assign  - Assign document to project");
    tracing::info!("  GET    /files/:filename       - Serve uploaded files");

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
