use crate::{
  AuthUser,
  error::CoreError,
  models::jwt::{AuthTokens, UserClaims},
  models::{CreateUser, SigninUser, UserId},
};
use async_trait::async_trait;

/// 认证服务契约
#[async_trait]
pub trait AuthService: Send + Sync {
  /// 用户注册
  async fn signup(
    &self,
    payload: &CreateUser,
    auth_context: Option<AuthContext>,
  ) -> Result<AuthTokens, CoreError>;

  /// 用户登录
  async fn signin(
    &self,
    payload: &SigninUser,
    auth_context: Option<AuthContext>,
  ) -> Result<Option<AuthTokens>, CoreError>;

  /// 刷新令牌
  async fn refresh_token(
    &self,
    refresh_token: &str,
    auth_context: Option<AuthContext>,
  ) -> Result<AuthTokens, CoreError>;

  /// 登出
  async fn logout(&self, refresh_token: &str) -> Result<(), CoreError>;

  /// 登出所有设备
  async fn logout_all(&self, user_id: UserId) -> Result<(), CoreError>;

  /// 验证令牌
  fn verify_token(&self, token: &str) -> Result<UserClaims, CoreError>;

  /// 从声明创建用户
  fn user_from_claims(&self, claims: UserClaims) -> AuthUser;
}

/// 工作空间服务契约
#[async_trait]
pub trait WorkspaceService: Send + Sync {
  /// 切换用户工作空间
  async fn switch_user_workspace(
    &self,
    user_id: UserId,
    workspace_id: crate::models::WorkspaceId,
  ) -> Result<SwitchWorkspaceResponse, CoreError>;

  /// 列出所有工作空间
  async fn list_all_workspaces(&self) -> Result<Vec<crate::models::Workspace>, CoreError>;

  /// 检查工作空间访问权限
  async fn check_workspace_access(
    &self,
    user_id: UserId,
    workspace_id: crate::models::WorkspaceId,
  ) -> Result<bool, CoreError>;
}

/// 认证上下文
#[derive(Debug, Clone, Default)]
pub struct AuthContext {
  pub user_agent: Option<String>,
  pub ip_address: Option<String>,
}

/// 切换工作空间响应
#[derive(Debug, Clone)]
pub struct SwitchWorkspaceResponse {
  pub success: bool,
  pub new_workspace_id: crate::models::WorkspaceId,
  pub message: String,
}
