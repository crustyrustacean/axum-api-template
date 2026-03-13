//! Axum API Template - Binary entry point.
//!
//! This is the main entry point for the application. It initializes
//! telemetry, loads configuration, and starts the HTTP server.

// dependencies
use anyhow::Context;
use axum_api_template_lib::configuration::get_configuration;
use axum_api_template_lib::startup::Application;
use axum_api_template_lib::telemetry::{get_subscriber, init_subscriber};

/// Main entry point for the application.
///
/// This function:
/// 1. Initializes structured logging with tracing
/// 2. Loads configuration from YAML files and environment variables
/// 3. Builds and starts the HTTP server
///
/// # Errors
///
/// Returns an error if:
/// - Configuration cannot be loaded
/// - The server cannot bind to the configured address
/// - A fatal error occurs while running
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    let subscriber = get_subscriber(
        "axum-api-template-server".into(),
        "info".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    // read in the app configuration
    let configuration = get_configuration().context("Failed to read configuration files.")?;

    // build an instance of the application and run it
    let application = Application::build(configuration.clone()).await?;
    application.run_until_stopped().await?;

    Ok(())
}
