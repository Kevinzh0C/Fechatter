use crate::{
  error::CoreError,
  models::jwt::{AuthTokens, UserClaims},
  models::{CreateUser, SigninUser, UserId},
  AuthUser,
};
use async_trait::async_trait;

/// Authentication service contract
#[async_trait]
pub trait AuthService: Send + Sync {
  /// User registration
  async fn signup(
    &self,
    payload: &CreateUser,
    auth_context: Option<AuthContext>,
  ) -> Result<AuthTokens, CoreError>;

  /// User login
  async fn signin(
    &self,
    payload: &SigninUser,
    auth_context: Option<AuthContext>,
  ) -> Result<Option<AuthTokens>, CoreError>;

  /// Refresh token
  async fn refresh_token(
    &self,
    refresh_token: &str,
    auth_context: Option<AuthContext>,
  ) -> Result<AuthTokens, CoreError>;

  /// Logout
  async fn logout(&self, refresh_token: &str) -> Result<(), CoreError>;

  /// Logout from all devices
  async fn logout_all(&self, user_id: UserId) -> Result<(), CoreError>;

  /// Verify token
  fn verify_token(&self, token: &str) -> Result<UserClaims, CoreError>;

  /// Create user from claims
  fn user_from_claims(&self, claims: UserClaims) -> AuthUser;
}

/// Workspace service contract
#[async_trait]
pub trait WorkspaceService: Send + Sync {
  /// Switch user workspace
  async fn switch_user_workspace(
    &self,
    user_id: UserId,
    workspace_id: crate::models::WorkspaceId,
  ) -> Result<SwitchWorkspaceResponse, CoreError>;

  /// List all workspaces
  async fn list_all_workspaces(&self) -> Result<Vec<crate::models::Workspace>, CoreError>;

  /// Check workspace access permissions
  async fn check_workspace_access(
    &self,
    user_id: UserId,
    workspace_id: crate::models::WorkspaceId,
  ) -> Result<bool, CoreError>;
}

/// Authentication context
#[derive(Debug, Clone, Default)]
pub struct AuthContext {
  pub user_agent: Option<String>,
  pub ip_address: Option<String>,
}

/// Switch workspace response
#[derive(Debug, Clone)]
pub struct SwitchWorkspaceResponse {
  pub success: bool,
  pub new_workspace_id: crate::models::WorkspaceId,
  pub message: String,
}
