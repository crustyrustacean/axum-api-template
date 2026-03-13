//! Telemetry and logging configuration.
//!
//! This module provides utilities for setting up structured logging
//! using the `tracing` crate with Bunyan formatting.
//!
//! # Features
//!
//! - Structured JSON logging with Bunyan format
//! - Request ID generation using UUID v4
//! - Configurable log levels via environment variables
//!
//! # Example
//!
//! ```rust,no_run
//! use axum_api_template_lib::telemetry::{get_subscriber, init_subscriber};
//!
//! // Set up logging to stdout with "info" level
//! let subscriber = get_subscriber(
//!     "my-app".into(),
//!     "info".into(),
//!     std::io::stdout,
//! );
//! init_subscriber(subscriber);
//! ```

// dependencies
use axum::http::Request;
use tower_http::request_id::{MakeRequestId, RequestId};
use tracing::Subscriber;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};
use uuid::Uuid;

/// Generates a new UUID v4 for each request.
///
/// This struct implements [`MakeRequestId`] to generate unique
/// identifiers for incoming HTTP requests.
#[derive(Clone, Copy, Debug)]
pub struct MakeRequestUuid;

impl MakeRequestId for MakeRequestUuid {
    fn make_request_id<B>(&mut self, _: &Request<B>) -> Option<RequestId> {
        let request_id = Uuid::new_v4().to_string();
        Some(RequestId::new(request_id.parse().unwrap()))
    }
}

/// Creates a configured tracing subscriber.
///
/// This function sets up a subscriber with:
/// - Environment-based log level filtering (via `RUST_LOG` environment variable)
/// - JSON storage layer for structured logging
/// - Bunyan formatting for human-readable output
///
/// # Arguments
///
/// * `name` - The name of the application (appears in logs)
/// * `env_filter` - The default log level if `RUST_LOG` is not set
/// * `sink` - The writer to output logs to (e.g., `std::io::stdout`)
///
/// # Example
///
/// ```rust,no_run
/// use axum_api_template_lib::telemetry::get_subscriber;
///
/// let subscriber = get_subscriber(
///     "my-app".into(),
///     "info".into(),
///     std::io::stdout,
/// );
/// ```
#[must_use]
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Sync + Send
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name, sink);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Initializes the global tracing subscriber.
///
/// This function must be called once at application startup.
/// It sets up the global subscriber and redirects `log` events to `tracing`.
///
/// # Panics
///
/// Panics if:
/// - The subscriber has already been initialized
/// - The `log` to `tracing` bridge fails to initialize
///
/// # Example
///
/// ```rust,no_run
/// use axum_api_template_lib::telemetry::{get_subscriber, init_subscriber};
///
/// let subscriber = get_subscriber(
///     "my-app".into(),
///     "info".into(),
///     std::io::stdout,
/// );
/// init_subscriber(subscriber);
/// ```
pub fn init_subscriber(subscriber: impl Subscriber + Sync + Send) {
    // Redirect logs to subscriber
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}
