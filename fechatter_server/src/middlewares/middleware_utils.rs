/// Middleware Utilities - Common helper functions for business middleware
/// 
/// This module contains utility functions and types that are shared across
/// different business middleware components.

use axum::{
  body::Body,
  extract::Request,
  http::{HeaderValue, Method, StatusCode, Uri},
  response::{IntoResponse, Response},
};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, warn};

// ============================================================================
// Path Utilities
// ============================================================================

/// Extract resource ID from URL path segments
pub fn extract_id_from_path(path: &str, resource: &str) -> Option<i64> {
  let pattern = format!("/api/{}/", resource);
  
  if let Some(start) = path.find(&pattern) {
    let after_prefix = &path[start + pattern.len()..];
    if let Some(end) = after_prefix.find('/') {
      let id_str = &after_prefix[..end];
      id_str.parse().ok()
    } else {
      after_prefix.parse().ok()
    }
  } else {
    None
  }
}

/// Extract chat ID from path
pub fn extract_chat_id(path: &str) -> Option<i64> {
  extract_id_from_path(path, "chat")
}

/// Extract workspace ID from path
pub fn extract_workspace_id(path: &str) -> Option<i64> {
  extract_id_from_path(path, "workspace")
}

/// Extract user ID from path
pub fn extract_user_id(path: &str) -> Option<i64> {
  extract_id_from_path(path, "users")
}

/// Check if path matches a pattern
pub fn path_matches_pattern(path: &str, patterns: &[&str]) -> bool {
  patterns.iter().any(|pattern| path.starts_with(pattern))
}

// ============================================================================
// Request Analysis
// ============================================================================

/// Check if request is a business operation that should be audited
pub fn is_business_operation(method: &Method, path: &str) -> bool {
  let business_paths = [
    "/api/chat",
    "/api/workspace",
    "/api/users",
    "/api/members",
    "/api/messages",
    "/api/files",
  ];
  
  let business_methods = [Method::POST, Method::PUT, Method::PATCH, Method::DELETE];
  
  path_matches_pattern(path, &business_paths) && business_methods.contains(method)
}

/// Check if request requires chat access validation
pub fn requires_chat_access(path: &str) -> bool {
  let chat_patterns = [
    "/api/chat/",
    "/api/messages/",
  ];
  
  path_matches_pattern(path, &chat_patterns)
}

/// Check if request requires workspace access validation
pub fn requires_workspace_access(path: &str) -> bool {
  let workspace_patterns = [
    "/api/workspace/",
    "/api/workspaces/",
  ];
  
  path_matches_pattern(path, &workspace_patterns)
}

/// Check if request is a public endpoint (no auth required)
pub fn is_public_endpoint(path: &str) -> bool {
  let public_patterns = [
    "/api/signup",
    "/api/signin", 
    "/api/refresh",
    "/health",
    "/metrics",
    "/api/system/health",
  ];
  
  path_matches_pattern(path, &public_patterns)
}

// ============================================================================
// Response Utilities
// ============================================================================

/// Create standardized error response
pub fn create_error_response(status: StatusCode, message: &str) -> Response {
  let error_body = serde_json::json!({
    "error": message,
    "status": status.as_u16(),
    "timestamp": current_timestamp()
  });
  
  (status, error_body.to_string()).into_response()
}

/// Create unauthorized response
pub fn unauthorized_response(message: &str) -> Response {
  create_error_response(StatusCode::UNAUTHORIZED, message)
}

/// Create forbidden response
pub fn forbidden_response(message: &str) -> Response {
  create_error_response(StatusCode::FORBIDDEN, message)
}

/// Create internal server error response
pub fn internal_error_response(message: &str) -> Response {
  create_error_response(StatusCode::INTERNAL_SERVER_ERROR, message)
}

// ============================================================================
// Time Utilities
// ============================================================================

/// Get current Unix timestamp
pub fn current_timestamp() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_secs()
}

/// Get current timestamp in milliseconds
pub fn current_timestamp_millis() -> u128 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_millis()
}

