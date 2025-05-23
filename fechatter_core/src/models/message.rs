use crate::error::CoreError;
use serde::{Deserialize, Serialize};
use std::future::Future;
use utoipa::ToSchema;
use uuid::Uuid;

use super::Message;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateMessage {
  pub content: String,
  #[serde(default)]
  pub files: Vec<String>,
  #[serde(default = "Uuid::now_v7")] // 默认使用UUID v7（时间序列UUID）
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

pub trait MessageRepository: Send + Sync {
  fn create_message(
    &self,
    input: &CreateMessage,
    chat_id: i64,
    user_id: i64,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<Message, CoreError>> + Send>>;

  fn list_messages(
    &self,
    input: &ListMessages,
    chat_id: i64,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<Message>, CoreError>> + Send>>;
}

// Public validate function that doesn't require database access
