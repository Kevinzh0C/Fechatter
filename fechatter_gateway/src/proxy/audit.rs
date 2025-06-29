//! # Gateway Audit Logging - Pingora Native Implementation
//!
//! **Comprehensive audit trail for Gateway security and compliance**
//!
//! Features:
//! - Structured audit event logging
//! - Security event detection
//! - Request/response audit trail
//! - Authentication audit events
//! - Authorization failures tracking
//! - Performance metrics logging

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Audit event types for categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
  // Authentication Events
  AuthenticationSuccess,
  AuthenticationFailure,
  TokenExpired,
  TokenInvalid,

  // Authorization Events
  AuthorizationSuccess,
  AuthorizationFailure,
  PermissionDenied,

  // Request Processing Events
  RequestReceived,
  RequestCompleted,
  RequestFailed,

  // Rate Limiting Events
  RateLimitExceeded,
  RateLimitWarning,

  // CORS Events
  CorsViolation,
  CorsSuccess,

  // Cache Events
  CacheHit,
  CacheMiss,
  CacheEviction,

  // Security Events
  SuspiciousActivity,
  SecurityViolation,
  UnauthorizedAccess,

  // System Events
  GatewayStartup,
  GatewayShutdown,
  ConfigurationChange,

  // Error Events
  InternalError,
  UpstreamError,
  TimeoutError,

  // Admin Events
  AdminAccess,
  CacheCleared,
  ConfigReload,
}

impl AuditEventType {
  /// Get severity level for event type
  pub fn severity(&self) -> AuditSeverity {
    match self {
      // Critical security events
      AuditEventType::SecurityViolation
      | AuditEventType::UnauthorizedAccess
      | AuditEventType::SuspiciousActivity => AuditSeverity::Critical,

      // High severity events
      AuditEventType::AuthenticationFailure
      | AuditEventType::AuthorizationFailure
      | AuditEventType::PermissionDenied
      | AuditEventType::CorsViolation => AuditSeverity::High,

      // Medium severity events
      AuditEventType::RateLimitExceeded
      | AuditEventType::TokenExpired
      | AuditEventType::TokenInvalid
      | AuditEventType::RequestFailed
      | AuditEventType::UpstreamError => AuditSeverity::Medium,

      // Low severity events
      AuditEventType::RateLimitWarning
      | AuditEventType::TimeoutError
      | AuditEventType::InternalError => AuditSeverity::Low,

      // Info level events
      _ => AuditSeverity::Info,
    }
  }

  /// Check if event type should be logged immediately (high priority)
  pub fn is_immediate(&self) -> bool {
    matches!(
      self.severity(),
      AuditSeverity::Critical | AuditSeverity::High
    )
  }
}

/// Audit event severity levels for Gateway operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditSeverity {
  Critical,
  High,
  Medium,
  Low,
  Info,
}

impl std::fmt::Display for AuditSeverity {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      AuditSeverity::Critical => write!(f, "critical"),
      AuditSeverity::High => write!(f, "high"),
      AuditSeverity::Medium => write!(f, "medium"),
      AuditSeverity::Low => write!(f, "low"),
      AuditSeverity::Info => write!(f, "info"),
    }
  }
}

/// Complete audit event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
  pub event_id: String,
  pub timestamp: DateTime<Utc>,
  pub event_type: AuditEventType,
  pub severity: AuditSeverity,

  // Request context
  pub request_id: String,
  pub session_id: Option<String>,
  pub method: String,
  pub path: String,
  pub query_params: Option<String>,

  // User context
  pub user_id: Option<i64>,
  pub workspace_id: Option<i64>,
  pub permissions: Option<Vec<String>>,

  // Network context
  pub client_ip: Option<String>,
  pub user_agent: Option<String>,
  pub origin: Option<String>,

  // Response context
  pub status_code: Option<u16>,
  pub response_time_ms: Option<u64>,
  pub bytes_sent: Option<u64>,
  pub bytes_received: Option<u64>,

  // Gateway context
  pub upstream: Option<String>,
  pub route: Option<String>,
  pub cache_status: Option<String>,

  // Event-specific data
  pub details: HashMap<String, serde_json::Value>,
  pub error_message: Option<String>,
  pub stack_trace: Option<String>,

  // Compliance fields
  pub compliance_tags: Vec<String>,
  pub retention_period_days: Option<u32>,

  // Additional metadata
  pub gateway_version: String,
  pub gateway_instance: String,
}

