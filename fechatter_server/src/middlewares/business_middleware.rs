/// Business Middleware Functions - Core server-side logic only
/// 
/// This module contains all business middleware that should remain on the server side,
/// while gateway-related functionality (JWT, CORS, Rate Limiting, HTTP Cache) has been moved to Gateway.

use axum::{
  body::Body,
  extract::{ConnectInfo, Request, State},
  http::{Method, StatusCode, Uri},
  middleware::Next,
  response::{IntoResponse, Response},
};
use std::net::SocketAddr;
use std::time::Instant;
use tracing::{debug, info, warn};

use crate::middlewares::{RequestContext, Permission, PermissionChecker};
use crate::AppState;

// ============================================================================
// Core Business Middleware Functions
// ============================================================================

/// State validation middleware function
pub async fn validate_state(
  State(state): State<AppState>,
  req: Request<Body>,
  next: Next,
) -> Response {
  debug!("🔧 [STATE_VALIDATION] Validating application state");
  
  // Verify critical components exist
  let _pool = state.pool(); // This returns Arc<PgPool>
  let _token_manager = state.token_manager(); // This returns Arc<TokenManager>
  
  debug!("🔧 [STATE_VALIDATION] ✅ AppState validated - all components available");
  next.run(req).await
}

/// Request context creation middleware function
pub async fn create_request_context(
  mut req: Request<Body>,
  next: Next,
) -> Response {
  debug!("📋 [REQUEST_CONTEXT] Creating request context");
  
  // Check if RequestContext already exists
  if req.extensions().get::<RequestContext>().is_some() {
    debug!("📋 [REQUEST_CONTEXT] RequestContext already exists, skipping creation");
    return next.run(req).await;
  }
  
  // Create RequestContext from AuthUser (handle cloning properly)
  let auth_user_clone = req.extensions().get::<fechatter_core::AuthUser>().cloned();
  if let Some(auth_user) = auth_user_clone {
    let context = RequestContext::new(auth_user.clone());
    req.extensions_mut().insert(context);
    debug!("📋 [REQUEST_CONTEXT] ✅ RequestContext created for user: {}", auth_user.id);
  } else {
    debug!("📋 [REQUEST_CONTEXT] ⚠️ No AuthUser found, RequestContext not created");
  }
  
  next.run(req).await
}

/// Business audit logging middleware function  
pub async fn audit_business_operation(
  State(_state): State<AppState>,
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  req: Request<Body>,
  next: Next,
) -> Response {
  let method = req.method().clone();
  let uri = req.uri().clone();
  let start_time = Instant::now();
  
  debug!("📊 [AUDIT] Recording business operation: {} {}", method, uri.path());
  
  // Check if this operation should be audited
  if should_audit_business_operation(&method, uri.path()) {
    let response = next.run(req).await;
    let duration = start_time.elapsed();
    let status = response.status();
    
    // Async audit logging for business operations
    tokio::spawn(async move {
      log_business_operation(method, uri, status, duration, addr.ip().to_string()).await;
    });
    
    response
  } else {
    next.run(req).await
  }
}

/// Permission enforcement middleware function
pub async fn enforce_permission(
  permission: Permission,
  req: Request<Body>,
  next: Next,
) -> Response {
  debug!("🛡️ [PERMISSION] Enforcing permission: {:?}", permission);
  
  // Get RequestContext with user permissions
  let context = match req.extensions().get::<RequestContext>() {
    Some(ctx) => ctx.clone(),
    None => {
      // Try to create from AuthUser if missing
      match req.extensions().get::<fechatter_core::AuthUser>() {
        Some(user) => {
          let mut new_context = RequestContext::new(user.clone());
          
          // Add route-specific permissions
          if req.uri().path().contains("/chat/") {
            new_context.add_permission(Permission::ChatMember);
          }
          
          new_context
        }
        None => {
          warn!("🛡️ [PERMISSION] No authentication context found");
          return (
            StatusCode::UNAUTHORIZED,
            "Authentication required for permission check".to_string(),
          ).into_response();
        }
      }
    }
  };
  
  // Check permission
  if !PermissionChecker::has_permission(&context.permissions, permission) {
    warn!(
      "🛡️ [PERMISSION] Permission denied: user {} lacks {:?}",
      context.user.id, permission
    );
    return (
      StatusCode::FORBIDDEN,
      format!("Missing required permission: {}", permission.display_name()),
    ).into_response();
  }
  
  debug!("🛡️ [PERMISSION] ✅ Permission granted for user: {}", context.user.id);
  next.run(req).await
}

