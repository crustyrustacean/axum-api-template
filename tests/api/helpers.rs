//! Test helpers for API integration tests.
//!
//! This module provides utilities for spinning up test instances
//! of the application and making HTTP requests against it.

// dependencies
use axum_api_template_lib::get_configuration;
use axum_api_template_lib::startup::Application;
use axum_api_template_lib::telemetry::{get_subscriber, init_subscriber};
use std::sync::LazyLock;

// set up a static variable for the tracing configuration
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

/// Represents a test application instance.
///
/// This struct contains the address of the spawned application
/// and an HTTP client for making requests.
#[allow(dead_code)]
pub struct TestApp {
    /// The base address of the test application (e.g., `http://localhost:1234`).
    pub address: String,
    /// The port the test application is listening on.
    pub port: u16,
    /// An HTTP client for making requests to the test application.
    pub api_client: reqwest::Client,
}

/// Spawns an instance of the application for testing.
///
/// This function:
/// 1. Initializes tracing (once, using `LazyLock`)
/// 2. Loads configuration with a random port (port 0)
/// 3. Starts the application in a background task
/// 4. Returns a `TestApp` with the application's address
///
/// # Panics
///
/// Panics if:
/// - Configuration cannot be loaded
/// - The application fails to build
/// - The HTTP client cannot be created
///
/// # Example
///
/// ```rust,no_run
/// use crate::helpers::spawn_app;
///
/// #[tokio::test]
/// async fn test_something() {
///     let app = spawn_app().await;
///     let response = reqwest::Client::new()
///         .get(format!("{}/health_check", app.address))
///         .send()
///         .await
///         .unwrap();
///     assert!(response.status().is_success());
/// }
/// ```
pub async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        c.application.port = 0;
        c
    };

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let application_port = application.port();
    tokio::spawn(application.run_until_stopped());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    TestApp {
        address: format!("http://localhost:{}", application_port),
        port: application_port,
        api_client: client,
    }
}
