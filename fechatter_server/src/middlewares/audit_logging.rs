//! # Audit Logging Middleware
//!
//! **Responsibility**: Records important operations and events in the system, providing audit trails
//!
//! ## Design Principles
//! - Record all sensitive operations (login, permission changes, data modifications, etc.)
//! - Structured log format for easy analysis
//! - Asynchronous logging to avoid impacting main business flows
//! - Support different storage backends (database, file, external services)
//!
//! ## Audit Content
//! - User authentication events (login, logout, token refresh)
//! - Permission operations (role changes, permission grants/revokes)
//! - Resource operations (create, modify, delete chats/messages/files)
//! - Management operations (cache clearing, search index rebuilding, etc.)

use axum::{
  body::Body,
  extract::{ConnectInfo, State},
  http::{Method, Request, Uri},
  middleware::Next,
  response::Response,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{AppState, error::AppError};
use fechatter_core::models::{AuthUser, UserId};

// ============================================================================
// Audit Event Type Definitions
// ============================================================================

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
  // Authentication events
  UserLogin,
  UserLogout,
  UserLogoutAll,
  TokenRefresh,
  AuthenticationFailed,

  // User management
  UserCreated,
  UserUpdated,
  UserDeleted,
  UserInvited,
  UserBanned,
  UserUnbanned,

  // Workspace operations
  WorkspaceCreated,
  WorkspaceUpdated,
  WorkspaceDeleted,
  WorkspaceUserAdded,
  WorkspaceUserRemoved,
  WorkspaceSettingsChanged,

  // Chat operations
  ChatCreated,
  ChatUpdated,
  ChatDeleted,
  ChatMemberAdded,
  ChatMemberRemoved,
  ChatOwnershipTransferred,
  ChatSettingsChanged,

  // Message operations
  MessageSent,
  MessageUpdated,
  MessageDeleted,
  MessagePinned,
  MessageUnpinned,

  // File operations
  FileUploaded,
  FileDeleted,
  FileShared,
  FileAccessGranted,
  FileAccessRevoked,

  // Permission operations
  PermissionGranted,
  PermissionRevoked,
  RoleChanged,

  // System operations
  CacheCleared,
  SearchIndexRebuilt,
  SystemConfigChanged,
  DatabaseMaintenance,

  // Security events
  SuspiciousActivity,
  UnauthorizedAccess,
  RateLimitExceeded,
  SecurityPolicyViolation,
}

/// Audit event severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditSeverity {
  Info,     // Normal information events
  Warning,  // Warning events
  Critical, // Critical events
  Security, // Security-related events
}

/// Audit event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
  /// Unique event ID
  pub id: String,
  /// Event type
  pub event_type: AuditEventType,
  /// Event severity level
  pub severity: AuditSeverity,
  /// User ID (if available)
  pub user_id: Option<UserId>,
  /// User IP address
  pub ip_address: Option<String>,
  /// User agent
  pub user_agent: Option<String>,
  /// Request ID (for correlation)
  pub request_id: Option<String>,
  /// HTTP method
  pub method: String,
  /// Request path
  pub path: String,
  /// Event description
  pub description: String,
  /// Additional metadata
  pub metadata: serde_json::Value,
  /// Event timestamp
  pub timestamp: u64,
  /// Workspace ID (if applicable)
  pub workspace_id: Option<i64>,
  /// Target resource ID (if applicable)
  pub resource_id: Option<String>,
  /// Operation result (success/failure)
  pub success: bool,
  /// Error message (if failed)
  pub error_message: Option<String>,
}

impl AuditEvent {
  /// Create new audit event
  pub fn new(event_type: AuditEventType, severity: AuditSeverity, description: String) -> Self {
    Self {
      id: Uuid::new_v4().to_string(),
      event_type,
      severity,
      user_id: None,
      ip_address: None,
      user_agent: None,
      request_id: None,
      method: String::new(),
      path: String::new(),
      description,
      metadata: serde_json::Value::Null,
      timestamp: SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs(),
      workspace_id: None,
      resource_id: None,
      success: true,
      error_message: None,
    }
  }

  /// Set user information
  pub fn with_user(mut self, user: &AuthUser) -> Self {
    self.user_id = Some(user.id);
    self.workspace_id = Some(user.workspace_id.into());
    self
  }

  /// Set request information
  pub fn with_request(mut self, method: &Method, uri: &Uri, request_id: Option<String>) -> Self {
    self.method = method.to_string();
    self.path = uri.path().to_string();
    self.request_id = request_id;
    self
  }

  /// Set network information
  pub fn with_network_info(mut self, ip: Option<String>, user_agent: Option<String>) -> Self {
    self.ip_address = ip;
    self.user_agent = user_agent;
    self
  }

