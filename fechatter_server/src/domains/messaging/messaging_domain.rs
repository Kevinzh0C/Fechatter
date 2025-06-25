// Messaging domain logic - business rules and orchestration

use async_trait::async_trait;
use std::sync::Arc;

use super::repository::MessageRepository;
use fechatter_core::{CreateMessage, ListMessages, Message, error::CoreError};

/// Domain service trait for messaging business logic
#[async_trait]
pub trait MessageDomainService: Send + Sync {
  async fn send_message(
    &self,
    message: CreateMessage,
    chat_id: i64,
    user_id: i64,
  ) -> Result<Message, CoreError>;
  async fn get_message(&self, id: i64) -> Result<Option<Message>, CoreError>;
  async fn list_messages(
    &self,
    input: ListMessages,
    chat_id: i64,
    user_id: i64,
  ) -> Result<Vec<Message>, CoreError>;
  async fn edit_message(
    &self,
    id: i64,
    content: String,
    editor_id: i64,
  ) -> Result<Message, CoreError>;
  async fn delete_message(&self, id: i64, user_id: i64) -> Result<(), CoreError>;
  async fn get_messages_count(&self, chat_id: i64) -> Result<i64, CoreError>;
  async fn get_chat_members(&self, chat_id: i64) -> Result<Vec<i64>, CoreError>;

  async fn mark_message_delivered(&self, message_id: i64, user_id: i64) -> Result<(), CoreError>;

  async fn mark_message_read(&self, message_id: i64, user_id: i64) -> Result<(), CoreError>;

  async fn mark_messages_read_batch(
    &self,
    message_ids: &[i64],
    user_id: i64,
  ) -> Result<(), CoreError>;

  async fn get_unread_count(&self, chat_id: i64, user_id: i64) -> Result<i64, CoreError>;

  // =============================================================================
  // MENTIONS MANAGEMENT
  // =============================================================================

  /// Get mentions for a specific message
  async fn get_message_mentions(
    &self,
    message_id: i64,
  ) -> Result<Vec<(i64, String, String, String)>, CoreError>;

  /// Get unread mentions for a user across all chats
  async fn get_unread_mentions_for_user(
    &self,
    user_id: i64,
  ) -> Result<Vec<(i64, i64, String, String, chrono::DateTime<chrono::Utc>, String)>, CoreError>;

  // =============================================================================
  // DETAILED RECEIPTS MANAGEMENT
  // =============================================================================

  /// Get detailed read receipts for a message
  async fn get_detailed_message_receipts(
    &self,
    message_id: i64,
  ) -> Result<Vec<(i64, String, String, String, chrono::DateTime<chrono::Utc>)>, CoreError>;

  // =============================================================================
  // ENHANCED READ TRACKING
  // =============================================================================

  /// Mark message as read with enhanced tracking (handles mentions)
  async fn mark_message_read_enhanced(
    &self,
    user_id: i64,
    chat_id: i64,
    message_id: i64,
  ) -> Result<(), CoreError>;
}

#[derive(Debug, Clone)]
pub struct MessageConfig {
  pub cache_enabled: bool,
  pub cache_ttl: u64,
  pub max_content_length: usize,
  pub max_file_count: usize,
}

impl Default for MessageConfig {
  fn default() -> Self {
    Self {
      cache_enabled: true,
      cache_ttl: 3600,
      max_content_length: 10000,
      max_file_count: 10,
    }
  }
}

#[derive(Clone)]
pub struct MessageDomainServiceImpl {
  repository: Arc<MessageRepository>,
  config: MessageConfig,
}

impl MessageDomainServiceImpl {
  pub fn new(repository: Arc<MessageRepository>, config: MessageConfig) -> Self {
    Self { repository, config }
  }

  /// Business logic for validating message content
  fn validate_message(&self, message: &CreateMessage) -> Result<(), CoreError> {
    // Check content length
    if message.content.len() > self.config.max_content_length {
      return Err(CoreError::Validation(format!(
        "Message content too long. Max {} characters allowed",
        self.config.max_content_length
      )));
    }

    // Check file count - handle Option<Vec<String>>
    if let Some(files) = &message.files {
      if files.len() > self.config.max_file_count {
        return Err(CoreError::Validation(format!(
          "Too many files. Max {} files allowed",
          self.config.max_file_count
        )));
      }
    }

    // Validate that message has either content or files
    let has_content = !message.content.trim().is_empty();
    let has_files = message
      .files
      .as_ref()
      .map_or(false, |files| !files.is_empty());

    if !has_content && !has_files {
      return Err(CoreError::Validation(
        "Message must contain either text content or attachments".into(),
      ));
    }

    Ok(())
  }
}

