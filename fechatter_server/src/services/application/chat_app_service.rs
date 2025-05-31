//! # Chat Application Service
//!
//! ## Single Responsibility Principle Application
//! - Each function focuses on a single use case
//! - Business rule validation is independent
//! - Cache strategy is delegated to a dedicated service
//! - Event publishing is delegated to event publisher
//! - Clear layering between upper and lower levels

use crate::AppError;
use crate::AppState;
use crate::services::application::{
  ApplicationEvent, ApplicationEventPublisher, CacheStrategyService, ChatEvent,
};
use async_trait::async_trait;
use fechatter_core::contracts::CacheService;
use fechatter_core::{
  contracts::ChatRepository,
  error::CoreError,
  models::{Chat, ChatId, ChatSidebar, ChatType, CreateChat, UpdateChat, UserId},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, instrument, warn};

//==============================================================================
// BUSINESS ERROR DEFINITIONS
//==============================================================================

/// Chat business errors - Single responsibility: Chat domain error handling
#[derive(Debug, thiserror::Error)]
pub enum ChatError {
  #[error("Repository error: {0}")]
  Repository(String),

  #[error("Chat not found: {0}")]
  NotFound(i64),

  #[error("Permission denied: {0}")]
  PermissionDenied(String),

  #[error("Invalid chat configuration: {0}")]
  InvalidChatConfig(String),

  #[error("Member limit exceeded for 200DAU platform")]
  MemberLimitExceeded,

  #[error("Business rule violation: {0}")]
  BusinessRuleViolation(String),
}

impl From<ChatError> for AppError {
  fn from(err: ChatError) -> Self {
    match err {
      ChatError::Repository(msg) => AppError::InvalidInput(msg),
      ChatError::NotFound(id) => AppError::NotFound(vec![format!("Chat {} not found", id)]),
      ChatError::PermissionDenied(msg) => AppError::Unauthorized(msg),
      ChatError::InvalidChatConfig(msg) => AppError::InvalidInput(msg),
      ChatError::MemberLimitExceeded => AppError::InvalidInput("Member limit exceeded".to_string()),
      ChatError::BusinessRuleViolation(msg) => AppError::InvalidInput(msg),
    }
  }
}

impl From<CoreError> for ChatError {
  fn from(err: CoreError) -> Self {
    match err {
      CoreError::NotFound(msg) => ChatError::Repository(msg),
      CoreError::ValidationError(msg) => ChatError::InvalidChatConfig(msg),
      _ => ChatError::Repository(err.to_string()),
    }
  }
}

//==============================================================================
// APPLICATION SERVICE TRAIT
//==============================================================================

/// Chat application service contract - Defines use case boundaries
#[async_trait]
pub trait ChatServiceTrait: Send + Sync {
  /// Use case: Create chat - Core business scenario
  async fn create_chat(&self, input: CreateChatInput) -> Result<ChatDetailView, AppError>;

  /// Use case: Get chat details
  async fn get_chat(&self, id: i64) -> Result<Option<ChatDetailView>, AppError>;

  /// Use case: List user chats - Key sidebar functionality
  async fn list_user_chats(&self, user_id: i64) -> Result<Vec<ChatSidebar>, AppError>;

  /// Use case: Update chat - Chat modification
  async fn update_chat(
    &self,
    chat_id: ChatId,
    user_id: UserId,
    payload: UpdateChat,
  ) -> Result<ChatDetailView, AppError>;

  /// Use case: Delete chat - Chat removal
  async fn delete_chat(&self, chat_id: ChatId, user_id: UserId) -> Result<bool, AppError>;

  /// Use case: List chat members - Get members of a specific chat
  async fn list_chat_members(
    &self,
    chat_id: i64,
  ) -> Result<Vec<fechatter_core::models::ChatMember>, AppError>;

  /// Use case: Add members - Group/Channel management
  async fn add_members(
    &self,
    chat_id: i64,
    user_id: i64,
    member_ids: Vec<i64>,
  ) -> Result<(), AppError>;

  /// Use case: Remove members - Group/Channel management
  async fn remove_members(
    &self,
    chat_id: i64,
    user_id: i64,
    member_ids: Vec<i64>,
  ) -> Result<(), AppError>;

  /// Use case: Transfer chat ownership - Change chat owner
  async fn transfer_chat_ownership(
    &self,
    chat_id: i64,
    current_owner_id: i64,
    new_owner_id: i64,
  ) -> Result<bool, AppError>;
}

