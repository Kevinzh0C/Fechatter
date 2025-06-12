//! # Request Context - Unified Request State
//!
//! **Responsibility**: Define unified request context containing auth user info, permissions etc.
//! **Principles**: Single data structure, clear state management

use super::permissions::Permission;
use fechatter_core::{AuthUser, UserId, WorkspaceId};

/// Unified request context - Contains all authentication and authorization related information
#[derive(Debug, Clone)]
pub struct RequestContext {
  /// Authenticated user information
  pub user: AuthUser,
  /// Workspace ID
  pub workspace_id: WorkspaceId,
  /// Chat ID (if in chat context)
  pub chat_id: Option<i64>,
  /// User permissions in current context
  pub permissions: Vec<Permission>,
}

impl RequestContext {
  /// Create new request context
  pub fn new(user: AuthUser) -> Self {
    Self {
      workspace_id: user.workspace_id,
      user,
      chat_id: None,
      permissions: vec![Permission::WorkspaceMember],
    }
  }

  /// Add chat context
  pub fn with_chat(mut self, chat_id: i64) -> Self {
    self.chat_id = Some(chat_id);
    self.permissions.push(Permission::ChatMember);
    self
  }

  /// Add permission
  pub fn add_permission(&mut self, permission: Permission) {
    if !self.permissions.contains(&permission) {
      self.permissions.push(permission);
    }
  }

  /// Check if has specific permission
  pub fn has_permission(&self, permission: Permission) -> bool {
    self.permissions.contains(&permission)
  }

  /// Get user ID
  pub fn user_id(&self) -> UserId {
    self.user.id
  }

  /// Get workspace ID
  pub fn workspace_id(&self) -> WorkspaceId {
    self.workspace_id
  }
}
