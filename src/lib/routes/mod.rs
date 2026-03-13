//! API route handlers.
//!
//! This module contains all HTTP route handlers for the application.
//! Each handler is defined in its own submodule for organization.

// module declarations
pub mod health_check;

// re-exports
pub use health_check::*;
