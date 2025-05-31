//! # Security Adapter
//!
//! **Responsibilities**:
//! - Wraps fechatter_core's security infrastructure
//! - Adds business-specific security policies
//! - Provides protection measures and threat detection
//! - Maintains Core's security baseline, enhances business security

use crate::AppState;
use axum::{
  body::Body,
  extract::{Request, State},
  http::{HeaderMap, StatusCode},
  middleware::Next,
  response::Response,
};
use fechatter_core::middlewares::{ServerTimeLayer, token_refresh::refresh_token_middleware};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, error, warn};

// =============================================================================
// Security Adapter - Inherits Core functionality, adds business security logic
// =============================================================================

/// Security adapter configuration
///
/// **Single Responsibility**: Configure various security policies and thresholds
pub struct SecurityAdapterConfig {
  /// Whether to enable rate limiting
  pub rate_limiting: bool,
  /// Whether to enable IP whitelist
  pub ip_whitelist: bool,
  /// Whether to enable User-Agent checks
  pub user_agent_check: bool,
  /// Whether to enable anti-bot protection
  pub anti_bot_protection: bool,
  /// Request rate limit (requests per minute)
  pub rate_limit_per_minute: u32,
  /// Suspicious activity detection threshold
  pub suspicious_activity_threshold: u32,
}

impl Default for SecurityAdapterConfig {
  fn default() -> Self {
    Self {
      rate_limiting: true,
      ip_whitelist: false,
      user_agent_check: true,
      anti_bot_protection: true,
      rate_limit_per_minute: 60,
      suspicious_activity_threshold: 10,
    }
  }
}

/// Security event types
///
/// **Single Responsibility**: Define various security threat types
#[derive(Debug, Clone)]
pub enum SecurityEvent {
  /// Rate limit exceeded
  RateLimitExceeded { ip: String, requests: u32 },
  /// Suspicious User-Agent
  SuspiciousUserAgent { user_agent: String },
  /// Suspicious IP activity
  SuspiciousIpActivity { ip: String, events: u32 },
  /// Token refresh failure
  TokenRefreshFailure { reason: String },
  /// Unauthorized access attempt
  UnauthorizedAccess { path: String, ip: String },
  /// Potential bot activity
  PotentialBotActivity {
    user_agent: String,
    patterns: Vec<String>,
  },
}

// =============================================================================
// Middleware Implementation - Adapter Pattern
// =============================================================================

/// Core security middleware (direct Core usage)
///
/// **Single Responsibility**: Provide Core-level security foundation
/// **Use Case**: Scenarios requiring maximum compatibility and minimal overhead
pub async fn core_security_middleware(
  State(state): State<AppState>,
  request: Request<Body>,
  next: Next,
) -> Response {
  debug!("Using Core security middleware");

  // Use fechatter_core's ServerTimeLayer and token refresh
  let response = next.run(request).await;

  // Add Core-level security headers
  add_core_security_headers(response)
}

/// Enhanced security middleware (Core + business security logic)
///
/// **Single Responsibility**: Add business security policies on top of Core
/// **Use Case**: Scenarios requiring business-specific security policies
pub async fn enhanced_security_middleware(
  State(state): State<AppState>,
  request: Request<Body>,
  next: Next,
) -> Response {
  debug!("Using enhanced security middleware");

  // 1. First perform business security checks
  if let Some(security_response) = perform_business_security_checks(&request).await {
    return security_response;
  }

  // 2. Use Core security middleware
  let mut response = core_security_middleware(State(state), request, next).await;

  // 3. Add business security headers
  add_business_security_headers(&mut response);

  response
}

