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
    assign_document, batch_assign_documents, create_email_filter, create_email_rule,
    create_forum_message, create_project, create_reply, create_voice_message, delete_document,
    delete_email_filter, delete_email_rule, email_webhook_status, get_document, get_project,
    get_vapid_key, list_email_filters, list_email_rules, list_forum_messages, list_inbox,
    list_project_documents, list_projects, list_replies, push_subscribe, push_unsubscribe,
    receive_inbound_email, toggle_task_item, update_document_category, update_document_notes,
    update_document_status, update_project_details, update_project_status, upload_document,
    user_handlers,
};
use crate::services::document_service::UPLOADS_DIR;
use crate::services::email_service::EmailService;
use crate::services::PushService;

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
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());

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

    // Initialize VAPID keys for push notifications
    if let Err(e) = PushService::init_vapid(&pool).await {
        tracing::warn!(
            "Failed to initialize VAPID keys: {}. Push notifications disabled.",
            e
        );
    }

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
        .route("/projects/:id/details", patch(update_project_details))
        // Forum endpoints
        .route(
            "/projects/:id/forum",
            post(create_forum_message).get(list_forum_messages),
        )
        .route("/projects/:id/forum/voice", post(create_voice_message))
        .route(
            "/forum/:msg_id/replies",
            get(list_replies).post(create_reply),
        )
        .route("/tasks/:item_id/toggle", patch(toggle_task_item))
        // Document endpoints - with increased body limit for uploads (100MB)
        .route("/upload", post(upload_document))
        .route("/documents/inbox", get(list_inbox))
        .route("/documents/batch-assign", patch(batch_assign_documents))
        .route("/documents/:id/assign", patch(assign_document))
        .route("/documents/:id/notes", patch(update_document_notes))
        .route("/documents/:id/status", patch(update_document_status))
        .route("/documents/:id/category", patch(update_document_category))
        .route("/documents/:id", get(get_document).delete(delete_document))
        // Push notification endpoints
        .route("/push/vapid-key", get(get_vapid_key))
        .route("/push/subscribe", post(push_subscribe))
        .route("/push/unsubscribe", post(push_unsubscribe))
        // Add shared state (database pool)
        // User profile endpoints
        .nest("/profiles", user_handlers::router())
        // Add shared state (database pool)
        .with_state(pool.clone())
        // Allow larger uploads (100MB)
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024))
        // API key authentication
        .layer(middleware::from_fn(auth::api_key_auth));

    // Email webhook routes - no API key auth (uses webhook signing for security)
    let email_routes = Router::new()
        .route("/status", get(email_webhook_status))
        .route("/inbound", post(receive_inbound_email))
        // Email rules and filters (API key protected)
        .route("/rules", get(list_email_rules).post(create_email_rule))
        .route("/rules/:id", delete(delete_email_rule))
        .route(
            "/filters",
            get(list_email_filters).post(create_email_filter),
        )
        .route("/filters/:id", delete(delete_email_filter))
        .with_state(pool)
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024));

    // Serve the Flutter web app with no-cache headers
    // The fallback serves index.html for SPA client-side routing
    let web_app =
        ServeDir::new(WEB_DIR).not_found_service(ServeFile::new(format!("{}/index.html", WEB_DIR)));

    let app = Router::new()
        // API routes under /api prefix
        .nest("/api", api_routes)
        // Email webhook routes (no auth required)
        .nest("/api/email", email_routes)
        // Static file serving for uploaded documents (cached 24h — images don't change)
        .nest_service(
            "/files",
            ServeDir::new(UPLOADS_DIR)
                .precompressed_gzip()
                .precompressed_br()
                .precompressed_deflate(),
        )
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
    tracing::info!("  POST   /api/email/inbound         - Email webhook endpoint");
    tracing::info!("  GET    /api/email/rules           - List email routing rules");
    tracing::info!("  POST   /api/email/rules           - Create email routing rule");
    tracing::info!("  DELETE /api/email/rules/:id       - Delete email routing rule");
    tracing::info!("  GET    /api/email/filters         - List attachment filters");
    tracing::info!("  POST   /api/email/filters         - Create attachment filter");
    tracing::info!("  DELETE /api/email/filters/:id     - Delete attachment filter");

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
