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
    Ok(None) => return StatusCode::NOT_FOUND.into_response(),
    Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  };

  // Add workspace context to request extensions
  let ctx = WorkspaceContext::new(workspace);
  request.extensions_mut().insert(ctx);

  // Continue processing request
  next.run(request).await
}
