//! # Type-safe Layer - Compile-time State Management
//!
//! **Design Principles**:
//! - Use type system to ensure middleware execution order
//! - Compile-time validation of permission checks
//! - Zero runtime overhead state machine design
//! - Clear type-level documentation

use fechatter_core::models::AuthUser;
use std::marker::PhantomData;

// =============================================================================
// Authentication State Types - Single Responsibility: Represent Auth State
// =============================================================================

/// Unauthenticated State
///
/// **Single Responsibility**: Mark request as not yet authenticated
/// **Type Safety**: Prevent access to authenticated resources at compile-time
pub struct Unauthenticated;

/// Authenticated State
///
/// **Single Responsibility**: Mark request as authenticated, containing user info
/// **Type Parameter**: U - User type, typically AuthUser
/// **Type Safety**: Ensure user info exists at compile-time
#[derive(Debug, Clone)]
pub struct Authenticated<U> {
  pub user: U,
}

impl<U> Authenticated<U> {
  /// Create authenticated state
  ///
  /// **Single Responsibility**: Create auth state from user info
  pub fn new(user: U) -> Self {
    Self { user }
  }

  /// Get user info
  ///
  /// **Single Responsibility**: Provide immutable access to user info
  pub fn user(&self) -> &U {
    &self.user
  }

  /// Extract user info
  ///
  /// **Single Responsibility**: Consume state, return user info
  pub fn into_user(self) -> U {
    self.user
  }
}

// =============================================================================
// Workspace State Types - Single Responsibility: Represent Workspace Permission State
// =============================================================================

/// No Workspace Permission State
///
/// **Single Responsibility**: Mark request as not having workspace permission
/// **Type Safety**: Prevent access to workspace resources at compile-time
pub struct NoWorkspace;

/// With Workspace Permission State
///
/// **Single Responsibility**: Mark request as having workspace permission
/// **Type Parameter**: W - Workspace ID type, typically i64
/// **Type Safety**: Ensure workspace permission exists at compile-time
#[derive(Debug, Clone)]
pub struct WithWorkspace<W> {
  pub workspace: W,
}

impl<W> WithWorkspace<W> {
  /// Create workspace permission state
  ///
  /// **Single Responsibility**: Create permission state from workspace ID
  pub fn new(workspace: W) -> Self {
    Self { workspace }
  }

  /// Get workspace ID
  ///
  /// **Single Responsibility**: Provide immutable access to workspace ID
  pub fn workspace(&self) -> &W {
    &self.workspace
  }

  /// Extract workspace ID
  ///
  /// **Single Responsibility**: Consume state, return workspace ID
  pub fn into_workspace(self) -> W {
    self.workspace
  }
}

// =============================================================================
// Chat State Types - Single Responsibility: Represent Chat Permission State
// =============================================================================

/// No Chat Permission State
///
/// **Single Responsibility**: Mark request as not having chat permission
/// **Type Safety**: Prevent access to chat resources at compile-time
pub struct NoChat;

/// With Chat Permission State
///
/// **Single Responsibility**: Mark request as having chat permission
/// **Type Parameter**: C - Chat ID type, typically i64
/// **Type Safety**: Ensure chat permission exists at compile-time
#[derive(Debug, Clone)]
pub struct WithChat<C> {
  pub chat: C,
}

impl<C> WithChat<C> {
  /// Create chat permission state
  ///
  /// **Single Responsibility**: Create permission state from chat ID
  pub fn new(chat: C) -> Self {
    Self { chat }
  }

  /// Get chat ID
  ///
  /// **Single Responsibility**: Provide immutable access to chat ID
  pub fn chat(&self) -> &C {
    &self.chat
  }

  /// Extract chat ID
  ///
  /// **Single Responsibility**: Consume state, return chat ID
  pub fn into_chat(self) -> C {
    self.chat
  }
}

// =============================================================================
// Request Context Container - Single Responsibility: Combine All State Types
// =============================================================================

/// Type-safe Request Context
///
/// **Single Responsibility**: Combine auth, workspace and chat states
/// **Type Parameters**:
/// - Auth: Authentication state type (Unauthenticated | Authenticated<U>)
/// - Workspace: Workspace state type (NoWorkspace | WithWorkspace<W>)
/// - Chat: Chat state type (NoChat | WithChat<C>)
/// **Type Safety**: Ensure state transitions correctness at compile-time
#[derive(Debug, Clone)]
pub struct RequestContext<Auth = Unauthenticated, Workspace = NoWorkspace, Chat = NoChat> {
  auth: Auth,
  workspace: Workspace,
  chat: Chat,
}