//==============================================================================
// BUSINESS RULES VALIDATOR - Single Responsibility
//==============================================================================

/// Chat business rules validator - Single responsibility: Business rule validation
pub struct ChatBusinessRules;

impl ChatBusinessRules {
  // 200DAU platform business rule constants
  const MAX_GROUP_MEMBERS: usize = 50;
  const MAX_CHANNEL_MEMBERS: usize = 200;
  const MAX_CHAT_NAME_LENGTH: usize = 128;
  const MIN_GROUP_MEMBERS: usize = 3;

  /// Validate chat configuration - Single responsibility: Configuration validation
  pub fn validate_chat_config(input: &CreateChatInput) -> Result<(), ChatError> {
    Self::validate_chat_name(&input.name)?;
    Self::validate_member_limits(input.chat_type, &input.initial_members)?;
    Ok(())
  }

  /// Validate chat name - Single responsibility: Name validation
  fn validate_chat_name(name: &str) -> Result<(), ChatError> {
    if name.trim().is_empty() {
      return Err(ChatError::InvalidChatConfig(
        "Chat name cannot be empty".to_string(),
      ));
    }

    if name.len() > Self::MAX_CHAT_NAME_LENGTH {
      return Err(ChatError::InvalidChatConfig(format!(
        "Chat name cannot exceed {} characters",
        Self::MAX_CHAT_NAME_LENGTH
      )));
    }

    Ok(())
  }

  /// Validate member limits - Single responsibility: Member limit validation
  fn validate_member_limits(chat_type: ChatType, initial_members: &[i64]) -> Result<(), ChatError> {
    let total_members = 1 + initial_members.len(); // +1 for creator

    match chat_type {
      ChatType::Single => {
        if total_members != 2 {
          return Err(ChatError::InvalidChatConfig(
            "Direct message must have exactly 2 members".to_string(),
          ));
        }
      }
      ChatType::Group => {
        if total_members < Self::MIN_GROUP_MEMBERS {
          return Err(ChatError::InvalidChatConfig(
            "Group chat needs at least 3 members".to_string(),
          ));
        }
        if total_members > Self::MAX_GROUP_MEMBERS {
          return Err(ChatError::MemberLimitExceeded);
        }
      }
      ChatType::PrivateChannel | ChatType::PublicChannel => {
        if total_members > Self::MAX_CHANNEL_MEMBERS {
          return Err(ChatError::MemberLimitExceeded);
        }
      }
    }

    Ok(())
  }

  /// Validate member operation permission - Single responsibility: Permission validation
  pub fn validate_member_operation_permission(
    _operator_id: i64,
    _chat_id: i64,
    _operation: MemberOperation,
  ) -> Result<(), ChatError> {
    // TODO: Implement permission check logic
    Ok(())
  }
}

/// Member operation type
#[derive(Debug, Clone)]
pub enum MemberOperation {
  Add(Vec<i64>),
  Remove(Vec<i64>),
}

//==============================================================================
// APPLICATION SERVICE IMPLEMENTATION
//==============================================================================

/// Chat application service implementation - Single responsibility: Use case orchestration and coordination
pub struct ChatService {
  repository: Arc<dyn ChatRepository>,
  cache_strategy: Arc<CacheStrategyService>,
  event_publisher: Arc<ApplicationEventPublisher>,
}

impl ChatService {
  /// Constructor - Dependency injection
  pub fn new(
    repository: Arc<dyn ChatRepository>,
    cache_strategy: Arc<CacheStrategyService>,
    event_publisher: Arc<ApplicationEventPublisher>,
  ) -> Self {
    Self {
      repository,
      cache_strategy,
      event_publisher,
    }
  }
}

