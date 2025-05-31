use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// 更新用户资料请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateUserProfileRequest {
  #[validate(length(
    min = 2,
    max = 50,
    message = "Full name must be between 2 and 50 characters"
  ))]
  #[schema(example = "John Doe")]
  pub fullname: Option<String>,

  #[validate(email(message = "Invalid email format"))]
  #[schema(example = "newemail@example.com")]
  pub email: Option<String>,

  #[schema(example = "+1234567890")]
  pub phone: Option<String>,

  #[schema(example = "Software Engineer")]
  pub title: Option<String>,

  #[schema(example = "Engineering")]
  pub department: Option<String>,

  #[schema(example = "https://example.com/avatar.jpg")]
  pub avatar_url: Option<String>,

  #[validate(length(max = 500, message = "Bio cannot exceed 500 characters"))]
  #[schema(example = "Passionate software engineer with 5 years of experience")]
  pub bio: Option<String>,

  #[schema(example = "America/New_York")]
  pub timezone: Option<String>,

  #[schema(example = "en")]
  pub language: Option<String>,
}

/// 用户设置更新请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateUserSettingsRequest {
  #[schema(example = true)]
  pub email_notifications: Option<bool>,

  #[schema(example = false)]
  pub push_notifications: Option<bool>,

  #[schema(example = true)]
  pub desktop_notifications: Option<bool>,

  #[schema(example = "all")]
  pub notification_sound: Option<String>, // "all", "mentions", "none"

  #[schema(example = true)]
  pub show_online_status: Option<bool>,

  #[schema(example = false)]
  pub auto_away: Option<bool>,

  #[schema(example = 15)]
  pub auto_away_minutes: Option<i32>,

  #[schema(example = "dark")]
  pub theme: Option<String>, // "light", "dark", "auto"

  #[schema(example = "compact")]
  pub message_display: Option<String>, // "comfortable", "compact"
}

/// 用户状态更新请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateUserStatusRequest {
  #[schema(example = "online")]
  pub status: String, // "online", "away", "busy", "offline"

  #[schema(example = "In a meeting")]
  pub status_message: Option<String>,

  #[schema(example = "2024-12-31T23:59:59Z")]
  pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 搜索用户请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct SearchUsersRequest {
  #[validate(length(
    min = 1,
    max = 100,
    message = "Query must be between 1 and 100 characters"
  ))]
  #[schema(example = "john")]
  pub query: String,

  #[schema(example = 1)]
  pub workspace_id: Option<i64>,

  #[schema(example = "Engineering")]
  pub department: Option<String>,

  #[schema(example = "Software Engineer")]
  pub title: Option<String>,

  #[validate(range(min = 1, max = 100, message = "Limit must be between 1 and 100"))]
  #[schema(example = 20)]
  pub limit: Option<i32>,

  #[validate(range(min = 0, message = "Offset must be non-negative"))]
  #[schema(example = 0)]
  pub offset: Option<i32>,
}

/// 用户权限更新请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateUserPermissionsRequest {
  #[schema(example = "admin")]
  pub role: Option<String>, // "admin", "moderator", "member"

  #[schema(example = "[\"create_chat\", \"delete_message\"]")]
  pub permissions: Option<Vec<String>>,

  #[schema(example = true)]
  pub can_create_public_channels: Option<bool>,

  #[schema(example = false)]
  pub can_delete_messages: Option<bool>,

  #[schema(example = true)]
  pub can_manage_users: Option<bool>,
}

/// 阻止用户请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BlockUserRequest {
  #[schema(example = 123)]
  pub user_id: i64,

  #[schema(example = "Inappropriate behavior")]
  pub reason: Option<String>,
}

/// 解除阻止用户请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UnblockUserRequest {
  #[schema(example = 123)]
  pub user_id: i64,
}

/// 用户活动日志请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UserActivityRequest {
  #[schema(example = "2024-01-01T00:00:00Z")]
  pub start_date: Option<chrono::DateTime<chrono::Utc>>,

  #[schema(example = "2024-12-31T23:59:59Z")]
  pub end_date: Option<chrono::DateTime<chrono::Utc>>,

  #[schema(example = "login")]
  pub activity_type: Option<String>, // "login", "message", "file_upload", etc.

  #[validate(range(min = 1, max = 100, message = "Limit must be between 1 and 100"))]
  #[schema(example = 50)]
  pub limit: Option<i32>,
}

/// 删除用户账户请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct DeleteUserAccountRequest {
  #[validate(length(min = 6, message = "Password is required for account deletion"))]
  #[schema(example = "password123")]
  pub password: String,

  #[schema(example = "No longer need the account")]
  pub reason: Option<String>,

  #[schema(example = true)]
  pub delete_all_data: Option<bool>,
}

/// 导出用户数据请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ExportUserDataRequest {
  #[schema(example = "json")]
  pub format: Option<String>, // "json", "csv"

  #[schema(example = true)]
  pub include_messages: Option<bool>,

  #[schema(example = true)]
  pub include_files: Option<bool>,

  #[schema(example = false)]
  pub include_deleted_data: Option<bool>,
}

/// 用户邀请请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct InviteUserRequest {
  #[validate(email(message = "Invalid email format"))]
  #[schema(example = "newuser@example.com")]
  pub email: String,

  #[schema(example = "member")]
  pub role: Option<String>,

  #[schema(example = "Welcome to our workspace!")]
  pub message: Option<String>,

  #[schema(example = 7)]
  pub expires_in_days: Option<i32>,
}

/// 批量用户操作请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct BatchUserOperationRequest {
  #[validate(length(min = 1, message = "At least one user ID is required"))]
  #[schema(example = "[1, 2, 3]")]
  pub user_ids: Vec<i64>,

  #[schema(example = "deactivate")]
  pub operation: String, // "activate", "deactivate", "delete", "change_role"

  #[schema(example = "member")]
  pub new_role: Option<String>, // 用于 change_role 操作

  #[schema(example = "Bulk operation")]
  pub reason: Option<String>,
}
