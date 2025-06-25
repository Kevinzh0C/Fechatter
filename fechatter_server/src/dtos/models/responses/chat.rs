use crate::dtos::core::{BaseDto, ConversionError, ResponseDto};
use crate::services::application::ChatDetailView;
use fechatter_core::{ChatType, models::chat::ChatSidebar};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 聊天侧边栏响应 (列表视图)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChatSidebarDto {
  #[schema(example = 123)]
  pub id: i64,

  #[schema(example = "Project Discussion")]
  pub name: String,

  #[schema(example = "Group")]
  pub chat_type: ChatType,

  #[schema(example = "Discussion about the new project")]
  pub description: Option<String>,

  #[schema(example = "Hello everyone!")]
  pub last_message: Option<String>,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub last_message_at: Option<chrono::DateTime<chrono::Utc>>,

  #[schema(example = 5)]
  pub unread_count: i32,

  #[schema(example = true)]
  pub is_creator: bool,

  #[schema(example = false)]
  pub is_muted: bool,

  #[schema(example = false)]
  pub is_pinned: bool,
}

impl BaseDto for ChatSidebarDto {
  fn dto_type() -> &'static str {
    "ChatSidebarDto"
  }

  fn validate(&self) -> Result<(), crate::dtos::core::DtoValidationError> {
    Ok(())
  }
}

impl ResponseDto for ChatSidebarDto {
  type DomainModel = ChatSidebar;

  fn from_domain(domain: &Self::DomainModel) -> Result<Self, ConversionError> {
    Ok(Self {
      id: domain.id.into(),
      name: domain.name.clone(),
      chat_type: domain.chat_type.clone(),
      description: None, // ChatSidebar doesn't have description, can be added if needed
      last_message: domain.last_message.as_ref().map(|msg| msg.content.clone()),
      last_message_at: None, // Can be added to domain if needed
      unread_count: 0,       // Can be added to domain if needed
      is_creator: domain.is_creator,
      is_muted: false,  // Can be added to domain if needed
      is_pinned: false, // Can be added to domain if needed
    })
  }

  fn from_domain_collection(domains: &[Self::DomainModel]) -> Result<Vec<Self>, ConversionError> {
    domains.iter().map(Self::from_domain).collect()
  }
}

/// 聊天详情响应 (详细视图)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChatDetailDto {
  #[schema(example = 123)]
  pub id: i64,

  #[schema(example = "Project Discussion")]
  pub name: String,

  #[schema(example = "Group")]
  pub chat_type: ChatType,

  #[schema(example = "Discussion about the new project")]
  pub description: Option<String>,

  #[schema(example = 789)]
  pub created_by: i64,

  #[schema(example = 456)]
  pub workspace_id: Option<i64>,

  #[schema(example = 5)]
  pub member_count: i32,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub created_at: chrono::DateTime<chrono::Utc>,

  #[schema(example = "2024-01-01T13:00:00Z")]
  pub updated_at: Option<chrono::DateTime<chrono::Utc>>,

  #[schema(example = false)]
  pub is_archived: bool,

  #[schema(example = true)]
  pub is_public: bool,
}

impl BaseDto for ChatDetailDto {
  fn dto_type() -> &'static str {
    "ChatDetailDto"
  }

  fn validate(&self) -> Result<(), crate::dtos::core::DtoValidationError> {
    Ok(())
  }
}

impl ResponseDto for ChatDetailDto {
  type DomainModel = ChatDetailView;

  fn from_domain(domain: &Self::DomainModel) -> Result<Self, ConversionError> {
    Ok(Self {
      id: domain.id,
      name: domain.name.clone(),
      chat_type: domain.chat_type.clone(),
      description: domain.description.clone(),
      created_by: domain.created_by,
      workspace_id: domain.workspace_id,
      member_count: domain.member_count,
      created_at: domain.created_at,
      updated_at: Some(domain.updated_at),
      is_archived: false, // Can be added to domain if needed
      is_public: false,   // Can be added to domain if needed
    })
  }

