// Messaging domain logic - business rules and orchestration

use async_trait::async_trait;
use std::sync::Arc;

use super::{events::*, repository::MessageRepository};
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

    // Check file count
    if message.files.len() > self.config.max_file_count {
      return Err(CoreError::Validation(format!(
        "Too many files. Max {} files allowed",
        self.config.max_file_count
      )));
    }

    // Validate that message has either content or files
    if message.content.trim().is_empty() && message.files.is_empty() {
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
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::Utc;
  use fechatter_core::{ChatId, CreateMessage, Message, MessageId, UserId};
  use uuid;

  struct MockMessageRepository;

  impl MockMessageRepository {
    fn new() -> Self {
      Self
    }
  }

  fn create_test_message() -> CreateMessage {
    CreateMessage {
      content: "Test message content".to_string(),
      files: Vec::new(),
      idempotency_key: uuid::Uuid::now_v7(),
    }
  }

  #[tokio::test]
  async fn validate_message_should_check_content_length() {
    let config = MessageConfig::default();
    let service = MessageDomainServiceImpl {
      repository: Arc::new(MockMessageRepository::new()),
      config: config.clone(),
    };

    // Valid content should pass
    let valid_message = create_test_message();
    assert!(service.validate_message(&valid_message).is_ok());

    // Too long content should fail
    let mut long_message = create_test_message();
    long_message.content = "a".repeat(10001); // Above max of 10000
    let result = service.validate_message(&long_message);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("too long"));
    assert!(result.unwrap_err().to_string().contains("10000 characters"));

    // Exactly max length should pass
    let mut max_message = create_test_message();
    max_message.content = "a".repeat(10000);
    assert!(service.validate_message(&max_message).is_ok());
  }

  #[tokio::test]
  async fn validate_message_should_check_file_count() {
    let config = MessageConfig::default();
    let service = MessageDomainServiceImpl {
      repository: Arc::new(MockMessageRepository::new()),
      config: config.clone(),
    };

    // Valid file count should pass
    let mut valid_message = create_test_message();
    valid_message.files = vec!["file1.jpg".to_string(), "file2.png".to_string()];
    assert!(service.validate_message(&valid_message).is_ok());

    // Too many files should fail
    let mut many_files_message = create_test_message();
    many_files_message.files = (0..11) // 11 files, above max of 10
      .map(|i| format!("file{}.jpg", i))
      .collect();
    let result = service.validate_message(&many_files_message);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Too many files"));
    assert!(result.unwrap_err().to_string().contains("10 files allowed"));

    // Exactly max files should pass
    let mut max_files_message = create_test_message();
    max_files_message.files = (0..10) // Exactly 10 files
      .map(|i| format!("file{}.jpg", i))
      .collect();
    assert!(service.validate_message(&max_files_message).is_ok());
  }

  #[tokio::test]
  async fn validate_message_should_require_content_or_files() {
    let config = MessageConfig::default();
    let service = MessageDomainServiceImpl {
      repository: Arc::new(MockMessageRepository::new()),
      config,
    };

    // Message with content but no files should pass
    let content_only_message = create_test_message();
    assert!(service.validate_message(&content_only_message).is_ok());

    // Message with files but no content should pass
    let mut files_only_message = create_test_message();
    files_only_message.content = "".to_string();
    files_only_message.files = vec!["file1.jpg".to_string()];
    assert!(service.validate_message(&files_only_message).is_ok());

    // Message with both content and files should pass
    let mut both_message = create_test_message();
    both_message.files = vec!["file1.jpg".to_string()];
    assert!(service.validate_message(&both_message).is_ok());

    // Message with neither content nor files should fail
    let mut empty_message = create_test_message();
    empty_message.content = "".to_string();
    empty_message.files = Vec::new();
    let result = service.validate_message(&empty_message);
    assert!(result.is_err());
    assert!(
      result
        .unwrap_err()
        .to_string()
        .contains("must contain either text content or attachments")
    );

    // Message with whitespace-only content and no files should fail
    let mut whitespace_message = create_test_message();
    whitespace_message.content = "   \n\t   ".to_string();
    whitespace_message.files = Vec::new();
    let result = service.validate_message(&whitespace_message);
    assert!(result.is_err());
    assert!(
      result
        .unwrap_err()
        .to_string()
        .contains("must contain either text content or attachments")
    );
  }

  #[tokio::test]
  async fn validate_message_with_custom_config() {
    let config = MessageConfig {
      cache_enabled: true,
      cache_ttl: 3600,
      max_content_length: 500,
      max_file_count: 3,
    };
    let service = MessageDomainServiceImpl {
      repository: Arc::new(MockMessageRepository::new()),
      config,
    };

    // Test custom content length limit
    let mut long_message = create_test_message();
    long_message.content = "a".repeat(501); // Above custom max of 500
    let result = service.validate_message(&long_message);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("500 characters"));

    // Test custom file count limit
    let mut many_files_message = create_test_message();
    many_files_message.files = vec![
      "file1.jpg".to_string(),
      "file2.jpg".to_string(),
      "file3.jpg".to_string(),
      "file4.jpg".to_string(), // 4 files, above custom max of 3
    ];
    let result = service.validate_message(&many_files_message);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("3 files allowed"));

    // Valid message with custom limits should pass
    let mut valid_message = create_test_message();
    valid_message.content = "a".repeat(500); // Exactly at limit
    valid_message.files = vec!["file1.jpg".to_string(), "file2.jpg".to_string()]; // Under limit
    assert!(service.validate_message(&valid_message).is_ok());
  }

  #[tokio::test]
  async fn message_config_should_have_reasonable_defaults() {
    let config = MessageConfig::default();

    assert!(config.cache_enabled);
    assert_eq!(config.cache_ttl, 3600);
    assert_eq!(config.max_content_length, 10000);
    assert_eq!(config.max_file_count, 10);
  }
}
