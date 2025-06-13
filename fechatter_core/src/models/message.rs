use crate::error::CoreError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::future::Future;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::{ChatId, Message, MessageId, UserId, WorkspaceId};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMessage {
  pub content: String,
  #[serde(default)]
  pub files: Vec<String>,
  #[serde(default = "Uuid::now_v7")] // Default to UUID v7 (time-based UUID)
  #[schema(value_type = String, format = "uuid", example = "01834abd-8c37-7d82-9206-54b2f6b4f7c4")]
  pub idempotency_key: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListMessages {
  #[serde(default)]
  pub last_id: Option<i64>,
  #[serde(default)]
  pub limit: i64,
}

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
