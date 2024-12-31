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
  AppError, AppState,
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
pub async fn ensure_workspace_member(
  State(state): State<AppState>,
  Extension(auth_user): Extension<AuthUser>,
  Path(ws_id): Path<i64>,
  request: Request<Body>,
  next: Next,
) -> Result<Response, StatusCode> {
  // Validate user is a member of the workspace
  if auth_user.workspace_id != ws_id {
    return Err(StatusCode::FORBIDDEN);
  }

  // Continue processing request
  Ok(next.run(request).await)
}

/// Extract workspace context from the request
pub async fn with_workspace_context(
  State(state): State<AppState>,
  Extension(auth_user): Extension<AuthUser>,
  mut request: Request<Body>,
  next: Next,
) -> Result<Response, StatusCode> {
  // Find workspace
  let workspace = state
    .find_workspace_by_id(auth_user.workspace_id, &state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

  // Add workspace context to request extensions
  let ctx = WorkspaceContext::new(workspace);
  request.extensions_mut().insert(ctx);

  // Continue processing request
  Ok(next.run(request).await)
}
