use axum::{
  extract::{Request, State},
  http::StatusCode,
  middleware::Next,
  response::Response,
};
use tracing::{info, warn};

use crate::{AppState, error::AppError};
use fechatter_core::{AuthUser, UserId};

/// =============================================================================
/// Authorization Middleware - Unified Permission Validation
/// Responsibility: Ensure all requests reaching Service layer have passed permission validation
/// =============================================================================

/// Permission type enum
#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
  /// Chat view permission
  ChatView(i64),
  /// Chat message send permission
  ChatSendMessage(i64),
  /// Chat management permission
  ChatManage(i64),
  /// Message edit permission
  MessageEdit(i64),
  /// Message delete permission
  MessageDelete(i64),
  /// Workspace access permission
  WorkspaceAccess(i64),
}

/// Authorization context - Passed to Handler
#[derive(Debug, Clone)]
pub struct AuthContext {
  pub user: AuthUser,
  pub verified_permissions: Vec<Permission>,
  pub chat_id: Option<i64>,
  pub message_id: Option<i64>,
  pub workspace_id: i64,
}

/// Main authorization middleware
pub async fn authorization_middleware(
  State(state): State<AppState>,
  mut request: Request,
  next: Next,
) -> Result<Response, AppError> {
  // 1. Get authenticated user
  let user = request
    .extensions()
    .get::<AuthUser>()
    .ok_or_else(|| AppError::Unauthorized("User not authenticated".to_string()))?
    .clone();

  // 2. Parse request path to determine required permissions
  let required_permissions = parse_required_permissions(&request)?;

  // 3. Verify permissions
  let mut verified_permissions = Vec::new();
  for permission in required_permissions {
    verify_permission(&state, &user, &permission).await?;
    verified_permissions.push(permission);
  }

  // 4. Build authorization context
  let auth_context = AuthContext {
    user: user.clone(),
    verified_permissions: verified_permissions.clone(),
    chat_id: extract_chat_id(&request),
    message_id: extract_message_id(&request),
    workspace_id: i64::from(user.workspace_id),
  };

  // 5. Add context to request
  request.extensions_mut().insert(auth_context);

  info!(
    user_id = %user.id,
    permissions = ?verified_permissions,
    "Authorization successful"
  );

  Ok(next.run(request).await)
}

/// Parse required permissions from request
fn parse_required_permissions(request: &Request) -> Result<Vec<Permission>, AppError> {
  let method = request.method();
  let path = request.uri().path();
  let mut permissions = Vec::new();

  // Chat related permissions
  if let Some(chat_id) = extract_chat_id(request) {
    match method.as_str() {
      "GET" => permissions.push(Permission::ChatView(chat_id)),
      "POST" if path.contains("/messages") => {
        permissions.push(Permission::ChatSendMessage(chat_id));
      }
      "PUT" | "PATCH" if path.contains("/chats/") => {
        permissions.push(Permission::ChatManage(chat_id));
      }
      _ => permissions.push(Permission::ChatView(chat_id)),
    }
  }

  // Message related permissions
  if let Some(message_id) = extract_message_id(request) {
    match method.as_str() {
      "PUT" | "PATCH" => permissions.push(Permission::MessageEdit(message_id)),
      "DELETE" => permissions.push(Permission::MessageDelete(message_id)),
      _ => {} // Other operations covered by chat permissions
    }
  }

  Ok(permissions)
}

/// Verify single permission
async fn verify_permission(
  state: &AppState,
  user: &AuthUser,
  permission: &Permission,
) -> Result<(), AppError> {
  match permission {
    Permission::ChatView(chat_id) | Permission::ChatSendMessage(chat_id) => {
      // TODO: Verify user is chat member
      // state.ensure_user_is_chat_member(*chat_id, i64::from(user.id)).await?;
    }

    Permission::ChatManage(chat_id) => {
      // TODO: Verify chat management permission (creator or admin)
      // state.ensure_user_is_chat_member(*chat_id, i64::from(user.id)).await?;
      // TODO: Additional admin check
    }

    Permission::MessageEdit(message_id) | Permission::MessageDelete(message_id) => {
      // Verify message ownership
      verify_message_ownership(state, user.id, *message_id).await?;
    }

    Permission::WorkspaceAccess(workspace_id) => {
      // Verify workspace access permission
      if i64::from(user.workspace_id) != *workspace_id {
        return Err(AppError::ChatPermissionError(
          "User does not belong to this workspace".to_string(),
        ));
      }
    }
  }

  Ok(())
}

/// Verify message ownership
async fn verify_message_ownership(
  _state: &AppState,
  _user_id: UserId,
  _message_id: i64,
) -> Result<(), AppError> {
  // TODO: Implement message ownership verification
  // 1. Query message sender
  // 2. Verify if current user
  // 3. Check time limit (15 min edit window)
  Ok(())
}

/// Extract chat_id from request
fn extract_chat_id(request: &Request) -> Option<i64> {
  let path = request.uri().path();

  // Match /api/chats/{id}
  if let Some(prefix) = path.strip_prefix("/api/chats/") {
    prefix.split('/').next()?.parse().ok()
  } else {
    // TODO: Extract chat_id from query params
    None
  }
}

/// Extract message_id from request
fn extract_message_id(request: &Request) -> Option<i64> {
  let path = request.uri().path();

  // Match /api/messages/{id}
  if let Some(prefix) = path.strip_prefix("/api/messages/") {
    prefix.split('/').next()?.parse().ok()
  } else {
    None
  }
}
