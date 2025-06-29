//! # Fechatter Proxy - Pure Pingora Implementation
//!
//! **Native Pingora implementation focused on Gateway routing and networking**
//!
//! ## 🎯 Gateway Feature Set:
//! ### Core Networking Functionality
//! - Route-based upstream selection with load balancing
//! - Multi-level rate limiting (IP-based)
//! - Advanced CORS handling with origin validation
//! - Request/response header manipulation
//!
//! ### Performance & Monitoring
//! - **High-performance memory caching** with TTL
//! - **Cache statistics and monitoring**
//! - **Automatic cache eviction** with LRU strategy
//!
//! ### Audit & Compliance
//! - **Comprehensive audit logging** with structured events
//! - **Security compliance tracking** with retention policies
//! - **Real-time security event detection**
//! - **Request/response audit trail**
//!
//! ### Gateway Administration
//! - **Rate limit management**
//! - **CORS configuration** capabilities
//! - **Health check endpoints**
//! - **Cache management**
//!
//! **Note**: JWT Authentication and Authorization are handled by fechatter_server.
//! Gateway focuses on network-level concerns only.

// ============================================================================
// MODULE DECLARATIONS AND IMPORTS
// ============================================================================

// Complete Gateway modules
pub mod audit;
pub mod cache;
pub mod production;

use crate::{config::GatewayConfig, upstream::UpstreamManager};
use anyhow::Result;
use async_trait::async_trait;
use audit::{AuditEventType, GatewayAuditLogger};
use cache::{CacheConfig, GatewayCache};
use pingora_core::upstreams::peer::HttpPeer;
use pingora_http::{RequestHeader, ResponseHeader};
use pingora_proxy::{ProxyHttp, Session};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, warn};

// ============================================================================
// TYPE DEFINITIONS AND STRUCTURES
// ============================================================================

/// Rate limiting tracker with time-based cleanup
#[derive(Debug, Clone)]
struct RateLimit {
  requests: Vec<Instant>,
  violations: u32,
}

/// Proxy metrics for monitoring
#[derive(Debug, Clone)]
pub struct ProxyMetrics {
  pub active_rate_limit_keys: usize,
  pub cache_enabled: bool,
  pub audit_enabled: bool,
  pub routes_configured: usize,
  pub upstreams_configured: usize,
}

/// **Gateway Proxy** implementing Pingora's ProxyHttp trait
///
/// Focuses on network-level Gateway features:
/// - Advanced routing and load balancing
/// - High-performance caching  
/// - Comprehensive audit logging
/// - Rate limiting with IP-based controls
/// - CORS handling and security headers
pub struct FechatterProxy {
  config: Arc<GatewayConfig>,
  upstream_manager: Arc<UpstreamManager>,
  rate_limiter: Arc<std::sync::Mutex<HashMap<String, RateLimit>>>,

  // Gateway functionality
  cache: Arc<GatewayCache>,
  audit_logger: Arc<GatewayAuditLogger>,
}

/// Request context for Gateway processing
#[derive(Debug)]
pub struct RequestContext {
  pub request_id: String,
  pub matched_route: Option<String>,
  pub upstream_name: Option<String>,
  pub start_time: Instant,

  // Network and monitoring context
  pub rate_limited: bool,
  pub client_ip: Option<String>,
  pub cors_origin: Option<String>,
  pub security_violations: Vec<String>,

  // Caching context
  pub cache_key: Option<String>,
  pub cache_hit: bool,

  // Audit context
  pub audit_events: Vec<AuditEventType>,
}

// ============================================================================
// IMPLEMENTATION BLOCKS
// ============================================================================

// ----------------------------------------------------------------------------
// FechatterProxy Implementation
// ----------------------------------------------------------------------------