  fn from_domain_collection(domains: &[Self::DomainModel]) -> Result<Vec<Self>, ConversionError> {
    domains.iter().map(Self::from_domain).collect()
  }
}

/// 聊天创建响应
#[derive(Debug, Serialize, ToSchema)]
pub struct ChatCreateResponse {
  #[schema(example = true)]
  pub success: bool,

  #[schema(example = "Chat created successfully")]
  pub message: String,

  pub data: ChatDetailDto,
}

/// 聊天列表响应
pub type ChatListResponse = Vec<ChatSidebarDto>;

/// 聊天操作响应
#[derive(Debug, Serialize, ToSchema)]
pub struct ChatOperationResponse {
  #[schema(example = true)]
  pub success: bool,

  #[schema(example = "Chat updated successfully")]
  pub message: String,

  #[schema(example = 1)]
  pub affected_rows: Option<i64>,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ChatOperationResponse {
  pub fn success(message: String) -> Self {
    Self {
      success: true,
      message,
      affected_rows: Some(1),
      timestamp: chrono::Utc::now(),
    }
  }

  pub fn with_affected_rows(message: String, affected_rows: i64) -> Self {
    Self {
      success: true,
      message,
      affected_rows: Some(affected_rows),
      timestamp: chrono::Utc::now(),
    }
  }
}

/// 聊天成员响应
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChatMemberDto {
  #[schema(example = 123)]
  pub user_id: i64,

  #[schema(example = 456)]
  pub chat_id: i64,

  #[schema(example = "john_doe")]
  pub username: String,

  #[schema(example = "John Doe")]
  pub display_name: Option<String>,

  #[schema(example = "member")]
  pub role: String,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub joined_at: chrono::DateTime<chrono::Utc>,

  #[schema(example = true)]
  pub is_online: bool,

  #[schema(example = true)]
  pub is_creator: bool,
}

impl BaseDto for ChatMemberDto {
  fn dto_type() -> &'static str {
    "ChatMemberDto"
  }

  fn validate(&self) -> Result<(), crate::dtos::core::DtoValidationError> {
    Ok(())
  }
}

// Note: We'll need to define a domain model for ChatMember if it doesn't exist
// For now, we'll implement a simple conversion
impl ChatMemberDto {
  pub fn new(
    user_id: i64,
    chat_id: i64,
    username: String,
    display_name: Option<String>,
    role: String,
    joined_at: chrono::DateTime<chrono::Utc>,
    is_online: bool,
    is_creator: bool,
  ) -> Self {
    Self {
      user_id,
      chat_id,
      username,
      display_name,
      role,
      joined_at,
      is_online,
      is_creator,
    }
  }
}

/// 聊天成员操作响应
#[derive(Debug, Serialize, ToSchema)]
pub struct ChatMemberOperationResponse {
  #[schema(example = true)]
  pub success: bool,

  #[schema(example = "Member added successfully")]
  pub message: String,

  #[schema(example = 1)]
  pub affected_count: i32,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ChatMemberOperationResponse {
  pub fn success(message: String, affected_count: i32) -> Self {
    Self {
      success: true,
      message,
      affected_count,
      timestamp: chrono::Utc::now(),
    }
  }
}

/// 转移所有权响应
#[derive(Debug, Serialize, ToSchema)]
pub struct TransferOwnershipResponse {
  #[schema(example = true)]
  pub success: bool,

  #[schema(example = "Chat ownership transferred successfully")]
  pub message: String,

  #[schema(example = 123)]
  pub old_owner_id: i64,

  #[schema(example = 456)]
  pub new_owner_id: i64,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 聊天设置响应
#[derive(Debug, Serialize, ToSchema)]
pub struct ChatSettingsDto {
  #[schema(example = 123)]
  pub chat_id: i64,

  #[schema(example = false)]
  pub mute_notifications: bool,

  #[schema(example = true)]
  pub pin_chat: bool,

  #[schema(example = true)]
  pub show_previews: bool,

  #[schema(example = "all")]
  pub notification_level: String,
}
