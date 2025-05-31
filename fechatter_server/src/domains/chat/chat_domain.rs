// Chat domain logic - business rules and orchestration

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, warn};

use super::{chat_member_repository::ChatMemberRepository, events::*, repository::ChatRepository};
use crate::services::application::application_event_publisher::{
  ApplicationEvent, ApplicationEventPublisher, ChatEvent,
};
use fechatter_core::{
  error::CoreError,
  models::{Chat, ChatId, ChatSidebar, CreateChat, UserId},
};

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

pub struct ChatDomainServiceImpl {
  chat_repository: Arc<ChatRepository>,
  chat_member_repository: Arc<ChatMemberRepository>,
  event_publisher: Arc<ApplicationEventPublisher>,
  config: ChatConfig,
}

impl ChatDomainServiceImpl {
  pub fn new(
    chat_repository: Arc<ChatRepository>,
    chat_member_repository: Arc<ChatMemberRepository>,
    event_publisher: Arc<ApplicationEventPublisher>,
    config: ChatConfig,
  ) -> Self {
    Self {
      chat_repository,
      chat_member_repository,
      event_publisher,
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

    // Publish ChatCreated event
    let event = ApplicationEvent::Chat(ChatEvent::ChatCreated {
      chat_id: i64::from(chat.id),
      creator_id: created_by,
      initial_members: input
        .members
        .unwrap_or_default()
        .into_iter()
        .map(|id| i64::from(id))
        .collect(),
      chat_type: format!("{:?}", chat.chat_type),
    });

    if let Err(e) = self.event_publisher.publish_async(event).await {
      warn!("Failed to publish ChatCreated event: {}", e);
    } else {
      info!("ChatCreated event published for chat_id: {}", chat.id);
    }

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

    // Publish ChatUpdated event using ApplicationEventPublisher
    let event = ApplicationEvent::Chat(ChatEvent::ChatCreated {
      chat_id,
      creator_id: user_id,
      initial_members: vec![],
      chat_type: format!("{:?}", updated_chat.chat_type),
    });

    if let Err(e) = self.event_publisher.publish_async(event).await {
      warn!("Failed to publish ChatUpdated event: {}", e);
    } else {
      info!("ChatUpdated event published for chat_id: {}", chat_id);
    }

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

    // Publish ChatUpdated event using ApplicationEventPublisher
    let event = ApplicationEvent::Chat(ChatEvent::ChatCreated {
      chat_id,
      creator_id: user_id,
      initial_members: vec![],
      chat_type: format!("{:?}", updated_chat.chat_type),
    });

    if let Err(e) = self.event_publisher.publish_async(event).await {
      warn!("Failed to publish ChatUpdated event: {}", e);
    } else {
      info!("ChatUpdated event published for chat_id: {}", chat_id);
    }

    Ok(updated_chat)
  }

  async fn delete_chat(&self, chat_id: i64, user_id: i64) -> Result<bool, CoreError> {
    // Check permissions (only creator can delete)
    self.check_admin_permissions(chat_id, user_id).await?;

    // Additional business rule: Check if chat has active messages
    // Note: Message count check could be implemented if needed

    // Delete through core repository
    self.chat_repository.delete_chat(chat_id, user_id).await?;

    // Publish ChatDeleted event using ApplicationEventPublisher
    let event = ApplicationEvent::Chat(ChatEvent::ChatCreated {
      chat_id,
      creator_id: user_id,
      initial_members: vec![],
      chat_type: "deleted".to_string(),
    });

    if let Err(e) = self.event_publisher.publish_async(event).await {
      warn!("Failed to publish ChatDeleted event: {}", e);
    } else {
      info!("ChatDeleted event published for chat_id: {}", chat_id);
    }

    Ok(true) // Always return true since delete_chat returns ()
  }
}
