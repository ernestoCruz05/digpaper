//! Services module
//!
//! This module exposes the service layer which contains all business logic.
//! Services abstract away database operations and provide a clean API
//! for the HTTP handlers to use.

pub mod document_service;
pub mod project_service;

pub use document_service::DocumentService;
pub use project_service::ProjectService;
