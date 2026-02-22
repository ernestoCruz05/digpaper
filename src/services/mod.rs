//! Services module
//!
//! This module exposes the service layer which contains all business logic.
//! Services abstract away database operations and provide a clean API
//! for the HTTP handlers to use.

pub mod document_service;
pub mod email_service;
pub mod forum_service;
pub mod project_service;
pub mod push_service;
pub mod user_service;

pub use document_service::DocumentService;
pub use email_service::EmailService;
pub use forum_service::ForumService;
pub use project_service::ProjectService;
pub use push_service::PushService;
pub use user_service::UserService;
