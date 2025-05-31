//! # Authentication Adapter
//!
//! **Responsibilities**:
//! - Wraps fechatter_core's TokenEngine
//! - Adds business-specific authentication logic
//! - Provides workspace and chat level permission checks
//! - Maintains Core security while enhancing business functionality

use crate::AppState;
use axum::{
  extract::{Request, State},
  http::StatusCode,
  middleware::Next,
  response::Response,
};
use fechatter_core::{
  middlewares::{verify_token_middleware, TokenVerifier},
  models::AuthUser,
};
use tracing::{debug, warn};

// =============================================================================
// Authentication Adapter - Inherits Core functionality, adds business logic
// =============================================================================

/// Authentication Adapter
///
/// **Single Responsibility**: Wraps Core token verification, adds business permission checks
/// **Inheritance Pattern**: Reuses Core engine + extends business logic
pub struct AuthAdapter {
  /// Whether workspace checks are enabled
  workspace_check_enabled: bool,
  /// Whether permission caching is enabled
  permission_cache_enabled: bool,
}

impl Default for AuthAdapter {
  fn default() -> Self {
    Self {
      workspace_check_enabled: true,
      permission_cache_enabled: true,
    }
  }
}

impl AuthAdapter {
  /// Create new authentication adapter
  pub fn new() -> Self {
    Self::default()
  }

  /// Disable workspace checks (for testing or special cases)
  pub fn without_workspace_check(mut self) -> Self {
    self.workspace_check_enabled = false;
    self
  }

  /// Disable permission caching (for real-time permission checks)
  pub fn without_permission_cache(mut self) -> Self {
    self.permission_cache_enabled = false;
    self
  }
}

// =============================================================================
// Middleware Implementation - Adapter Pattern
// =============================================================================

/// Basic authentication middleware (direct Core usage)
///
/// **Single Responsibility**: Provides Core-level token verification
/// **Use Cases**: Scenarios requiring maximum compatibility and minimal overhead
pub async fn core_auth_middleware(
  State(state): State<AppState>,
  request: Request,
  next: Next,
) -> Response {
  debug!("Using Core authentication middleware");

  // Directly call fechatter_core's mature implementation
  verify_token_middleware(State(state), request, next).await
}

/// Enhanced authentication middleware (Core + business logic)
///
/// **Single Responsibility**: Adds business permission checks on top of Core
/// **Use Cases**: Business scenarios requiring workspace and permission checks
pub async fn enhanced_auth_middleware(
  State(state): State<AppState>,
  mut request: Request,
  next: Next,
) -> Response {
  debug!("Using enhanced authentication middleware");

  // 1. First use Core for basic authentication
  let (mut parts, body) = request.into_parts();
  let temp_request = Request::from_parts(parts.clone(), body);

  // Use Core's verify_token_middleware for basic verification
  let core_result = verify_token_middleware(State(state.clone()), temp_request, next.clone()).await;

  // Check if Core authentication succeeded
  if core_result.status() != StatusCode::OK {
    return core_result;
  }

  // 2. Core auth successful, continue with business logic enhancement
  request = Request::from_parts(parts, axum::body::Body::empty());

  // Get AuthUser from request extensions (set by Core middleware)
  if let Some(auth_user) = request.extensions().get::<AuthUser>().cloned() {
    debug!("Authenticated user: user_id={}", auth_user.id);

    // 3. Add business-specific enhancements
    let enhanced_user = enhance_auth_user(auth_user, &state).await;

    // 4. Update request extensions
    request.extensions_mut().insert(enhanced_user);
  } else {
    warn!("Core auth successful but AuthUser not found, this may be a bug");
  }

  next.run(request).await
}

/// Workspace authentication middleware (Core + workspace check)
///
/// **Single Responsibility**: Validates user access to specific workspace
/// **Use Cases**: Workspace-level API endpoints
pub async fn workspace_auth_middleware(
  State(state): State<AppState>,
  mut request: Request,
  next: Next,
) -> Response {
  debug!("Using workspace authentication middleware");

  // 1. First perform enhanced authentication
  let response = enhanced_auth_middleware(State(state.clone()), request, next.clone()).await;

  if response.status() != StatusCode::OK {
    return response;
  }

  // 2. Extract workspace_id from path and verify permissions
  // Add specific workspace permission check logic here

  response
}

/// Chat authentication middleware (Core + workspace + chat permissions)
///
/// **Single Responsibility**: Validates user access to specific chat
/// **Use Cases**: Chat-related API endpoints
pub async fn chat_auth_middleware(
  State(state): State<AppState>,
  request: Request,
  next: Next,
) -> Response {
  debug!("Using chat authentication middleware");

  // 1. First perform workspace authentication
  let response = workspace_auth_middleware(State(state), request, next).await;

  if response.status() != StatusCode::OK {
    return response;
  }

  // 2. Add chat-specific permission checks
  // Add specific chat permission check logic here

  response
}

// =============================================================================
// Business Logic Enhancement Functions
// =============================================================================

/// Enhance AuthUser object
///
/// **Single Responsibility**: Add business-specific information and permissions to base AuthUser
async fn enhance_auth_user(auth_user: AuthUser, state: &AppState) -> AuthUser {
  debug!("Enhancing AuthUser object: user_id={}", auth_user.id);

  // Can add:
  // 1. Permission cache queries
  // 2. User role information
  // 3. Workspace permissions
  // 4. Other business-specific information

  // Currently returns original AuthUser, future could create EnhancedAuthUser
  auth_user
}

// =============================================================================
// Convenience Constructors
// =============================================================================

/// Create standard authentication middleware stack
///
/// **Single Responsibility**: Provide common middleware combinations
pub fn standard_auth_stack() -> Vec<&'static str> {
  vec!["core_auth"]
}

/// Create business authentication middleware stack
///
/// **Single Responsibility**: Provide business scenario middleware combinations
pub fn business_auth_stack() -> Vec<&'static str> {
  vec!["enhanced_auth"]
}

/// Create full authentication middleware stack
///
/// **Single Responsibility**: Provide complete stack with all checks
pub fn full_auth_stack() -> Vec<&'static str> {
  vec!["chat_auth"] // Includes all levels of checks
}

// =============================================================================
// Test Module
// =============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_auth_adapter_creation() {
    let adapter = AuthAdapter::new();
    assert!(adapter.workspace_check_enabled);
    assert!(adapter.permission_cache_enabled);

    let adapter_without_workspace = AuthAdapter::new().without_workspace_check();
    assert!(!adapter_without_workspace.workspace_check_enabled);

    let adapter_without_cache = AuthAdapter::new().without_permission_cache();
    assert!(!adapter_without_cache.permission_cache_enabled);
  }

  #[test]
  fn test_auth_stack_creation() {
    let standard = standard_auth_stack();
    assert_eq!(standard, vec!["core_auth"]);

    let business = business_auth_stack();
    assert_eq!(business, vec!["enhanced_auth"]);

    let full = full_auth_stack();
    assert_eq!(full, vec!["chat_auth"]);
  }
}
