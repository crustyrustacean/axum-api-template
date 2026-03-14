//! Error types for the API.
//!
//! This module provides the [`ApiError`] enum which represents all possible
//! HTTP error responses from the API. It implements [`axum::response::IntoResponse`]
//! for seamless integration with Axum handlers.
//!
//! # Response Format
//!
//! Error responses use the same envelope format as [`ApiResponse`](crate::response::ApiResponse):
//!
//! ```json
//! {
//!   "success": false,
//!   "message": "Resource not found: User 123",
//!   "time": "2024-01-15T10:30:00Z"
//! }
//! ```
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
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::Serialize;

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
/// // This will return a 404 response with the error envelope
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

impl ApiError {
    /// Returns the HTTP status code for this error.
    #[must_use]
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Returns the error message for this error.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            ApiError::BadRequest(msg) => format!("Bad Request: {}", msg),
            ApiError::Unauthorized(msg) => format!("Unauthorized: {}", msg),
            ApiError::Forbidden(msg) => format!("Forbidden: {}", msg),
            ApiError::NotFound(msg) => format!("Not Found: {}", msg),
            ApiError::Conflict(msg) => format!("Conflict: {}", msg),
            ApiError::UnprocessableEntity(msg) => format!("Unprocessable Entity: {}", msg),
            ApiError::Internal(msg) => format!("Internal Server Error: {}", msg),
        }
    }
}

/// Error response envelope matching the [`ApiResponse`](crate::response::ApiResponse) format.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponse {
    /// Always `false` for error responses.
    success: bool,
    /// The error message.
    message: String,
    /// Timestamp when the error response was generated.
    time: DateTime<Utc>,
}

// implement the IntoResponse trait for the ApiError type
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let message = self.message();

        tracing::error!("Error occurred: {:?}", self);

        let error_response = ErrorResponse {
            success: false,
            message,
            time: Utc::now(),
        };

        (status, Json(error_response)).into_response()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_code_mapping() {
        assert_eq!(ApiError::BadRequest("test".into()).status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(ApiError::Unauthorized("test".into()).status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(ApiError::Forbidden("test".into()).status_code(), StatusCode::FORBIDDEN);
        assert_eq!(ApiError::NotFound("test".into()).status_code(), StatusCode::NOT_FOUND);
        assert_eq!(ApiError::Conflict("test".into()).status_code(), StatusCode::CONFLICT);
        assert_eq!(ApiError::UnprocessableEntity("test".into()).status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(ApiError::Internal("test".into()).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_message_formatting() {
        let error = ApiError::NotFound("User 123".into());
        assert_eq!(error.message(), "Not Found: User 123");
    }

    #[test]
    fn test_error_response_serialization() {
        let error = ApiError::BadRequest("Invalid input".into());
        let response = ErrorResponse {
            success: false,
            message: error.message(),
            time: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"message\":\"Bad Request: Invalid input\""));
    }
}
