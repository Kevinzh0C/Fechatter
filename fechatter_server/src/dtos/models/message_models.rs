//! # Message Data Transfer Objects
//!
//! **职责**: 消息相关的数据传输对象和视图模型
//! **原则**: 清晰的数据结构，分离领域模型和API模型

use chrono::{DateTime, Utc};
use fechatter_core::models::Message;
use serde::{Deserialize, Serialize};

/// 消息视图模型 - 应用层消息表示
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageView {
    pub id: i64,
    pub chat_id: i64,
    pub sender_id: i64,
    pub content: String,
    pub files: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub reply_to: Option<i64>,
    pub mentions: Option<Vec<i64>>,
    pub is_edited: bool,
    pub idempotency_key: Option<String>,
}

impl From<Message> for MessageView {
    fn from(message: Message) -> Self {
        Self {
            id: message.id.into(),
            chat_id: message.chat_id.into(),
            sender_id: message.sender_id.into(),
            content: message.content,
            files: message.files,
            created_at: message.created_at,
            reply_to: None,   // TODO: 如果需要支持回复功能，需要在core模型中添加
            mentions: None,   // TODO: 如果需要支持@功能，需要在core模型中添加
            is_edited: false, // TODO: 如果需要支持编辑标记，需要在core模型中添加
            idempotency_key: message.idempotency_key.map(|uuid| uuid.to_string()),
        }
    }
}

/// 消息发送输入参数
#[derive(Debug, Clone)]
pub struct SendMessageInput {
    pub sender_id: fechatter_core::UserId,
    pub chat_id: fechatter_core::ChatId,
    pub content: String,
    pub files: Option<Vec<String>>,
    pub idempotency_key: Option<uuid::Uuid>,
    pub reply_to: Option<fechatter_core::MessageId>,
    pub mentions: Option<Vec<fechatter_core::UserId>>,
}

/// 消息查询参数
#[derive(Debug, Clone)]
pub struct GetMessagesInput {
    pub user_id: fechatter_core::UserId,
    pub chat_id: fechatter_core::ChatId,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub before: Option<fechatter_core::MessageId>,
    pub after: Option<fechatter_core::MessageId>,
}