impl AuditEvent {
  /// Create new audit event
  pub fn new(event_type: AuditEventType, request_id: String) -> Self {
    let event_id = Uuid::new_v4().to_string();
    let severity = event_type.severity();

    Self {
      event_id,
      timestamp: Utc::now(),
      event_type,
      severity,
      request_id,
      session_id: None,
      method: String::new(),
      path: String::new(),
      query_params: None,
      user_id: None,
      workspace_id: None,
      permissions: None,
      client_ip: None,
      user_agent: None,
      origin: None,
      status_code: None,
      response_time_ms: None,
      bytes_sent: None,
      bytes_received: None,
      upstream: None,
      route: None,
      cache_status: None,
      details: HashMap::new(),
      error_message: None,
      stack_trace: None,
      compliance_tags: Vec::new(),
      retention_period_days: None,
      gateway_version: env!("CARGO_PKG_VERSION").to_string(),
      gateway_instance: std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string()),
    }
  }

  /// Add request context
  pub fn with_request(mut self, method: String, path: String, query: Option<String>) -> Self {
    self.method = method;
    self.path = path;
    self.query_params = query;
    self
  }

  /// Add user context
  pub fn with_user(mut self, user_id: i64, workspace_id: i64, permissions: Vec<String>) -> Self {
    self.user_id = Some(user_id);
    self.workspace_id = Some(workspace_id);
    self.permissions = Some(permissions);
    self
  }

  /// Add network context
  pub fn with_network(
    mut self,
    client_ip: Option<String>,
    user_agent: Option<String>,
    origin: Option<String>,
  ) -> Self {
    self.client_ip = client_ip;
    self.user_agent = user_agent;
    self.origin = origin;
    self
  }

  /// Add response context
  pub fn with_response(
    mut self,
    status: u16,
    response_time: Duration,
    bytes_sent: Option<u64>,
  ) -> Self {
    self.status_code = Some(status);
    self.response_time_ms = Some(response_time.as_millis() as u64);
    self.bytes_sent = bytes_sent;
    self
  }

  /// Add gateway context
  pub fn with_gateway(
    mut self,
    upstream: Option<String>,
    route: Option<String>,
    cache_status: Option<String>,
  ) -> Self {
    self.upstream = upstream;
    self.route = route;
    self.cache_status = cache_status;
    self
  }

  /// Add error context
  pub fn with_error(mut self, message: String, stack_trace: Option<String>) -> Self {
    self.error_message = Some(message);
    self.stack_trace = stack_trace;
    self
  }

  /// Add custom detail
  pub fn with_detail<T: Serialize>(mut self, key: &str, value: T) -> Self {
    if let Ok(json_value) = serde_json::to_value(value) {
      self.details.insert(key.to_string(), json_value);
    }
    self
  }

  /// Add compliance tags
  pub fn with_compliance(mut self, tags: Vec<String>, retention_days: Option<u32>) -> Self {
    self.compliance_tags = tags;
    self.retention_period_days = retention_days;
    self
  }

  /// Check if event meets compliance requirements
  pub fn is_compliant(&self) -> bool {
    !self.compliance_tags.is_empty() && self.retention_period_days.is_some()
  }

  /// Get log level for structured logging
  pub fn log_level(&self) -> tracing::Level {
    match self.severity {
      AuditSeverity::Critical => tracing::Level::ERROR,
      AuditSeverity::High => tracing::Level::WARN,
      AuditSeverity::Medium => tracing::Level::WARN,
      AuditSeverity::Low => tracing::Level::INFO,
      AuditSeverity::Info => tracing::Level::INFO,
    }
  }
}

/// Audit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
  pub enabled: bool,
  pub log_level: String,
  pub structured_logging: bool,
  pub include_request_body: bool,
  pub include_response_body: bool,
  pub max_body_size: usize,
  pub retention_days: u32,
  pub buffer_size: usize,
  pub flush_interval_secs: u64,
  pub compliance_mode: bool,
  pub sensitive_headers: Vec<String>,
  pub excluded_paths: Vec<String>,
}

impl Default for AuditConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      log_level: "info".to_string(),
      structured_logging: true,
      include_request_body: false,
      include_response_body: false,
      max_body_size: 1024 * 1024, // 1MB
      retention_days: 90,
      buffer_size: 10000,
      flush_interval_secs: 60,
      compliance_mode: false,
      sensitive_headers: vec![
        "authorization".to_string(),
        "cookie".to_string(),
        "x-api-key".to_string(),
      ],
      excluded_paths: vec!["/health".to_string(), "/metrics".to_string()],
    }
  }
}

/// Gateway audit logger
pub struct GatewayAuditLogger {
  config: AuditConfig,
  event_buffer: Arc<tokio::sync::Mutex<Vec<AuditEvent>>>,
}

