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

mod auth;
mod db;
mod error;
mod handlers;
mod models;
mod services;

use axum::{
    extract::DefaultBodyLimit,
    http::header,
    middleware,
    routing::{delete, get, patch, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::handlers::{
    assign_document, create_project, delete_document, get_project, list_inbox,
    list_project_documents, list_projects, update_project_status, upload_document,
};
use crate::services::document_service::UPLOADS_DIR;

/// Default database URL for SQLite (used if DATABASE_URL env var not set)
/// The `?mode=rwc` flag creates the database if it doesn't exist
const DEFAULT_DATABASE_URL: &str = "sqlite:./digpaper.db?mode=rwc";

/// Server bind address
const SERVER_ADDR: &str = "0.0.0.0:3000";

/// Directory where the Flutter web app is served from
const WEB_DIR: &str = "./web";

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

    // Get database URL from environment or use default
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
    
    // Ensure SQLite URL has mode=rwc for auto-creation
    let database_url = if database_url.starts_with("sqlite:") && !database_url.contains("mode=") {
        format!("{}?mode=rwc", database_url)
    } else {
        database_url
    };

    tracing::info!("Using database: {}", database_url);

    // Initialize database connection pool and run migrations
    let pool = db::init_db(&database_url).await;

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
    // API routes are prefixed with /api for clean separation from web app
    let api_routes = Router::new()
        // Project endpoints
        .route("/projects", post(create_project).get(list_projects))
        .route("/projects/:id", get(get_project))
        .route("/projects/:id/documents", get(list_project_documents))
        .route("/projects/:id/status", patch(update_project_status))
        // Document endpoints - with increased body limit for uploads (100MB)
        .route("/upload", post(upload_document))
        .route("/documents/inbox", get(list_inbox))
        .route("/documents/:id/assign", patch(assign_document))
        .route("/documents/:id", delete(delete_document))
        // Add shared state (database pool)
        .with_state(pool)
        // Allow larger uploads (100MB)
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024))
        // API key authentication
        .layer(middleware::from_fn(auth::api_key_auth));

    // Serve the Flutter web app with no-cache headers
    // The fallback serves index.html for SPA client-side routing
    let web_app = ServeDir::new(WEB_DIR)
        .not_found_service(ServeFile::new(format!("{}/index.html", WEB_DIR)));

    let app = Router::new()
        // API routes under /api prefix
        .nest("/api", api_routes)
        // Static file serving for uploaded documents
        .nest_service("/files", ServeDir::new(UPLOADS_DIR))
        // Serve web app for all other routes (must be last)
        .fallback_service(web_app)
        // Add cache-control header to prevent aggressive caching
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CACHE_CONTROL,
            header::HeaderValue::from_static("no-cache, no-store, must-revalidate"),
        ))
        // Add middleware layers
        .layer(TraceLayer::new_for_http()) // Request/response logging
        .layer(cors); // CORS headers

    // Parse server address
    let addr: SocketAddr = SERVER_ADDR
        .parse()
        .expect("Invalid server address configuration");

    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("Web app served from: {}", WEB_DIR);
    tracing::info!("API Documentation:");
    tracing::info!("  POST   /api/projects              - Create project");
    tracing::info!("  GET    /api/projects              - List projects (?status=active)");
    tracing::info!("  GET    /api/projects/:id          - Get project");
    tracing::info!("  GET    /api/projects/:id/documents - List project documents");
    tracing::info!("  POST   /api/upload                - Upload file (multipart)");
    tracing::info!("  GET    /api/documents/inbox       - List inbox documents");
    tracing::info!("  PATCH  /api/documents/:id/assign  - Assign document to project");
    tracing::info!("  GET    /files/:filename           - Serve uploaded files");

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