/// Chat access validation middleware function
pub async fn validate_chat_access(
  state: AppState,
  req: Request<Body>,
  next: Next,
) -> Response {
  let path = req.uri().path().to_string();
  
  debug!("🔒 [CHAT_ACCESS] Validating chat access for path: {}", path);
  
  // Extract chat_id from path
  let chat_id = match extract_chat_id_from_path(&path) {
    Some(id) => {
      debug!("🔒 [CHAT_ACCESS] Extracted chat_id: {}", id);
      id
    }
    None => {
      debug!("🔒 [CHAT_ACCESS] No chat_id in path, skipping validation");
      return next.run(req).await;
    }
  };
  
  // Get authenticated user
  let auth_user = match req.extensions().get::<fechatter_core::AuthUser>() {
    Some(user) => {
      debug!("🔒 [CHAT_ACCESS] Found authenticated user: {}", user.id);
      user
    }
    None => {
      warn!("🔒 [CHAT_ACCESS] No authenticated user found");
      return (
        StatusCode::UNAUTHORIZED,
        "Authentication required for chat access".to_string(),
      ).into_response();
    }
  };
  
  // Verify chat membership
  let user_id: i64 = auth_user.id.into();
  match verify_chat_membership(&state, user_id, chat_id).await {
    Ok(true) => {
      debug!("🔒 [CHAT_ACCESS] ✅ Chat access granted for user {} in chat {}", user_id, chat_id);
      next.run(req).await
    }
    Ok(false) => {
      warn!("🔒 [CHAT_ACCESS] ❌ User {} not a member of chat {}", user_id, chat_id);
      (
        StatusCode::FORBIDDEN,
        format!("Access denied to chat {}", chat_id),
      ).into_response()
    }
    Err(e) => {
      warn!("🔒 [CHAT_ACCESS] Error verifying chat membership: {}", e);
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Failed to verify chat access".to_string(),
      ).into_response()
    }
  }
}

/// Workspace validation middleware function (simplified)
pub async fn validate_workspace_access(
  _state: AppState,
  req: Request<Body>,
  next: Next,
) -> Response {
  debug!("🏢 [WORKSPACE] Validating workspace access (simplified)");
  
  if req.extensions().get::<fechatter_core::AuthUser>().is_some() {
    debug!("🏢 [WORKSPACE] ✅ AuthUser found, proceeding");
    next.run(req).await
  } else {
    warn!("🏢 [WORKSPACE] No authenticated user found");
    (
      StatusCode::UNAUTHORIZED,
      "Authentication required for workspace access".to_string(),
    ).into_response()
  }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Authentication requirement middleware function
pub fn require_auth() -> impl Fn(Request<Body>, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Clone {
  |req: Request<Body>, next: Next| {
    Box::pin(async move {
      if req.extensions().get::<fechatter_core::AuthUser>().is_some() {
        next.run(req).await
      } else {
        (
          StatusCode::UNAUTHORIZED,
          "Authentication required".to_string(),
        ).into_response()
      }
    })
  }
}

/// Extract chat_id from URL path
pub fn extract_chat_id_from_path(path: &str) -> Option<i64> {
  if let Some(start) = path.find("/api/chat/") {
    let after_prefix = &path[start + "/api/chat/".len()..];
    if let Some(end) = after_prefix.find('/') {
      let chat_id_str = &after_prefix[..end];
      chat_id_str.parse().ok()
    } else {
      after_prefix.parse().ok()
    }
  } else {
    None
  }
}

/// Check if business operation should be audited
fn should_audit_business_operation(method: &Method, path: &str) -> bool {
  // Audit important business operations
  let audit_paths = [
    "/api/chat",
    "/api/workspace", 
    "/api/users",
    "/api/members",
    "/api/messages",
  ];
  
  let audit_methods = [Method::POST, Method::PUT, Method::PATCH, Method::DELETE];
  
  audit_paths.iter().any(|pattern| path.starts_with(pattern)) &&
  audit_methods.contains(method)
}

/// Verify chat membership (business logic)
async fn verify_chat_membership(
  state: &AppState,
  user_id: i64,
  chat_id: i64,
) -> Result<bool, crate::AppError> {
  let chat_service = state.application_services().chat_application_service();
  chat_service.is_user_chat_member(user_id, chat_id).await
}

/// Log business operation (async)
async fn log_business_operation(
  method: Method,
  uri: Uri,
  status: StatusCode,
  duration: std::time::Duration,
  client_ip: String,
) {
  info!(
    "📊 [BUSINESS_AUDIT] {} {} -> {} ({}ms) from {}",
    method,
    uri.path(),
    status.as_u16(),
    duration.as_millis(),
    client_ip
  );
}

// ============================================================================
// Middleware Function Exports
// ============================================================================

/// Get state validation middleware function
pub fn get_state_validation() -> impl Fn(State<AppState>, Request<Body>, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Clone {
  |state, req, next| Box::pin(validate_state(state, req, next))
}

/// Get request context middleware function  
pub fn get_request_context() -> impl Fn(Request<Body>, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Clone {
  |req, next| Box::pin(create_request_context(req, next))
}

/// Get audit middleware function
pub fn get_audit_middleware() -> impl Fn(State<AppState>, ConnectInfo<SocketAddr>, Request<Body>, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Clone {
  |state, addr, req, next| Box::pin(audit_business_operation(state, addr, req, next))
}

/// Get permission middleware function
pub fn get_permission_middleware(permission: Permission) -> impl Fn(Request<Body>, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Clone {
  move |req, next| {
    let perm = permission;
    Box::pin(enforce_permission(perm, req, next))
  }
}

/// Get chat access middleware function
pub fn get_chat_access() -> impl Fn(AppState, Request<Body>, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Clone {
  |state, req, next| Box::pin(validate_chat_access(state, req, next))
}

/// Get workspace access middleware function
pub fn get_workspace_access() -> impl Fn(AppState, Request<Body>, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Clone {
  |state, req, next| Box::pin(validate_workspace_access(state, req, next))
} 