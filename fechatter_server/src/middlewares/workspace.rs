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
  // Find workspace
  let workspace = match state.find_by_id_with_pool(auth_user.workspace_id).await {
    Ok(Some(workspace)) => workspace,
    Ok(None) => {
      // 工作区不存在，可能是数据错误
      // 为调试目的，打印更多信息
      tracing::error!(
        "Workspace ID {} not found for user {} (email: {})",
        auth_user.workspace_id,
        auth_user.id,
        auth_user.email
      );

      // 允许继续请求，使用空的工作区上下文
      let workspace = Workspace {
        id: auth_user.workspace_id,
        name: "Default".to_string(),
        owner_id: auth_user.id,
        created_at: chrono::Utc::now(),
      };

      // 尝试创建这个工作区以修复问题
      if let Err(e) = state
        .create_workspace_with_pool(&workspace.name, auth_user.id)
        .await
      {
        tracing::warn!("Failed to create workspace: {:?}", e);
      }

      workspace
    }
    Err(e) => {
      tracing::error!("Database error when finding workspace: {:?}", e);
      return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
  };

  // Add workspace context to request extensions
  let ctx = WorkspaceContext::new(workspace);
  request.extensions_mut().insert(ctx);

  // Continue processing request
  next.run(request).await
}