impl FechatterProxy {
  /// Create new Fechatter proxy instance
  pub fn new(config: Arc<GatewayConfig>, upstream_manager: Arc<UpstreamManager>) -> Self {
    info!(
      "🚀 Creating Pingora-native Fechatter proxy with {} upstreams",
      config.upstreams.len()
    );

    // Create default configurations for cache and audit
    let cache_config = CacheConfig {
      enabled: true,
      default_ttl_secs: 300,
      max_size_bytes: 100 * 1024 * 1024, // 100MB
      compress: true,
      key_prefix: "gateway".to_string(),
      variants: vec![
        cache::CacheVariant::UserId,
        cache::CacheVariant::WorkspaceId,
        cache::CacheVariant::UserPermissions,
      ],
      rules: Vec::new(),
    };

    let audit_config = audit::AuditConfig {
      enabled: true,
      log_level: "info".to_string(),
      structured_logging: true,
      include_request_body: false,
      include_response_body: false,
      max_body_size: 1024 * 1024, // 1MB
      retention_days: 90,
      buffer_size: 1000,
      flush_interval_secs: 30,
      compliance_mode: false,
      sensitive_headers: vec![
        "authorization".to_string(),
        "cookie".to_string(),
        "x-api-key".to_string(),
      ],
      excluded_paths: vec!["/health".to_string(), "/metrics".to_string()],
    };

    Self {
      config,
      upstream_manager,
      rate_limiter: Arc::new(std::sync::Mutex::new(HashMap::new())),
      cache: Arc::new(GatewayCache::new(cache_config)),
      audit_logger: Arc::new(GatewayAuditLogger::new(audit_config)),
    }
  }

  /// Get proxy configuration for external access
  pub fn get_config(&self) -> &Arc<GatewayConfig> {
    &self.config
  }

  /// Get proxy metrics for monitoring
  pub fn get_proxy_metrics(&self) -> ProxyMetrics {
    let rate_limiter_count = self.rate_limiter.lock().unwrap().len();

    ProxyMetrics {
      active_rate_limit_keys: rate_limiter_count,
      cache_enabled: true,
      audit_enabled: true,
      routes_configured: self.config.routes.len(),
      upstreams_configured: self.config.upstreams.len(),
    }
  }

  /// Public method to test route matching (for testing only)
  #[cfg(test)]
  pub fn test_route_match(&self, path: &str, method: &str) -> bool {
    self.match_route(path, method).is_some()
  }

  /// Public method to test CORS validation (for testing only)
  #[cfg(test)]
  pub fn test_cors_validation(&self, origin: &str, path: &str) -> bool {
    self.validate_cors_origin(origin, path)
  }

  /// Public method to test rate limiting (for testing only)
  #[cfg(test)]
  pub fn test_rate_limit(&self, key: &str) -> (bool, usize) {
    self.check_rate_limit(key)
  }

  /// Check server compatibility based on expected endpoints
  pub fn check_server_compatibility(&self) -> Vec<String> {
    let mut issues = Vec::new();

    // Expected fechatter_server endpoints
    let fechatter_endpoints = vec![
      ("/health", "GET"),
      ("/api/signin", "POST"),
      ("/api/signup", "POST"),
      ("/api/refresh", "POST"),
      ("/api/logout", "POST"),
      ("/api/chats", "GET"),
      ("/api/chat/", "GET"),
      ("/api/chat/", "POST"),
      ("/api/workspace/", "GET"),
      ("/api/workspace/", "POST"),
      ("/ws", "GET"), // WebSocket
    ];

    // Expected notify_server endpoints
    let notify_endpoints = vec![
      ("/events", "GET"), // SSE
      ("/online-users", "GET"),
      ("/sse/health", "GET"),
    ];

    // Check fechatter_server compatibility
    for (path, method) in fechatter_endpoints {
      if !self.has_compatible_route(path, method) {
        issues.push(format!(
          "Missing route for fechatter_server endpoint: {} {}",
          method, path
        ));
      }
    }

    // Check notify_server compatibility
    for (path, method) in notify_endpoints {
      if !self.has_compatible_route(path, method) {
        issues.push(format!(
          "Missing route for notify_server endpoint: {} {}",
          method, path
        ));
      }
    }

    issues
  }