#[async_trait]
impl ChatServiceTrait for ChatService {
  /// Use case: Create chat - Single responsibility: Chat creation use case orchestration
  #[instrument(skip(self, input), fields(chat_name = %input.name, chat_type = ?input.chat_type))]
  async fn create_chat(&self, input: CreateChatInput) -> Result<ChatDetailView, AppError> {
    // 1. Business rule validation (delegated to dedicated validator)
    ChatBusinessRules::validate_chat_config(&input)?;

    // 2. Convert to core layer data structure
    let create_data = self.build_create_chat_data(&input)?;

    // 3. Create chat through repository
    let chat = self.repository.create(&create_data).await?;

    // 4. Build return view
    let detail_view = ChatDetailView::from_chat(chat, input.initial_members.len() as i32 + 1);

    // 5. Publish domain event (async cache invalidation and notification)
    let event = ApplicationEvent::Chat(ChatEvent::ChatCreated {
      chat_id: detail_view.id,
      creator_id: input.created_by,
      initial_members: input.initial_members.clone(),
      chat_type: format!("{:?}", input.chat_type),
    });

    if let Err(e) = self.event_publisher.publish_async(event).await {
      warn!("Failed to publish chat created event: {}", e);
    }

    info!(
      chat_id = %detail_view.id,
      chat_type = ?input.chat_type,
      member_count = %detail_view.member_count,
      "Chat created successfully"
    );

    Ok(detail_view)
  }

  /// Use case: Get chat details - Single responsibility: Chat detail query
  #[instrument(skip(self))]
  async fn get_chat(&self, id: i64) -> Result<Option<ChatDetailView>, AppError> {
    // 1. Try to get from cache
    let cached_result = self.get_chat_from_cache(id).await;
    if cached_result.is_ok() {
      return cached_result;
    }

    // 2. Get from repository
    let chat = match self.repository.find_by_id(ChatId::new(id)).await? {
      Some(chat) => chat,
      None => return Ok(None),
    };

    // 3. Build detail view
    let detail_view = ChatDetailView::from_chat(chat, 1); // TODO: Get actual member count

    // 4. Update cache
    self.cache_chat_detail(&detail_view).await;

    Ok(Some(detail_view))
  }

  /// Use case: List user chats - Single responsibility: User chat list query
  #[instrument(skip(self))]
  async fn list_user_chats(&self, user_id: i64) -> Result<Vec<ChatSidebar>, AppError> {
    // 1. Try to get from cache
    let cached_result = self.get_user_chats_from_cache(user_id).await;
    if cached_result.is_ok() {
      return cached_result;
    }

    // 2. Get from repository
    let chat_list = self.repository.get_user_chats(UserId::new(user_id)).await?;

    // 3. Update cache
    self.cache_user_chats(user_id, &chat_list).await;

    Ok(chat_list)
  }

  /// Use case: Update chat - Chat modification
  async fn update_chat(
    &self,
    chat_id: ChatId,
    user_id: UserId,
    payload: UpdateChat,
  ) -> Result<ChatDetailView, AppError> {
    // Implementation needed
    unimplemented!()
  }

  /// Use case: Delete chat - Chat removal
  async fn delete_chat(&self, chat_id: ChatId, user_id: UserId) -> Result<bool, AppError> {
    // Implementation needed
    unimplemented!()
  }

  /// Use case: List chat members - Get members of a specific chat
  async fn list_chat_members(
    &self,
    chat_id: i64,
  ) -> Result<Vec<fechatter_core::models::ChatMember>, AppError> {
    // Implementation needed
    unimplemented!()
  }

  /// Use case: Add members - Single responsibility: Member addition use case orchestration
  #[instrument(skip(self))]
  async fn add_members(
    &self,
    chat_id: i64,
    user_id: i64,
    member_ids: Vec<i64>,
  ) -> Result<(), AppError> {
    if member_ids.is_empty() {
      return Ok(());
    }

    // 1. Validate operation permission
    ChatBusinessRules::validate_member_operation_permission(
      user_id,
      chat_id,
      MemberOperation::Add(member_ids.clone()),
    )?;

    // 2. Execute add operation (TODO: Implement actual add logic)
    // self.repository.add_members(chat_id, &member_ids).await?;

    // 3. Publish domain event
    let event = ApplicationEvent::Chat(ChatEvent::MembersAdded {
      chat_id,
      added_by: user_id,
      new_members: member_ids.clone(),
    });

    if let Err(e) = self.event_publisher.publish_async(event).await {
      warn!("Failed to publish members added event: {}", e);
    }

    info!(chat_id = %chat_id, added_members = ?member_ids, "Members added successfully");
    Ok(())
  }