// =============================================================================
// Initial State Construction - Single Responsibility: Create Initial State
// =============================================================================

impl Default for RequestContext {
  /// Create default unauthenticated context
  ///
  /// **Single Responsibility**: Provide default initial state
  fn default() -> Self {
    Self::new()
  }
}

impl RequestContext {
  /// Create new request context
  ///
  /// **Single Responsibility**: Create initial unauthenticated state
  pub fn new() -> Self {
    Self {
      auth: Unauthenticated,
      workspace: NoWorkspace,
      chat: NoChat,
    }
  }
}

// =============================================================================
// State Transitions - Single Responsibility: Type-safe State Transitions
// =============================================================================

/// Transition from unauthenticated to authenticated state
impl RequestContext<Unauthenticated, NoWorkspace, NoChat> {
  /// User authentication transition
  ///
  /// **Single Responsibility**: Transition from unauthenticated to authenticated state
  /// **Type Safety**: Can only be called in unauthenticated state
  /// **Returns**: Authenticated state without workspace permission
  pub fn authenticate(
    self,
    user: AuthUser,
  ) -> RequestContext<Authenticated<AuthUser>, NoWorkspace, NoChat> {
    RequestContext {
      auth: Authenticated::new(user),
      workspace: NoWorkspace,
      chat: NoChat,
    }
  }
}

/// Add workspace permission to authenticated state
impl<U> RequestContext<Authenticated<U>, NoWorkspace, NoChat> {
  /// Add workspace permission
  ///
  /// **Single Responsibility**: Add workspace permission for authenticated user
  /// **Type Safety**: Can only be called when authenticated without workspace permission
  /// **Returns**: Authenticated state with workspace permission but no chat permission
  pub fn with_workspace(
    self,
    workspace_id: i64,
  ) -> RequestContext<Authenticated<U>, WithWorkspace<i64>, NoChat> {
    RequestContext {
      auth: self.auth,
      workspace: WithWorkspace::new(workspace_id),
      chat: NoChat,
    }
  }
}

/// Add chat permission to workspace permission state
impl<U> RequestContext<Authenticated<U>, WithWorkspace<i64>, NoChat> {
  /// Add chat permission
  ///
  /// **Single Responsibility**: Add chat permission for user with workspace permission
  /// **Type Safety**: Can only be called with workspace permission but no chat permission
  /// **Returns**: State with full permissions
  pub fn with_chat(
    self,
    chat_id: i64,
  ) -> RequestContext<Authenticated<U>, WithWorkspace<i64>, WithChat<i64>> {
    RequestContext {
      auth: self.auth,
      workspace: self.workspace,
      chat: WithChat::new(chat_id),
    }
  }
}

// =============================================================================
// State Accessors - Single Responsibility: Provide Type-safe State Access
// =============================================================================

/// Accessors for authenticated state
impl<U, W, C> RequestContext<Authenticated<U>, W, C> {
  /// Get user info
  ///
  /// **Single Responsibility**: Provide access to authenticated user info
  /// **Type Safety**: Can only be called in authenticated state
  pub fn user(&self) -> &U {
    self.auth.user()
  }

  /// Get authentication state
  ///
  /// **Single Responsibility**: Provide complete access to auth state
  pub fn auth(&self) -> &Authenticated<U> {
    &self.auth
  }
}

/// Accessors for workspace permission state
impl<A, W, C> RequestContext<A, WithWorkspace<W>, C> {
  /// Get workspace ID
  ///
  /// **Single Responsibility**: Provide access to workspace ID
  /// **Type Safety**: Can only be called with workspace permission
  pub fn workspace_id(&self) -> &W {
    self.workspace.workspace()
  }

  /// Get workspace state
  ///
  /// **Single Responsibility**: Provide complete access to workspace state
  pub fn workspace(&self) -> &WithWorkspace<W> {
    &self.workspace
  }
}

/// Accessors for chat permission state
impl<A, W, C> RequestContext<A, W, WithChat<C>> {
  /// Get chat ID
  ///
  /// **Single Responsibility**: Provide access to chat ID
  /// **Type Safety**: Can only be called with chat permission
  pub fn chat_id(&self) -> &C {
    self.chat.chat()
  }

  /// Get chat state
  ///
  /// **Single Responsibility**: Provide complete access to chat state
  pub fn chat(&self) -> &WithChat<C> {
    &self.chat
  }
}