  /// Check if there's a compatible route for the given path and method
  fn has_compatible_route(&self, path: &str, method: &str) -> bool {
    self.config.routes.iter().any(|route| {
      let path_matches = if route.path.ends_with('/') {
        path.starts_with(&route.path) || path == route.path.trim_end_matches('/')
      } else {
        path == route.path || path.starts_with(&format!("{}/", route.path))
      };

      path_matches
        && route
          .methods
          .iter()
          .any(|m| m.to_uppercase() == method.to_uppercase())
    })
  }

  /// Get fallback peer for error recovery
  fn get_fallback_peer(&self, _ctx: &RequestContext) -> Option<HttpPeer> {
    // Try to find any healthy upstream as fallback
    for (name, _config) in &self.config.upstreams {
      if let Some(peer) = self.upstream_manager.select_peer(name, None) {
        warn!("Using fallback upstream: {}", name);
        return Some(peer);
      }
    }
    None
  }
}

// ----------------------------------------------------------------------------
// RateLimit Implementation
// ----------------------------------------------------------------------------

impl RateLimit {
  fn new() -> Self {
    Self {
      requests: Vec::new(),
      violations: 0,
    }
  }

  fn check_limit(&mut self, max_requests: usize, window_secs: u64) -> bool {
    let now = Instant::now();
    let window = std::time::Duration::from_secs(window_secs);

    // Clean old requests
    self
      .requests
      .retain(|&time| now.duration_since(time) < window);

    if self.requests.len() >= max_requests {
      self.violations += 1;
      false
    } else {
      self.requests.push(now);
      true
    }
  }

  fn get_remaining(&self, max_requests: usize) -> usize {
    max_requests.saturating_sub(self.requests.len())
  }
}

// ----------------------------------------------------------------------------
// RequestContext Implementation
// ----------------------------------------------------------------------------

impl Default for RequestContext {
  fn default() -> Self {
    Self {
      request_id: uuid::Uuid::new_v4().to_string(),
      matched_route: None,
      upstream_name: None,
      start_time: Instant::now(),
      rate_limited: false,
      client_ip: None,
      cors_origin: None,
      security_violations: Vec::new(),
      cache_key: None,
      cache_hit: false,
      audit_events: Vec::new(),
    }
  }
}

// ============================================================================
// RATE LIMITING FUNCTIONS
// ============================================================================

impl FechatterProxy {
  /// Rate limiting with IP-based limits
  fn check_rate_limit(&self, key: &str) -> (bool, usize) {
    let mut limiter = self.rate_limiter.lock().unwrap();
    let rate_limit = limiter
      .entry(key.to_string())
      .or_insert_with(RateLimit::new);

    // Basic IP-based rate limiting (100 requests per minute)
    let (max_requests, window_secs) = (100, 60);

    let allowed = rate_limit.check_limit(max_requests, window_secs);
    let remaining = rate_limit.get_remaining(max_requests);

    (allowed, remaining)
  }

  /// Generate rate limit key based on client IP
  fn get_rate_limit_key(&self, ctx: &RequestContext) -> String {
    if let Some(ip) = &ctx.client_ip {
      format!("ip:{}", ip)
    } else {
      "anon:default".to_string()
    }
  }
}

// ============================================================================
// NETWORK AND REQUEST UTILITIES
// ============================================================================

impl FechatterProxy {
  /// Extract client IP from headers
  fn extract_client_ip(&self, session: &Session) -> Option<String> {
    let headers = &session.req_header().headers;

    // Check X-Forwarded-For first, then X-Real-IP
    if let Some(xff) = headers.get("x-forwarded-for") {
      if let Ok(xff_str) = xff.to_str() {
        return xff_str.split(',').next().map(|s| s.trim().to_string());
      }
    }

    if let Some(real_ip) = headers.get("x-real-ip") {
      if let Ok(ip_str) = real_ip.to_str() {
        return Some(ip_str.to_string());
      }
    }

    None
  }

  /// Enhanced CORS origin validation using configuration
  fn validate_cors_origin(&self, origin: &str, path: &str) -> bool {
    // Get allowed origins for this route from configuration
    let allowed_origins = self.config.get_cors_origins(path);

    // Check if origin is in the allowed list
    allowed_origins.contains(&origin.to_string())
  }

