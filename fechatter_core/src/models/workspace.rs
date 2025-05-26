use crate::{
  error::CoreError,
  models::{ChatUser, UserId, Workspace, WorkspaceId},
};
use serde::{Deserialize, Serialize};
use std::future::Future;
use utoipa::ToSchema;

pub trait WorkspaceRepository: Send + Sync {
  fn create(&self, name: &str) -> impl Future<Output = Result<Workspace, CoreError>> + Send;

  fn find_by_id(
    &self,
    id: WorkspaceId,
  ) -> impl Future<Output = Result<Option<Workspace>, CoreError>> + Send;

  fn find_by_name(
    &self,
    name: &str,
  ) -> impl Future<Output = Result<Option<Workspace>, CoreError>> + Send;

  fn update_owner(
    &self,
    id: WorkspaceId,
    owner_id: UserId,
  ) -> impl Future<Output = Result<Workspace, CoreError>> + Send;

  fn fetch_all_users(
    &self,
    workspace_id: WorkspaceId,
  ) -> impl Future<Output = Result<Vec<ChatUser>, CoreError>> + Send;

  fn add_user_to_workspace(
    &self,
    workspace_id: WorkspaceId,
    user_id: UserId,
  ) -> impl Future<Output = Result<Workspace, CoreError>> + Send;
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateWorkspace {
  pub name: String,
}

// Validation functions that don't require database access
pub fn validate_workspace_name(name: &str) -> Result<(), CoreError> {
  if name.trim().is_empty() {
    return Err(CoreError::Validation(
      "Workspace name cannot be empty".into(),
    ));
  }

  if name.len() > 100 {
    return Err(CoreError::Validation(
      "Workspace name too long (max 100 characters)".into(),
    ));
  }

  Ok(())
}