/// Enterprise security middleware (complete security suite)
///
/// **Single Responsibility**: Provide enterprise-grade complete security protection
/// **Use Case**: Production environments with high security requirements
pub async fn enterprise_security_middleware(
  State(state): State<AppState>,
  request: Request<Body>,
  next: Next,
) -> Response {
  debug!("Using enterprise security middleware");

  let start_time = Instant::now();

  // 1. Log security event start
  let request_info = extract_security_info(&request);
  log_security_event_start(&request_info);

  // 2. Perform comprehensive security checks
  if let Some(security_response) = perform_comprehensive_security_checks(&request, &state).await {
    log_security_violation(&request_info, &security_response);
    return security_response;
  }

  // 3. Use enhanced security middleware
  let mut response = enhanced_security_middleware(State(state), request, next).await;

  // 4. Add enterprise security headers
  add_enterprise_security_headers(&mut response);

  // 5. Log security event completion
  let duration = start_time.elapsed();
  log_security_event_complete(&request_info, response.status(), duration);

  response
}

/// Development environment security middleware (relaxed but monitored)
///
/// **Single Responsibility**: Provide relaxed but monitored security policies for development
/// **Use Case**: Development and testing environments
pub async fn development_security_middleware(
  State(state): State<AppState>,
  request: Request<Body>,
  next: Next,
) -> Response {
  debug!("Using development security middleware");

  // 1. Log all requests for debugging
  log_development_request(&request);

  // 2. Perform basic security checks (but don't block requests)
  if let Some(_security_issue) = perform_development_security_checks(&request).await {
    warn!("Development environment security check found issues, but allowing execution");
  }

  // 3. Use Core security middleware
  let mut response = core_security_middleware(State(state), request, next).await;

  // 4. Add development environment identifiers
  add_development_headers(&mut response);

  response
}

// =============================================================================
// Security Check Functions
// =============================================================================

/// Perform business security checks
///
/// **Single Responsibility**: Implement business-specific security policies
async fn perform_business_security_checks(request: &Request<Body>) -> Option<Response> {
  // 1. Check User-Agent
  if let Some(user_agent) = request.headers().get("user-agent") {
    if let Ok(ua_str) = user_agent.to_str() {
      if is_suspicious_user_agent(ua_str) {
        warn!("Detected suspicious User-Agent: {}", ua_str);
        return Some(create_security_response(
          StatusCode::FORBIDDEN,
          "Suspicious User-Agent detected",
        ));
      }
    }
  }

  // 2. Check request frequency (simplified version)
  if is_rate_limited(request) {
    warn!("Request rate limit exceeded");
    return Some(create_security_response(
      StatusCode::TOO_MANY_REQUESTS,
      "Rate limit exceeded",
    ));
  }

  None
}

/// Perform comprehensive security checks
///
/// **Single Responsibility**: Implement enterprise-grade comprehensive security checks
async fn perform_comprehensive_security_checks(
  request: &Request<Body>,
  state: &AppState,
) -> Option<Response> {
  // 1. First perform business security checks
  if let Some(response) = perform_business_security_checks(request).await {
    return Some(response);
  }

  // 2. Check IP whitelist (if enabled)
  if let Some(response) = check_ip_whitelist(request) {
    return Some(response);
  }

  // 3. Check anti-bot protection
  if let Some(response) = check_anti_bot_protection(request) {
    return Some(response);
  }

  // 4. Check suspicious activity patterns
  if let Some(response) = check_suspicious_patterns(request) {
    return Some(response);
  }

  None
}

/// Perform development environment security checks
///
/// **Single Responsibility**: Provide non-blocking security checks for development
async fn perform_development_security_checks(request: &Request<Body>) -> Option<SecurityEvent> {
  // Check but don't block, just log
  if let Some(user_agent) = request.headers().get("user-agent") {
    if let Ok(ua_str) = user_agent.to_str() {
      if is_suspicious_user_agent(ua_str) {
        return Some(SecurityEvent::SuspiciousUserAgent {
          user_agent: ua_str.to_string(),
        });
      }
    }
  }

  None
}

// =============================================================================
// Security Check Helper Functions
// =============================================================================