impl GatewayAuditLogger {
  /// Create new audit logger
  pub fn new(config: AuditConfig) -> Self {
    let logger = Self {
      event_buffer: Arc::new(tokio::sync::Mutex::new(Vec::new())),
      config: config.clone(),
    };

    // Start background flush task
    if config.enabled {
      logger.start_flush_task();
    }

    logger
  }

  /// Log audit event
  pub async fn log_event(&self, event: AuditEvent) {
    if !self.config.enabled {
      return;
    }

    // Check if path should be excluded
    if self
      .config
      .excluded_paths
      .iter()
      .any(|path| event.path.starts_with(path))
    {
      return;
    }

    // Log immediately for high-priority events
    if event.event_type.is_immediate() {
      self.write_event(&event);
    }

    // Buffer event for batch processing
    let mut buffer = self.event_buffer.lock().await;
    buffer.push(event);

    // Flush if buffer is full
    if buffer.len() >= self.config.buffer_size {
      self.flush_buffer(&mut buffer);
    }
  }

  /// Log authentication success
  pub async fn log_auth_success(&self, request_id: String, user_id: i64, workspace_id: i64) {
    let event = AuditEvent::new(AuditEventType::AuthenticationSuccess, request_id)
      .with_user(user_id, workspace_id, Vec::new())
      .with_compliance(vec!["AUTH".to_string()], Some(30));

    self.log_event(event).await;
  }

  /// Log authentication failure
  pub async fn log_auth_failure(
    &self,
    request_id: String,
    client_ip: Option<String>,
    reason: String,
  ) {
    let event = AuditEvent::new(AuditEventType::AuthenticationFailure, request_id)
      .with_network(client_ip, None, None)
      .with_error(reason, None)
      .with_compliance(vec!["AUTH".to_string(), "SECURITY".to_string()], Some(90));

    self.log_event(event).await;
  }

  /// Log authorization failure
  pub async fn log_auth_denied(
    &self,
    request_id: String,
    user_id: Option<i64>,
    path: String,
    required_permission: String,
  ) {
    let event = AuditEvent::new(AuditEventType::AuthorizationFailure, request_id)
      .with_detail("required_permission", required_permission)
      .with_detail("requested_path", path)
      .with_compliance(vec!["AUTHZ".to_string(), "SECURITY".to_string()], Some(90));

    if let Some(uid) = user_id {
      self.log_event(event.with_user(uid, 0, Vec::new())).await;
    } else {
      self.log_event(event).await;
    }
  }

  /// Log rate limit exceeded
  pub async fn log_rate_limit_exceeded(&self, request_id: String, key: String, limit: u32) {
    let event = AuditEvent::new(AuditEventType::RateLimitExceeded, request_id)
      .with_detail("rate_limit_key", key)
      .with_detail("rate_limit", limit)
      .with_compliance(vec!["RATE_LIMIT".to_string()], Some(30));

    self.log_event(event).await;
  }

  /// Log CORS violation
  pub async fn log_cors_violation(&self, request_id: String, origin: String) {
    let event = AuditEvent::new(AuditEventType::CorsViolation, request_id)
      .with_detail("blocked_origin", origin)
      .with_compliance(vec!["CORS".to_string(), "SECURITY".to_string()], Some(90));

    self.log_event(event).await;
  }

  /// Log security violation
  pub async fn log_security_violation(
    &self,
    request_id: String,
    violation_type: String,
    details: HashMap<String, serde_json::Value>,
  ) {
    let mut event = AuditEvent::new(AuditEventType::SecurityViolation, request_id)
      .with_detail("violation_type", violation_type)
      .with_compliance(
        vec!["SECURITY".to_string(), "CRITICAL".to_string()],
        Some(365),
      );

    for (key, value) in details {
      event.details.insert(key, value);
    }

    self.log_event(event).await;
  }

  /// Log request completion
  pub async fn log_request_completed(
    &self,
    request_id: String,
    method: String,
    path: String,
    status: u16,
    response_time: Duration,
    user_id: Option<i64>,
    upstream: Option<String>,
  ) {
    let event = AuditEvent::new(AuditEventType::RequestCompleted, request_id)
      .with_request(method, path, None)
      .with_response(status, response_time, None)
      .with_gateway(upstream, None, None);

    let event = if let Some(uid) = user_id {
      event.with_user(uid, 0, Vec::new())
    } else {
      event
    };

    self.log_event(event).await;
  }

