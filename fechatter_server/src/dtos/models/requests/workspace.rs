use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// 创建工作空间请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateWorkspaceRequest {
  #[validate(length(
    min = 2,
    max = 100,
    message = "Workspace name must be between 2 and 100 characters"
  ))]
  #[schema(example = "My Company")]
  pub name: String,

  #[validate(length(max = 500, message = "Description cannot exceed 500 characters"))]
  #[schema(example = "Our company workspace for team collaboration")]
  pub description: Option<String>,

  #[schema(example = "https://example.com/logo.png")]
  pub logo_url: Option<String>,

  #[schema(example = "company")]
  pub workspace_type: Option<String>, // "company", "team", "personal", "community"
}

/// 更新工作空间请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateWorkspaceRequest {
  #[validate(length(
    min = 2,
    max = 100,
    message = "Workspace name must be between 2 and 100 characters"
  ))]
  #[schema(example = "Updated Company Name")]
  pub name: Option<String>,

  #[validate(length(max = 500, message = "Description cannot exceed 500 characters"))]
  #[schema(example = "Updated description for our workspace")]
  pub description: Option<String>,

  #[schema(example = "https://example.com/new-logo.png")]
  pub logo_url: Option<String>,

  #[schema(example = "company")]
  pub workspace_type: Option<String>,
}

/// 邀请用户到工作空间请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct InviteWorkspaceMemberRequest {
  #[validate(email(message = "Invalid email format"))]
  #[schema(example = "newmember@example.com")]
  pub email: String,

  #[schema(example = "member")]
  pub role: String, // "owner", "admin", "member"

  #[schema(example = "Welcome to our workspace!")]
  pub message: Option<String>,

  #[schema(example = 7)]
  pub expires_in_days: Option<i32>,
}

/// 更新工作空间成员角色请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateWorkspaceMemberRoleRequest {
  #[schema(example = "admin")]
  pub role: String, // "owner", "admin", "member"

  #[schema(example = "Promoting to admin role")]
  pub reason: Option<String>,
}

/// 移除工作空间成员请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RemoveWorkspaceMemberRequest {
  #[schema(example = "User violated workspace policies")]
  pub reason: Option<String>,

  #[schema(example = false)]
  pub delete_user_data: Option<bool>,
}

/// 转移工作空间所有权请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TransferWorkspaceOwnershipRequest {
  #[schema(example = 123)]
  pub new_owner_id: i64,

  #[schema(example = "Transferring ownership due to role change")]
  pub reason: Option<String>,
}

/// 工作空间设置更新请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateWorkspaceSettingsRequest {
  #[schema(example = true)]
  pub allow_public_channels: Option<bool>,

  #[schema(example = false)]
  pub require_approval_for_join: Option<bool>,

  #[schema(example = true)]
  pub allow_guest_users: Option<bool>,

  #[schema(example = 30)]
  pub message_retention_days: Option<i32>,

  #[schema(example = 100)]
  pub max_file_size_mb: Option<i32>,

  #[schema(example = "[\"jpg\", \"png\", \"pdf\", \"docx\"]")]
  pub allowed_file_types: Option<Vec<String>>,

  #[schema(example = "America/New_York")]
  pub default_timezone: Option<String>,

  #[schema(example = "en")]
  pub default_language: Option<String>,
}

/// 工作空间统计请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WorkspaceStatsRequest {
  #[schema(example = "2024-01-01T00:00:00Z")]
  pub start_date: Option<chrono::DateTime<chrono::Utc>>,

  #[schema(example = "2024-12-31T23:59:59Z")]
  pub end_date: Option<chrono::DateTime<chrono::Utc>>,

  #[schema(example = "day")]
  pub group_by: Option<String>, // "hour", "day", "week", "month"

  #[schema(example = true)]
  pub include_user_activity: Option<bool>,

  #[schema(example = true)]
  pub include_message_stats: Option<bool>,
}

/// 搜索工作空间请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct SearchWorkspacesRequest {
  #[validate(length(
    min = 1,
    max = 100,
    message = "Query must be between 1 and 100 characters"
  ))]
  #[schema(example = "tech")]
  pub query: String,

  #[schema(example = "company")]
  pub workspace_type: Option<String>,

  #[validate(range(min = 1, max = 100, message = "Limit must be between 1 and 100"))]
  #[schema(example = 20)]
  pub limit: Option<i32>,

  #[validate(range(min = 0, message = "Offset must be non-negative"))]
  #[schema(example = 0)]
  pub offset: Option<i32>,
}

/// 工作空间备份请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateWorkspaceBackupRequest {
  #[schema(example = true)]
  pub include_messages: Option<bool>,

  #[schema(example = true)]
  pub include_files: Option<bool>,

  #[schema(example = false)]
  pub include_deleted_data: Option<bool>,

  #[schema(example = "Monthly backup")]
  pub description: Option<String>,
}

/// 工作空间恢复请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RestoreWorkspaceRequest {
  #[schema(example = "backup_id_here")]
  pub backup_id: String,

  #[schema(example = false)]
  pub overwrite_existing: Option<bool>,

  #[schema(example = "Restoring from monthly backup")]
  pub reason: Option<String>,
}

/// 工作空间集成设置请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateWorkspaceIntegrationsRequest {
  #[schema(example = true)]
  pub enable_slack_integration: Option<bool>,

  #[schema(example = false)]
  pub enable_teams_integration: Option<bool>,

  #[schema(example = true)]
  pub enable_email_integration: Option<bool>,

  #[schema(example = "webhook_url_here")]
  pub webhook_url: Option<String>,

  #[schema(example = "api_key_here")]
  pub api_key: Option<String>,
}

/// 工作空间安全设置请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateWorkspaceSecurityRequest {
  #[schema(example = true)]
  pub require_2fa: Option<bool>,

  #[schema(example = false)]
  pub allow_password_login: Option<bool>,

  #[schema(example = true)]
  pub require_sso: Option<bool>,

  #[schema(example = 30)]
  pub session_timeout_minutes: Option<i32>,

  #[schema(example = "[\"192.168.1.0/24\"]")]
  pub allowed_ip_ranges: Option<Vec<String>>,

  #[schema(example = 5)]
  pub max_login_attempts: Option<i32>,
}

/// 删除工作空间请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct DeleteWorkspaceRequest {
  #[validate(length(min = 6, message = "Password is required for workspace deletion"))]
  #[schema(example = "password123")]
  pub password: String,

  #[schema(example = "No longer needed")]
  pub reason: Option<String>,

  #[schema(example = true)]
  pub delete_all_data: Option<bool>,

  #[schema(example = "DELETE")]
  pub confirmation: String, // 必须输入 "DELETE" 确认
}
