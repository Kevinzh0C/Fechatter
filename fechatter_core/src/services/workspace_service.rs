use crate::{
  error::CoreError,
  models::{
    user::UserRepository, workspace::WorkspaceRepository, User, UserId, Workspace, WorkspaceId,
  },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SwitchWorkspaceRequest {
  pub workspace_id: WorkspaceId,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SwitchWorkspaceResponse {
  pub success: bool,
  pub message: String,
  pub user: User,
  pub workspace: Workspace,
}

/// Service trait for workspace operations
pub trait WorkspaceService: Send + Sync {
  /// Switch user's workspace
  async fn switch_user_workspace(
    &self,
    user_id: UserId,
    workspace_id: WorkspaceId,
  ) -> Result<SwitchWorkspaceResponse, CoreError>;

  /// List all available workspaces
  async fn list_all_workspaces(&self) -> Result<Vec<Workspace>, CoreError>;

  /// Check if user has access to the specified workspace
  async fn check_workspace_access(
    &self,
    user_id: UserId,
    workspace_id: WorkspaceId,
  ) -> Result<bool, CoreError>;
}

/// Core workspace service implementation
pub struct CoreWorkspaceService<U, W> {
  user_repository: U,
  workspace_repository: W,
}

impl<U, W> CoreWorkspaceService<U, W>
where
  U: UserRepository + Sync,
  W: WorkspaceRepository + Sync,
{
  pub fn new(user_repository: U, workspace_repository: W) -> Self {
    Self {
      user_repository,
      workspace_repository,
    }
  }
}

impl<U, W> WorkspaceService for CoreWorkspaceService<U, W>
where
  U: UserRepository + Sync,
  W: WorkspaceRepository + Sync,
{
  async fn switch_user_workspace(
    &self,
    user_id: UserId,
    workspace_id: WorkspaceId,
  ) -> Result<SwitchWorkspaceResponse, CoreError> {
    // First verify that the workspace exists
    let workspace = self
      .workspace_repository
      .find_by_id(workspace_id)
      .await?
      .ok_or_else(|| {
        CoreError::NotFound(format!("Workspace {} not found", workspace_id.inner()))
      })?;

    // Check if user has access to this workspace
    let has_access = self
      .workspace_repository
      .check_user_access(user_id, workspace_id)
      .await?;

    if !has_access {
      return Err(CoreError::Unauthorized(
        "User does not have access to this workspace".to_string(),
      ));
    }

    // Switch user's workspace
    let updated_user = self
      .user_repository
      .switch_workspace(user_id, workspace_id)
      .await?;

    Ok(SwitchWorkspaceResponse {
      success: true,
      message: format!("Successfully switched to workspace '{}'", workspace.name),
      user: updated_user,
      workspace,
    })
  }

  async fn list_all_workspaces(&self) -> Result<Vec<Workspace>, CoreError> {
    self.workspace_repository.list_all().await
  }

  async fn check_workspace_access(
    &self,
    user_id: UserId,
    workspace_id: WorkspaceId,
  ) -> Result<bool, CoreError> {
    self
      .workspace_repository
      .check_user_access(user_id, workspace_id)
      .await
  }
}
