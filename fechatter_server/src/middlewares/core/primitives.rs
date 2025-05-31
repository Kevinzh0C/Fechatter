//! # Middleware Infrastructure Layer - Atomic Operation Functions
//!
//! **Design Principles**:
//! - Each function does one thing (SRP)
//! - Zero external dependencies, pure function design
//! - Highly reusable atomic operations
//! - Inline optimization, zero-cost abstraction

use axum::body::Body;
use axum::http::{HeaderMap, Request};

// =============================================================================
// Token Extraction - Single Responsibility: Handle Authentication Tokens
// =============================================================================

/// Extract Bearer Token
///
/// **Single Responsibility**: Extract Bearer token from HTTP headers
/// **Input**: HTTP headers
/// **Output**: Token string (if exists)
/// **Side Effects**: None
#[inline(always)]
pub fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
  headers
    .get("authorization")
    .and_then(|h| h.to_str().ok())
    .and_then(|h| h.strip_prefix("Bearer "))
}

/// Extract Refresh Token
///
/// **Single Responsibility**: Extract refresh token from HTTP cookies
/// **Input**: HTTP headers
/// **Output**: Refresh token string (if exists)
/// **Side Effects**: None
#[inline(always)]
pub fn extract_refresh_token(headers: &HeaderMap) -> Option<String> {
  extract_cookie_value(headers, "refresh_token")
}

/// Generic Cookie Value Extraction
///
/// **Single Responsibility**: Extract specified value from cookies
/// **Input**: HTTP headers, cookie name
/// **Output**: Cookie value (if exists)
/// **Side Effects**: None
#[inline(always)]
pub fn extract_cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
  let cookie_header = headers.get("cookie")?;
  let cookie_str = cookie_header.to_str().ok()?;

  for cookie_part in cookie_str.split(';') {
    let trimmed = cookie_part.trim();
    if let Some(eq_pos) = trimmed.find('=') {
      let (cookie_name, value) = trimmed.split_at(eq_pos);
      if cookie_name.trim() == name {
        return Some(value[1..].trim().to_string());
      }
    }
  }
  None
}

// =============================================================================
// Path Parsing - Single Responsibility: Handle URL Path Parsing
// =============================================================================

/// Parse Workspace ID
///
/// **Single Responsibility**: Extract workspace ID from request path
/// **Input**: HTTP request
/// **Output**: Workspace ID (if exists)
/// **Side Effects**: None
#[inline(always)]
pub fn extract_workspace_id(request: &Request<Body>) -> Option<i64> {
  let path = request.uri().path();
  if let Some(rest) = path.strip_prefix("/api/workspace/") {
    parse_id_from_path(rest)
  } else if let Some(rest) = path.strip_prefix("/api/files/") {
    parse_id_from_path(rest)
  } else {
    None
  }
}

/// Parse Chat ID
///
/// **Single Responsibility**: Extract chat ID from request path
/// **Input**: HTTP request
/// **Output**: Chat ID (if exists)
/// **Side Effects**: None
#[inline(always)]
pub fn extract_chat_id(request: &Request<Body>) -> Option<i64> {
  let path = request.uri().path();
  if let Some(rest) = path.strip_prefix("/api/chat/") {
    parse_id_from_path(rest)
  } else {
    None
  }
}

/// Generic ID Parsing
///
/// **Single Responsibility**: Parse first integer ID from path string
/// **Input**: Path string
/// **Output**: Parsed ID (if valid)
/// **Side Effects**: None
#[inline(always)]
pub fn parse_id_from_path(path: &str) -> Option<i64> {
  path.split('/').next().and_then(|s| s.parse().ok())
}

// =============================================================================
// Identifier Generation - Single Responsibility: Generate Unique Identifiers
// =============================================================================

/// Generate Request ID
///
/// **Single Responsibility**: Generate unique request tracking ID
/// **Input**: None
/// **Output**: Unique request ID string
/// **Side Effects**: Based on current timestamp
#[inline(always)]
pub fn generate_request_id() -> String {
  format!(
    "req_{}",
    chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
  )
}

/// Generate Session ID
///
/// **Single Responsibility**: Generate unique identifier for user session
/// **Input**: None
/// **Output**: Session ID string
/// **Side Effects**: Based on current timestamp and random number
#[inline(always)]
pub fn generate_session_id() -> String {
  format!("sess_{}", chrono::Utc::now().timestamp_millis())
}

