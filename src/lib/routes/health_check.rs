//! Health check endpoint handler.
//!
//! This module provides a simple health check endpoint that returns
//! HTTP 200 OK when the service is healthy.

// dependencies
use crate::response::ApiResponse;

/// Health check endpoint.
///
/// Returns `HTTP 200 OK` to indicate the service is healthy and running.
///
/// # Example
///
/// ```text
/// GET /health_check HTTP/1.1
///
/// HTTP/1.1 200 OK
/// ```
pub async fn health_check() -> ApiResponse<()> {
    ApiResponse::ok(())
}