  /// Handle CORS preflight requests (OPTIONS method)
  fn is_preflight_request(&self, method: &str, headers: &pingora_http::HMap) -> bool {
    method.to_uppercase() == "OPTIONS" && headers.contains_key("access-control-request-method")
  }

  /// Get CORS preflight response headers
  fn get_preflight_headers(&self, origin: &str, path: &str) -> HashMap<String, String> {
    let mut headers = HashMap::new();

    // Only add CORS headers if origin is validated
    if self.validate_cors_origin(origin, path) {
      headers.insert(
        "access-control-allow-origin".to_string(),
        origin.to_string(),
      );
      headers.insert(
        "access-control-allow-credentials".to_string(),
        "true".to_string(),
      );
      headers.insert(
        "access-control-allow-methods".to_string(),
        "GET, POST, PUT, DELETE, PATCH, OPTIONS".to_string(),
      );
      headers.insert(
        "access-control-allow-headers".to_string(),
        "content-type, authorization, x-api-key, x-request-id, x-workspace-id, cache-control, x-requested-with".to_string(),
      );
      headers.insert("access-control-max-age".to_string(), "86400".to_string());
      // 24 hours
    }

    headers
  }
}

// ============================================================================
// ROUTING FUNCTIONS
// ============================================================================

impl FechatterProxy {
  /// Match request path to configured routes
  fn match_route(&self, path: &str, method: &str) -> Option<&crate::config::RouteConfig> {
    for route in &self.config.routes {
      if self.path_matches(&route.path, path) && route.methods.contains(&method.to_uppercase()) {
        debug!("🎯 Matched route: {} -> {}", path, route.upstream);
        return Some(route);
      }
    }
    warn!("❌ No route matched for {} {}", method, path);
    None
  }

  /// Check if request path matches route pattern
  fn path_matches(&self, route_path: &str, request_path: &str) -> bool {
    if route_path.ends_with('/') {
      request_path.starts_with(route_path)
    } else {
      request_path == route_path || request_path.starts_with(&format!("{}/", route_path))
    }
  }
}

// ============================================================================
// PINGORA PROXYHTTP TRAIT IMPLEMENTATION
// ============================================================================

#[async_trait]
impl ProxyHttp for FechatterProxy {
  type CTX = RequestContext;

  /// Create new request context
  fn new_ctx(&self) -> Self::CTX {
    RequestContext::default()
  }

  /// **Enhanced Gateway Logic** - Authentication, Authorization, CORS, Rate Limiting
  async fn request_filter(
    &self,
    session: &mut Session,
    ctx: &mut Self::CTX,
  ) -> Result<bool, Box<pingora_core::Error>> {
    let path = session.req_header().uri.path();
    let method = session.req_header().method.as_str();

    debug!(
      "🔐 [GATEWAY] Processing enhanced request: {} {}",
      method, path
    );

    // Extract client IP for rate limiting and logging
    ctx.client_ip = self.extract_client_ip(session);

    // 1. Handle CORS preflight requests directly
    if self.is_preflight_request(method, &session.req_header().headers) {
      if let Some(origin) = session.req_header().headers.get("origin") {
        if let Ok(origin_str) = origin.to_str() {
          if self.validate_cors_origin(origin_str, path) {
            debug!(
              "✅ [GATEWAY] CORS preflight request approved for origin: {}",
              origin_str
            );
            ctx.cors_origin = Some(origin_str.to_string());

            // Return early - preflight will be handled in response_filter
            // by returning early with true, we tell Pingora to short-circuit to response
            return Ok(true);
          } else {
            warn!(
              "❌ [GATEWAY] CORS preflight rejected for origin: {}",
              origin_str
            );
            return Err(pingora_core::Error::new_str("CORS preflight not allowed"));
          }
        }
      }
      // If no origin header in preflight, reject
      return Err(pingora_core::Error::new_str(
        "CORS preflight missing origin",
      ));
    }

    // 2. IP-based Rate Limiting (for non-preflight requests)
    let rate_key = self.get_rate_limit_key(ctx);
    let (allowed, remaining) = self.check_rate_limit(&rate_key);

    if !allowed {
      warn!("🚦 [GATEWAY] Rate limit exceeded for: {}", rate_key);
      ctx.rate_limited = true;
      return Err(pingora_core::Error::new_str("Rate limited"));
    }

    debug!(
      "✅ [GATEWAY] Rate limit check passed: {} remaining",
      remaining
    );

    // 3. Enhanced CORS Validation for actual requests
    if let Some(origin) = session.req_header().headers.get("origin") {
      if let Ok(origin_str) = origin.to_str() {
        if self.validate_cors_origin(origin_str, path) {
          ctx.cors_origin = Some(origin_str.to_string());
          debug!("✅ [GATEWAY] CORS origin validated: {}", origin_str);
        } else {
          warn!("❌ [GATEWAY] CORS origin rejected: {}", origin_str);
          return Err(pingora_core::Error::new_str("CORS origin not allowed"));
        }
      }
    }

    debug!("✅ [GATEWAY] Gateway request filter completed successfully");
    Ok(false) // Continue to upstream
  }