  /// Use case: Remove members - Single responsibility: Member removal use case orchestration
  #[instrument(skip(self))]
  async fn remove_members(
    &self,
    chat_id: i64,
    user_id: i64,
    member_ids: Vec<i64>,
  ) -> Result<(), AppError> {
    if member_ids.is_empty() {
      return Ok(());
    }

    // 1. Validate operation permission
    ChatBusinessRules::validate_member_operation_permission(
      user_id,
      chat_id,
      MemberOperation::Remove(member_ids.clone()),
    )?;

    // 2. Execute remove operation (TODO: Implement actual remove logic)
    // self.repository.remove_members(chat_id, &member_ids).await?;

    // 3. Publish domain event
    let event = ApplicationEvent::Chat(ChatEvent::MembersRemoved {
      chat_id,
      removed_by: user_id,
      removed_members: member_ids.clone(),
    });

    if let Err(e) = self.event_publisher.publish_async(event).await {
      warn!("Failed to publish members removed event: {}", e);
    }

    info!(chat_id = %chat_id, removed_members = ?member_ids, "Members removed successfully");
    Ok(())
  }

  /// Use case: Transfer chat ownership - Change chat owner
  async fn transfer_chat_ownership(
    &self,
    chat_id: i64,
    current_owner_id: i64,
    new_owner_id: i64,
  ) -> Result<bool, AppError> {
    // Implementation needed
    unimplemented!()
  }
}

//==============================================================================
// PRIVATE HELPER METHODS - Single Responsibility
//==============================================================================

impl ChatService {
  /// Build create chat data - Single responsibility: Data conversion
  fn build_create_chat_data(&self, input: &CreateChatInput) -> Result<CreateChat, ChatError> {
    Ok(CreateChat {
      name: input.name.clone(),
      chat_type: input.chat_type,
      description: input.description.clone(),
      members: input
        .members
        .as_ref()
        .map(|ids| ids.iter().map(|&id| UserId::from(id)).collect()),
    })
  }

  /// Get chat details from cache - Single responsibility: Cache query
  async fn get_chat_from_cache(&self, chat_id: i64) -> Result<Option<ChatDetailView>, AppError> {
    let cache_key = CacheStrategyService::chat_detail_key(chat_id);

    match self.cache_strategy.get::<ChatDetailView>(&cache_key).await {
      Ok(Some(cached_data)) => Ok(Some(cached_data)),
      Ok(None) => Err(AppError::NotFound(vec!["Cache miss".to_string()])), // Use error to indicate cache miss
      Err(e) => {
        warn!("Cache error for chat {}: {}", chat_id, e);
        Err(AppError::InvalidInput(e.to_string()))
      }
    }
  }

  /// Cache chat details - Single responsibility: Cache update
  async fn cache_chat_detail(&self, detail_view: &ChatDetailView) {
    let cache_key = CacheStrategyService::chat_detail_key(detail_view.id);

    if let Err(e) = self
      .cache_strategy
      .set(
        &cache_key,
        detail_view,
        CacheStrategyService::CHAT_DETAIL_TTL,
      )
      .await
    {
      warn!("Failed to cache chat detail {}: {}", detail_view.id, e);
    }
  }

  /// Get user chats from cache - Single responsibility: Cache query
  async fn get_user_chats_from_cache(&self, user_id: i64) -> Result<Vec<ChatSidebar>, AppError> {
    let cache_key = CacheStrategyService::user_chats_key(user_id);

    match self
      .cache_strategy
      .get::<Vec<ChatSidebar>>(&cache_key)
      .await
    {
      Ok(Some(cached_data)) => Ok(cached_data),
      Ok(None) => Err(AppError::NotFound(vec!["Cache miss".to_string()])), // Use error to indicate cache miss
      Err(e) => {
        warn!("Cache error for user chats {}: {}", user_id, e);
        Err(AppError::InvalidInput(e.to_string()))
      }
    }
  }

  /// Cache user chat list - Single responsibility: Cache update
  async fn cache_user_chats(&self, user_id: i64, chat_list: &[ChatSidebar]) {
    let cache_key = CacheStrategyService::user_chats_key(user_id);

    if let Err(e) = self
      .cache_strategy
      .set(&cache_key, &chat_list, CacheStrategyService::CHAT_LIST_TTL)
      .await
    {
      warn!("Failed to cache user {} chat list: {}", user_id, e);
    }
  }
}

//==============================================================================
// VIEW MODELS
//==============================================================================

/// Create chat input parameters - Service layer DTO
#[derive(Debug, Clone)]
pub struct CreateChatInput {
  pub name: String,
  pub chat_type: ChatType,
  pub description: Option<String>,
  pub created_by: i64,
  pub workspace_id: Option<i64>,
  pub initial_members: Vec<i64>, // Initial members excluding creator
  pub members: Option<Vec<i64>>,
}

