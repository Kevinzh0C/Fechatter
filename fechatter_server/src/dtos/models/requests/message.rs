use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// 发送消息请求
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct SendMessageRequest {
    #[validate(length(
        min = 1,
        max = 4000,
        message = "Message content must be between 1 and 4000 characters"
    ))]
    #[schema(example = "Hello, this is a test message!")]
    pub content: String,

    #[schema(example = "['/files/1/abc/def/123.jpg', '/files/1/xyz/uvw/456.pdf']")]
    pub files: Option<Vec<String>>,

    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub idempotency_key: Option<Uuid>,

    #[schema(example = 789)]
    pub reply_to: Option<i64>, // 回复的消息ID

    #[schema(example = "[2, 3]")]
    pub mentions: Option<Vec<i64>>, // 提及的用户ID列表
}

/// 编辑消息请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct EditMessageRequest {
    #[validate(length(
        min = 1,
        max = 4000,
        message = "Message content must be between 1 and 4000 characters"
    ))]
    #[schema(example = "This is the edited message content")]
    pub content: String,
}

/// 删除消息请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeleteMessageRequest {
    #[schema(example = "Inappropriate content")]
    pub reason: Option<String>,

    #[schema(example = false)]
    pub delete_for_everyone: Option<bool>, // 是否为所有人删除
}

/// 获取消息列表请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ListMessagesRequest {
    #[validate(range(min = 1, max = 100, message = "Limit must be between 1 and 100"))]
    #[schema(example = 20)]
    pub limit: Option<i32>,

    #[schema(example = 1000)]
    pub before_id: Option<i64>, // 获取此ID之前的消息

    #[schema(example = 500)]
    pub after_id: Option<i64>, // 获取此ID之后的消息

    #[schema(example = "2024-01-01T00:00:00Z")]
    pub before_timestamp: Option<chrono::DateTime<chrono::Utc>>,

    #[schema(example = "2024-01-01T00:00:00Z")]
    pub after_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

/// 搜索消息请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct SearchMessagesRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Query must be between 1 and 100 characters"
    ))]
    #[schema(example = "project update")]
    pub query: String,

    #[schema(example = 123)]
    pub chat_id: Option<i64>, // 限制在特定聊天中搜索

    #[schema(example = 2)]
    pub sender_id: Option<i64>, // 限制特定发送者的消息

    #[schema(example = "2024-01-01T00:00:00Z")]
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,

    #[schema(example = "2024-12-31T23:59:59Z")]
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,

    #[schema(example = true)]
    pub has_files: Option<bool>, // 是否包含文件

    #[validate(range(min = 1, max = 100, message = "Limit must be between 1 and 100"))]
    #[schema(example = 20)]
    pub limit: Option<i32>,

    #[validate(range(min = 0, message = "Offset must be non-negative"))]
    #[schema(example = 0)]
    pub offset: Option<i32>,
}

/// 标记消息为已读请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MarkMessagesReadRequest {
    #[schema(example = "[1, 2, 3, 4, 5]")]
    pub message_ids: Vec<i64>,
}

/// 消息反应请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct MessageReactionRequest {
    #[validate(length(
        min = 1,
        max = 10,
        message = "Emoji must be between 1 and 10 characters"
    ))]
    #[schema(example = "👍")]
    pub emoji: String,

    #[schema(example = true)]
    pub add: bool, // true为添加反应，false为移除反应
}

/// 转发消息请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ForwardMessageRequest {
    #[validate(length(min = 1, message = "At least one target chat is required"))]
    #[schema(example = "[456, 789]")]
    pub target_chat_ids: Vec<i64>,

    #[schema(example = "Forwarding this important message")]
    pub comment: Option<String>,
}

/// 固定消息请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PinMessageRequest {
    #[schema(example = true)]
    pub pin: bool, // true为固定，false为取消固定

    #[schema(example = "Important announcement")]
    pub reason: Option<String>,
}

/// 批量删除消息请求
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct BatchDeleteMessagesRequest {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Must specify between 1 and 100 message IDs"
    ))]
    #[schema(example = "[1, 2, 3, 4, 5]")]
    pub message_ids: Vec<i64>,

    #[schema(example = "Cleanup old messages")]
    pub reason: Option<String>,

    #[schema(example = false)]
    pub delete_for_everyone: Option<bool>,
}

/// 消息统计请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MessageStatsRequest {
    #[schema(example = "2024-01-01T00:00:00Z")]
    pub start_date: chrono::DateTime<chrono::Utc>,

    #[schema(example = "2024-12-31T23:59:59Z")]
    pub end_date: chrono::DateTime<chrono::Utc>,

    #[schema(example = "day")]
    pub group_by: Option<String>, // "hour", "day", "week", "month"
}

/// 导出消息请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ExportMessagesRequest {
    #[schema(example = "2024-01-01T00:00:00Z")]
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,

    #[schema(example = "2024-12-31T23:59:59Z")]
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,

    #[schema(example = "json")]
    pub format: Option<String>, // "json", "csv", "txt"

    #[schema(example = true)]
    pub include_files: Option<bool>,
}
