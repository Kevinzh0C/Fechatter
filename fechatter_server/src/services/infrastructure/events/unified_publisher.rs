//! # DEPRECATED: Use enhanced_publisher instead
//!
//! **🚨 This module is DEPRECATED and will be removed in future versions**
//!
//! **Migration Guide:**
//! - Use `crate::services::infrastructure::event::enhanced_publisher` instead
//! - New enhanced_publisher provides notify_server compatibility
//! - Complete message content delivery for SSE broadcasting
//! - Better integration with real-time notification system
//!
//! **Replacement:**
//! ```rust
//! // OLD (deprecated):
//! use crate::services::infrastructure::events::UnifiedEventPublisher;
//!
//! // NEW (recommended):
//! use crate::services::infrastructure::event::EnhancedEventPublisher;
//! ```

use crate::error::AppError;
use async_nats::Client as NatsClient;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use fechatter_core::{ChatId, MessageId, UserId};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// =============================================================================
// EVENT DEFINITIONS - Protobuf-like structures for immediate use
// =============================================================================

/// Event envelope with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deprecated(
    since = "2.0.0",
    note = "Use enhanced_publisher from event module instead. This provides notify_server compatibility and complete message content delivery."
)]
pub struct EventEnvelope {
    pub metadata: EventMetadata,
    pub payload: EventPayload,
}

/// Event metadata for all events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deprecated(since = "2.0.0", note = "Use enhanced_publisher instead")]
pub struct EventMetadata {
    pub event_id: String,
    pub version: String,
    pub occurred_at: DateTime<Utc>,
    pub server_ts: DateTime<Utc>,
    pub source_service: String,
    pub source_instance: String,
    pub target_users: Vec<i64>,
    pub target_chats: Vec<i64>,
    pub workspace_id: i64,
    pub signature: Option<String>,
    pub retry_count: i32,
    pub trace_context: HashMap<String, String>,
}

/// Event payload types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deprecated(since = "2.0.0", note = "Use enhanced_publisher instead")]
#[serde(tag = "type", content = "data")]
pub enum EventPayload {
    MessageCreated(MessageCreatedEvent),
    MessageEdited(MessageEditedEvent),
    MessageDeleted(MessageDeletedEvent),
    ChatMemberJoined(ChatMemberJoinedEvent),
    ChatMemberLeft(ChatMemberLeftEvent),
    MessageReadReceipt(MessageReadReceiptEvent),
    DuplicateMessage(DuplicateMessageEvent),
}

/// Message created event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deprecated(since = "2.0.0", note = "Use enhanced_publisher instead")]
pub struct MessageCreatedEvent {
    pub message_id: i64,
    pub chat_id: i64,
    pub sender_id: i64,
    pub content: String,
    pub files: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub idempotency_key: String,
    pub sender_name: String,
    pub chat_name: String,
    pub message_type: String,
    pub mentioned_users: Vec<i64>,
    pub sequence_number: Option<i64>,
    pub has_attachments: bool,
    pub content_length: i32,
}

/// Message edited event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deprecated(since = "2.0.0", note = "Use enhanced_publisher instead")]
pub struct MessageEditedEvent {
    pub message_id: i64,
    pub chat_id: i64,
    pub editor_id: i64,
    pub old_content: String,
    pub new_content: String,
    pub edited_at: DateTime<Utc>,
    pub edit_reason: Option<String>,
}

/// Message deleted event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deprecated(since = "2.0.0", note = "Use enhanced_publisher instead")]
pub struct MessageDeletedEvent {
    pub message_id: i64,
    pub chat_id: i64,
    pub deleted_by: i64,
    pub deleted_at: DateTime<Utc>,
    pub deletion_reason: Option<String>,
    pub soft_delete: bool,
}

/// Chat member joined event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deprecated(since = "2.0.0", note = "Use enhanced_publisher instead")]
pub struct ChatMemberJoinedEvent {
    pub chat_id: i64,
    pub user_id: i64,
    pub added_by: i64,
    pub joined_at: DateTime<Utc>,
    pub join_method: String,
}

/// Chat member left event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deprecated(since = "2.0.0", note = "Use enhanced_publisher instead")]
pub struct ChatMemberLeftEvent {
    pub chat_id: i64,
    pub user_id: i64,
    pub removed_by: i64,
    pub left_at: DateTime<Utc>,
    pub leave_reason: String,
}

/// Message read receipt event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deprecated(since = "2.0.0", note = "Use enhanced_publisher instead")]
pub struct MessageReadReceiptEvent {
    pub chat_id: i64,
    pub user_id: i64,
    pub message_ids: Vec<i64>,
    pub read_at: DateTime<Utc>,
    pub last_read_message_id: i64,
}

/// Duplicate message event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deprecated(since = "2.0.0", note = "Use enhanced_publisher instead")]
pub struct DuplicateMessageEvent {
    pub idempotency_key: String,
    pub chat_id: i64,
    pub sender_id: i64,
    pub attempted_at: DateTime<Utc>,
    pub original_message_id: Option<i64>,
}
