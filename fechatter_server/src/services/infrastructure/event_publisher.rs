//! Event publisher for NATS-based messaging

use crate::error::AppError;
use async_nats::Client as NatsClient;
use chrono::{DateTime, Utc};
use fechatter_core::Message;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

/// NATS subjects configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsSubjectsConfig {
  pub message_created: String,
  pub message_updated: String,
  pub message_deleted: String,
  pub chat_created: String,
  pub chat_updated: String,
  pub user_joined_chat: String,
  pub user_left_chat: String,
  pub duplicate_message_attempted: String,
}

impl Default for NatsSubjectsConfig {
  fn default() -> Self {
    Self {
      message_created: "fechatter.messages.created".to_string(),
      message_updated: "fechatter.messages.updated".to_string(),
      message_deleted: "fechatter.messages.deleted".to_string(),
      chat_created: "fechatter.chats.created".to_string(),
      chat_updated: "fechatter.chats.updated".to_string(),
      user_joined_chat: "fechatter.chats.member.joined".to_string(),
      user_left_chat: "fechatter.chats.member.left".to_string(),
      duplicate_message_attempted: "fechatter.messages.duplicate".to_string(),
    }
  }
}

/// Chat information for indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatInfo {
  pub chat_id: i64,
  pub chat_name: String,
  pub workspace_id: i64,
}

/// Message index event for search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageIndexEvent {
  pub message: Message,
  pub chat_info: ChatInfo,
}

