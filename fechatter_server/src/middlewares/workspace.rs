use axum::body::Body;
use axum::extract::{Extension, Path, State};
use axum::http::Request;

use axum::{
  http::StatusCode,
  middleware::Next,
  response::{IntoResponse, Response},
};
use std::sync::Arc;

use crate::{
  AppState,
  models::{AuthUser, Workspace},
};

/// Workspace context structure, containing current request workspace information
#[derive(Clone)]
pub struct WorkspaceContext {
  pub workspace: Arc<Workspace>,
}

impl WorkspaceContext {
  /// Create a new workspace context from a workspace
  pub fn new(workspace: Workspace) -> Self {
    Self {
      workspace: Arc::new(workspace),
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
  // Validate user is a member of the workspace
  if auth_user.workspace_id != ws_id {
    return StatusCode::FORBIDDEN.into_response();
  }

  // Continue processing request
  next.run(request).await
}

/// Extract workspace context from the request
pub async fn with_workspace_context(
  State(state): State<AppState>,
  Extension(auth_user): Extension<AuthUser>,
  mut request: Request<Body>,
  next: Next,
) -> Response {
  tracing::debug!(
    "with_workspace_context called for user_id={}, email={}, workspace_id={}",
    auth_user.id,
    auth_user.email,
    auth_user.workspace_id
  );

  // Find workspace
  let workspace = match state.find_by_id_with_pool(auth_user.workspace_id).await {
    Ok(Some(workspace)) => {
      tracing::debug!(
        "Workspace found: id={}, name={}, owner_id={}",
        workspace.id,
        workspace.name,
        workspace.owner_id
      );
      workspace
    }
    Ok(None) => {
      // 工作区不存在，可能是数据错误
      // 为调试目的，打印更多信息
      tracing::error!(
        "Workspace ID {} not found for user {} (email: {})",
        auth_user.workspace_id,
        auth_user.id,
        auth_user.email
      );

      // 尝试创建这个工作区以修复问题
      let new_workspace_name = format!("Workspace-{}", auth_user.workspace_id);
      tracing::info!(
        "Attempting to create missing workspace with name '{}' for user_id={}",
        new_workspace_name,
        auth_user.id
      );

      match state
        .create_workspace_with_pool(&new_workspace_name, auth_user.id)
        .await
      {
        Ok(new_workspace) => {
          tracing::info!(
            "Successfully created new workspace id={}, name={} for user_id={}",
            new_workspace.id,
            new_workspace.name,
            auth_user.id
          );
          new_workspace
        }
        Err(e) => {
          tracing::error!("Failed to create workspace: {:?}", e);

          // 返回401而不是继续 - 这可能是造成集成测试问题的原因
          tracing::error!(
            "Returning UNAUTHORIZED due to workspace creation failure. This is likely causing the 401 errors in tests."
          );
          return StatusCode::UNAUTHORIZED.into_response();
        }
      }
    }
    Err(e) => {
      tracing::error!(
        "Database error when finding workspace id={} for user_id={}: {:?}",
        auth_user.workspace_id,
        auth_user.id,
        e
      );
      return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
  };

  // Add workspace context to request extensions
  let workspace_copy = workspace.clone(); // Keep a copy for logging
  let ctx = WorkspaceContext::new(workspace.clone()); // Clone to avoid ownership issues
  request.extensions_mut().insert(workspace); // Insert workspace directly
  request.extensions_mut().insert(ctx); // Insert context too

  tracing::debug!(
    "Workspace context added to request. Proceeding with user_id={}, workspace_id={}",
    auth_user.id,
    workspace_copy.id
  );

  // Continue processing request
  next.run(request).await
}