#[async_trait]
impl MessageDomainService for MessageDomainServiceImpl {
  async fn send_message(
    &self,
    message: CreateMessage,
    chat_id: i64,
    user_id: i64,
  ) -> Result<Message, CoreError> {
    // Validate business rules
    self.validate_message(&message)?;

    // Create through repository - now using core models directly
    let saved_message = self
      .repository
      .create_message(message, chat_id, user_id)
      .await?;

    // TODO: Publish MessageSent event
    // This would be handled by infrastructure layer

    Ok(saved_message)
  }

  async fn get_message(&self, id: i64) -> Result<Option<Message>, CoreError> {
    self.repository.get_message_by_id(id).await
  }

  async fn list_messages(
    &self,
    input: ListMessages,
    chat_id: i64,
    user_id: i64,
  ) -> Result<Vec<Message>, CoreError> {
    // Use core model directly
    self.repository.list_messages(input, chat_id).await
  }

  async fn edit_message(
    &self,
    id: i64,
    content: String,
    editor_id: i64,
  ) -> Result<Message, CoreError> {
    // Validate new content
    if content.len() > self.config.max_content_length {
      return Err(CoreError::Validation(format!(
        "Message content too long. Max {} characters allowed",
        self.config.max_content_length
      )));
    }

    if content.trim().is_empty() {
      return Err(CoreError::Validation(
        "Message content cannot be empty".to_string(),
      ));
    }

    // Update through repository
    let updated_message = self
      .repository
      .update_message(id, content, editor_id)
      .await?;

    // TODO: Publish MessageEdited event
    // This would be handled by infrastructure layer

    Ok(updated_message)
  }

  async fn delete_message(&self, id: i64, user_id: i64) -> Result<(), CoreError> {
    // Delete through repository
    self.repository.delete_message(id, user_id).await?;

    // TODO: Publish MessageDeleted event
    // This would be handled by infrastructure layer

    Ok(())
  }

  async fn get_messages_count(&self, chat_id: i64) -> Result<i64, CoreError> {
    self.repository.get_messages_count(chat_id).await
  }

  async fn get_chat_members(&self, chat_id: i64) -> Result<Vec<i64>, CoreError> {
    self.repository.get_chat_members(chat_id).await
  }

  async fn mark_message_delivered(&self, message_id: i64, user_id: i64) -> Result<(), CoreError> {
    self
      .repository
      .mark_message_delivered(message_id, user_id)
      .await
  }

  async fn mark_message_read(&self, message_id: i64, user_id: i64) -> Result<(), CoreError> {
    self.repository.mark_message_read(message_id, user_id).await
  }

  async fn mark_messages_read_batch(
    &self,
    message_ids: &[i64],
    user_id: i64,
  ) -> Result<(), CoreError> {
    self
      .repository
      .mark_messages_read_batch(message_ids, user_id)
      .await
  }

  async fn get_unread_count(&self, chat_id: i64, user_id: i64) -> Result<i64, CoreError> {
    self.repository.get_unread_count(chat_id, user_id).await
  }

  // =============================================================================
  // MENTIONS MANAGEMENT
  // =============================================================================

  /// Get mentions for a specific message
  async fn get_message_mentions(
    &self,
    message_id: i64,
  ) -> Result<Vec<(i64, String, String, String)>, CoreError> {
    self.repository.get_message_mentions(message_id).await
  }

  /// Get unread mentions for a user across all chats
  async fn get_unread_mentions_for_user(
    &self,
    user_id: i64,
  ) -> Result<Vec<(i64, i64, String, String, chrono::DateTime<chrono::Utc>, String)>, CoreError> {
    self.repository.get_unread_mentions_for_user(user_id).await
  }

  // =============================================================================
  // DETAILED RECEIPTS MANAGEMENT
  // =============================================================================

  /// Get detailed read receipts for a message
  async fn get_detailed_message_receipts(
    &self,
    message_id: i64,
  ) -> Result<Vec<(i64, String, String, String, chrono::DateTime<chrono::Utc>)>, CoreError> {
    self.repository.get_detailed_message_receipts(message_id).await
  }

  // =============================================================================
  // ENHANCED READ TRACKING
  // =============================================================================

  /// Mark message as read with enhanced tracking (handles mentions)
  async fn mark_message_read_enhanced(
    &self,
    user_id: i64,
    chat_id: i64,
    message_id: i64,
  ) -> Result<(), CoreError> {
    self.repository.mark_message_read_enhanced(user_id, chat_id, message_id).await
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn message_config_should_have_reasonable_defaults() {
    let config = MessageConfig::default();

    assert!(config.cache_enabled);
    assert_eq!(config.cache_ttl, 3600);
    assert_eq!(config.max_content_length, 10000);
    assert_eq!(config.max_file_count, 10);
  }

  // Note: Database-dependent tests are disabled for now
  // TODO: Implement proper mock repository for unit testing
}
