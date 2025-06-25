// Chat domain logic - business rules and orchestration

use async_trait::async_trait;
use std::sync::Arc;
use tracing::info;

use super::{chat_member_repository::ChatMemberRepository, repository::ChatRepository};
use crate::services::infrastructure::flows::SimplifiedEventPublisher;
use fechatter_core::{
  error::CoreError,
  models::{Chat, ChatSidebar, CreateChat},
};
use sqlx::Row;

/// Domain service trait for chat business logic
#[async_trait]
pub trait ChatDomainService: Send + Sync {
  async fn create_chat(
    &self,
    input: CreateChat,
    created_by: i64,
    workspace_id: Option<i64>,
  ) -> Result<Chat, CoreError>;

  async fn get_chat(&self, chat_id: i64) -> Result<Option<Chat>, CoreError>;

  async fn get_user_chats(&self, user_id: i64) -> Result<Vec<ChatSidebar>, CoreError>;

  async fn update_chat_name(
    &self,
    chat_id: i64,
    user_id: i64,
    new_name: String,
  ) -> Result<Chat, CoreError>;

  async fn update_chat_description(
    &self,
    chat_id: i64,
    user_id: i64,
    new_description: String,
  ) -> Result<Chat, CoreError>;

  async fn delete_chat(&self, chat_id: i64, user_id: i64) -> Result<bool, CoreError>;

  async fn transfer_ownership(
    &self,
    chat_id: i64,
    current_owner_id: i64,
    new_owner_id: i64,
  ) -> Result<(), CoreError>;
}

#[derive(Debug, Clone)]
pub struct ChatConfig {
  pub cache_enabled: bool,
  pub cache_ttl: u64,
  pub max_name_length: usize,
  pub max_description_length: usize,
}

impl Default for ChatConfig {
  fn default() -> Self {
    Self {
      cache_enabled: true,
      cache_ttl: 300, // 5 minutes
      max_name_length: 128,
      max_description_length: 500,
    }
  }
}

#[derive(Clone)]
pub struct ChatDomainServiceImpl {
  chat_repository: Arc<ChatRepository>,
  chat_member_repository: Arc<ChatMemberRepository>,
  _event_publisher: Arc<SimplifiedEventPublisher>, // 暂时不使用，避免编译错误
  config: ChatConfig,
}

impl ChatDomainServiceImpl {
  pub fn new(
    chat_repository: Arc<ChatRepository>,
    chat_member_repository: Arc<ChatMemberRepository>,
    event_publisher: Arc<SimplifiedEventPublisher>,
    config: ChatConfig,
  ) -> Self {
    Self {
      chat_repository,
      chat_member_repository,
      _event_publisher: event_publisher,
      config,
    }
  }

  /// Business logic for validating chat creation
  fn validate_chat_creation(&self, input: &CreateChat) -> Result<(), CoreError> {
    // Validate chat name
    fechatter_core::models::chat::validate_chat_name(&input.name)?;

    // Additional validation for description length
    if let Some(description) = &input.description {
      if description.len() > self.config.max_description_length {
        return Err(CoreError::Validation(format!(
          "Description too long. Max {} characters allowed",
          self.config.max_description_length
        )));
      }
    }

    Ok(())
  }

  /// Business logic for validating chat updates
  fn validate_chat_update(&self, name: &str, description: Option<&str>) -> Result<(), CoreError> {
    // Validate name
    fechatter_core::models::chat::validate_chat_name(name)?;

    // Validate description if provided
    if let Some(desc) = description {
      if desc.len() > self.config.max_description_length {
        return Err(CoreError::Validation(format!(
          "Description too long. Max {} characters allowed",
          self.config.max_description_length
        )));
      }
    }

    Ok(())
  }

  /// Business logic for checking chat permissions
  async fn check_chat_permissions(&self, chat_id: i64, user_id: i64) -> Result<(), CoreError> {
    // Check if user is a member or creator of the chat
    let is_member = self
      .chat_member_repository
      .is_user_member(chat_id, user_id)
      .await?;
    if !is_member {
      return Err(CoreError::Unauthorized(
        "User is not a member of this chat".to_string(),
      ));
    }
    Ok(())
  }

  /// Business logic for checking admin permissions
  async fn check_admin_permissions(&self, chat_id: i64, user_id: i64) -> Result<(), CoreError> {
    // Check if user is the creator of the chat
    let is_creator = self
      .chat_member_repository
      .is_user_creator(chat_id, user_id)
      .await?;
    if !is_creator {
      return Err(CoreError::Unauthorized(
        "Only chat creator can perform this action".to_string(),
      ));
    }
    Ok(())
  }
}