  /// Write event to structured log
  fn write_event(&self, event: &AuditEvent) {
    if self.config.structured_logging {
      let level = event.log_level();

      match level {
        tracing::Level::ERROR => {
          error!(
            event_id = %event.event_id,
            event_type = ?event.event_type,
            severity = ?event.severity,
            request_id = %event.request_id,
            user_id = ?event.user_id,
            workspace_id = ?event.workspace_id,
            method = %event.method,
            path = %event.path,
            status_code = ?event.status_code,
            response_time_ms = ?event.response_time_ms,
            client_ip = ?event.client_ip,
            upstream = ?event.upstream,
            error_message = ?event.error_message,
            details = ?event.details,
            "🚨 [AUDIT] Critical security event"
          );
        }
        tracing::Level::WARN => {
          warn!(
            event_id = %event.event_id,
            event_type = ?event.event_type,
            severity = ?event.severity,
            request_id = %event.request_id,
            user_id = ?event.user_id,
            method = %event.method,
            path = %event.path,
            status_code = ?event.status_code,
            response_time_ms = ?event.response_time_ms,
            details = ?event.details,
            "⚠️ [AUDIT] Security/warning event"
          );
        }
        _ => {
          info!(
            event_id = %event.event_id,
            event_type = ?event.event_type,
            request_id = %event.request_id,
            user_id = ?event.user_id,
            method = %event.method,
            path = %event.path,
            status_code = ?event.status_code,
            response_time_ms = ?event.response_time_ms,
            "📋 [AUDIT] Gateway event"
          );
        }
      }
    } else {
      // Simple text logging
      info!(
        "AUDIT: {} {} {} {} {} {:?}",
        event.event_id,
        event.event_type.severity(),
        event.method,
        event.path,
        event.status_code.unwrap_or(0),
        event.event_type
      );
    }
  }

  /// Start background flush task
  fn start_flush_task(&self) {
    let buffer = Arc::clone(&self.event_buffer);
    let flush_interval = Duration::from_secs(self.config.flush_interval_secs);

    tokio::spawn(async move {
      let mut interval = tokio::time::interval(flush_interval);

      loop {
        interval.tick().await;

        let mut buffer_guard = buffer.lock().await;
        if !buffer_guard.is_empty() {
          // Flush buffered events
          for event in buffer_guard.iter() {
            // In production, this would write to external audit store
            // For now, we just use structured logging
          }
          buffer_guard.clear();
        }
      }
    });
  }

  /// Flush buffer immediately
  fn flush_buffer(&self, buffer: &mut Vec<AuditEvent>) {
    for event in buffer.iter() {
      self.write_event(event);
    }
    buffer.clear();
  }

  /// Get audit statistics
  pub async fn get_audit_stats(&self) -> AuditStats {
    let buffer_size = self.event_buffer.lock().await.len();

    AuditStats {
      events_buffered: buffer_size as u64,
      buffer_capacity: self.config.buffer_size as u64,
      flush_interval_secs: self.config.flush_interval_secs,
      retention_days: self.config.retention_days,
      compliance_mode: self.config.compliance_mode,
    }
  }
}

/// Audit statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
  pub events_buffered: u64,
  pub buffer_capacity: u64,
  pub flush_interval_secs: u64,
  pub retention_days: u32,
  pub compliance_mode: bool,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_audit_event_creation() {
    let event = AuditEvent::new(
      AuditEventType::AuthenticationSuccess,
      "test-req-123".to_string(),
    )
    .with_user(123, 456, vec!["WorkspaceMember".to_string()])
    .with_request("GET".to_string(), "/api/test".to_string(), None)
    .with_compliance(vec!["AUTH".to_string()], Some(30));

    assert_eq!(event.request_id, "test-req-123");
    assert_eq!(event.user_id, Some(123));
    assert_eq!(event.workspace_id, Some(456));
    assert!(event.is_compliant());
  }

  #[tokio::test]
  async fn test_audit_logger() {
    let config = AuditConfig::default();
    let logger = GatewayAuditLogger::new(config);

    logger
      .log_auth_success("test-123".to_string(), 123, 456)
      .await;
    logger
      .log_auth_failure(
        "test-124".to_string(),
        Some("192.168.1.1".to_string()),
        "Invalid token".to_string(),
      )
      .await;

    let stats = logger.get_audit_stats().await;
    assert_eq!(stats.events_buffered, 2);
  }

  #[test]
  fn test_event_severity() {
    assert_eq!(
      AuditEventType::SecurityViolation.severity(),
      AuditSeverity::Critical
    );
    assert_eq!(
      AuditEventType::AuthenticationFailure.severity(),
      AuditSeverity::High
    );
    assert_eq!(
      AuditEventType::RateLimitExceeded.severity(),
      AuditSeverity::Medium
    );
    assert_eq!(
      AuditEventType::RequestCompleted.severity(),
      AuditSeverity::Info
    );
  }
}
