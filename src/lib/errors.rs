//! Error types for the API.
//!
//! This module provides the [`ApiError`] enum which represents all possible
//! HTTP error responses from the API. It implements [`axum::response::IntoResponse`]
//! for seamless integration with Axum handlers.
//!
//! # Example
//!
//! ```rust
//! use axum_api_template_lib::errors::ApiError;
//! use axum::response::IntoResponse;
//!
//! async fn handler() -> Result<&'static str, ApiError> {
//!     Err(ApiError::NotFound("User not found".to_string()))
//! }
//! ```

// dependencies
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// API error types with HTTP status code mapping.
///
/// Each variant maps to an appropriate HTTP status code and provides
/// a descriptive error message.
///
/// # HTTP Status Code Mapping
///
/// | Variant | Status Code |
/// |---------|-------------|
/// | `BadRequest` | 400 Bad Request |
/// | `Unauthorized` | 401 Unauthorized |
/// | `Forbidden` | 403 Forbidden |
/// | `NotFound` | 404 Not Found |
/// | `Conflict` | 409 Conflict |
/// | `UnprocessableEntity` | 422 Unprocessable Entity |
/// | `Internal` | 500 Internal Server Error |
///
/// # Example
///
/// ```rust
/// use axum_api_template_lib::errors::ApiError;
///
/// let error = ApiError::NotFound("User 123 not found".to_string());
/// // This will return a 404 response with the body "Not Found: User 123 not found"
/// ```
#[derive(thiserror::Error)]
#[non_exhaustive]
pub enum ApiError {
    /// The request was malformed or invalid.
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Authentication is required for this resource.
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// The authenticated user does not have permission.
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// The requested resource was not found.
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// The request conflicts with the current state.
    #[error("Conflict: {0}")]
    Conflict(String),

    /// The request was well-formed but semantically invalid.
    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),

    /// An internal server error occurred.
    #[error("Internal server error: {0}")]
    Internal(String),
}

// implement the IntoResponse trait for the ApiError type
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            ApiError::BadRequest(err) => (StatusCode::BAD_REQUEST, format!("Bad Request: {}", err)),
            ApiError::Unauthorized(err) => {
                (StatusCode::UNAUTHORIZED, format!("Unauthorized: {}", err))
            }
            ApiError::Forbidden(err) => (StatusCode::FORBIDDEN, format!("Forbidden: {}", err)),
            ApiError::NotFound(err) => (StatusCode::NOT_FOUND, format!("Not Found: {}", err)),
            ApiError::Conflict(err) => (StatusCode::CONFLICT, format!("Conflict: {}", err)),
            ApiError::UnprocessableEntity(err) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Unprocessable Entity: {}", err),
            ),
            ApiError::Internal(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal Server Error: {}", err),
            ),
        };
        tracing::error!("Error occurred: {:?}", self);
        (status, msg).into_response()
    }
}

// implement the Debug trait for ApiError
impl std::fmt::Debug for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

/// Formats an error and its cause chain for display.
///
/// This utility function walks the error source chain and formats
/// each error with its cause, providing a complete error trace.
///
/// # Example Output
///
/// ```text
/// Bad request: Invalid email
///
/// Caused by:
///     Invalid email format
/// ```
pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