// =============================================================================
// Convenient Type Aliases - Single Responsibility: Simplify Common Type Combinations
// =============================================================================

/// Basic Authentication Context
///
/// **Single Responsibility**: Simplify authenticated state without other permissions
pub type AuthContext = RequestContext<Authenticated<AuthUser>, NoWorkspace, NoChat>;

/// Workspace Context
///
/// **Single Responsibility**: Simplify authenticated state with workspace permission
pub type WorkspaceContext = RequestContext<Authenticated<AuthUser>, WithWorkspace<i64>, NoChat>;

/// Chat Context
///
/// **Single Responsibility**: Simplify state with full permissions
pub type ChatContext = RequestContext<Authenticated<AuthUser>, WithWorkspace<i64>, WithChat<i64>>;

/// Unauthenticated Context
///
/// **Single Responsibility**: Simplify unauthenticated state
pub type UnauthenticatedContext = RequestContext<Unauthenticated, NoWorkspace, NoChat>;

// =============================================================================
// Permission Validation Helpers - Single Responsibility: Compile-time Permission Checks
// =============================================================================

/// Permission Validation Trait
///
/// **Single Responsibility**: Provide compile-time permission validation interface
pub trait HasPermission<P> {
  /// Check if has specific permission
  fn has_permission(&self) -> bool;
}

/// Authentication Permission Marker
pub struct AuthPermission;

/// Workspace Permission Marker
pub struct WorkspacePermission;

/// Chat Permission Marker
pub struct ChatPermission;

// Implement auth permission for authenticated state
impl<U, W, C> HasPermission<AuthPermission> for RequestContext<Authenticated<U>, W, C> {
  fn has_permission(&self) -> bool {
    true
  }
}

// Implement workspace permission for workspace permission state
impl<A, W, C> HasPermission<WorkspacePermission> for RequestContext<A, WithWorkspace<W>, C> {
  fn has_permission(&self) -> bool {
    true
  }
}

// Implement chat permission for chat permission state
impl<A, W, C> HasPermission<ChatPermission> for RequestContext<A, W, WithChat<C>> {
  fn has_permission(&self) -> bool {
    true
  }
}

// =============================================================================
// Test Module - Single Responsibility: Validate Type Safety
// =============================================================================

#[cfg(test)]
mod tests {
  use super::*;
  use fechatter_core::models::AuthUser;

  #[test]
  fn test_state_transitions() {
    // Create initial context
    let ctx = RequestContext::new();

    // Authentication transition
    let user = AuthUser {
      id: 1.into(),
      fullname: "Test User".to_string(),
      email: "test@example.com".to_string(),
      status: fechatter_core::UserStatus::Active,
      created_at: chrono::Utc::now(),
      workspace_id: 1,
    };

    let auth_ctx = ctx.authenticate(user);
    assert_eq!(auth_ctx.user().id, 1.into());

    // Workspace permission transition
    let workspace_ctx = auth_ctx.with_workspace(123);
    assert_eq!(*workspace_ctx.workspace_id(), 123);

    // Chat permission transition
    let chat_ctx = workspace_ctx.with_chat(456);
    assert_eq!(*chat_ctx.chat_id(), 456);
  }

  #[test]
  fn test_permission_checking() {
    let user = AuthUser {
      id: 1.into(),
      fullname: "Test User".to_string(),
      email: "test@example.com".to_string(),
      status: fechatter_core::UserStatus::Active,
      created_at: chrono::Utc::now(),
      workspace_id: 1,
    };

    let ctx = RequestContext::new()
      .authenticate(user)
      .with_workspace(123)
      .with_chat(456);

    // Validate permissions
    assert!(ctx.has_permission::<AuthPermission>());
    assert!(ctx.has_permission::<WorkspacePermission>());
    assert!(ctx.has_permission::<ChatPermission>());
  }

  #[test]
  fn test_type_aliases() {
    let user = AuthUser {
      id: 1.into(),
      fullname: "Test User".to_string(),
      email: "test@example.com".to_string(),
      status: fechatter_core::UserStatus::Active,
      created_at: chrono::Utc::now(),
      workspace_id: 1,
    };

    let _auth_ctx: AuthContext = RequestContext::new().authenticate(user.clone());
    let _workspace_ctx: WorkspaceContext = RequestContext::new()
      .authenticate(user.clone())
      .with_workspace(123);
    let _chat_ctx: ChatContext = RequestContext::new()
      .authenticate(user)
      .with_workspace(123)
      .with_chat(456);
  }
}
