use chrono::{DateTime, Utc};
use fechatter_core::{Chat, Message};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMemberEventData {
  pub chat_id: i64,
  pub user_id: i64,
  pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageReadEvent {
  pub message_id: i64,
  pub chat_id: i64,
  pub reader_id: i64,
  pub read_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageUnreadEvent {
  pub message_id: i64,
  pub chat_id: i64,
  pub user_id: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TypingEvent {
  pub chat_id: i64,
  pub user_id: i64,
  pub user_name: Option<String>,
  pub is_typing: bool, // true for started, false for stopped
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserPresenceEvent {
  pub user_id: i64,
  pub status: String, // "online", "offline", "away"
  pub last_seen: Option<String>,
}

// Duplicate message event payload
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DuplicateMessagePayload {
  pub idempotency_key: Uuid,
  pub chat_id: i64,
  pub sender_id: i64,
}

/// Unified notification event type
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum NotifyEvent {
  NewChat(Chat),
  UserJoinedChat(ChatMemberEventData),
  UserLeftChat(ChatMemberEventData),
  NewMessage(Message),
  DuplicateMessageAttempted(DuplicateMessagePayload),

  // Realtime stream events (low-latency delivery)
  MessageRead(MessageReadEvent),
  MessageUnread(MessageUnreadEvent),
  TypingStatus(TypingEvent),
  UserPresence(UserPresenceEvent),

  // Generic event extensibility
  Generic(serde_json::Value),
}

/// Broadcast event, with explicit target users
#[derive(Debug, Serialize, Deserialize)]
pub struct BroadcastEvent {
  /// List of user IDs to receive this event
  pub target_users: Vec<i64>,
  /// Actual event data
  pub event_data: serde_json::Value,
}

/// Realtime stream event structure (from fechatter_server dual-stream architecture)
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum RealtimeStreamEvent {
  MessageReceived {
    message: StreamMessage,
    chat_id: i64,
    recipients: Vec<i64>,
  },
  MessageRead {
    message_id: i64,
    chat_id: i64,
    reader_id: i64,
    read_at: String,
  },
  MessageUnread {
    message_id: i64,
    chat_id: i64,
    user_id: i64,
  },
  TypingStarted {
    chat_id: i64,
    user_id: i64,
    user_name: String,
  },
  TypingStopped {
    chat_id: i64,
    user_id: i64,
  },
  UserPresence {
    user_id: i64,
    status: String,
    last_seen: Option<String>,
  },
  MessageDeleted {
    message_id: i64,
    chat_id: i64,
    deleted_by: i64,
    deleted_at: String,
  },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamMessage {
  pub id: String,
  pub chat_id: i64,
  pub sender_id: i64,
  pub content: String,
  pub files: Vec<String>,
  pub timestamp: i64,
}