/// Search index event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndexEvent {
  pub message_id: i64,
  pub chat_id: i64,
  pub chat_name: String,
  pub content: String,
  pub sender_id: i64,
  pub created_at: DateTime<Utc>,
  pub files: Vec<String>,
  pub new_member_id: Option<i64>,
  pub dedup_key: String,
  pub event_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageCreatedEvent {
  pub message: Message,
  pub chat_members: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMemberEvent {
  pub chat_id: i64,
  pub user_id: i64,
  pub joined_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DuplicateMessageEvent {
  pub idempotency_key: uuid::Uuid,
  pub chat_id: i64,
  pub sender_id: i64,
}

#[derive(Clone)]
pub struct EventPublisher {
  nats_client: NatsClient,
  subjects: NatsSubjectsConfig,
}

impl EventPublisher {
  pub fn new(nats_client: NatsClient, subjects: NatsSubjectsConfig) -> Self {
    Self {
      nats_client,
      subjects,
    }
  }

  /// Create with default subjects
  pub fn with_default_subjects(nats_client: NatsClient) -> Self {
    Self::new(nats_client, NatsSubjectsConfig::default())
  }

  /// Get the NATS client for health checks
  pub fn nats_client(&self) -> &NatsClient {
    &self.nats_client
  }

  /// Publish message created event
  pub async fn publish_message_created(
    &self,
    message: &Message,
    chat_members: Vec<i64>,
  ) -> Result<(), AppError> {
    let event = MessageCreatedEvent {
      message: message.clone(),
      chat_members,
    };

    let payload = match serde_json::to_vec(&event) {
      Ok(payload) => payload,
      Err(e) => {
        error!("Failed to serialize message created event: {}", e);
        return Err(AppError::EventPublishingError(format!(
          "Failed to serialize message created event: {}",
          e
        )));
      }
    };

    info!(
      "Publishing message created event: message_id={}, chat_members={:?}",
      message.id, event.chat_members
    );

    if let Err(e) = self
      .nats_client
      .publish(self.subjects.message_created.clone(), payload.into())
      .await
    {
      error!("Failed to publish message created event: {}", e);
      return Err(AppError::NatsError(format!(
        "Failed to publish message created event: {}",
        e
      )));
    }

    info!(
      "Successfully published message created event: message_id={}",
      message.id
    );
    Ok(())
  }

  /// Publish user joined chat event
  pub async fn publish_user_joined_chat(
    &self,
    chat_id: i64,
    user_id: i64,
    joined_at: DateTime<Utc>,
  ) -> Result<(), AppError> {
    let event = ChatMemberEvent {
      chat_id,
      user_id,
      joined_at,
    };

    let payload = match serde_json::to_vec(&event) {
      Ok(payload) => payload,
      Err(e) => {
        error!("Failed to serialize user joined chat event: {}", e);
        return Err(AppError::EventPublishingError(format!(
          "Failed to serialize user joined chat event: {}",
          e
        )));
      }
    };

    info!(
      "Publishing user joined chat event: user_id={}, chat_id={}",
      user_id, chat_id
    );

    if let Err(e) = self
      .nats_client
      .publish(self.subjects.user_joined_chat.clone(), payload.into())
      .await
    {
      error!("Failed to publish user joined chat event: {}", e);
      return Err(AppError::NatsError(format!(
        "Failed to publish user joined chat event: {}",
        e
      )));
    }

    info!(
      "Successfully published user joined chat event: user_id={}, chat_id={}",
      user_id, chat_id
    );
    Ok(())
  }

  /// Publish user left chat event
  pub async fn publish_user_left_chat(
    &self,
    chat_id: i64,
    user_id: i64,
    left_at: chrono::DateTime<chrono::Utc>,
  ) -> Result<(), AppError> {
    let event = ChatMemberEvent {
      chat_id,
      user_id,
      joined_at: left_at, // Reuse structure, keep field name consistent
    };

    let payload = match serde_json::to_vec(&event) {
      Ok(payload) => payload,
      Err(e) => {
        error!("Failed to serialize user left chat event: {}", e);
        return Err(AppError::EventPublishingError(format!(
          "Failed to serialize user left chat event: {}",
          e
        )));
      }
    };

    info!(
      "Publishing user left chat event: user_id={}, chat_id={}",
      user_id, chat_id
    );

    if let Err(e) = self
      .nats_client
      .publish(self.subjects.user_left_chat.clone(), payload.into())
      .await
    {
      error!("Failed to publish user left chat event: {}", e);
      return Err(AppError::NatsError(format!(
        "Failed to publish user left chat event: {}",
        e
      )));
    }

    info!(
      "Successfully published user left chat event: user_id={}, chat_id={}",
      user_id, chat_id
    );
    Ok(())
  }

  /// Publish duplicate message attempt event
  pub async fn publish_duplicate_message_attempted(
    &self,
    idempotency_key: uuid::Uuid,
    chat_id: i64,
    sender_id: i64,
  ) -> Result<(), AppError> {
    let event = DuplicateMessageEvent {
      idempotency_key,
      chat_id,
      sender_id,
    };

    let payload = match serde_json::to_vec(&event) {
      Ok(payload) => payload,
      Err(e) => {
        error!("Failed to serialize duplicate message event: {}", e);
        return Err(AppError::EventPublishingError(format!(
          "Failed to serialize duplicate message event: {}",
          e
        )));
      }
    };

    info!(
      "Publishing duplicate message attempted event: idempotency_key={}, sender_id={}, chat_id={}",
      idempotency_key, sender_id, chat_id
    );

    if let Err(e) = self
      .nats_client
      .publish(
        self.subjects.duplicate_message_attempted.clone(),
        payload.into(),
      )
      .await
    {
      error!("Failed to publish duplicate message attempted event: {}", e);
      return Err(AppError::NatsError(format!(
        "Failed to publish duplicate message attempted event: {}",
        e
      )));
    }

    info!(
      "Successfully published duplicate message attempted event: idempotency_key={}",
      idempotency_key
    );
    Ok(())
  }

  /// Publish search index event
  pub async fn publish_search_index_event(
    &self,
    message: &Message,
    chat_info: &ChatInfo,
  ) -> Result<(), AppError> {
    let event = MessageIndexEvent {
      message: message.clone(),
      chat_info: chat_info.clone(),
    };

    let payload = match serde_json::to_vec(&event) {
      Ok(payload) => payload,
      Err(e) => {
        error!("Failed to serialize search index event: {}", e);
        return Err(AppError::EventPublishingError(format!(
          "Failed to serialize search index event: {}",
          e
        )));
      }
    };

    info!(
      "Publishing search index event: message_id={}, chat_name={}",
      message.id, chat_info.chat_name
    );

    let subject = "fechatter.search.index";
    if let Err(e) = self
      .nats_client
      .publish(subject.clone(), payload.into())
      .await
    {
      error!("Failed to publish search index event: {}", e);
      return Err(AppError::NatsError(format!(
        "Failed to publish search index event: {}",
        e
      )));
    }

    info!(
      "Successfully published search index event: message_id={}, subject={}",
      message.id, subject
    );
    Ok(())
  }

  /// Publish unified search index event
  pub async fn publish_unified_search_index_event(
    &self,
    message: &Message,
    chat_name: &str,
    event_type: &str,
    new_member_id: Option<i64>,
  ) -> Result<(), AppError> {
    let dedup_key = match new_member_id {
      Some(member_id) => format!("{}_{}_{}", message.id, member_id, message.created_at),
      None => format!("{}_{}", message.id, message.created_at),
    };

    let event = SearchIndexEvent {
      message_id: message.id.into(),
      chat_id: message.chat_id.into(),
      chat_name: chat_name.to_string(),
      content: message.content.clone(),
      sender_id: message.sender_id.into(),
      created_at: message.created_at,
      files: message.files.clone().unwrap_or_default(),
      new_member_id,
      dedup_key: dedup_key.clone(),
      event_type: event_type.to_string(),
    };

    let payload = match serde_json::to_vec(&event) {
      Ok(payload) => payload,
      Err(e) => {
        error!("Failed to serialize unified search index event: {}", e);
        return Err(AppError::EventPublishingError(format!(
          "Failed to serialize unified search index event: {}",
          e
        )));
      }
    };

    info!(
      "Publishing unified search index event: message_id={}, chat_name={}, event_type={}, dedup_key={}",
      message.id, chat_name, event_type, dedup_key
    );

    let subject = "fechatter.search.index";
    if let Err(e) = self
      .nats_client
      .publish(subject.clone(), payload.into())
      .await
    {
      error!("Failed to publish unified search index event: {}", e);
      return Err(AppError::NatsError(format!(
        "Failed to publish unified search index event: {}",
        e
      )));
    }

    info!(
      "Successfully published unified search index event: message_id={}, subject={}, event_type={}",
      message.id, subject, event_type
    );
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::Utc;
  use fechatter_core::Message;
  use uuid::Uuid;

  // Create test NATS subject configuration
  fn create_test_subjects() -> NatsSubjectsConfig {
    NatsSubjectsConfig {
      message_created: "test.messages.created".to_string(),
      message_updated: "test.messages.updated".to_string(),
      message_deleted: "test.messages.deleted".to_string(),
      chat_created: "test.chats.created".to_string(),
      chat_updated: "test.chats.updated".to_string(),
      user_joined_chat: "test.chats.member.joined".to_string(),
      user_left_chat: "test.chats.member.left".to_string(),
      duplicate_message_attempted: "test.messages.duplicate".to_string(),
    }
  }

  // Create test message
  fn create_test_message() -> Message {
    Message {
      id: fechatter_core::MessageId(1),
      chat_id: fechatter_core::ChatId(100),
      sender_id: fechatter_core::UserId(10),
      content: "Test message content".to_string(),
      files: Some(vec!["file1.txt".to_string(), "file2.jpg".to_string()]),
      created_at: Utc::now(),
      idempotency_key: Some(Uuid::new_v4()),
    }
  }

  #[test]
  fn test_event_publisher_config() {
    // This test verifies that EventPublisher can be created correctly
    // Since we need a real NATS client, we test the config itself
    let subjects = create_test_subjects();

    // We can't easily mock async_nats::Client, so test config parsing
    assert_eq!(subjects.message_created, "test.messages.created");
    assert_eq!(subjects.user_joined_chat, "test.chats.member.joined");
    assert_eq!(
      subjects.duplicate_message_attempted,
      "test.messages.duplicate"
    );
  }

  #[test]
  fn test_message_created_event_serialization() {
    // Test event serialization
    let message = create_test_message();
    let chat_members = vec![1, 2, 3];

    let event = MessageCreatedEvent {
      message: message.clone(),
      chat_members: chat_members.clone(),
    };

    // Test serialization
    let serialized = serde_json::to_vec(&event).expect("Failed to serialize event");
    assert!(!serialized.is_empty());

    // Test deserialization
    let deserialized: MessageCreatedEvent =
      serde_json::from_slice(&serialized).expect("Failed to deserialize event");

    assert_eq!(deserialized.message.id, message.id);
    assert_eq!(deserialized.message.content, message.content);
    assert_eq!(deserialized.chat_members, chat_members);
  }

  #[test]
  fn test_chat_member_event_serialization() {
    let event = ChatMemberEvent {
      chat_id: 100,
      user_id: 10,
      joined_at: Utc::now(),
    };

    // Test serialization and deserialization
    let serialized = serde_json::to_vec(&event).expect("Failed to serialize event");
    let deserialized: ChatMemberEvent =
      serde_json::from_slice(&serialized).expect("Failed to deserialize event");

    assert_eq!(deserialized.chat_id, event.chat_id);
    assert_eq!(deserialized.user_id, event.user_id);
    assert_eq!(deserialized.joined_at, event.joined_at);
  }

  #[test]
  fn test_duplicate_message_event_serialization() {
    let event = DuplicateMessageEvent {
      idempotency_key: Uuid::new_v4(),
      chat_id: 100,
      sender_id: fechatter_core::UserId(10).into(),
    };

    // Test serialization and deserialization
    let serialized = serde_json::to_vec(&event).expect("Failed to serialize event");
    let deserialized: DuplicateMessageEvent =
      serde_json::from_slice(&serialized).expect("Failed to deserialize event");

    assert_eq!(deserialized.idempotency_key, event.idempotency_key);
    assert_eq!(deserialized.chat_id, event.chat_id);
    assert_eq!(deserialized.sender_id, event.sender_id);
  }

  #[test]
  fn test_event_structures_debug() {
    // Test Debug trait for all event structures
    let message = create_test_message();
    let chat_members = vec![1, 2, 3];

    let message_event = MessageCreatedEvent {
      message: message.clone(),
      chat_members,
    };

    let chat_member_event = ChatMemberEvent {
      chat_id: 100,
      user_id: 10,
      joined_at: Utc::now(),
    };

    let duplicate_event = DuplicateMessageEvent {
      idempotency_key: Uuid::new_v4(),
      chat_id: 100,
      sender_id: fechatter_core::UserId(10).into(),
    };

    // Ensure Debug trait works correctly
    let message_debug = format!("{:?}", message_event);
    let chat_debug = format!("{:?}", chat_member_event);
    let duplicate_debug = format!("{:?}", duplicate_event);

    assert!(message_debug.contains("MessageCreatedEvent"));
    assert!(chat_debug.contains("ChatMemberEvent"));
    assert!(duplicate_debug.contains("DuplicateMessageEvent"));
  }

  #[test]
  fn test_subjects_config_completeness() {
    // Ensure all required subjects are configured
    let subjects = create_test_subjects();

    // Verify all fields are non-empty
    assert!(!subjects.message_created.is_empty());
    assert!(!subjects.message_updated.is_empty());
    assert!(!subjects.message_deleted.is_empty());
    assert!(!subjects.chat_created.is_empty());
    assert!(!subjects.chat_updated.is_empty());
    assert!(!subjects.user_joined_chat.is_empty());
    assert!(!subjects.user_left_chat.is_empty());
    assert!(!subjects.duplicate_message_attempted.is_empty());

    // Verify subject name format
    assert!(subjects.message_created.contains("messages"));
    assert!(subjects.user_joined_chat.contains("member"));
    assert!(subjects.duplicate_message_attempted.contains("duplicate"));
  }

  #[test]
  fn test_message_with_optional_fields() {
    // Test optional fields in message
    let mut message = create_test_message();

    // Test when files is None
    message.files = None;
    let event = MessageCreatedEvent {
      message: message.clone(),
      chat_members: vec![1, 2],
    };

    let serialized = serde_json::to_vec(&event).expect("Failed to serialize event");
    let deserialized: MessageCreatedEvent =
      serde_json::from_slice(&serialized).expect("Failed to deserialize event");

    assert_eq!(deserialized.message.files, None);

    // Test when idempotency_key is None
    message.idempotency_key = None;
    let event = MessageCreatedEvent {
      message: message.clone(),
      chat_members: vec![1, 2],
    };

    let serialized = serde_json::to_vec(&event).expect("Failed to serialize event");
    let deserialized: MessageCreatedEvent =
      serde_json::from_slice(&serialized).expect("Failed to deserialize event");

    assert_eq!(deserialized.message.idempotency_key, None);
  }

  #[test]
  fn test_error_handling_serialization() {
    // Test error handling for invalid data serialization
    // Since our structs are simple types, we mainly test edge cases

    let chat_members = vec![i64::MAX, i64::MIN, 0, -1];
    let message = create_test_message();

    let event = MessageCreatedEvent {
      message,
      chat_members,
    };

    // Even extreme values should serialize correctly
    let result = serde_json::to_vec(&event);
    assert!(result.is_ok());
  }
}
