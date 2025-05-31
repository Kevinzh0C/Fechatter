//! # Request Adapter
//!
//! **Responsibilities**:
//! - Wraps fechatter_core's request ID generation functionality
//! - Adds business-specific request tracking logic
//! - Provides performance monitoring and debug information
//! - Maintains Core standardization while enhancing business value

use axum::{
  extract::Request,
  http::{HeaderMap, HeaderValue},
  middleware::Next,
  response::Response,
};
use fechatter_core::middlewares::request_id_middleware;
use std::time::Instant;
use tracing::{debug, info};

// =============================================================================
// Request Adapter - Inherits Core functionality, adds business logic
// =============================================================================

/// Request Adapter Configuration
///
/// **Single Responsibility**: Configure various request handling options
pub struct RequestAdapterConfig {
  /// Whether performance monitoring is enabled
  pub performance_monitoring: bool,
  /// Whether detailed logging is enabled
  pub detailed_logging: bool,
  /// Whether to add business headers
  pub business_headers: bool,
  /// Request timeout in milliseconds
  pub timeout_ms: Option<u64>,
}

impl Default for RequestAdapterConfig {
  fn default() -> Self {
    Self {
      performance_monitoring: true,
      detailed_logging: false,
      business_headers: true,
      timeout_ms: Some(30000), // 30 second default timeout
    }
  }
}

// =============================================================================
// Middleware Implementation - Adapter Pattern
// =============================================================================

/// Basic Request ID Middleware (Direct Core Usage)
///
/// **Single Responsibility**: Provide Core-level request ID generation
/// **Use Cases**: Scenarios requiring maximum compatibility and minimal overhead
pub async fn core_request_id_middleware(request: Request, next: Next) -> Response {
  debug!("Using Core request ID middleware");

  // Directly call fechatter_core's standard implementation (UUID v7)
  request_id_middleware(request, next).await
}

/// Enhanced Request Middleware (Core + Business Logic)
///
/// **Single Responsibility**: Add performance monitoring and business tracking on top of Core
/// **Use Cases**: Business scenarios requiring detailed monitoring and debugging
pub async fn enhanced_request_middleware(mut request: Request, next: Next) -> Response {
  let start_time = Instant::now();

  debug!("Using enhanced request middleware");

  // 1. First use Core to generate standard request ID
  let mut response = core_request_id_middleware(request, next).await;

  // 2. Add performance monitoring information
  let duration = start_time.elapsed();
  let duration_ms = duration.as_millis();

  info!("Request processing completed: took {}ms", duration_ms);

  // 3. Add business-related response headers
  let headers = response.headers_mut();

  // Add processing time header
  if let Ok(duration_header) = HeaderValue::from_str(&duration_ms.to_string()) {
    headers.insert("x-processing-time-ms", duration_header);
  }

  // Add service identifier
  let service_header = HeaderValue::from_static("fechatter-server");
  headers.insert("x-service-name", service_header);

  response
}

/// Business Request Middleware (Core + Complete Business Logic)
///
/// **Single Responsibility**: Provide complete business request handling flow
/// **Use Cases**: Production environment with complete business scenarios
pub async fn business_request_middleware(request: Request, next: Next) -> Response {
  let start_time = Instant::now();

  debug!("Using business request middleware");

  // 1. Log request start
  let method = request.method().clone();
  let uri = request.uri().clone();
  let user_agent = request
    .headers()
    .get("user-agent")
    .and_then(|h| h.to_str().ok())
    .unwrap_or("unknown");

  info!(
    "Starting request processing: {} {} - User-Agent: {}",
    method, uri, user_agent
  );

  // 2. Use enhanced request middleware
  let mut response = enhanced_request_middleware(request, next).await;

  // 3. Add more business headers
  let headers = response.headers_mut();

  // Add API version
  let version_header = HeaderValue::from_static("v1");
  headers.insert("x-api-version", version_header);

  // Add environment information
  let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
  if let Ok(env_header) = HeaderValue::from_str(&env) {
    headers.insert("x-environment", env_header);
  }

  // 4. Log request completion
  let duration = start_time.elapsed();
  let status = response.status();

  info!(
    "Request completed: {} {} - Status: {} - Duration: {:?}",
    method, uri, status, duration
  );

  response
}

