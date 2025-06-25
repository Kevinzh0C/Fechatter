/// Event contracts shared between fechatter_server and notify_server
/// This module serves as the single source of truth for event definitions
use crate::{ChatId, Message, MessageId, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event versioning for backward compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventVersion {
  V1,
}

impl Default for EventVersion {
  fn default() -> Self {
    EventVersion::V1
  }
}

/// Base trait for all events with versioning support
pub trait VersionedEvent {
  fn version(&self) -> EventVersion;
}

/// Message lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageLifecycle {
  Created,
  Updated,
  Deleted,
}

/// Message event with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEvent {
  #[serde(default)]
  pub version: EventVersion,
  pub kind: MessageLifecycle,
  pub msg: Message,
  pub members: Vec<UserId>,
  pub occurred_at: DateTime<Utc>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sig: Option<String>,
}

impl VersionedEvent for MessageEvent {
  fn version(&self) -> EventVersion {
    self.version
  }
}

/// Chat member joined event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMemberJoinedEvent {
  #[serde(default)]
  pub version: EventVersion,
  pub chat_id: ChatId,
  pub user_id: UserId,
  pub occurred_at: DateTime<Utc>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sig: Option<String>,
}

impl VersionedEvent for ChatMemberJoinedEvent {
  fn version(&self) -> EventVersion {
    self.version
  }
}

/// Chat member left event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMemberLeftEvent {
  #[serde(default)]
  pub version: EventVersion,
  pub chat_id: ChatId,
  pub user_id: UserId,
  pub occurred_at: DateTime<Utc>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sig: Option<String>,
}

impl VersionedEvent for ChatMemberLeftEvent {
  fn version(&self) -> EventVersion {
    self.version
  }
}

/// Duplicate message attempt event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateMessageEvent {
  #[serde(default)]
  pub version: EventVersion,
  pub idempotency_key: Uuid,
  pub chat_id: ChatId,
  pub sender_id: UserId,
  pub occurred_at: DateTime<Utc>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sig: Option<String>,
}

impl VersionedEvent for DuplicateMessageEvent {
  fn version(&self) -> EventVersion {
    self.version
  }
}

/// Event subjects/topics constants
pub mod subjects {
  pub const MESSAGE_CREATED: &str = "fechatter.message.created";
  pub const MESSAGE_UPDATED: &str = "fechatter.message.updated";
  pub const MESSAGE_DELETED: &str = "fechatter.message.deleted";
  pub const CHAT_MEMBER_JOINED: &str = "fechatter.chat.joined";
  pub const CHAT_MEMBER_LEFT: &str = "fechatter.chat.left";
  pub const DUPLICATE_MESSAGE: &str = "fechatter.message.duplicate";
  pub const SEARCH_INDEX: &str = "fechatter.search.index";
}

/// Signature verification interface
pub trait SignatureVerifier {
  fn verify_signature(&self, payload: &[u8], signature: &str, secret: &[u8]) -> bool;
}

/// Default HMAC-SHA256 signature verifier
pub struct HmacSha256Verifier;

impl SignatureVerifier for HmacSha256Verifier {
  fn verify_signature(&self, payload: &[u8], signature: &str, secret: &[u8]) -> bool {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    // Validate signature format
    if signature.len() != 64 || !signature.chars().all(|c| c.is_ascii_hexdigit()) {
      return false;
    }

    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret) else {
      return false;
    };

    mac.update(payload);

    let Ok(sig_bytes) = hex::decode(signature) else {
      return false;
    };

    mac.verify_slice(&sig_bytes).is_ok()
  }
}

/// Event envelope for transport with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
  pub id: Uuid,
  pub subject: String,
  pub timestamp: DateTime<Utc>,
  pub payload: T,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub correlation_id: Option<Uuid>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub causation_id: Option<Uuid>,
}

impl<T> EventEnvelope<T> {
  pub fn new(subject: String, payload: T) -> Self {
    Self {
      id: Uuid::new_v4(),
      subject,
      timestamp: Utc::now(),
      payload,
      correlation_id: None,
      causation_id: None,
    }
  }

  pub fn with_correlation(mut self, correlation_id: Uuid) -> Self {
    self.correlation_id = Some(correlation_id);
    self
  }

  pub fn with_causation(mut self, causation_id: Uuid) -> Self {
    self.causation_id = Some(causation_id);
    self
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_event_versioning() {
    let event = MessageEvent {
      version: EventVersion::V1,
      kind: MessageLifecycle::Created,
      msg: Message {
        id: MessageId(1),
        chat_id: ChatId(1),
        sender_id: UserId(1),
        content: "test".to_string(),
        files: None,
        created_at: Utc::now(),
        idempotency_key: None,
      },
      members: vec![UserId(1), UserId(2)],
      occurred_at: Utc::now(),
      sig: None,
    };

    assert_eq!(event.version(), EventVersion::V1);
  }

  #[test]
  fn test_signature_verification() {
    let verifier = HmacSha256Verifier;
    let secret = b"test-secret";
    let payload = b"test payload";

    // Create valid signature
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac = Hmac::<Sha256>::new_from_slice(secret).unwrap();
    mac.update(payload);
    let signature = hex::encode(mac.finalize().into_bytes());

    assert!(verifier.verify_signature(payload, &signature, secret));
    assert!(!verifier.verify_signature(payload, "invalid", secret));
  }
}