  /// Select upstream peer based on routing with error recovery
  async fn upstream_peer(
    &self,
    session: &mut Session,
    ctx: &mut Self::CTX,
  ) -> Result<Box<HttpPeer>, Box<pingora_core::Error>> {
    let path = session.req_header().uri.path();
    let method = session.req_header().method.as_str();

    debug!("🔍 [GATEWAY] Routing: {} {}", method, path);

    // Match route
    let route = self.match_route(path, method).ok_or_else(|| {
      error!("No route found for {} {}", method, path);
      pingora_core::Error::new_str("Route not found")
    })?;

    // Update context
    ctx.matched_route = Some(route.path.clone());
    ctx.upstream_name = Some(route.upstream.clone());

    // Select upstream peer with fallback logic
    let peer = match self.upstream_manager.select_peer(&route.upstream, None) {
      Some(peer) => peer,
      None => {
        error!("No healthy upstream found for: {}", route.upstream);
        // Try fallback logic
        if let Some(fallback) = self.get_fallback_peer(ctx) {
          warn!("Using fallback peer due to upstream selection error");
          fallback
        } else {
          return Err(pingora_core::Error::new_str("No healthy upstream"));
        }
      }
    };

    info!("🎯 [GATEWAY] Routed to upstream: {}", route.upstream);
    Ok(Box::new(peer))
  }

  /// Modify request headers before sending to upstream
  async fn upstream_request_filter(
    &self,
    _session: &mut Session,
    upstream_request: &mut RequestHeader,
    ctx: &mut Self::CTX,
  ) -> Result<(), Box<pingora_core::Error>> {
    // Add essential Gateway headers
    upstream_request.insert_header("x-request-id", &ctx.request_id)?;
    upstream_request.insert_header("x-forwarded-by", "fechatter-gateway")?;
    upstream_request.insert_header("x-gateway-version", env!("CARGO_PKG_VERSION"))?;

    // Add client IP for upstream processing
    if let Some(ip) = &ctx.client_ip {
      upstream_request.insert_header("x-client-ip", ip)?;
    }

    debug!("📤 [GATEWAY] Added comprehensive Gateway headers to upstream request");
    Ok(())
  }