#[async_trait]
impl ChatDomainService for ChatDomainServiceImpl {
  async fn create_chat(
    &self,
    input: CreateChat,
    created_by: i64,
    workspace_id: Option<i64>,
  ) -> Result<Chat, CoreError> {
    // Validate business rules
    self.validate_chat_creation(&input)?;

    // Create through repository
    let chat = self
      .chat_repository
      .create_chat(input.clone(), created_by, workspace_id)
      .await?;

    // TODO: 重新设计事件发布机制
    // 暂时注释掉事件发布代码以解决编译错误
    info!("Chat created: {}", chat.id);

    Ok(chat)
  }

  async fn get_chat(&self, chat_id: i64) -> Result<Option<Chat>, CoreError> {
    self.chat_repository.find_chat_by_id(chat_id).await
  }

  async fn get_user_chats(&self, user_id: i64) -> Result<Vec<ChatSidebar>, CoreError> {
    self.chat_repository.get_sidebar_chats(user_id).await
  }

  async fn update_chat_name(
    &self,
    chat_id: i64,
    user_id: i64,
    new_name: String,
  ) -> Result<Chat, CoreError> {
    // Check permissions
    self.check_admin_permissions(chat_id, user_id).await?;

    // Validate new name
    self.validate_chat_update(&new_name, None)?;

    // Update through core repository
    let updated_chat = self
      .chat_repository
      .update_chat_name(chat_id, user_id, &new_name)
      .await?;

    // TODO: 重新设计事件发布机制
    info!("Chat {} name updated by user {}", chat_id, user_id);

    Ok(updated_chat)
  }

  async fn update_chat_description(
    &self,
    chat_id: i64,
    user_id: i64,
    new_description: String,
  ) -> Result<Chat, CoreError> {
    // Check permissions
    self.check_admin_permissions(chat_id, user_id).await?;

    // Validate new description
    self.validate_chat_update("dummy", Some(&new_description))?; // Name validation will be skipped

    // Update through core repository
    let updated_chat = self
      .chat_repository
      .update_chat_description(chat_id, user_id, &new_description)
      .await?;

    // TODO: 重新设计事件发布机制
    info!("Chat {} description updated by user {}", chat_id, user_id);

    Ok(updated_chat)
  }

  async fn delete_chat(&self, chat_id: i64, user_id: i64) -> Result<bool, CoreError> {
    // Check permissions (only creator can delete)
    self.check_admin_permissions(chat_id, user_id).await?;

    // Additional business rule: Check if chat has active messages
    // Note: Message count check could be implemented if needed

    // Delete through core repository
    self.chat_repository.delete_chat(chat_id, user_id).await?;

    // TODO: 重新设计事件发布机制
    info!("Chat {} deleted by user {}", chat_id, user_id);

    Ok(true) // Always return true since delete_chat returns ()
  }

  async fn transfer_ownership(
    &self,
    chat_id: i64,
    current_owner_id: i64,
    new_owner_id: i64,
  ) -> Result<(), CoreError> {
    // Business rule: Cannot transfer to self
    if current_owner_id == new_owner_id {
      return Err(CoreError::Validation(
        "Cannot transfer ownership to yourself".to_string(),
      ));
    }

    // Check if current user is actually the owner
    self
      .check_admin_permissions(chat_id, current_owner_id)
      .await?;

    // Check if new owner is a member of the chat
    let is_member = self
      .chat_member_repository
      .is_user_member(chat_id, new_owner_id)
      .await?;

    if !is_member {
      return Err(CoreError::Validation(
        "New owner must be a member of the chat".to_string(),
      ));
    }

    // Check if chat exists and get its details
    let chat = self
      .chat_repository
      .find_chat_by_id(chat_id)
      .await?
      .ok_or_else(|| CoreError::NotFound(format!("Chat {} not found", chat_id)))?;

    // Business rule: Cannot transfer ownership of direct messages
    if matches!(chat.chat_type, fechatter_core::models::ChatType::Single) {
      return Err(CoreError::Validation(
        "Cannot transfer ownership of direct messages".to_string(),
      ));
    }

    // Perform the ownership transfer
    let transferred = self
      .chat_member_repository
      .transfer_ownership(chat_id, current_owner_id, new_owner_id)
      .await?;

    if !transferred {
      return Err(CoreError::Internal(
        "Failed to transfer ownership".to_string(),
      ));
    }

    // TODO: 重新设计事件发布机制
    info!(
      "Chat {} ownership transferred from user {} to user {}",
      chat_id, current_owner_id, new_owner_id
    );

    Ok(())
  }
}
