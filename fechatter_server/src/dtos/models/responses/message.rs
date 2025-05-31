use crate::dtos::core::{ConversionError, ResponseDto};
use fechatter_core::Message;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// 单个消息响应
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageResponse {
  #[schema(example = 123)]
  pub id: i64,

  #[schema(example = 456)]
  pub chat_id: i64,

  #[schema(example = 789)]
  pub sender_id: i64,

  #[schema(example = "Hello, this is a test message!")]
  pub content: String,

  #[schema(example = "['/files/1/abc/def/123.jpg']")]
  pub files: Option<Vec<String>>,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub created_at: chrono::DateTime<chrono::Utc>,

  #[schema(example = 100)]
  pub reply_to: Option<i64>,

  #[schema(example = "[2, 3]")]
  pub mentions: Option<Vec<i64>>,

  #[schema(example = false)]
  pub is_edited: bool,

  #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
  pub idempotency_key: Option<String>,
}

impl ResponseDto for MessageResponse {
  type DomainModel = Message;

  fn from_domain(domain: &Self::DomainModel) -> Result<Self, ConversionError> {
    Ok(Self {
      id: domain.id.into(),
      chat_id: domain.chat_id.into(),
      sender_id: domain.sender_id.into(),
      content: domain.content.clone(),
      files: domain.files.clone(),
      created_at: domain.created_at,
      reply_to: domain.reply_to.map(|id| id.into()),
      mentions: domain.mentions.clone(),
      is_edited: false, // TODO: 从domain获取编辑状态
      idempotency_key: domain.idempotency_key.clone(),
    })
  }

  fn from_domain_collection(domains: &[Self::DomainModel]) -> Result<Vec<Self>, ConversionError> {
    domains.iter().map(Self::from_domain).collect()
  }
}

/// 消息创建响应
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageCreateResponse {
  #[schema(example = true)]
  pub success: bool,

  #[schema(example = "Message sent successfully")]
  pub message: String,

  pub data: MessageResponse,
}

/// 消息列表响应（内部使用分页包装器）
pub type MessageListResponse = Vec<MessageResponse>;

/// 消息操作响应
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageOperationResponse {
  #[schema(example = true)]
  pub success: bool,

  #[schema(example = "Message updated successfully")]
  pub message: String,

  #[schema(example = 1)]
  pub affected_rows: Option<i64>,

  #[schema(example = "2024-01-01T12:00:00Z")]
  pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl MessageOperationResponse {
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