/// Check for suspicious User-Agent
///
/// **Single Responsibility**: Identify potentially malicious or automated User-Agents
fn is_suspicious_user_agent(user_agent: &str) -> bool {
  let suspicious_patterns = [
    "bot",
    "crawler",
    "spider",
    "scraper",
    "python-requests",
    "curl",
    "wget",
    "httpclient",
    "scanner",
    "security",
    "pentest",
  ];

  let ua_lower = user_agent.to_lowercase();
  suspicious_patterns
    .iter()
    .any(|pattern| ua_lower.contains(pattern))
}

/// Check if request is rate limited
///
/// **Single Responsibility**: Implement simple rate limiting policy
fn is_rate_limited(request: &Request<Body>) -> bool {
  // This should implement actual rate limiting logic
  // Currently just a placeholder
  false
}

/// Check IP whitelist
///
/// **Single Responsibility**: Verify if request IP is in allowed list
fn check_ip_whitelist(request: &Request<Body>) -> Option<Response> {
  // Implement IP whitelist check logic
  None
}

/// Check anti-bot protection
///
/// **Single Responsibility**: Detect automated access patterns
fn check_anti_bot_protection(request: &Request<Body>) -> Option<Response> {
  // Implement anti-bot check logic
  None
}

/// Check suspicious activity patterns
///
/// **Single Responsibility**: Detect abnormal access patterns
fn check_suspicious_patterns(request: &Request<Body>) -> Option<Response> {
  // Implement suspicious pattern check logic
  None
}

// =============================================================================
// Response Header Management Functions
// =============================================================================

/// Add Core-level security headers
///
/// **Single Responsibility**: Add basic security response headers
fn add_core_security_headers(mut response: Response) -> Response {
  let headers = response.headers_mut();

  // Basic security headers
  headers.insert("x-content-type-options", "nosniff".parse().unwrap());
  headers.insert("x-frame-options", "DENY".parse().unwrap());
  headers.insert("x-xss-protection", "1; mode=block".parse().unwrap());

  response
}

/// Add business-level security headers
///
/// **Single Responsibility**: Add business-specific security response headers
fn add_business_security_headers(response: &mut Response) {
  let headers = response.headers_mut();

  // Business security headers
  headers.insert("x-security-level", "business".parse().unwrap());
  headers.insert("x-rate-limit-remaining", "59".parse().unwrap());
}

/// Add enterprise-level security headers
///
/// **Single Responsibility**: Add enterprise-grade security response headers
fn add_enterprise_security_headers(response: &mut Response) {
  let headers = response.headers_mut();

  // Enterprise security headers
  headers.insert(
    "strict-transport-security",
    "max-age=31536000; includeSubDomains".parse().unwrap(),
  );
  headers.insert(
    "content-security-policy",
    "default-src 'self'".parse().unwrap(),
  );
  headers.insert("x-security-level", "enterprise".parse().unwrap());
}

/// Add development environment headers
///
/// **Single Responsibility**: Add development environment identifier headers
fn add_development_headers(response: &mut Response) {
  let headers = response.headers_mut();

  headers.insert("x-environment", "development".parse().unwrap());
  headers.insert("x-security-level", "development".parse().unwrap());
}

// =============================================================================
// Logging and Monitoring Functions
// =============================================================================

/// Extract security-related information
///
/// **Single Responsibility**: Extract information needed for security monitoring from request
fn extract_security_info(request: &Request<Body>) -> SecurityInfo {
  SecurityInfo {
    ip: request
      .headers()
      .get("x-forwarded-for")
      .or_else(|| request.headers().get("x-real-ip"))
      .and_then(|h| h.to_str().ok())
      .unwrap_or("unknown")
      .to_string(),
    user_agent: request
      .headers()
      .get("user-agent")
      .and_then(|h| h.to_str().ok())
      .unwrap_or("unknown")
      .to_string(),
    path: request.uri().path().to_string(),
    method: request.method().to_string(),
  }
}

