//! Configuration management for the application.
//!
//! This module provides types and functions for loading application
//! configuration from YAML files and environment variables.
//!
//! # Configuration Sources
//!
//! Configuration is loaded in the following order (later sources override earlier):
//! 1. `configuration/base.yaml` - Base configuration
//! 2. `configuration/{environment}.yaml` - Environment-specific configuration
//! 3. Environment variables with `APP_` prefix
//!
//! # Environment Variables
//!
//! Environment variables can override YAML configuration:
//! - `APP_ENVIRONMENT`: Set the environment (`local` or `production`)
//! - `APP_APPLICATION__PORT`: Override the port
//! - `APP_APPLICATION__HOST`: Override the host
//!
//! # Example
//!
//! ```rust,no_run
//! use axum_api_template_lib::configuration::get_configuration;
//!
//! let config = get_configuration().expect("Failed to load configuration");
//! println!("Server will bind to {}:{}", config.application.host, config.application.port);
//! ```

// dependencies
use serde_aux::field_attributes::deserialize_number_from_string;
use std::convert::TryFrom;
use std::str::FromStr;

/// Top-level application settings.
///
/// Contains all configuration for the application, loaded from
/// YAML files and environment variables.
#[derive(serde::Deserialize, Clone, Debug)]
pub struct Settings {
    /// Application-specific settings.
    pub application: ApplicationSettings,
}

/// Application-specific configuration settings.
///
/// These settings control how the application binds and runs.
#[derive(serde::Deserialize, Clone, Debug)]
pub struct ApplicationSettings {
    /// The TCP port the application will listen on.
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    /// The host address to bind to.
    pub host: String,
    /// The base URL for the application (used for generating links).
    pub base_url: String,
}

/// Loads application configuration from YAML files and environment variables.
///
/// # Errors
///
/// Returns a [`ConfigurationError`] if:
/// - The configuration files cannot be read
/// - The configuration is invalid
/// - The environment is invalid
///
/// # Example
///
/// ```rust,no_run
/// use axum_api_template_lib::configuration::get_configuration;
///
/// match get_configuration() {
///     Ok(config) => println!("Loaded configuration for port {}", config.application.port),
///     Err(e) => eprintln!("Failed to load configuration: {}", e),
/// }
/// ```
pub fn get_configuration() -> Result<Settings, ConfigurationError> {
    let base_path = std::env::current_dir().map_err(|e| {
        ConfigurationError::Io(format!("Failed to determine the current directory: {}", e))
    })?;
    let configuration_directory = base_path.join("configuration");

    // Detect the running environment.
    // Default to `local` if unspecified.
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .parse()
        .map_err(ConfigurationError::InvalidEnvironment)?;
    let environment_filename = format!("{}.yaml", environment.as_str());
    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        // Add in settings from environment variables (with a prefix of APP and '__' as separator)
        // E.g. `APP_APPLICATION__PORT=5001 would set `Settings.application.port`
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()
        .map_err(ConfigurationError::Config)?;

    settings
        .try_deserialize::<Settings>()
        .map_err(ConfigurationError::Config)
}

/// The possible runtime environments for the application.
///
/// The environment determines which configuration file is loaded
/// and how certain features behave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    /// Local development environment.
    Local,
    /// Production environment.
    Production,
}

impl Environment {
    /// Returns the environment as a string for use in configuration filenames.
    ///
    /// # Example
    ///
    /// ```
    /// use axum_api_template_lib::configuration::Environment;
    ///
    /// let env = Environment::Local;
    /// assert_eq!(env.as_str(), "local");
    /// ```
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

/// Errors that can occur when loading configuration.
#[derive(thiserror::Error, Debug)]
pub enum ConfigurationError {
    /// An I/O error occurred while reading configuration files.
    #[error("I/O error: {0}")]
    Io(String),

    /// An error occurred while parsing configuration files.
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    /// An invalid environment was specified.
    #[error("Invalid environment: {0}")]
    InvalidEnvironment(String),
}
