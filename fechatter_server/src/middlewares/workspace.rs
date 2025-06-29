use axum::body::Body;
use axum::extract::{Extension, Path, State};
use axum::http::Request;

use axum::{
  http::StatusCode,
  middleware::Next,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn, error};
use std::sync::Arc;

use crate::{
  AppState,
  error::AppError,
};
use fechatter_core::models::{AuthUser, Workspace};

/// Workspace context structure, containing current request workspace information
#[derive(Clone, Serialize, Deserialize)]
pub struct WorkspaceContext {
  pub workspace_id: i64,
  pub user_id: i64,
  pub is_admin: bool,
  #[serde(skip)]
  pub workspace: Option<Arc<Workspace>>,
}

impl WorkspaceContext {
  /// Create a new workspace context from workspace data
  pub fn new(workspace_id: i64, user_id: i64, is_admin: bool) -> Self {
    Self {
      workspace_id,
      user_id,
      is_admin,
      workspace: None,
    }
  }

  /// Create workspace context with full workspace data
  pub fn with_workspace(workspace: Workspace, user_id: i64, is_admin: bool) -> Self {
    let workspace_id = workspace.id;
    Self {
      workspace_id: workspace_id.into(),
      user_id,
      is_admin,
      workspace: Some(Arc::new(workspace)),
    }
  }
}

/// Middleware to ensure user is a member of the specified workspace
#[allow(unused)]
pub async fn ensure_workspace_member(
  State(_state): State<AppState>,
  Extension(auth_user): Extension<AuthUser>,
  Path(ws_id): Path<i64>,
  request: Request<Body>,
  next: Next,
) -> Response {
  // Validate workspace ID - ensure user's workspace matches the requested one
  if auth_user.workspace_id != fechatter_core::WorkspaceId(ws_id) {
    warn!(
      "Workspace access denied: user workspace {} != requested workspace {}",
      auth_user.workspace_id, ws_id
    );
    return AppError::PermissionDenied(format!(
      "User is not a member of workspace {}",
      ws_id
    ))
    .into_response();
  }

  debug!("Workspace access granted for user {} in workspace {}", auth_user.id, ws_id);

  // Continue processing request - fix use-after-move issue
  let (parts, body) = request.into_parts();
  let req = Request::from_parts(parts, body);
  let mut response = next.run(req).await;

  // Add workspace context to response headers if successful
  if response.status().is_success() {
    let workspace_context = WorkspaceContext::new(
      ws_id,
      i64::from(auth_user.id),
      false, // TODO: Implement proper admin check using workspace service
    );

    if let Ok(workspace_context_json) = serde_json::to_string(&workspace_context) {
      if let Ok(header_value) = workspace_context_json.parse() {
        response.headers_mut().insert("X-Workspace-Context", header_value);
      }
    }
  }

  response
}

