//! Handlers module
//!
//! This module exposes all HTTP handlers for the API endpoints.
//! Handlers are organized by domain (projects, documents).

pub mod document_handlers;
pub mod project_handlers;

pub use document_handlers::*;
pub use project_handlers::*;
