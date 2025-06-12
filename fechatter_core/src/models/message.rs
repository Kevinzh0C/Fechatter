use crate::error::CoreError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::future::Future;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::{ChatId, Message, MessageId, UserId, WorkspaceId};

// ── Domain Models ──────────────────────────────────────────────────────────

fn default_uuid() -> Option<Uuid> {
  Some(Uuid::new_v4())
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMessage {
  pub content: String,
  #[serde(default)]
  pub files: Option<Vec<String>>,
  #[serde(default = "default_uuid")] // Default to UUID v7 (time-based UUID)
  #[schema(value_type = String, format = "uuid", example = "01834abd-8c37-7d82-9206-54b2f6b4f7c4")]
  pub idempotency_key: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListMessages {
  #[serde(default)]
  pub last_id: Option<i64>,
  #[serde(default)]
  pub limit: i64,
}

// ── Application Models ─────────────────────────────────────────────────────

/// Sender information for message view
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessageSender {
  pub id: i64,
  pub fullname: String,
  pub username: Option<String>,
  pub email: Option<String>,
}

/// Message view model for application layer
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessageView {
  pub id: i64,
  pub chat_id: i64,
  pub sender_id: i64,
  pub sender: Option<MessageSender>, // Added sender information
  pub content: String,
  pub files: Option<Vec<String>>,
  pub created_at: DateTime<Utc>,
  pub reply_to: Option<i64>,
  pub mentions: Option<Vec<i64>>,
  pub is_edited: bool,
  pub sequence_number: Option<i64>,
  pub idempotency_key: Option<String>,
}

impl From<Message> for MessageView {
  fn from(message: Message) -> Self {
    Self {
      id: message.id.into(),
      chat_id: message.chat_id.into(),
      sender_id: message.sender_id.into(),
      sender: None, // Will be populated by query JOIN
      content: message.content,
      files: message.files,
      created_at: message.created_at,
      reply_to: None,        // TODO: Add to core Message if needed
      mentions: None,        // TODO: Add to core Message if needed
      is_edited: false,      // TODO: Add to core Message if needed
      sequence_number: None, // TODO: Add to core Message if needed
      idempotency_key: message.idempotency_key.map(|uuid| uuid.to_string()),
    }
  }
}

/// Input for sending a message
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SendMessageInput {
  pub content: String,
  pub files: Option<Vec<String>>,
  pub idempotency_key: Option<Uuid>,
  pub reply_to: Option<MessageId>,
  pub mentions: Option<Vec<UserId>>,
}

impl From<SendMessageInput> for CreateMessage {
  fn from(input: SendMessageInput) -> Self {
    Self {
      content: input.content,
      files: input.files,
      idempotency_key: input.idempotency_key,
    }
  }
}

/// Input for querying messages
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetMessagesInput {
  pub limit: Option<i64>,
  pub offset: Option<i64>,
  pub before: Option<MessageId>,
  pub after: Option<MessageId>,
  pub last_id: Option<i64>,
}

impl From<GetMessagesInput> for ListMessages {
  fn from(input: GetMessagesInput) -> Self {
    Self {
      last_id: input.last_id.or_else(|| input.before.map(|id| id.into())),
      limit: input.limit.unwrap_or(20),
    }
  }
}

// ── Event Models ───────────────────────────────────────────────────────────

/// Message Event Types for the event system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageEvent {
  MessageSent {
    message_id: i64,
    chat_id: i64,
    sender_id: i64,
    content_preview: String,
    mentioned_users: Vec<i64>,
  },
  MessageEdited {
    message_id: i64,
    chat_id: i64,
    editor_id: i64,
  },
  MessageDeleted {
    message_id: i64,
    chat_id: i64,
    deleted_by: i64,
  },
}

/// Stream message for real-time messaging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMessage {
  pub id: String,
  pub chat_id: i64,
  pub sender_id: i64,
  pub content: String,
  pub files: Vec<String>,
  pub timestamp: i64,
}

// ── Search Models ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct SearchMessages {
  /// 搜索关键词
  #[validate(length(min = 1, max = 200))]
  pub query: String,

  /// 工作空间ID（从用户token自动填充）
  #[serde(skip)]
  pub workspace_id: WorkspaceId,

  /// 聊天ID（从URL路径自动填充）
  #[serde(skip)]
  pub chat_id: Option<ChatId>,

  /// 分页偏移
  #[serde(default)]
  #[validate(range(min = 0, max = 10000))]
  pub offset: i64,

  /// 每页结果数
  #[serde(default = "default_limit")]
  #[validate(range(min = 1, max = 100))]
  pub limit: i64,
}

fn default_limit() -> i64 {
  20
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchableMessage {
  pub id: MessageId,
  pub chat_id: ChatId,
  pub sender_id: UserId,
  pub sender_name: String,
  pub content: String,
  pub files: Option<Vec<String>>,
  pub created_at: DateTime<Utc>,
  /// 相关性得分（可选）
  pub relevance_score: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SearchResult {
  /// 搜索到的消息
  pub messages: Vec<SearchableMessage>,
  /// 总命中数
  pub total_hits: usize,
  /// 是否有更多结果
  pub has_more: bool,
  /// 查询耗时（毫秒）
  pub query_time_ms: u64,
}

// ── NATS Event Models ──────────────────────────────────────────────────────

/// Event published to NATS when a message is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageCreatedEvent {
  pub message_id: i64,
  pub chat_id: i64,
  pub sender_id: i64,
  pub content: String,
  pub files: Vec<String>,
  pub created_at: String, // ISO timestamp
  pub chat_members: Vec<i64>,
  pub mentions: Vec<i64>,
  pub sequence_number: Option<i64>,
}

// ── Repository Trait ───────────────────────────────────────────────────────

pub trait MessageRepository: Send + Sync {
  fn create_message(
    &self,
    input: &CreateMessage,
    chat_id: ChatId,
    user_id: UserId,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<Message, CoreError>> + Send>>;

  fn list_messages(
    &self,
    input: &ListMessages,
    chat_id: ChatId,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<Message>, CoreError>> + Send>>;
}