  /// Set metadata
  pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
    self.metadata = metadata;
    self
  }

  /// Set operation result
  pub fn with_result(mut self, success: bool, error_message: Option<String>) -> Self {
    self.success = success;
    self.error_message = error_message;
    self
  }

  /// Set resource ID
  pub fn with_resource_id(mut self, resource_id: String) -> Self {
    self.resource_id = Some(resource_id);
    self
  }
}

// ============================================================================
// Audit Log Storage Interface
// ============================================================================

/// Audit log storage trait
#[async_trait::async_trait]
pub trait AuditStorage: Send + Sync {
  async fn store_event(&self, event: AuditEvent) -> Result<(), AppError>;
  async fn query_events(
    &self,
    user_id: Option<UserId>,
    event_type: Option<AuditEventType>,
    start_time: Option<u64>,
    end_time: Option<u64>,
    limit: Option<usize>,
  ) -> Result<Vec<AuditEvent>, AppError>;
}

/// Tracing-based audit storage implementation
pub struct TracingAuditStorage;

#[async_trait::async_trait]
impl AuditStorage for TracingAuditStorage {
  async fn store_event(&self, event: AuditEvent) -> Result<(), AppError> {
    let event_json = serde_json::to_string(&event)
      .map_err(|e| AppError::Internal(format!("Failed to serialize audit event: {}", e)))?;

    match event.severity {
      AuditSeverity::Info => {
        info!(
            event_type = ?event.event_type,
            user_id = ?event.user_id,
            ip_address = ?event.ip_address,
            path = %event.path,
            success = event.success,
            audit_event = %event_json,
            "{}",
            event.description
        );
      }
      AuditSeverity::Warning => {
        warn!(
            event_type = ?event.event_type,
            user_id = ?event.user_id,
            ip_address = ?event.ip_address,
            path = %event.path,
            success = event.success,
            audit_event = %event_json,
            "{}",
            event.description
        );
      }
      AuditSeverity::Critical | AuditSeverity::Security => {
        error!(
            event_type = ?event.event_type,
            user_id = ?event.user_id,
            ip_address = ?event.ip_address,
            path = %event.path,
            success = event.success,
            audit_event = %event_json,
            "{}",
            event.description
        );
      }
    }

    Ok(())
  }

  async fn query_events(
    &self,
    _user_id: Option<UserId>,
    _event_type: Option<AuditEventType>,
    _start_time: Option<u64>,
    _end_time: Option<u64>,
    _limit: Option<usize>,
  ) -> Result<Vec<AuditEvent>, AppError> {
    // Tracing storage doesn't support queries, returns empty result
    // Should use database storage in production
    Ok(vec![])
  }
}

// ============================================================================
// Audit Logging Middleware
// ============================================================================

/// Audit logging middleware - Records important API operations
pub async fn audit_logging_middleware(
  State(state): State<AppState>,
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  req: Request<Body>,
  next: Next,
) -> Response {
  let method = req.method().clone();
  let uri = req.uri().clone();
  let path = uri.path();

  // Extract request information
  let request_id = req
    .headers()
    .get("x-request-id")
    .and_then(|h| h.to_str().ok())
    .map(String::from);

  let user_agent = req
    .headers()
    .get("user-agent")
    .and_then(|h| h.to_str().ok())
    .map(String::from);

  let ip_address = Some(addr.ip().to_string());

  // Check if this path needs auditing
  if should_audit_path(&path, &method) {
    // Execute request
    let response = next.run(req).await;
    let status = response.status();

    // Asynchronously record audit event
    tokio::spawn(async move {
      if let Err(e) = log_api_access_event(
        &state,
        method,
        uri,
        status.as_u16(),
        request_id,
        ip_address,
        user_agent,
      )
      .await
      {
        error!("Failed to log audit event: {:?}", e);
      }
    });

    response
  } else {
    next.run(req).await
  }
}

/// Check if path needs auditing
fn should_audit_path(path: &str, method: &Method) -> bool {
  // Paths that need auditing
  let audit_patterns = [
    "/api/signin",
    "/api/signup",
    "/api/logout",
    "/api/chat",
    "/api/workspace",
    "/api/cache",
    "/api/admin",
  ];

  // Sensitive operation methods that need auditing
  let audit_methods = [Method::POST, Method::PUT, Method::PATCH, Method::DELETE];

  // Check if matches audit patterns or sensitive methods
  audit_patterns
    .iter()
    .any(|pattern| path.starts_with(pattern))
    || (audit_methods.contains(method) && !path.starts_with("/health"))
}

