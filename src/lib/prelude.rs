//! Common imports for the axum-api-template library.
//!
//! This module re-exports the most commonly used types and traits,
//! allowing users to import everything with a single `use` statement:
//!
//! ```
//! use axum_api_template_lib::prelude::*;
//! ```

pub use crate::configuration::{ApplicationSettings, Environment, Settings};
pub use crate::errors::ApiError;
pub use crate::startup::Application;
