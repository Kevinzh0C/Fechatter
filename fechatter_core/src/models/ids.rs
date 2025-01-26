//! Newtype wrappers for various ID types in the system
//!
//! These types provide type safety and prevent mixing up different kinds of IDs

use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::fmt;
use utoipa::ToSchema;

/// User ID newtype
#[derive(
  Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Type, ToSchema,
)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct UserId(pub i64);

/// Chat ID newtype
#[derive(
  Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Type, ToSchema,
)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct ChatId(pub i64);

/// Workspace ID newtype
#[derive(
  Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Type, ToSchema,
)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct WorkspaceId(pub i64);

/// Message ID newtype
#[derive(
  Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Type, ToSchema,
)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct MessageId(pub i64);

// Implement Display for all ID types
impl fmt::Display for UserId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl fmt::Display for ChatId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl fmt::Display for WorkspaceId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl fmt::Display for MessageId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

// Implement From<i64> for convenience
impl From<i64> for UserId {
  fn from(id: i64) -> Self {
    UserId(id)
  }
}

impl From<i64> for ChatId {
  fn from(id: i64) -> Self {
    ChatId(id)
  }
}

impl From<i64> for WorkspaceId {
  fn from(id: i64) -> Self {
    WorkspaceId(id)
  }
}

impl From<i64> for MessageId {
  fn from(id: i64) -> Self {
    MessageId(id)
  }
}

// Implement Into<i64> for convenience
impl From<UserId> for i64 {
  fn from(id: UserId) -> Self {
    id.0
  }
}

impl From<ChatId> for i64 {
  fn from(id: ChatId) -> Self {
    id.0
  }
}

impl From<WorkspaceId> for i64 {
  fn from(id: WorkspaceId) -> Self {
    id.0
  }
}

impl From<MessageId> for i64 {
  fn from(id: MessageId) -> Self {
    id.0
  }
}

// Add methods for each ID type
impl UserId {
  /// Create a new UserId
  pub fn new(id: i64) -> Self {
    UserId(id)
  }

  /// Get the inner value
  pub fn inner(&self) -> i64 {
    self.0
  }
}

impl ChatId {
  /// Create a new ChatId
  pub fn new(id: i64) -> Self {
    ChatId(id)
  }

  /// Get the inner value
  pub fn inner(&self) -> i64 {
    self.0
  }
}

impl WorkspaceId {
  /// Create a new WorkspaceId
  pub fn new(id: i64) -> Self {
    WorkspaceId(id)
  }

  /// Get the inner value
  pub fn inner(&self) -> i64 {
    self.0
  }
}

impl MessageId {
  /// Create a new MessageId
  pub fn new(id: i64) -> Self {
    MessageId(id)
  }

  /// Get the inner value
  pub fn inner(&self) -> i64 {
    self.0
  }
}

// Implement Default for ID types (useful for tests)
impl Default for UserId {
  fn default() -> Self {
    UserId(0)
  }
}

impl Default for ChatId {
  fn default() -> Self {
    ChatId(0)
  }
}

impl Default for WorkspaceId {
  fn default() -> Self {
    WorkspaceId(0)
  }
}

impl Default for MessageId {
  fn default() -> Self {
    MessageId(0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_id_creation() {
    let user_id = UserId::new(123);
    assert_eq!(user_id.inner(), 123);
    assert_eq!(user_id.0, 123);
  }

  #[test]
  fn test_id_conversion() {
    let user_id = UserId::from(456);
    let id_value: i64 = user_id.into();
    assert_eq!(id_value, 456);
  }

  #[test]
  fn test_id_display() {
    let chat_id = ChatId::new(789);
    assert_eq!(format!("{}", chat_id), "789");
  }

  #[test]
  fn test_id_equality() {
    let id1 = WorkspaceId::new(100);
    let id2 = WorkspaceId::new(100);
    let id3 = WorkspaceId::new(200);

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
  }

  #[test]
  fn test_id_ordering() {
    let id1 = MessageId::new(10);
    let id2 = MessageId::new(20);

    assert!(id1 < id2);
    assert!(id2 > id1);
  }
}