/// Record API access event
async fn log_api_access_event(
  state: &AppState,
  method: Method,
  uri: Uri,
  status_code: u16,
  request_id: Option<String>,
  ip_address: Option<String>,
  user_agent: Option<String>,
) -> Result<(), AppError> {
  // Determine event type and severity
  let (event_type, severity) = determine_audit_event_type(&uri.path(), &method, status_code);

  let success = status_code < 400;
  let description = format!("{} {} - Status: {}", method, uri.path(), status_code);

  let event = AuditEvent::new(event_type, severity, description)
    .with_request(&method, &uri, request_id)
    .with_network_info(ip_address, user_agent)
    .with_result(
      success,
      if success {
        None
      } else {
        Some(format!("HTTP {}", status_code))
      },
    );

  // Add user info to audit event if available
  // Note: We can't get user info directly from middleware since auth happens later
  // In practice, user-related audit events should be recorded elsewhere after auth

  // Store audit event
  let storage = TracingAuditStorage;
  storage.store_event(event).await?;

  Ok(())
}

/// Determine audit event type based on path and method
fn determine_audit_event_type(
  path: &str,
  method: &Method,
  status_code: u16,
) -> (AuditEventType, AuditSeverity) {
  // Authentication related
  if path == "/api/signin" {
    return if status_code < 300 {
      (AuditEventType::UserLogin, AuditSeverity::Info)
    } else {
      (
        AuditEventType::AuthenticationFailed,
        AuditSeverity::Security,
      )
    };
  }

  if path == "/api/logout" {
    return (AuditEventType::UserLogout, AuditSeverity::Info);
  }

  if path == "/api/logout-all" {
    return (AuditEventType::UserLogoutAll, AuditSeverity::Info);
  }

  if path == "/api/signup" {
    return (AuditEventType::UserCreated, AuditSeverity::Info);
  }

  // Chat related
  if path.starts_with("/api/chat") {
    match method {
      &Method::POST if path == "/api/chat" => (AuditEventType::ChatCreated, AuditSeverity::Info),
      &Method::PATCH => (AuditEventType::ChatUpdated, AuditSeverity::Info),
      &Method::DELETE => (AuditEventType::ChatDeleted, AuditSeverity::Warning),
      _ => (AuditEventType::ChatUpdated, AuditSeverity::Info),
    }
  }
  // Cache related
  else if path.starts_with("/api/cache") {
    (AuditEventType::CacheCleared, AuditSeverity::Warning)
  }
  // Admin operations
  else if path.starts_with("/api/admin") {
    (AuditEventType::SystemConfigChanged, AuditSeverity::Critical)
  }
  // Other operations
  else {
    let severity = if status_code >= 400 {
      AuditSeverity::Warning
    } else {
      AuditSeverity::Info
    };
    (AuditEventType::UserUpdated, severity)
  }
}

// ============================================================================
// Convenient Audit Logging Functions
// ============================================================================

/// Record authentication event
pub async fn log_authentication_event(
  user: &AuthUser,
  event_type: AuditEventType,
  ip_address: Option<String>,
  user_agent: Option<String>,
  success: bool,
  error_message: Option<String>,
) {
  let description = match event_type {
    AuditEventType::UserLogin => format!("User {} logged in", user.id),
    AuditEventType::UserLogout => format!("User {} logged out", user.id),
    AuditEventType::TokenRefresh => format!("User {} refreshed token", user.id),
    _ => format!("Authentication event for user {}", user.id),
  };

  let severity = if success {
    AuditSeverity::Info
  } else {
    AuditSeverity::Security
  };

  let event = AuditEvent::new(event_type, severity, description)
    .with_user(user)
    .with_network_info(ip_address, user_agent)
    .with_result(success, error_message);

  let storage = TracingAuditStorage;
  if let Err(e) = storage.store_event(event).await {
    error!("Failed to log authentication event: {:?}", e);
  }
}

/// Record resource operation event
pub async fn log_resource_event(
  user: &AuthUser,
  event_type: AuditEventType,
  resource_id: String,
  description: String,
  metadata: Option<serde_json::Value>,
) {
  let event = AuditEvent::new(event_type, AuditSeverity::Info, description)
    .with_user(user)
    .with_resource_id(resource_id)
    .with_metadata(metadata.unwrap_or(serde_json::Value::Null));

  let storage = TracingAuditStorage;
  if let Err(e) = storage.store_event(event).await {
    error!("Failed to log resource event: {:?}", e);
  }
}

/// Record security event
pub async fn log_security_event(
  event_type: AuditEventType,
  description: String,
  user_id: Option<UserId>,
  ip_address: Option<String>,
  metadata: Option<serde_json::Value>,
) {
  let mut event = AuditEvent::new(event_type, AuditSeverity::Security, description)
    .with_network_info(ip_address, None)
    .with_metadata(metadata.unwrap_or(serde_json::Value::Null));

  if let Some(uid) = user_id {
    event.user_id = Some(uid);
  }

  let storage = TracingAuditStorage;
  if let Err(e) = storage.store_event(event).await {
    error!("Failed to log security event: {:?}", e);
  }
}
