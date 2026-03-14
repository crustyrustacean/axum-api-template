//! Standardized API response types.
//!
//! This module provides a consistent JSON envelope for all API responses,
//! ensuring clients can rely on a predictable response structure.
//!
//! # Response Format
//!
//! All successful responses follow this envelope format:
//!
//! ```json
//! {
//!   "success": true,
//!   "message": "Optional message",
//!   "time": "2024-01-15T10:30:00Z",
//!   "data": { ... }
//! }
//! ```
//!
//! Error responses from [`ApiError`](crate::errors::ApiError) use the same envelope
//! format with `success: false` and the error message in the `message` field.
//!
//! # Example
//!
//! ```rust
//! use axum_api_template_lib::response::ApiResponse;
//! use axum::http::StatusCode;
//!
//! // Create a simple success response
//! let response = ApiResponse::ok("Hello, World!");
//!
//! // Create a response with a custom status code
//! let response = ApiResponse::with_status(StatusCode::CREATED, 42);
//! ```

// dependencies
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::Serialize;

/// Result type for API handlers.
///
/// This type alias combines [`ApiResponse`] with [`ApiError`](crate::errors::ApiError)
/// for ergonomic handler return types.
///
/// # Example
///
/// ```rust,no_run
/// use axum_api_template_lib::response::{ApiResult, ApiResponse};
/// use axum_api_template_lib::errors::ApiError;
///
/// async fn get_item(id: u64) -> ApiResult<String> {
///     // Simulate looking up an item
///     if id == 0 {
///         return Err(ApiError::NotFound("Item not found".into()));
///     }
///     Ok(ApiResponse::ok(format!("Item {}", id)))
/// }
/// ```
pub type ApiResult<T> = Result<ApiResponse<T>, crate::errors::ApiError>;

/// A standardized API response envelope.
///
/// This struct provides a consistent JSON structure for all API responses,
/// making it easy for clients to parse and handle responses uniformly.
///
/// # Fields
///
/// - `success` - Indicates whether the request was successful
/// - `message` - Optional message providing additional context
/// - `time` - Timestamp when the response was generated (UTC)
/// - `data` - The response payload (omitted from JSON if `None`)
///
/// # Example
///
/// ```rust
/// use axum_api_template_lib::response::ApiResponse;
/// use axum::http::StatusCode;
///
/// // Simple OK response
/// let response = ApiResponse::ok(vec![1, 2, 3]);
///
/// // Created response (HTTP 201)
/// let response = ApiResponse::with_status(StatusCode::CREATED, "new-resource-id");
///
/// // Response with a custom message
/// let response = ApiResponse::with_message("data", "User created successfully");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    /// Indicates whether the request was successful.
    pub success: bool,

    /// Optional message providing additional context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Timestamp when the response was generated.
    pub time: DateTime<Utc>,

    /// The response payload, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    /// Creates a successful response with HTTP 200 OK.
    ///
    /// # Example
    ///
    /// ```rust
    /// use axum_api_template_lib::response::ApiResponse;
    ///
    /// let response = ApiResponse::ok("Hello, World!");
    /// // Returns: { "success": true, "time": "...", "data": "Hello, World!" }
    /// ```
    #[must_use = "ApiResponse must be returned as a response"]
    pub fn ok(data: T) -> Self {
        Self::with_status(StatusCode::OK, data)
    }

    /// Creates a successful response with a custom HTTP status code.
    ///
    /// The `success` field is set to `true` for 2xx status codes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use axum_api_template_lib::response::ApiResponse;
    /// use axum::http::StatusCode;
    ///
    /// let response = ApiResponse::with_status(StatusCode::CREATED, "new-resource-id");
    /// // Returns HTTP 201 with: { "success": true, "time": "...", "data": "new-resource-id" }
    /// ```
    #[must_use = "ApiResponse must be returned as a response"]
    pub fn with_status(status: StatusCode, data: T) -> Self {
        Self {
            success: status.is_success(),
            message: None,
            time: Utc::now(),
            data: Some(data),
        }
    }

    /// Creates a successful response with a custom message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use axum_api_template_lib::response::ApiResponse;
    ///
    /// let response = ApiResponse::with_message("item-123", "Item created successfully");
    /// // Returns: { "success": true, "message": "Item created successfully", "time": "...", "data": "item-123" }
    /// ```
    #[must_use = "ApiResponse must be returned as a response"]
    pub fn with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: Some(message.into()),
            time: Utc::now(),
            data: Some(data),
        }
    }
}

impl ApiResponse<()> {
    /// Creates an empty success response (no data payload).
    ///
    /// Useful for endpoints that don't return data, like DELETE endpoints.
    ///
    /// # Example
    ///
    /// ```rust
    /// use axum_api_template_lib::response::ApiResponse;
    ///
    /// let response = ApiResponse::empty();
    /// // Returns: { "success": true, "time": "..." }
    /// ```
    #[must_use = "ApiResponse must be returned as a response"]
    pub fn empty() -> Self {
        Self {
            success: true,
            message: None,
            time: Utc::now(),
            data: None,
        }
    }
}

/// Implement `IntoResponse` for `ApiResponse<T>`.
///
/// This allows `ApiResponse` to be returned directly from Axum handlers.
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        // Determine status code based on success field
        // For success responses, use OK; error responses come through ApiError
        let status = if self.success {
            StatusCode::OK
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };

        (status, Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok_response() {
        let response = ApiResponse::ok("test data");
        assert!(response.success);
        assert!(response.message.is_none());
        assert_eq!(response.data, Some("test data"));
    }

    #[test]
    fn test_with_status_created() {
        let response = ApiResponse::with_status(StatusCode::CREATED, 42);
        assert!(response.success);
        assert_eq!(response.data, Some(42));
    }

    #[test]
    fn test_with_message() {
        let response = ApiResponse::with_message("data", "Custom message");
        assert!(response.success);
        assert_eq!(response.message, Some("Custom message".to_string()));
    }

    #[test]
    fn test_empty_response() {
        let response = ApiResponse::empty();
        assert!(response.success);
        assert!(response.data.is_none());
    }

    #[test]
    fn test_serialization() {
        let response = ApiResponse::ok("hello");
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"data\":\"hello\""));
        assert!(!json.contains("\"message\":")); // should be omitted
    }
}