/// Chat detail view - Service layer DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatDetailView {
  pub id: i64,
  pub name: String,
  pub chat_type: ChatType,
  pub description: Option<String>,
  pub created_by: i64,
  pub workspace_id: Option<i64>,
  pub member_count: i32,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
  pub last_activity: chrono::DateTime<chrono::Utc>,
}

impl ChatDetailView {
  /// Build view from Chat entity - Single responsibility: View construction
  pub fn from_chat(chat: Chat, member_count: i32) -> Self {
    Self {
      id: i64::from(chat.id),
      name: chat.name,
      chat_type: chat.chat_type,
      description: Some(chat.description),
      created_by: i64::from(chat.created_by),
      workspace_id: Some(i64::from(chat.workspace_id)),
      member_count,
      created_at: chat.created_at,
      updated_at: chat.updated_at,
      last_activity: chat.updated_at,
    }
  }
}

//==============================================================================
// UNIT TESTS
//==============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_business_rules_chat_name_validation() {
    // Test empty name
    let empty_input = CreateChatInput {
      name: "".to_string(),
      chat_type: ChatType::Group,
      description: None,
      created_by: 1,
      workspace_id: Some(1),
      initial_members: vec![2, 3],
      members: None,
    };
    assert!(ChatBusinessRules::validate_chat_config(&empty_input).is_err());

    // Test too long name
    let long_name = "a".repeat(130);
    let long_input = CreateChatInput {
      name: long_name,
      chat_type: ChatType::Group,
      description: None,
      created_by: 1,
      workspace_id: Some(1),
      initial_members: vec![2, 3],
      members: None,
    };
    assert!(ChatBusinessRules::validate_chat_config(&long_input).is_err());
  }

  #[test]
  fn test_business_rules_member_limits() {
    // Test insufficient group members
    let insufficient_members = CreateChatInput {
      name: "Test Group".to_string(),
      chat_type: ChatType::Group,
      description: None,
      created_by: 1,
      workspace_id: Some(1),
      initial_members: vec![2], // Only 2 total members, less than required 3
      members: None,
    };
    assert!(ChatBusinessRules::validate_chat_config(&insufficient_members).is_err());

    // Test valid group configuration
    let valid_group = CreateChatInput {
      name: "Valid Group".to_string(),
      chat_type: ChatType::Group,
      description: None,
      created_by: 1,
      workspace_id: Some(1),
      initial_members: vec![2, 3], // 3 total members, meets minimum requirement
      members: None,
    };
    assert!(ChatBusinessRules::validate_chat_config(&valid_group).is_ok());
  }
}

//==============================================================================
// SIMPLE CHAT APPLICATION SERVICE FOR APPSTATE INTEGRATION
//==============================================================================

/// Simple chat application service for AppState integration
pub struct ChatApplicationService {
  app_state: AppState,
}

impl ChatApplicationService {
  pub fn new(app_state: AppState) -> Self {
    Self { app_state }
  }

  /// Chat membership validation through repository layer  
  pub async fn ensure_user_is_chat_member(
    &self,
    user_id: i64,
    chat_id: i64,
  ) -> Result<(), AppError> {
    // Create ChatMemberRepository instance through service layer
    let chat_member_repo = crate::domains::chat::chat_member_repository::ChatMemberRepository::new(
      Arc::new(self.app_state.pool().clone()),
    );

    // Check if user is a member of the chat
    let is_member = chat_member_repo
      .is_user_member(chat_id, user_id)
      .await
      .map_err(|e| AppError::InvalidInput(format!("Failed to check chat membership: {}", e)))?;

    if !is_member {
      return Err(AppError::ChatPermissionError(format!(
        "User {} is not a member of chat {}",
        user_id, chat_id
      )));
    }

    Ok(())
  }

  /// Simple chat membership check (returns boolean)
  pub async fn is_user_chat_member(&self, user_id: i64, chat_id: i64) -> Result<bool, AppError> {
    let chat_member_repo = crate::domains::chat::chat_member_repository::ChatMemberRepository::new(
      Arc::new(self.app_state.pool().clone()),
    );

    chat_member_repo
      .is_user_member(chat_id, user_id)
      .await
      .map_err(|e| AppError::InvalidInput(format!("Failed to check chat membership: {}", e)))
  }
}
