/// Business permission checker - no token validation, pure business logic
/// 
/// This module handles business-level permission checking after authentication
/// has been completed by either Gateway or auth middleware.

use axum::{
  body::Body,
  extract::Request,
  http::StatusCode,
  middleware::Next,
  response::{IntoResponse, Response},
};
use tracing::{debug, warn};

use crate::middlewares::{Permission, PermissionChecker, RequestContext};
use crate::AppError;

/// Permission middleware factory - checks business permissions
/// 
/// This assumes authentication has already happened and AuthUser/RequestContext exists
pub fn require_permission(
  permission: Permission,
) -> impl Fn(
  Request<Body>,
  Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
+ Clone {
  move |req: Request<Body>, next: Next| {
    let permission = permission;
    Box::pin(async move {
      debug!("🛡️ [PERMISSION] Checking permission: {:?}", permission);
      
      // Try to get existing RequestContext
      let context = match req.extensions().get::<RequestContext>() {
        Some(ctx) => {
          debug!("🛡️ [PERMISSION] Found RequestContext for user: {}", ctx.user.id);
          ctx.clone()
        }
        None => {
          // If no RequestContext, try to create one from AuthUser
          match req.extensions().get::<fechatter_core::AuthUser>() {
            Some(user) => {
              debug!("🛡️ [PERMISSION] Creating RequestContext from AuthUser: {}", user.id);
              let mut new_context = RequestContext::new(user.clone());
              
              // For message/file endpoints, add basic chat member permission if this is a chat route
              if req.uri().path().contains("/chat/") && req.uri().path().contains("/messages") {
                new_context.add_permission(Permission::ChatMember);
                debug!("🛡️ [PERMISSION] Added ChatMember permission for chat route");
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

      debug!(
        "🛡️ [PERMISSION] Permission granted: user {} has {:?}", 
        context.user.id, permission
      );
      
      next.run(req).await
    })
  }
}

/// Authentication check middleware - verifies AuthUser exists
pub fn require_auth() -> impl Fn(
  Request<Body>,
  Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
+ Clone {
  |req: Request<Body>, next: Next| {
    Box::pin(async move {
      debug!("🔐 [AUTH_CHECK] Verifying authentication state");
      
      if req.extensions().get::<fechatter_core::AuthUser>().is_none() {
        warn!("🔐 [AUTH_CHECK] No AuthUser found in request extensions");
        return (
          StatusCode::UNAUTHORIZED,
          "Authentication required".to_string(),
        ).into_response();
      }
      
      debug!("🔐 [AUTH_CHECK] Authentication verified");
      next.run(req).await
    })
  }
}

/// Chat admin permission check
pub fn require_chat_admin() -> impl Fn(Request<Body>, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Clone {
  require_permission(Permission::ChatAdmin)
}

/// Chat creator permission check  
pub fn require_chat_creator() -> impl Fn(Request<Body>, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Clone {
  require_permission(Permission::ChatCreator)
}

/// Workspace admin permission check
pub fn require_workspace_admin() -> impl Fn(Request<Body>, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Clone {
  require_permission(Permission::WorkspaceAdmin)
}

/// System admin permission check
pub fn require_system_admin() -> impl Fn(Request<Body>, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Clone {
  require_permission(Permission::SystemAdmin)
} 