/// Debug Request Middleware (Includes Detailed Debug Information)
///
/// **Single Responsibility**: Provide detailed request debug information
/// **Use Cases**: Development and debugging environments
pub async fn debug_request_middleware(request: Request, next: Next) -> Response {
  debug!("Using debug request middleware");

  // 1. Log detailed request information
  log_request_details(&request);

  // 2. Use business request middleware
  let mut response = business_request_middleware(request, next).await;

  // 3. Add debug headers
  let headers = response.headers_mut();

  // Add debug mode identifier
  let debug_header = HeaderValue::from_static("true");
  headers.insert("x-debug-mode", debug_header);

  // Add timestamp
  let timestamp = chrono::Utc::now().timestamp().to_string();
  if let Ok(timestamp_header) = HeaderValue::from_str(&timestamp) {
    headers.insert("x-timestamp", timestamp_header);
  }

  response
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Log Detailed Request Information
///
/// **Single Responsibility**: Output detailed debug information about the request
fn log_request_details(request: &Request) {
  debug!("=== Request Details ===");
  debug!("Method: {}", request.method());
  debug!("URI: {}", request.uri());
  debug!("Version: {:?}", request.version());

  debug!("--- Headers ---");
  for (name, value) in request.headers() {
    if let Ok(value_str) = value.to_str() {
      debug!("{}: {}", name, value_str);
    }
  }

  debug!("--- Extensions ---");
  debug!("Extension count: {}", request.extensions().len());

  debug!("==================");
}

/// Extract Key Request Information
///
/// **Single Responsibility**: Extract key monitoring information from request
pub fn extract_request_info(request: &Request) -> RequestInfo {
  RequestInfo {
    method: request.method().to_string(),
    uri: request.uri().to_string(),
    user_agent: request
      .headers()
      .get("user-agent")
      .and_then(|h| h.to_str().ok())
      .map(|s| s.to_string()),
    content_length: request
      .headers()
      .get("content-length")
      .and_then(|h| h.to_str().ok())
      .and_then(|s| s.parse().ok()),
    request_id: request
      .headers()
      .get("x-request-id")
      .and_then(|h| h.to_str().ok())
      .map(|s| s.to_string()),
  }
}

/// Request Information Structure
///
/// **Single Responsibility**: Store key request information for monitoring and logging
#[derive(Debug, Clone)]
pub struct RequestInfo {
  pub method: String,
  pub uri: String,
  pub user_agent: Option<String>,
  pub content_length: Option<u64>,
  pub request_id: Option<String>,
}

// =============================================================================
// Convenience Constructors
// =============================================================================

/// Create Standard Request Middleware Stack
///
/// **Single Responsibility**: Provide common request handling middleware combinations
pub fn standard_request_stack() -> Vec<&'static str> {
  vec!["core_request_id"]
}

/// Create Business Request Middleware Stack
///
/// **Single Responsibility**: Provide business scenario request handling middleware combinations
pub fn business_request_stack() -> Vec<&'static str> {
  vec!["enhanced_request"]
}

/// Create Full Request Middleware Stack
///
/// **Single Responsibility**: Provide complete stack with all functionality
pub fn full_request_stack() -> Vec<&'static str> {
  vec!["business_request"]
}

/// Create Debug Request Middleware Stack
///
/// **Single Responsibility**: Provide request handling stack for debugging
pub fn debug_request_stack() -> Vec<&'static str> {
  vec!["debug_request"]
}

// =============================================================================
// Performance Monitoring Tools
// =============================================================================

/// Request Performance Monitor
///
/// **Single Responsibility**: Monitor and record request performance metrics
pub struct RequestPerformanceMonitor {
  /// Slow request threshold (milliseconds)
  slow_request_threshold_ms: u64,
  /// Whether detailed monitoring is enabled
  detailed_monitoring: bool,
}

impl Default for RequestPerformanceMonitor {
  fn default() -> Self {
    Self {
      slow_request_threshold_ms: 1000, // 1 second
      detailed_monitoring: false,
    }
  }
}

impl RequestPerformanceMonitor {
  /// Create new performance monitor
  pub fn new(slow_threshold_ms: u64) -> Self {
    Self {
      slow_request_threshold_ms: slow_threshold_ms,
      detailed_monitoring: false,
    }
  }

  /// Enable detailed monitoring
  pub fn with_detailed_monitoring(mut self) -> Self {
    self.detailed_monitoring = true;
    self
  }

  /// Record request performance
  pub fn record_request_performance(&self, info: &RequestInfo, duration_ms: u128) {
    if duration_ms > self.slow_request_threshold_ms as u128 {
      tracing::warn!(
        "Slow request detected: {} {} - Duration: {}ms",
        info.method,
        info.uri,
        duration_ms
      );
    }

    if self.detailed_monitoring {
      tracing::info!(
        "Request performance: {} {} - Duration: {}ms - User-Agent: {:?}",
        info.method,
        info.uri,
        duration_ms,
        info.user_agent
      );
    }
  }
}

// =============================================================================
// Test Module
// =============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_request_adapter_config() {
    let config = RequestAdapterConfig::default();
    assert!(config.performance_monitoring);
    assert!(!config.detailed_logging);
    assert!(config.business_headers);
    assert_eq!(config.timeout_ms, Some(30000));
  }

  #[test]
  fn test_request_stack_creation() {
    let standard = standard_request_stack();
    assert_eq!(standard, vec!["core_request_id"]);

    let business = business_request_stack();
    assert_eq!(business, vec!["enhanced_request"]);

    let full = full_request_stack();
    assert_eq!(full, vec!["business_request"]);

    let debug = debug_request_stack();
    assert_eq!(debug, vec!["debug_request"]);
  }

  #[test]
  fn test_performance_monitor() {
    let monitor = RequestPerformanceMonitor::new(500);
    assert_eq!(monitor.slow_request_threshold_ms, 500);
    assert!(!monitor.detailed_monitoring);

    let detailed_monitor = RequestPerformanceMonitor::new(500).with_detailed_monitoring();
    assert!(detailed_monitor.detailed_monitoring);
  }
}
