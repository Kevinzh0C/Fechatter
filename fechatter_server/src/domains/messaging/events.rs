use fechatter_core::{ChatId, MessageId, UserId};
use serde::{Deserialize, Serialize};

/// Messaging domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagingDomainEvent {
  /// Message sent
  MessageSent {
    message_id: MessageId,
    chat_id: ChatId,
    sender_id: UserId,
    content_preview: String,
    mentioned_users: Vec<UserId>,
  },
  /// Message edited
  MessageEdited {
    message_id: MessageId,
    chat_id: ChatId,
    editor_id: UserId,
    old_content: String,
    new_content: String,
  },
  /// Message deleted
  MessageDeleted {
    message_id: MessageId,
    chat_id: ChatId,
    deleted_by: UserId,
  },
  /// Message reaction added
  ReactionAdded {
    message_id: MessageId,
    user_id: UserId,
    reaction: String,
  },
  /// Message reaction removed
  ReactionRemoved {
    message_id: MessageId,
    user_id: UserId,
    reaction: String,
  },
}
