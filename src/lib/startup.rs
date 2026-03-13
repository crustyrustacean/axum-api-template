//! Application startup and server configuration.
//!
//! This module contains the [`Application`] struct which manages the
//! HTTP server lifecycle, including graceful shutdown support.
//!
//! # Example
//!
//! ```rust,no_run
//! use axum_api_template_lib::configuration::get_configuration;
//! use axum_api_template_lib::startup::Application;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = get_configuration()?;
//!     let app = Application::build(config).await?;
//!     app.run_until_stopped().await?;
//!     Ok(())
//! }
//! ```

// dependencies
use crate::configuration::Settings;
use crate::routes::health_check;
use crate::telemetry::MakeRequestUuid;
use anyhow::{Context, Result};
use axum::{Router, http::HeaderName, routing::get};
use tokio::net::TcpListener;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    request_id::{PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

/// The header name used for request ID propagation.
const X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

/// Handles graceful shutdown signals.
///
/// This function listens for:
/// - `Ctrl+C` on all platforms
/// - `SIGTERM` on Unix systems
///
/// # Panics
///
/// Panics if the signal handlers cannot be installed.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

/// Represents the running application.
///
/// This struct wraps an Axum [`Router`] and manages the HTTP server lifecycle.
/// It provides methods for building the application and running it with
/// graceful shutdown support.
///
/// # Example
///
/// ```rust,no_run
/// use axum_api_template_lib::startup::Application;
/// use axum_api_template_lib::configuration::get_configuration;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let config = get_configuration()?;
///     let app = Application::build(config).await?;
///     
///     // Get the port the server bound to
///     println!("Server running on port {}", app.port());
///     
///     // Run until stopped (Ctrl+C or SIGTERM)
///     app.run_until_stopped().await?;
///     Ok(())
/// }
/// ```
pub struct Application {
    /// The port the server is bound to.
    port: u16,
    /// The TCP listener for incoming connections.
    listener: TcpListener,
    /// The Axum router handling requests.
    app: Router,
}

impl Application {
    /// Builds and configures the application.
    ///
    /// This method:
    /// 1. Binds to the configured address
    /// 2. Sets up the router with all routes and middleware
    /// 3. Configures tracing and request ID propagation
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The TCP listener cannot bind to the configured address
    /// - The router fails to build
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use axum_api_template_lib::startup::Application;
    /// use axum_api_template_lib::configuration::get_configuration;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let config = get_configuration()?;
    ///     let app = Application::build(config).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn build(config: Settings) -> Result<Self, anyhow::Error> {
        let address = format!("{}:{}", config.application.host, config.application.port);
        let listener = TcpListener::bind(address)
            .await
            .context("Unable to get a TCP listener...")?;
        let port = listener.local_addr()?.port();

        let app = build_router()
            .await
            .context("Failed to create the application.")?;

        Ok(Self {
            port,
            listener,
            app,
        })
    }

    /// Returns the port the server is bound to.
    ///
    /// This is useful when binding to port 0 (random port assignment),
    /// commonly used in tests.
    #[must_use = "The port information should be used"]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Runs the server until a shutdown signal is received.
    ///
    /// This method starts the HTTP server and blocks until:
    /// - `Ctrl+C` is pressed
    /// - `SIGTERM` is received (Unix only)
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to start or encounters
    /// a fatal error while running.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use axum_api_template_lib::startup::Application;
    /// use axum_api_template_lib::configuration::get_configuration;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let config = get_configuration()?;
    ///     let app = Application::build(config).await?;
    ///     app.run_until_stopped().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn run_until_stopped(self) -> Result<(), anyhow::Error> {
        axum::serve(self.listener, self.app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .context("Unable to start the app server...")?;
        Ok(())
    }
}

/// Builds the Axum router with all routes and middleware.
///
/// This function configures:
/// - HTTP request/response tracing
/// - Request ID generation and propagation
/// - All application routes
///
/// # Errors
///
/// Currently always returns `Ok`, but returns a `Result` for future
/// extensibility when routes may require async initialization.
///
/// # Example
///
/// ```rust,no_run
/// use axum_api_template_lib::startup::build_router;
/// use axum::Router;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let router = build_router().await?;
///     // Use the router...
///     Ok(())
/// }
/// ```
pub async fn build_router() -> Result<Router, anyhow::Error> {
    // define the tracing layer
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(
            DefaultMakeSpan::new()
                .include_headers(true)
                .level(Level::INFO),
        )
        .on_response(DefaultOnResponse::new().include_headers(true));

    // build the router with tracing
    let app = Router::new()
        .route("/health_check", get(health_check))
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::new(
                    X_REQUEST_ID.clone(),
                    MakeRequestUuid,
                ))
                .layer(trace_layer)
                .layer(PropagateRequestIdLayer::new(X_REQUEST_ID)),
        );

    Ok(app)
}
