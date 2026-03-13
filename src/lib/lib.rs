//! # Axum API Template
//!
//! An opinionated API template using [Axum](https://docs.rs/axum).
//!
//! ## Features
//!
//! - Configuration management via YAML files and environment variables
//! - Structured logging with tracing and Bunyan formatting
//! - Request ID propagation
//! - Graceful shutdown support
//! - Error handling with proper HTTP status codes
//!
//! ## Example
//!
//! ```rust,no_run
//! use axum_api_template_lib::configuration::get_configuration;
//! use axum_api_template_lib::startup::Application;
//! use axum_api_template_lib::telemetry::{get_subscriber, init_subscriber};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let subscriber = get_subscriber(
//!         "axum-api-template-server".into(),
//!         "info".into(),
//!         std::io::stdout,
//!     );
//!     init_subscriber(subscriber);
//!
//!     let configuration = get_configuration()?;
//!     let application = Application::build(configuration.clone()).await?;
//!     application.run_until_stopped().await?;
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

// module declarations
pub mod configuration;
pub mod errors;
pub mod prelude;
pub mod routes;
pub mod startup;
pub mod telemetry;

// re-exports
pub use configuration::*;
pub use errors::*;
pub use startup::*;
pub use telemetry::*;
