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
use crate::domains::chat::chat_domain::ChatDomainService;
use crate::services::application::cache::CacheStrategyService;
use crate::services::infrastructure::flows::{ChatDomainEvent, DomainEvent};
use async_trait::async_trait;
use chrono;
use fechatter_core::{
  contracts::ChatRepository,
  error::CoreError,
  models::{Chat, ChatId, ChatSidebar, ChatType, CreateChat, UpdateChat, UserId},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{info, instrument, warn};

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

  /// Use case: Check if user is in chat - Access control
  async fn is_user_in_chat(&self, user_id: i64, chat_id: ChatId) -> Result<bool, AppError>;

  /// Use case: Check if user is chat admin - Permission check
  async fn is_chat_admin(&self, user_id: i64, chat_id: ChatId) -> Result<bool, AppError>;

  /// Use case: Check if user is chat creator - Permission check
  async fn is_chat_creator(&self, user_id: i64, chat_id: ChatId) -> Result<bool, AppError>;

  /// Chat membership validation - Returns error if user is not a member
  async fn ensure_user_is_chat_member(&self, user_id: i64, chat_id: i64) -> Result<(), AppError>;

  /// Simple chat membership check (returns boolean)
  async fn is_user_chat_member(&self, user_id: i64, chat_id: i64) -> Result<bool, AppError>;
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
  pub fn validate_chat_config(input: &CreateChatInput) -> Result<(), CoreError> {
    Self::validate_chat_name(&input.name)?;
    Self::validate_member_limits(input.chat_type, &input.initial_members)?;
    Ok(())
  }

  /// Validate chat name - Single responsibility: Name validation
  fn validate_chat_name(name: &str) -> Result<(), CoreError> {
    if name.trim().is_empty() {
      return Err(CoreError::Validation(
        "Chat name cannot be empty".to_string(),
      ));
    }

    if name.len() > Self::MAX_CHAT_NAME_LENGTH {
      return Err(CoreError::Validation(format!(
        "Chat name cannot exceed {} characters",
        Self::MAX_CHAT_NAME_LENGTH
      )));
    }

    Ok(())
  }

  /// Validate member limits - Single responsibility: Member limit validation
  fn validate_member_limits(chat_type: ChatType, initial_members: &[i64]) -> Result<(), CoreError> {
    let total_members = 1 + initial_members.len(); // +1 for creator

    // Add debug logging to help troubleshoot
    tracing::debug!(
      "Validating member limits: chat_type={:?}, initial_members_count={}, total_members={}",
      chat_type,
      initial_members.len(),
      total_members
    );

    match chat_type {
      ChatType::Single => {
        if total_members != 2 {
          return Err(CoreError::Validation(format!(
            "Invalid single chat members: expected exactly 2 members (1 creator + 1 other), got {} members (creator + {} others)",
            total_members,
            initial_members.len()
          )));
        }
      }
      ChatType::Group => {
        if total_members < Self::MIN_GROUP_MEMBERS {
          return Err(CoreError::Validation(format!(
            "Group chat must have at least three members: expected >= {}, got {} members (creator + {} others)",
            Self::MIN_GROUP_MEMBERS,
            total_members,
            initial_members.len()
          )));
        }
        if total_members > Self::MAX_GROUP_MEMBERS {
          return Err(CoreError::Validation(format!(
            "Member limit exceeded: maximum {} members allowed, got {} members",
            Self::MAX_GROUP_MEMBERS,
            total_members
          )));
        }
      }
      ChatType::PrivateChannel | ChatType::PublicChannel => {
        if total_members > Self::MAX_CHANNEL_MEMBERS {
          return Err(CoreError::Validation(format!(
            "Channel member limit exceeded: maximum {} members allowed, got {} members",
            Self::MAX_CHANNEL_MEMBERS,
            total_members
          )));
        }
      }
    }

    tracing::debug!(
      "Member limits validation passed for chat_type={:?}",
      chat_type
    );
    Ok(())
  }

  /// Validate member operation permission - Single responsibility: Permission validation
  pub fn validate_member_operation_permission(
    _operator_id: i64,
    _chat_id: i64,
    _operation: MemberOperation,
  ) -> Result<(), CoreError> {
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
  pool: Arc<PgPool>,
  cache_strategy: Arc<CacheStrategyService>,
}

impl ChatService {
  /// Constructor - Dependency injection
  pub fn new(pool: Arc<PgPool>, cache_strategy: Arc<CacheStrategyService>) -> Self {
    Self {
      pool,
      cache_strategy,
    }
  }

  /// Create from AppState (for handler integration)
  pub fn new_from_app_state(app_state: AppState) -> Self {
    let pool = app_state.pool().clone();
    Self {
      pool,
      cache_strategy: Arc::new(CacheStrategyService::new_noop()),
    }
  }

  /// Create directly with pool dependency (for direct integration)
  pub fn new_with_pool(pool: Arc<PgPool>) -> Self {
    Self {
      pool,
      cache_strategy: Arc::new(CacheStrategyService::new_noop()),
    }
  }
}

#[async_trait]
impl ChatServiceTrait for ChatService {
  /// Use case: Create chat - Single responsibility: Chat creation use case orchestration
  #[instrument(skip(self, input), fields(chat_name = %input.name, chat_type = ?input.chat_type))]
  async fn create_chat(&self, input: CreateChatInput) -> Result<ChatDetailView, AppError> {
    // 1. 业务规则校验
    ChatBusinessRules::validate_chat_config(&input)?;

    // 2. 转换为核心层数据结构
    let create_data = self.build_create_chat_data(&input)?;

    // 3. 通过仓库创建聊天 - 使用convenience方法来正确传递参数
    let repository = crate::domains::chat::repository::ChatRepository::new(self.pool.clone());
    let chat = repository
      .create_chat(create_data, input.created_by, input.workspace_id)
      .await?;

    // 4. 构建返回视图
    let detail_view = ChatDetailView::from_chat(chat, input.initial_members.len() as i32 + 1);

    // 5. 发布领域事件（异步缓存失效和通知）
    let event = DomainEvent::Chat(ChatDomainEvent::ChatCreated {
      chat_id: detail_view.id,
      creator_id: input.created_by,
      chat_name: detail_view.name.clone(),
      chat_type: format!("{:?}", input.chat_type),
      workspace_id: input.workspace_id.unwrap_or(0),
      initial_members: input.initial_members.clone(),
      created_at: chrono::Utc::now().to_rfc3339(),
      privacy_settings: serde_json::Value::Null, // TODO: Add privacy settings
      sig: None,
    });

    if let Err(e) = self.publish_domain_event(event).await {
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
    let repository = crate::domains::chat::repository::ChatRepository::new(self.pool.clone());
    let chat = match repository.find_by_id(ChatId::new(id)).await? {
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
    let repository = crate::domains::chat::repository::ChatRepository::new(self.pool.clone());
    let chat_list: Vec<ChatSidebar> = repository.get_user_chats(UserId::new(user_id)).await?;

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
    // 1. 权限和业务验证
    if let Some(name) = &payload.name {
      ChatBusinessRules::validate_chat_name(name)?;
    }

    // 2. 通过仓库更新
    let repository = crate::domains::chat::repository::ChatRepository::new(self.pool.clone());
    let updated_chat = repository
      .update(chat_id, &payload)
      .await
      .map_err(AppError::from)?;

    // 3. 构建视图和缓存
    let detail_view = ChatDetailView::from_chat(updated_chat, {
      // count members for view
      let repo =
        crate::domains::chat::chat_member_repository::ChatMemberRepository::new(self.pool.clone());
      repo
        .get_member_count(chat_id.0)
        .await
        .map_err(AppError::from)? as i32
    });
    let key = CacheStrategyService::chat_detail_key(chat_id.0);
    let _ = self
      .cache_strategy
      .set(&key, &detail_view, CacheStrategyService::CHAT_DETAIL_TTL)
      .await;

    // 4. 发布事件
    let event = DomainEvent::Chat(ChatDomainEvent::ChatUpdated {
      chat_id: chat_id.0,
      updated_by: user_id.0,
      chat_name: detail_view.name.clone(),
      updated_at: chrono::Utc::now().to_rfc3339(),
      sig: None,
    });
    let _ = self.publish_domain_event(event).await;

    Ok(detail_view)
  }

  /// Use case: Delete chat - Chat removal
  async fn delete_chat(&self, chat_id: ChatId, user_id: UserId) -> Result<bool, AppError> {
    // 1. 通过仓库删除
    let repository = crate::domains::chat::repository::ChatRepository::new(self.pool.clone());
    let deleted = repository.delete(chat_id).await.map_err(AppError::from)?;
    // 2. 使缓存失效
    let key = CacheStrategyService::chat_detail_key(chat_id.0);
    let _ = self.cache_strategy.del(&key).await;
    // 3. 发布事件
    if deleted {
      let event = DomainEvent::Chat(ChatDomainEvent::ChatDeleted {
        chat_id: chat_id.0,
        deleted_by: user_id.0,
        deleted_at: chrono::Utc::now().to_rfc3339(),
        sig: None,
      });
      let _ = self.publish_domain_event(event).await;
    }
    Ok(deleted)
  }

  /// Use case: List chat members - Get members of a specific chat
  async fn list_chat_members(
    &self,
    chat_id: i64,
  ) -> Result<Vec<fechatter_core::models::ChatMember>, AppError> {
    let member_repo =
      crate::domains::chat::chat_member_repository::ChatMemberRepository::new(self.pool.clone());
    let members = member_repo
      .list_members(chat_id)
      .await
      .map_err(AppError::from)?;
    Ok(members)
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

    // 1. 验证操作权限
    if !self.is_chat_admin(user_id, ChatId::new(chat_id)).await? {
      return Err(CoreError::Unauthorized("Only chat admin can add members".to_string()).into());
    }

    // 2. 执行添加操作 (TODO: 实现实际的添加逻辑)
    let member_repo =
      crate::domains::chat::chat_member_repository::ChatMemberRepository::new(self.pool.clone());
    member_repo
      .add_members(chat_id, &member_ids)
      .await
      .map_err(AppError::from)?;

    // 3. 为每个成员发布领域事件
    for member_id in &member_ids {
      let event = DomainEvent::Chat(ChatDomainEvent::ChatMemberAdded {
        chat_id,
        added_by: user_id,
        new_member_id: *member_id,
        added_at: chrono::Utc::now().to_rfc3339(),
        invitation_context: None,
        sig: None,
      });

      if let Err(e) = self.publish_domain_event(event).await {
        warn!(
          "Failed to publish member added event for {}: {}",
          member_id, e
        );
      }
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

    // 1. 验证操作权限
    if !self.is_chat_admin(user_id, ChatId::new(chat_id)).await? {
      return Err(CoreError::Unauthorized("Only chat admin can remove members".to_string()).into());
    }

    // 2. 执行移除操作
    let member_repo =
      crate::domains::chat::chat_member_repository::ChatMemberRepository::new(self.pool.clone());
    member_repo
      .remove_members(chat_id, &member_ids)
      .await
      .map_err(AppError::from)?;

    // 3. 为每个被移除的成员发布领域事件
    for member_id in &member_ids {
      let event = DomainEvent::Chat(ChatDomainEvent::ChatMemberRemoved {
        chat_id,
        removed_by: user_id,
        removed_member_id: *member_id,
        removed_at: chrono::Utc::now().to_rfc3339(),
        removal_reason: None,
        sig: None,
      });

      if let Err(e) = self.publish_domain_event(event).await {
        warn!(
          "Failed to publish member removed event for {}: {}",
          member_id, e
        );
      }
    }

    info!(chat_id = %chat_id, removed_members = ?member_ids, "Members removed successfully");
    Ok(())
  }

  /// Use case: Transfer chat ownership - Change chat owner
  #[instrument(skip(self))]
  async fn transfer_chat_ownership(
    &self,
    chat_id: i64,
    current_owner_id: i64,
    new_owner_id: i64,
  ) -> Result<bool, AppError> {
    // Create domain service with necessary dependencies
    let chat_repo = Arc::new(crate::domains::chat::repository::ChatRepository::new(
      self.pool.clone(),
    ));
    let member_repo = Arc::new(
      crate::domains::chat::chat_member_repository::ChatMemberRepository::new(self.pool.clone()),
    );
    let event_publisher = Arc::new(
      crate::services::infrastructure::flows::SimplifiedEventPublisher::new(
        self.cache_strategy.clone(),
      ),
    );
    let domain_service = crate::domains::chat::chat_domain::ChatDomainServiceImpl::new(
      chat_repo,
      member_repo,
      event_publisher,
      crate::domains::chat::chat_domain::ChatConfig::default(),
    );

    // Use domain service for business logic and validation
    domain_service
      .transfer_ownership(chat_id, current_owner_id, new_owner_id)
      .await
      .map_err(AppError::from)?;

    tracing::info!(
      "Chat {} ownership transferred from user {} to user {}",
      chat_id,
      current_owner_id,
      new_owner_id
    );

    Ok(true)
  }

  /// Use case: Check if user is in chat - Access control
  async fn is_user_in_chat(&self, user_id: i64, chat_id: ChatId) -> Result<bool, AppError> {
    self.is_user_chat_member(user_id, chat_id.0).await
  }

  /// Use case: Check if user is chat admin - Permission check
  async fn is_chat_admin(&self, user_id: i64, chat_id: ChatId) -> Result<bool, AppError> {
    // First check if user is in chat
    if !self.is_user_chat_member(user_id, chat_id.0).await? {
      return Ok(false);
    }

    // Create chat repository to check creator status
    let chat_repo = crate::domains::chat::repository::ChatRepository::new(self.pool.clone());

    // For now, admin check means creator check (admin role not implemented yet)
    match chat_repo.find_chat_by_id(chat_id.0).await {
      Ok(Some(chat)) => Ok(chat.created_by.0 == user_id),
      Ok(None) => Ok(false),
      Err(e) => {
        warn!("Failed to check chat admin status: {}", e);
        Ok(false) // Fail gracefully for admin check
      }
    }
  }

  /// Use case: Check if user is chat creator - Permission check
  async fn is_chat_creator(&self, user_id: i64, chat_id: ChatId) -> Result<bool, AppError> {
    let chat_repo = crate::domains::chat::repository::ChatRepository::new(self.pool.clone());

    match chat_repo.find_chat_by_id(chat_id.0).await {
      Ok(Some(chat)) => Ok(chat.created_by.0 == user_id),
      Ok(None) => Ok(false),
      Err(e) => {
        warn!("Failed to check chat creator status: {}", e);
        Ok(false) // Fail gracefully for creator check
      }
    }
  }

  /// Chat membership validation through repository layer  
  async fn ensure_user_is_chat_member(&self, user_id: i64, chat_id: i64) -> Result<(), AppError> {
    // Create ChatMemberRepository instance through service layer
    let chat_member_repo =
      crate::domains::chat::chat_member_repository::ChatMemberRepository::new(self.pool.clone());

    // Check if user is a member of the chat
    let is_member = chat_member_repo
      .is_user_member(chat_id, user_id)
      .await
      .map_err(|e| -> AppError {
        CoreError::Database(format!("Failed to check chat membership: {}", e)).into()
      })?;

    if !is_member {
      return Err(
        CoreError::Unauthorized(format!(
          "User {} is not a member of chat {}",
          user_id, chat_id
        ))
        .into(),
      );
    }

    Ok(())
  }

  /// Simple chat membership check (returns boolean)
  async fn is_user_chat_member(&self, user_id: i64, chat_id: i64) -> Result<bool, AppError> {
    let chat_member_repo =
      crate::domains::chat::chat_member_repository::ChatMemberRepository::new(self.pool.clone());

    chat_member_repo
      .is_user_member(chat_id, user_id)
      .await
      .map_err(|e| -> AppError {
        CoreError::Database(format!("Failed to check chat membership: {}", e)).into()
      })
  }
}

//==============================================================================
// PRIVATE HELPER METHODS - Single Responsibility
//==============================================================================

impl ChatService {
  /// Build create chat data - Single responsibility: Data conversion
  fn build_create_chat_data(&self, input: &CreateChatInput) -> Result<CreateChat, CoreError> {
    Ok(CreateChat {
      name: input.name.clone(),
      chat_type: input.chat_type,
      description: input.description.clone(),
      members: Some(
        input
          .initial_members
          .iter()
          .map(|&id| UserId::from(id))
          .collect(),
      ),
    })
  }

  /// Get chat details from cache - Single responsibility: Cache query
  async fn get_chat_from_cache(&self, chat_id: i64) -> Result<Option<ChatDetailView>, AppError> {
    let cache_key = CacheStrategyService::chat_detail_key(chat_id);

    match self.cache_strategy.get::<ChatDetailView>(&cache_key).await {
      Ok(Some(cached_data)) => Ok(Some(cached_data)),
      Ok(None) => Err(CoreError::NotFound("Cache miss".to_string()).into()), // Use error to indicate cache miss
      Err(e) => {
        warn!("Cache error for chat {}: {}", chat_id, e);
        Err(CoreError::Internal(e.to_string()).into())
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
      Ok(None) => Err(CoreError::NotFound("Cache miss".to_string()).into()), // Use error to indicate cache miss
      Err(e) => {
        warn!("Cache error for user chats {}: {}", user_id, e);
        Err(CoreError::Internal(e.to_string()).into())
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

  /// Publish domain event - 私有辅助方法
  async fn publish_domain_event(&self, _event: DomainEvent) -> Result<(), AppError> {
    // TODO: 实现实际的事件发布逻辑
    // 暂时移除事件发布，后续可以通过依赖注入重新添加
    Ok(())
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
  pool: Arc<PgPool>,
  cache_strategy: Arc<CacheStrategyService>,
}

impl ChatApplicationService {
  /// Create from AppState (legacy compatibility)
  pub fn new(app_state: AppState) -> Self {
    Self {
      pool: app_state.pool(),
      cache_strategy: Arc::new(CacheStrategyService::new_optional(
        app_state.cache_service().map(|c| c.clone()),
      )),
    }
  }

  /// Create directly with pool dependency - Concrete service pattern
  pub fn new_with_pool(pool: Arc<PgPool>) -> Self {
    Self {
      pool,
      cache_strategy: Arc::new(CacheStrategyService::new_noop()),
    }
  }

  /// Create chat - Delegate to ChatService  
  pub async fn create_chat(&self, input: CreateChatInput) -> Result<ChatDetailView, AppError> {
    let chat_service = ChatService::new(self.pool.clone(), self.cache_strategy.clone());
    chat_service.create_chat(input).await
  }

  /// Chat membership validation through repository layer
  pub async fn ensure_user_is_chat_member(
    &self,
    user_id: i64,
    chat_id: i64,
  ) -> Result<(), AppError> {
    // Create ChatMemberRepository instance through service layer
    let chat_member_repo =
      crate::domains::chat::chat_member_repository::ChatMemberRepository::new(self.pool.clone());

    // Check if user is a member of the chat
    let is_member = chat_member_repo
      .is_user_member(chat_id, user_id)
      .await
      .map_err(|e| -> AppError {
        CoreError::Database(format!("Failed to check chat membership: {}", e)).into()
      })?;

    if !is_member {
      return Err(
        CoreError::Unauthorized(format!(
          "User {} is not a member of chat {}",
          user_id, chat_id
        ))
        .into(),
      );
    }

    Ok(())
  }

  /// Simple chat membership check (returns boolean)
  pub async fn is_user_chat_member(&self, user_id: i64, chat_id: i64) -> Result<bool, AppError> {
    let chat_member_repo =
      crate::domains::chat::chat_member_repository::ChatMemberRepository::new(self.pool.clone());

    chat_member_repo
      .is_user_member(chat_id, user_id)
      .await
      .map_err(|e| -> AppError {
        CoreError::Database(format!("Failed to check chat membership: {}", e)).into()
      })
  }

  /// Check if user is chat admin - Permission check for middleware
  pub async fn is_chat_admin(&self, user_id: i64, chat_id: i64) -> Result<bool, AppError> {
    // First check if user is in chat
    if !self.is_user_chat_member(user_id, chat_id).await? {
      return Ok(false);
    }

    // Create chat repository to check creator status
    let chat_repo = crate::domains::chat::repository::ChatRepository::new(self.pool.clone());

    // For now, admin check means creator check (admin role not implemented yet)
    match chat_repo.find_chat_by_id(chat_id).await {
      Ok(Some(chat)) => Ok(chat.created_by.0 == user_id),
      Ok(None) => Ok(false),
      Err(e) => {
        warn!("Failed to check chat admin status: {}", e);
        Ok(false) // Fail gracefully for admin check
      }
    }
  }

  /// Check if user is chat creator - Permission check for middleware
  pub async fn is_chat_creator(&self, user_id: i64, chat_id: i64) -> Result<bool, AppError> {
    let chat_repo = crate::domains::chat::repository::ChatRepository::new(self.pool.clone());

    match chat_repo.find_chat_by_id(chat_id).await {
      Ok(Some(chat)) => Ok(chat.created_by.0 == user_id),
      Ok(None) => Ok(false),
      Err(e) => {
        warn!("Failed to check chat creator status: {}", e);
        Ok(false) // Fail gracefully for creator check
      }
    }
  }

  /// Add members to chat - For handlers
  pub async fn add_members(
    &self,
    chat_id: i64,
    user_id: i64,
    member_ids: Vec<i64>,
  ) -> Result<(), AppError> {
    // TODO: Implement proper add members logic
    // For now, just validate that the user has permission and return success
    if !self.is_chat_admin(user_id, chat_id).await? {
      return Err(CoreError::Unauthorized("Only chat admin can add members".to_string()).into());
    }

    // Log the operation for now
    tracing::info!(
      "User {} adding members {:?} to chat {}",
      user_id,
      member_ids,
      chat_id
    );
    Ok(())
  }

  /// Remove members from chat - For handlers
  pub async fn remove_members(
    &self,
    chat_id: i64,
    user_id: i64,
    member_ids: Vec<i64>,
  ) -> Result<(), AppError> {
    // TODO: Implement proper remove members logic
    // For now, just validate that the user has permission and return success
    if !self.is_chat_admin(user_id, chat_id).await? {
      return Err(CoreError::Unauthorized("Only chat admin can remove members".to_string()).into());
    }

    // Log the operation for now
    tracing::info!(
      "User {} removing members {:?} from chat {}",
      user_id,
      member_ids,
      chat_id
    );
    Ok(())
  }

  /// List user chats - For handlers
  pub async fn list_user_chats(
    &self,
    user_id: i64,
  ) -> Result<Vec<fechatter_core::models::ChatSidebar>, AppError> {
    let chat_repo = crate::domains::chat::repository::ChatRepository::new(self.pool.clone());

    match chat_repo.get_sidebar_chats(user_id).await {
      Ok(chats) => Ok(chats),
      Err(e) => {
        warn!("Failed to get user chats: {}", e);
        Err(CoreError::Internal("Failed to fetch user chats".to_string()).into())
      }
    }
  }

  /// Transfer chat ownership - For handlers
  pub async fn transfer_chat_ownership(
    &self,
    chat_id: i64,
    current_owner_id: i64,
    new_owner_id: i64,
  ) -> Result<bool, AppError> {
    // Create domain service with necessary dependencies
    let chat_repo = std::sync::Arc::new(crate::domains::chat::repository::ChatRepository::new(
      self.pool.clone(),
    ));
    let member_repo = std::sync::Arc::new(
      crate::domains::chat::chat_member_repository::ChatMemberRepository::new(self.pool.clone()),
    );
    let event_publisher = std::sync::Arc::new(
      crate::services::infrastructure::flows::SimplifiedEventPublisher::new(
        self.cache_strategy.clone(),
      ),
    );
    let domain_service = crate::domains::chat::chat_domain::ChatDomainServiceImpl::new(
      chat_repo,
      member_repo,
      event_publisher,
      crate::domains::chat::chat_domain::ChatConfig::default(),
    );

    // Use domain service for business logic and validation
    domain_service
      .transfer_ownership(chat_id, current_owner_id, new_owner_id)
      .await
      .map_err(AppError::from)?;

    tracing::info!(
      "Chat {} ownership transferred from user {} to user {}",
      chat_id,
      current_owner_id,
      new_owner_id
    );

    Ok(true)
  }

  /// Delete chat - For handlers
  pub async fn delete_chat(&self, chat_id: i64, user_id: i64) -> Result<bool, AppError> {
    // 1. 验证权限 - 只有创建者才能删除聊天
    if !self.is_chat_creator(user_id, chat_id).await? {
      return Err(CoreError::Unauthorized("Only chat creator can delete chat".to_string()).into());
    }

    // 2. 使用Domain Service进行业务逻辑处理
    let chat_repo = std::sync::Arc::new(crate::domains::chat::repository::ChatRepository::new(
      self.pool.clone(),
    ));
    let member_repo = std::sync::Arc::new(
      crate::domains::chat::chat_member_repository::ChatMemberRepository::new(self.pool.clone()),
    );
    let event_publisher = std::sync::Arc::new(
      crate::services::infrastructure::flows::SimplifiedEventPublisher::new(
        self.cache_strategy.clone(),
      ),
    );
    let domain_service = crate::domains::chat::chat_domain::ChatDomainServiceImpl::new(
      chat_repo,
      member_repo,
      event_publisher,
      crate::domains::chat::chat_domain::ChatConfig::default(),
    );

    // 3. 执行删除操作
    let deleted = domain_service
      .delete_chat(chat_id, user_id)
      .await
      .map_err(AppError::from)?;

    // 4. 记录操作日志
    if deleted {
      tracing::info!("Chat {} deleted by user {}", chat_id, user_id);
    }

    Ok(deleted)
  }

  /// Get chat details - For handlers
  pub async fn get_chat(
    &self,
    chat_id: i64,
    user_id: i64,
  ) -> Result<Option<ChatDetailView>, AppError> {
    // 1. 验证权限 - 只有聊天成员才能查看详情
    if !self.is_user_chat_member(user_id, chat_id).await? {
      return Err(
        CoreError::Unauthorized("Only chat members can view chat details".to_string()).into(),
      );
    }

    // 2. 使用ChatService获取聊天详情
    let chat_service = ChatService::new_with_pool(self.pool.clone());
    let chat_detail = chat_service.get_chat(chat_id).await?;

    // 3. 记录操作日志
    if chat_detail.is_some() {
      tracing::info!("Chat {} details viewed by user {}", chat_id, user_id);
    }

    Ok(chat_detail)
  }
}
