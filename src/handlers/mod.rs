//! Handlers module
//!
//! This module exposes all HTTP handlers for the API endpoints.
//! Handlers are organized by domain (projects, documents, email).

pub mod document_handlers;
pub mod email_handlers;
pub mod forum_handlers;
pub mod project_handlers;
pub mod push_handlers;
pub mod user_handlers;

pub use document_handlers::*;
pub use email_handlers::*;
pub use forum_handlers::*;
pub use project_handlers::*;
pub use push_handlers::*;
pub use user_handlers::*;