// ============================================================================
// Header Utilities
// ============================================================================

/// Extract client IP from request headers
pub fn extract_client_ip(req: &Request<Body>) -> String {
  // Check X-Forwarded-For header first (proxy/load balancer)
  if let Some(forwarded_for) = req.headers().get("x-forwarded-for") {
    if let Ok(forwarded_str) = forwarded_for.to_str() {
      // Take the first IP (original client)
      if let Some(first_ip) = forwarded_str.split(',').next() {
        return first_ip.trim().to_string();
      }
    }
  }
  
  // Check X-Real-IP header (Nginx)
  if let Some(real_ip) = req.headers().get("x-real-ip") {
    if let Ok(ip_str) = real_ip.to_str() {
      return ip_str.to_string();
    }
  }
  
  // Fallback to unknown
  "unknown".to_string()
}

/// Extract User-Agent from request headers
pub fn extract_user_agent(req: &Request<Body>) -> String {
  req.headers()
    .get("user-agent")
    .and_then(|ua| ua.to_str().ok())
    .unwrap_or("unknown")
    .to_string()
}

/// Extract request ID if present
pub fn extract_request_id(req: &Request<Body>) -> Option<String> {
  req.headers()
    .get("x-request-id")
    .and_then(|id| id.to_str().ok())
    .map(|s| s.to_string())
}

// ============================================================================
// Validation Utilities
// ============================================================================

/// Validate resource ID format
pub fn validate_resource_id(id: i64) -> bool {
  id > 0
}

/// Validate that required extension exists in request
pub fn has_required_extension<T: Clone + Send + Sync + 'static>(req: &Request<Body>) -> bool {
  req.extensions().get::<T>().is_some()
}

// ============================================================================
// Logging Utilities
// ============================================================================

/// Log middleware entry with context
pub fn log_middleware_entry(middleware_name: &str, method: &Method, path: &str) {
  debug!(
    "[{}] Processing {} {}", 
    middleware_name.to_uppercase(), 
    method, 
    path
  );
}

/// Log middleware success
pub fn log_middleware_success(middleware_name: &str, user_id: Option<i64>) {
  if let Some(id) = user_id {
    debug!("[{}] Success for user {}", middleware_name.to_uppercase(), id);
  } else {
    debug!("[{}] Success", middleware_name.to_uppercase());
  }
}

/// Log middleware failure
pub fn log_middleware_failure(middleware_name: &str, reason: &str) {
  warn!("ERROR: [{}] Failed: {}", middleware_name.to_uppercase(), reason);
}

// ============================================================================
// Test Utilities (for unit testing)
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;
  use axum::http::Method;

  #[test]
  fn test_extract_chat_id() {
    assert_eq!(extract_chat_id("/api/chat/123"), Some(123));
    assert_eq!(extract_chat_id("/api/chat/456/messages"), Some(456));
    assert_eq!(extract_chat_id("/api/users/123"), None);
    assert_eq!(extract_chat_id("/api/chat/invalid"), None);
  }

  #[test]
  fn test_is_business_operation() {
    assert!(is_business_operation(&Method::POST, "/api/chat"));
    assert!(is_business_operation(&Method::DELETE, "/api/users/123"));
    assert!(!is_business_operation(&Method::GET, "/api/chat"));
    assert!(!is_business_operation(&Method::POST, "/api/signin"));
  }

  #[test]
  fn test_is_public_endpoint() {
    assert!(is_public_endpoint("/api/signup"));
    assert!(is_public_endpoint("/api/signin"));
    assert!(is_public_endpoint("/health"));
    assert!(!is_public_endpoint("/api/chat"));
    assert!(!is_public_endpoint("/api/users"));
  }

  #[test]
  fn test_validate_resource_id() {
    assert!(validate_resource_id(1));
    assert!(validate_resource_id(999));
    assert!(!validate_resource_id(0));
    assert!(!validate_resource_id(-1));
  }
} 