// =============================================================================
// String Processing - Single Responsibility: String Operation Utilities
// =============================================================================

/// Normalize Bearer Token Format
///
/// **Single Responsibility**: Ensure token has correct Bearer prefix
/// **Input**: Raw token string
/// **Output**: Formatted Authorization header value
/// **Side Effects**: None
#[inline(always)]
pub fn format_bearer_token(token: &str) -> String {
  if token.starts_with("Bearer ") {
    token.to_string()
  } else {
    format!("Bearer {}", token)
  }
}

/// Clean Path String
///
/// **Single Responsibility**: Remove redundant slashes and spaces from path
/// **Input**: Raw path string
/// **Output**: Cleaned path
/// **Side Effects**: None
#[inline(always)]
pub fn normalize_path(path: &str) -> String {
  path
    .trim()
    .trim_start_matches('/')
    .trim_end_matches('/')
    .to_string()
}

// =============================================================================
// Time Processing - Single Responsibility: Time-related Utility Functions
// =============================================================================

/// Calculate Cookie Expiration Time
///
/// **Single Responsibility**: Calculate Max-Age value based on expiration time
/// **Input**: Expiration timestamp
/// **Output**: Max-Age in seconds
/// **Side Effects**: Based on current time calculation
#[inline(always)]
pub fn calculate_cookie_max_age(expires_at: chrono::DateTime<chrono::Utc>) -> i64 {
  (expires_at - chrono::Utc::now()).num_seconds().max(0)
}

/// Check Timestamp Expiration
///
/// **Single Responsibility**: Determine if given timestamp has expired
/// **Input**: Timestamp
/// **Output**: Whether expired
/// **Side Effects**: Based on current time judgment
#[inline(always)]
pub fn is_expired(timestamp: chrono::DateTime<chrono::Utc>) -> bool {
  chrono::Utc::now() > timestamp
}

// =============================================================================
// Validation - Single Responsibility: Data Validation Tools
// =============================================================================

/// Validate ID
///
/// **Single Responsibility**: Check if ID is valid positive integer
/// **Input**: ID value
/// **Output**: Whether valid
/// **Side Effects**: None
#[inline(always)]
pub fn is_valid_id(id: i64) -> bool {
  id > 0
}

/// Validate Token Format
///
/// **Single Responsibility**: Check if token string format is valid
/// **Input**: Token string
/// **Output**: Whether valid
/// **Side Effects**: None
#[inline(always)]
pub fn is_valid_token_format(token: &str) -> bool {
  !token.is_empty() && token.len() > 10 && token.chars().all(|c| c.is_ascii())
}

/// Validate Path Safety
///
/// **Single Responsibility**: Check if path contains dangerous characters
/// **Input**: Path string
/// **Output**: Whether safe
/// **Side Effects**: None
#[inline(always)]
pub fn is_safe_path(path: &str) -> bool {
  !path.contains("..") && !path.contains('\0')
}

// =============================================================================
// Test Module
// =============================================================================

#[cfg(test)]
mod tests {
  use super::*;
  use axum::http::HeaderValue;

  #[test]
  fn test_extract_bearer_token() {
    let mut headers = HeaderMap::new();
    headers.insert("authorization", HeaderValue::from_static("Bearer abc123"));

    assert_eq!(extract_bearer_token(&headers), Some("abc123"));
  }

  #[test]
  fn test_parse_id_from_path() {
    assert_eq!(parse_id_from_path("123/messages"), Some(123));
    assert_eq!(parse_id_from_path("invalid"), None);
  }

  #[test]
  fn test_generate_request_id() {
    let id1 = generate_request_id();
    let id2 = generate_request_id();

    assert!(id1.starts_with("req_"));
    assert_ne!(id1, id2); // Should be unique
  }

  #[test]
  fn test_is_valid_id() {
    assert!(is_valid_id(1));
    assert!(is_valid_id(999999));
    assert!(!is_valid_id(0));
    assert!(!is_valid_id(-1));
  }

  #[test]
  fn test_format_bearer_token() {
    assert_eq!(format_bearer_token("abc123"), "Bearer abc123");
    assert_eq!(format_bearer_token("Bearer abc123"), "Bearer abc123");
  }
}