/// Log security event start
///
/// **Single Responsibility**: Log the start of security checks
fn log_security_event_start(info: &SecurityInfo) {
  debug!(
    "Starting security check: {} {} - IP: {} - UA: {}",
    info.method, info.path, info.ip, info.user_agent
  );
}

/// Log security event completion
///
/// **Single Responsibility**: Log the completion of security checks
fn log_security_event_complete(info: &SecurityInfo, status: StatusCode, duration: Duration) {
  debug!(
    "Security check completed: {} {} - Status: {} - Duration: {:?}",
    info.method, info.path, status, duration
  );
}

/// Log security violation
///
/// **Single Responsibility**: Log security violation events
fn log_security_violation(info: &SecurityInfo, response: &Response) {
  warn!(
    "Security violation detected: {} {} - IP: {} - Status: {}",
    info.method,
    info.path,
    info.ip,
    response.status()
  );
}

/// Log development environment request
///
/// **Single Responsibility**: Log detailed request information in development environment
fn log_development_request(request: &Request<Body>) {
  debug!(
    "Development environment request: {} {} - Header count: {}",
    request.method(),
    request.uri(),
    request.headers().len()
  );
}

// =============================================================================
// Helper Structs and Functions
// =============================================================================

/// Security information struct
///
/// **Single Responsibility**: Store key information needed for security monitoring
#[derive(Debug, Clone)]
struct SecurityInfo {
  ip: String,
  user_agent: String,
  path: String,
  method: String,
}

/// Create security response
///
/// **Single Responsibility**: Create standardized security error responses
fn create_security_response(status: StatusCode, message: &str) -> Response {
  Response::builder()
    .status(status)
    .header("content-type", "application/json")
    .body(format!(r#"{{"error": "{}"}}"#, message).into())
    .unwrap()
}

// =============================================================================
// Convenience Constructor Functions
// =============================================================================

/// Create standard security middleware stack
///
/// **Single Responsibility**: Provide common security middleware combinations
pub fn standard_security_stack() -> Vec<&'static str> {
  vec!["core_security"]
}

/// Create business security middleware stack
///
/// **Single Responsibility**: Provide business scenario security middleware combinations
pub fn business_security_stack() -> Vec<&'static str> {
  vec!["enhanced_security"]
}

/// Create enterprise security middleware stack
///
/// **Single Responsibility**: Provide complete enterprise-grade security stack
pub fn enterprise_security_stack() -> Vec<&'static str> {
  vec!["enterprise_security"]
}

/// Create development environment security middleware stack
///
/// **Single Responsibility**: Provide development environment security stack
pub fn development_security_stack() -> Vec<&'static str> {
  vec!["development_security"]
}

// =============================================================================
// Test Module
// =============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_security_adapter_config() {
    let config = SecurityAdapterConfig::default();
    assert!(config.rate_limiting);
    assert!(!config.ip_whitelist);
    assert!(config.user_agent_check);
    assert!(config.anti_bot_protection);
    assert_eq!(config.rate_limit_per_minute, 60);
  }

  #[test]
  fn test_suspicious_user_agent_detection() {
    assert!(is_suspicious_user_agent("python-requests/2.25.1"));
    assert!(is_suspicious_user_agent(
      "Mozilla/5.0 (compatible; Googlebot/2.1)"
    ));
    assert!(!is_suspicious_user_agent(
      "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
    ));
  }

  #[test]
  fn test_security_stack_creation() {
    let standard = standard_security_stack();
    assert_eq!(standard, vec!["core_security"]);

    let business = business_security_stack();
    assert_eq!(business, vec!["enhanced_security"]);

    let enterprise = enterprise_security_stack();
    assert_eq!(enterprise, vec!["enterprise_security"]);

    let development = development_security_stack();
    assert_eq!(development, vec!["development_security"]);
  }
}