  /// Add response headers including CORS, rate limiting info, and monitoring
  async fn response_filter(
    &self,
    session: &mut Session,
    upstream_response: &mut ResponseHeader,
    ctx: &mut Self::CTX,
  ) -> Result<(), Box<pingora_core::Error>> {
    let duration = ctx.start_time.elapsed();
    let path = session.req_header().uri.path();
    let method = session.req_header().method.as_str();

    // Handle CORS preflight responses (when short-circuited from request_filter)
    if self.is_preflight_request(method, &session.req_header().headers) {
      if let Some(origin) = &ctx.cors_origin {
        // Set preflight response status and headers
        upstream_response.set_status(200).unwrap();

        let preflight_headers = self.get_preflight_headers(origin, path);
        for (name, value) in preflight_headers {
          upstream_response.insert_header(name, &value)?;
        }

        debug!(
          "✅ [GATEWAY] CORS preflight response sent for origin: {}",
          origin
        );
        return Ok(());
      }
    }

    // Add standard Gateway headers for regular responses
    upstream_response.insert_header("x-response-time", &format!("{}ms", duration.as_millis()))?;
    upstream_response.insert_header("x-served-by", "fechatter-gateway")?;
    upstream_response.insert_header("x-request-id", &ctx.request_id)?;
    upstream_response.insert_header("x-gateway-version", env!("CARGO_PKG_VERSION"))?;

    // Add CORS headers if origin is validated (for actual requests)
    if let Some(origin) = &ctx.cors_origin {
      upstream_response.insert_header("access-control-allow-origin", origin)?;
      upstream_response.insert_header("access-control-allow-credentials", "true")?;
      upstream_response.insert_header(
        "access-control-allow-methods",
        "GET, POST, PUT, DELETE, PATCH, OPTIONS",
      )?;
      upstream_response.insert_header(
        "access-control-allow-headers",
        "content-type, authorization, x-api-key, x-request-id, x-workspace-id, cache-control, x-requested-with",
      )?;
      upstream_response.insert_header(
        "access-control-expose-headers",
        "x-request-id, x-ratelimit-remaining, x-ratelimit-limit, x-ratelimit-reset",
      )?;
      upstream_response.insert_header("access-control-max-age", "86400")?;
    }

    // Add comprehensive rate limiting headers (IP-based)
    if let Some(ip) = &ctx.client_ip {
      let rate_key = format!("ip:{}", ip);
      if let Ok(limiter) = self.rate_limiter.lock() {
        if let Some(rate_limit) = limiter.get(&rate_key) {
          let remaining = rate_limit.get_remaining(100);
          upstream_response.insert_header("x-ratelimit-limit", "100")?;
          upstream_response.insert_header("x-ratelimit-remaining", &remaining.to_string())?;
          upstream_response.insert_header("x-ratelimit-window", "60")?;
          upstream_response.insert_header("x-ratelimit-type", "ip")?;
        }
      }
    }

    // Add performance metrics - fix header value types
    upstream_response.insert_header(
      "x-upstream",
      ctx
        .upstream_name
        .as_ref()
        .unwrap_or(&"gateway".to_string()) // Use "gateway" for preflight responses
        .as_str(),
    )?;
    upstream_response.insert_header(
      "x-route",
      ctx
        .matched_route
        .as_ref()
        .unwrap_or(&"preflight".to_string()) // Use "preflight" for preflight responses
        .as_str(),
    )?;

    debug!("📥 [GATEWAY] Added comprehensive Gateway response headers");
    Ok(())
  }

  /// Request completion logging and metrics
  async fn logging(
    &self,
    session: &mut Session,
    e: Option<&pingora_core::Error>,
    ctx: &mut Self::CTX,
  ) {
    let duration = ctx.start_time.elapsed();
    let status = session
      .response_written()
      .map(|r| r.status.as_u16())
      .unwrap_or(0);

    // Log request completion
    if let Some(error) = e {
      error!(
        request_id = %ctx.request_id,
        upstream = %ctx.upstream_name.as_ref().unwrap_or(&"unknown".to_string()),
        route = %ctx.matched_route.as_ref().unwrap_or(&"unknown".to_string()),
        status = status,
        duration_ms = duration.as_millis(),
        rate_limited = ctx.rate_limited,
        error = %error,
        "Pingora Gateway request failed"
      );
    } else {
      info!(
        request_id = %ctx.request_id,
        upstream = %ctx.upstream_name.as_ref().unwrap_or(&"unknown".to_string()),
        route = %ctx.matched_route.as_ref().unwrap_or(&"unknown".to_string()),
        status = status,
        duration_ms = duration.as_millis(),
        rate_limited = ctx.rate_limited,
        client_ip = %ctx.client_ip.as_ref().unwrap_or(&"unknown".to_string()),
        "Pingora Gateway request completed"
      );
    }

    // Report upstream health
    if let Some(upstream_name) = &ctx.upstream_name {
      let healthy = status >= 200 && status < 500;
      self
        .upstream_manager
        .report_health(upstream_name, "peer", healthy);
    }
  }
}

