// Essential types for compilation compatibility - services moved to workers module

// Essential types for compilation compatibility
use crate::AppError;
use async_trait::async_trait;
use fechatter_core::UserStatus;
use serde::{Deserialize, Serialize};

/// User profile view model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileView {
  pub id: i64,
  pub fullname: String,
  pub email: String,
  pub status: String,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
  pub workspace_id: Option<i64>,
}

/// Update user input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserInput {
  pub fullname: Option<String>,
  pub email: Option<String>,
}

/// Change password input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordInput {
  pub current_password: String,
  pub new_password: String,
}

/// User service trait
#[async_trait]
pub trait UserServiceTrait: Send + Sync {
  async fn get_user_profile(&self, user_id: i64) -> Result<UserProfileView, AppError>;
  async fn update_user_profile(
    &self,
    user_id: i64,
    input: UpdateUserInput,
  ) -> Result<UserProfileView, AppError>;
  async fn change_password(&self, user_id: i64, input: ChangePasswordInput)
  -> Result<(), AppError>;
  async fn update_user_status(&self, user_id: i64, status: UserStatus) -> Result<(), AppError>;
  async fn validate_users_exist(&self, user_ids: Vec<i64>) -> Result<(), AppError>;
  async fn find_users_by_workspace(
    &self,
    workspace_id: i64,
  ) -> Result<Vec<UserProfileView>, AppError>;
}