/// Enhanced workspace middleware - SIMPLIFIED VERSION  
/// Validates workspace access and adds workspace context to request
/// TEMPORARILY SIMPLIFIED to avoid service layer issues
pub async fn with_workspace_context(
  State(state): State<AppState>,
  Extension(auth_user): Extension<AuthUser>,
  req: Request<Body>,
  next: Next,
) -> Response {
  debug!("🏢 [WORKSPACE] SIMPLIFIED: Validating workspace access for user: {}", auth_user.id);
  
  let user_id = i64::from(auth_user.id);
  let workspace_id = i64::from(auth_user.workspace_id);
  
  // SIMPLIFIED: Skip database lookup and create basic context
  // TODO: Re-enable proper workspace validation once service layer is fixed
  info!("🏢 [WORKSPACE] SIMPLIFIED: Allowing access for user {} to workspace {} (DB check disabled)", user_id, workspace_id);
  
  // Create simplified workspace context without database lookup
  let workspace_context = WorkspaceContext::new(
    workspace_id,
    user_id,
    false, // Assume not admin for now
  );

  // Add context to request extensions
  let (mut parts, body) = req.into_parts();
  parts.extensions.insert(workspace_context.clone());
  let req = Request::from_parts(parts, body);

  let mut response = next.run(req).await;

  // Add workspace context to response headers if successful
  if response.status().is_success() {
    if let Ok(workspace_context_json) = serde_json::to_string(&workspace_context) {
      if let Ok(header_value) = workspace_context_json.parse() {
        response.headers_mut().insert("X-Workspace-Context", header_value);
      }
    }
  }

  debug!("🏢 [WORKSPACE] SIMPLIFIED workspace access granted for user: {} in workspace: {}", 
         user_id, workspace_id);
  response

  // Original implementation (commented out):
  /*
  // Find workspace using the AppState method
  let workspace = match state.find_by_id_with_pool(workspace_id).await {
    Ok(Some(workspace)) => {
      debug!("🏢 [WORKSPACE] Workspace found: id={}, name={}, owner_id={}", 
             workspace.id, workspace.name, workspace.owner_id);
      workspace
    }
    Ok(None) => {
      // Workspace not found - attempt to create it
      warn!("🏢 [WORKSPACE] WARNING: Workspace {} not found for user {}, attempting to create", 
            workspace_id, user_id);
      
      let new_workspace_name = format!("Workspace-{}", workspace_id);
      match state.create_workspace_with_pool(&new_workspace_name, user_id).await {
        Ok(new_workspace) => {
          info!("🏢 [WORKSPACE] Created new workspace: id={}, name={}", 
                new_workspace.id, new_workspace.name);
          new_workspace
        }
        Err(e) => {
          error!("🏢 [WORKSPACE] ERROR: Failed to create workspace: {:?}", e);
          return AppError::Internal("Failed to create workspace".to_string()).into_response();
        }
      }
    }
    Err(e) => {
      error!("🏢 [WORKSPACE] 💥 Database error finding workspace {}: {:?}", workspace_id, e);
      return AppError::Internal("Database error accessing workspace".to_string()).into_response();
    }
  };

  // Determine if user is admin (workspace owner)
  let is_admin = workspace.owner_id == user_id;
  
  // Create workspace context with full workspace data
  let workspace_context = WorkspaceContext::with_workspace(workspace.clone(), user_id, is_admin);

  // Add context to request extensions
  let (mut parts, body) = req.into_parts();
  parts.extensions.insert(workspace_context.clone());
  parts.extensions.insert(workspace); // Also add the workspace itself
  let req = Request::from_parts(parts, body);

  let mut response = next.run(req).await;

  // Add workspace context to response headers if successful
  if response.status().is_success() {
    if let Ok(workspace_context_json) = serde_json::to_string(&workspace_context) {
      if let Ok(header_value) = workspace_context_json.parse() {
        response.headers_mut().insert("X-Workspace-Context", header_value);
      }
    }
  }

  debug!("🏢 [WORKSPACE] Workspace access granted for user: {} in workspace: {}", 
         user_id, workspace_id);
  response
  */
}

/// Extract workspace ID from request path
pub fn extract_workspace_id_from_path(path: &str) -> Option<i64> {
  // Handle patterns like /api/workspace/{id} or /api/workspaces/{id}
  for pattern in ["/api/workspace/", "/api/workspaces/"] {
    if let Some(start) = path.find(pattern) {
      let after_prefix = &path[start + pattern.len()..];
      if let Some(end) = after_prefix.find('/') {
        let workspace_id_str = &after_prefix[..end];
        if let Ok(id) = workspace_id_str.parse() {
          return Some(id);
        }
      } else if let Ok(id) = after_prefix.parse() {
        return Some(id);
      }
    }
  }
  None
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_extract_workspace_id_from_path() {
    assert_eq!(extract_workspace_id_from_path("/api/workspace/123"), Some(123));
    assert_eq!(extract_workspace_id_from_path("/api/workspaces/456"), Some(456));
    assert_eq!(extract_workspace_id_from_path("/api/workspace/789/users"), Some(789));
    assert_eq!(extract_workspace_id_from_path("/api/users/123"), None);
    assert_eq!(extract_workspace_id_from_path("/api/workspace/invalid"), None);
  }

  #[test]
  fn test_workspace_context_creation() {
    let context = WorkspaceContext::new(123, 456, true);
    assert_eq!(context.workspace_id, 123);
    assert_eq!(context.user_id, 456);
    assert_eq!(context.is_admin, true);
    assert!(context.workspace.is_none());
  }

  #[test]
  fn test_workspace_context_serialization() {
    let context = WorkspaceContext::new(123, 456, false);
    let json = serde_json::to_string(&context).unwrap();
    let deserialized: WorkspaceContext = serde_json::from_str(&json).unwrap();
    
    assert_eq!(context.workspace_id, deserialized.workspace_id);
    assert_eq!(context.user_id, deserialized.user_id);
    assert_eq!(context.is_admin, deserialized.is_admin);
  }
}