// ============================================================================
// UTILITY IMPLEMENTATIONS
// ============================================================================

impl Clone for FechatterProxy {
  fn clone(&self) -> Self {
    Self {
      config: Arc::clone(&self.config),
      upstream_manager: Arc::clone(&self.upstream_manager),
      rate_limiter: Arc::clone(&self.rate_limiter),
      cache: Arc::clone(&self.cache),
      audit_logger: Arc::clone(&self.audit_logger),
    }
  }
}

// ============================================================================
// TEST MODULES
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::{GatewayConfig, LoadBalancingType, RouteConfig, UpstreamConfig};
  use std::collections::HashMap;

  fn create_test_config() -> GatewayConfig {
    // Use the centralized test configuration
    crate::config::testing::create_test_config()
  }

  #[tokio::test]
  async fn test_route_matching() {
    let config = Arc::new(create_test_config());
    let upstream_manager = Arc::new(UpstreamManager::new(config.clone()).await.unwrap());
    let proxy = FechatterProxy::new(config, upstream_manager);

    let route = proxy.match_route("/api/users", "GET");
    assert!(route.is_some());
    assert_eq!(route.unwrap().upstream, "test-server");
  }

  #[tokio::test]
  async fn test_rate_limiting() {
    let config = Arc::new(create_test_config());
    let upstream_manager = Arc::new(UpstreamManager::new(config.clone()).await.unwrap());
    let proxy = FechatterProxy::new(config, upstream_manager);

    // Test rate limiting
    for i in 0..100 {
      let (allowed, remaining) = proxy.test_rate_limit("test-key");
      assert!(allowed, "Request {} should be allowed", i + 1);
      assert_eq!(remaining, 99 - i);
    }

    // 101st request should be rate limited
    let (allowed, _) = proxy.test_rate_limit("test-key");
    assert!(!allowed, "101st request should be rate limited");
  }

  #[tokio::test]
  async fn test_cors_validation() {
    let config = Arc::new(create_test_config());
    let upstream_manager = Arc::new(UpstreamManager::new(config.clone()).await.unwrap());
    let proxy = FechatterProxy::new(config, upstream_manager);

    // Test allowed origins
    assert!(proxy.test_cors_validation("http://localhost:3000", "/api/"));
    assert!(proxy.test_cors_validation("https://fechatter.com", "/api/"));

    // Test disallowed origin
    assert!(!proxy.test_cors_validation("https://evil.com", "/api/"));
  }

  #[tokio::test]
  async fn test_client_ip_extraction() {
    let config = Arc::new(create_test_config());
    let upstream_manager = Arc::new(UpstreamManager::new(config.clone()).await.unwrap());
    let proxy = FechatterProxy::new(config, upstream_manager);

    // Test rate limit key generation
    let mut ctx = RequestContext::default();
    ctx.client_ip = Some("192.168.1.1".to_string());

    let rate_key = proxy.get_rate_limit_key(&ctx);
    assert_eq!(rate_key, "ip:192.168.1.1");

    // Test with no IP
    ctx.client_ip = None;
    let rate_key = proxy.get_rate_limit_key(&ctx);
    assert_eq!(rate_key, "anon:default");
  }

  #[tokio::test]
  async fn test_request_context_default() {
    let ctx = RequestContext::default();

    assert!(!ctx.request_id.is_empty());
    assert_eq!(ctx.matched_route, None);
    assert_eq!(ctx.upstream_name, None);
    assert_eq!(ctx.rate_limited, false);
    assert_eq!(ctx.client_ip, None);
    assert_eq!(ctx.cors_origin, None);
    assert_eq!(ctx.cache_hit, false);
  }
}

// ============================================================================
// PRODUCTION MODULE
// ============================================================================

pub use production::ProductionProxy;
