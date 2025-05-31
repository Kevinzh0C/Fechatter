use fechatter_core::models::ChatType;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// 创建聊天请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateChatRequest {
  #[validate(length(
    min = 1,
    max = 128,
    message = "Chat name must be between 1 and 128 characters"
  ))]
  #[schema(example = "Project Discussion")]
  pub name: String,

  #[schema(example = "Group")]
  pub chat_type: ChatType,

  #[schema(example = "Discussion about the new project")]
  pub description: Option<String>,

  #[validate(length(
    min = 1,
    message = "At least one member is required for non-public chats"
  ))]
  #[schema(example = "[2, 3, 4]")]
  pub members: Option<Vec<i64>>,
}

/// 更新聊天请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateChatRequest {
  #[validate(length(
    min = 1,
    max = 128,
    message = "Chat name must be between 1 and 128 characters"
  ))]
  #[schema(example = "Updated Project Discussion")]
  pub name: Option<String>,

  #[schema(example = "Updated description for the project")]
  pub description: Option<String>,
}

/// 添加聊天成员请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct AddChatMembersRequest {
  #[validate(length(min = 1, message = "At least one member ID is required"))]
  #[schema(example = "[5, 6, 7]")]
  pub member_ids: Vec<i64>,
}

/// 移除聊天成员请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct RemoveChatMembersRequest {
  #[validate(length(min = 1, message = "At least one member ID is required"))]
  #[schema(example = "[5, 6]")]
  pub member_ids: Vec<i64>,
}

/// 加入公共聊天请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct JoinPublicChatRequest {
  #[schema(example = 123)]
  pub chat_id: i64,
}

/// 离开聊天请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LeaveChatRequest {
  #[schema(example = 123)]
  pub chat_id: i64,

  #[schema(example = "I'm leaving this chat")]
  pub reason: Option<String>,
}

/// 搜索聊天请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct SearchChatsRequest {
  #[validate(length(
    min = 1,
    max = 100,
    message = "Query must be between 1 and 100 characters"
  ))]
  #[schema(example = "project")]
  pub query: String,

  #[schema(example = "Group")]
  pub chat_type: Option<ChatType>,

  #[schema(example = 1)]
  pub workspace_id: Option<i64>,

  #[validate(range(min = 1, max = 100, message = "Limit must be between 1 and 100"))]
  #[schema(example = 20)]
  pub limit: Option<i32>,

  #[validate(range(min = 0, message = "Offset must be non-negative"))]
  #[schema(example = 0)]
  pub offset: Option<i32>,
}

/// 聊天设置更新请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateChatSettingsRequest {
  #[schema(example = true)]
  pub mute_notifications: Option<bool>,

  #[schema(example = false)]
  pub pin_chat: Option<bool>,

  #[schema(example = true)]
  pub show_previews: Option<bool>,

  #[schema(example = "all")]
  pub notification_level: Option<String>, // "all", "mentions", "none"
}

/// 聊天权限更新请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateChatPermissionsRequest {
  #[schema(example = true)]
  pub allow_member_invite: Option<bool>,

  #[schema(example = false)]
  pub allow_member_remove: Option<bool>,

  #[schema(example = true)]
  pub allow_message_edit: Option<bool>,

  #[schema(example = false)]
  pub allow_message_delete: Option<bool>,

  #[schema(example = true)]
  pub allow_file_upload: Option<bool>,
}

/// 批量操作聊天请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct BatchChatOperationRequest {
  #[validate(length(min = 1, message = "At least one chat ID is required"))]
  #[schema(example = "[1, 2, 3]")]
  pub chat_ids: Vec<i64>,

  #[schema(example = "archive")]
  pub operation: String, // "archive", "unarchive", "delete", "mute", "unmute"
}

/// 聊天邀请链接请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateChatInviteRequest {
  #[schema(example = 123)]
  pub chat_id: i64,

  #[schema(example = 7)]
  pub expires_in_days: Option<i32>,

  #[schema(example = 50)]
  pub max_uses: Option<i32>,
}

/// 使用邀请链接加入聊天请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct JoinChatByInviteRequest {
  #[schema(example = "invite_token_here")]
  pub invite_token: String,
}
