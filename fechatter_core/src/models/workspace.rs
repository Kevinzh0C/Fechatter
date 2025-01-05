use crate::{
  error::CoreError,
  models::{ChatUser, Workspace},
};
use serde::{Deserialize, Serialize};
use std::future::Future;

pub trait WorkspaceRepository: Send + Sync {
  fn create(&self, name: &str) -> impl Future<Output = Result<Workspace, CoreError>> + Send;

  fn find_by_id(
    &self,
    id: i64,
  ) -> impl Future<Output = Result<Option<Workspace>, CoreError>> + Send;

  fn find_by_name(
    &self,
    name: &str,
  ) -> impl Future<Output = Result<Option<Workspace>, CoreError>> + Send;

  fn update_owner(
    &self,
    id: i64,
    owner_id: i64,
  ) -> impl Future<Output = Result<Workspace, CoreError>> + Send;

  fn fetch_all_users(
    &self,
    workspace_id: i64,
  ) -> impl Future<Output = Result<Vec<ChatUser>, CoreError>> + Send;

  fn add_user_to_workspace(
    &self,
    workspace_id: i64,
    user_id: i64,
  ) -> impl Future<Output = Result<Workspace, CoreError>> + Send;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
