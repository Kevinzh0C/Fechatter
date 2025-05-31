//! # Application Service Adapters - Pure Interface Translation
//!
//! ## Correct Adapter Pattern Implementation
//!
//! **Core Principle**: Pure interface translation, NO business logic
//! **Responsibility**: Map existing AppState methods to Service interfaces
//! **Pattern**: Method Mapping + Type Conversion + Error Translation
//!
//! **Anti-Pattern**: Fake data, TODO calls, business logic

use crate::AppError;
use crate::AppState;
use crate::error::map_core_error_to_app_error;
use async_trait::async_trait;
use fechatter_core::{ChatId, UserId, error::CoreError};

// Import necessary service traits and types
use super::chat_app_service::{ChatDetailView, ChatServiceTrait, CreateChatInput};
use super::user_app_service::{
  ChangePasswordInput, UpdateUserInput, UserProfileView, UserServiceTrait,
};
use fechatter_core::UserStatus;
use fechatter_core::models::{ChatSidebar, UpdateChat};

// ============================================================================
// CHAT SERVICE ADAPTER - Pure Interface Translation
// ============================================================================

/// Pure Chat Service Adapter - Maps AppState methods to ChatServiceTrait
///
/// **PRINCIPLE**: Only method mapping and type conversion, NO business logic
#[derive(Clone)]
pub struct AppStateChatServiceAdapter {
  state: AppState,
}

impl AppStateChatServiceAdapter {
  pub fn new(state: AppState) -> Self {
    Self { state }
  }
}

#[async_trait]
impl ChatServiceTrait for AppStateChatServiceAdapter {
  /// INTERFACE MAPPING: CreateChatInput → AppState (when implemented)
  async fn create_chat(&self, _input: CreateChatInput) -> Result<ChatDetailView, AppError> {
    // ADAPTER PRINCIPLE: Return error if underlying method doesn't exist
    Err(AppError::Internal(
      "create_chat not yet implemented in AppState - use domain service directly".to_string(),
    ))
  }

  /// INTERFACE MAPPING: id → AppState (when implemented)  
  async fn get_chat(&self, _id: i64) -> Result<Option<ChatDetailView>, AppError> {
    // ADAPTER PRINCIPLE: Return error if underlying method doesn't exist
    Err(AppError::Internal(
      "get_chat not yet implemented in AppState - use domain service directly".to_string(),
    ))
  }

  /// INTERFACE MAPPING: user_id → AppState (when implemented)
  async fn list_user_chats(&self, _user_id: i64) -> Result<Vec<ChatSidebar>, AppError> {
    // ADAPTER PRINCIPLE: Return error if underlying method doesn't exist
    Err(AppError::Internal(
      "list_user_chats not yet implemented in AppState - use domain service directly".to_string(),
    ))
  }

  /// INTERFACE MAPPING: UpdateChat → AppState (when implemented)
  async fn update_chat(
    &self,
    _chat_id: ChatId,
    _user_id: UserId,
    _payload: UpdateChat,
  ) -> Result<ChatDetailView, AppError> {
    // ADAPTER PRINCIPLE: Return error if underlying method doesn't exist
    Err(AppError::Internal(
      "update_chat not yet implemented in AppState - use domain service directly".to_string(),
    ))
  }

  /// PURE INTERFACE MAPPING: Existing AppState method
  async fn delete_chat(&self, chat_id: ChatId, user_id: UserId) -> Result<bool, AppError> {
    // CORRECT MAPPING: Use existing AppState method with proper error conversion
    self
      .state
      .delete_chat_by_id(chat_id, user_id)
      .await
      .map_err(map_core_error_to_app_error)
  }

  /// INTERFACE MAPPING: Not implemented in AppState
  async fn list_chat_members(
    &self,
    _chat_id: i64,
  ) -> Result<Vec<fechatter_core::models::ChatMember>, AppError> {
    Err(AppError::Internal(
      "list_chat_members not yet implemented in AppState - use domain service directly".to_string(),
    ))
  }

  /// PURE INTERFACE MAPPING: Existing AppState method  
  async fn add_members(
    &self,
    chat_id: i64,
    user_id: i64,
    member_ids: Vec<i64>,
  ) -> Result<(), AppError> {
    // TYPE CONVERSION: i64 → TypedId
    let chat_id = ChatId(chat_id);
    let user_id = UserId(user_id);
    let member_user_ids: Vec<UserId> = member_ids.into_iter().map(UserId).collect();

    // INTERFACE MAPPING: Direct delegation to AppState
    self
      .state
      .add_chat_members(chat_id, user_id, member_user_ids)
      .await
      .map_err(map_core_error_to_app_error)
      .map(|_| ()) // Convert Vec<ChatMember> to ()
  }

  /// PURE INTERFACE MAPPING: Existing AppState method
  async fn remove_members(
    &self,
    chat_id: i64,
    user_id: i64,
    member_ids: Vec<i64>,
  ) -> Result<(), AppError> {
    // TYPE CONVERSION: i64 → TypedId
    let chat_id = ChatId(chat_id);
    let user_id = UserId(user_id);
    let member_user_ids: Vec<UserId> = member_ids.into_iter().map(UserId).collect();

    // INTERFACE MAPPING: Direct delegation to AppState
    self
      .state
      .remove_chat_members(chat_id, user_id, member_user_ids)
      .await
      .map_err(map_core_error_to_app_error)
      .map(|_| ()) // Convert bool to ()
  }

  /// PURE INTERFACE MAPPING: Existing AppState method
  async fn transfer_chat_ownership(
    &self,
    chat_id: i64,
    current_owner_id: i64,
    new_owner_id: i64,
  ) -> Result<bool, AppError> {
    // TYPE CONVERSION: i64 → TypedId
    let chat_id = ChatId(chat_id);
    let from_user_id = UserId(current_owner_id);
    let to_user_id = UserId(new_owner_id);

    // INTERFACE MAPPING: Direct delegation to AppState
    self
      .state
      .transfer_chat_ownership(chat_id, from_user_id, to_user_id)
      .await
      .map_err(map_core_error_to_app_error)
  }
}

// ============================================================================
// USER SERVICE ADAPTER - Pure Interface Translation
// ============================================================================

/// Pure User Service Adapter - Maps AppState methods to UserServiceTrait
#[derive(Clone)]
pub struct AppStateUserServiceAdapter {
  state: AppState,
}

impl AppStateUserServiceAdapter {
  pub fn new(state: AppState) -> Self {
    Self { state }
  }
}

#[async_trait]
impl UserServiceTrait for AppStateUserServiceAdapter {
  /// PURE INTERFACE MAPPING: Existing AppState method
  async fn get_user_profile(&self, user_id: i64) -> Result<UserProfileView, AppError> {
    // INTERFACE MAPPING: Use existing AppState method
    let user = self
      .state
      .find_user_by_id(user_id)
      .await?
      .ok_or_else(|| AppError::NotFound(vec![format!("User {} not found", user_id)]))?;

    // TYPE CONVERSION: fechatter_core::User → UserProfileView
    Ok(UserProfileView {
      id: i64::from(user.id),
      fullname: user.fullname,
      email: user.email,
      status: format!("{:?}", user.status),
      created_at: user.created_at,
      updated_at: user.created_at, // Use created_at as fallback
      workspace_id: Some(i64::from(user.workspace_id)),
    })
  }

  /// INTERFACE MAPPING: Not implemented in AppState
  async fn update_user_profile(
    &self,
    _user_id: i64,
    _input: UpdateUserInput,
  ) -> Result<UserProfileView, AppError> {
    Err(AppError::Internal(
      "update_user_profile not yet implemented in AppState - use domain service directly"
        .to_string(),
    ))
  }

  /// INTERFACE MAPPING: Not implemented in AppState
  async fn change_password(
    &self,
    _user_id: i64,
    _input: ChangePasswordInput,
  ) -> Result<(), AppError> {
    Err(AppError::Internal(
      "change_password not yet implemented in AppState - use domain service directly".to_string(),
    ))
  }

  /// INTERFACE MAPPING: Not implemented in AppState
  async fn update_user_status(&self, _user_id: i64, _status: UserStatus) -> Result<(), AppError> {
    Err(AppError::Internal(
      "update_user_status not yet implemented in AppState - use domain service directly"
        .to_string(),
    ))
  }

  /// INTERFACE MAPPING: Not implemented in AppState  
  async fn validate_users_exist(&self, _user_ids: Vec<i64>) -> Result<(), AppError> {
    Err(AppError::Internal(
      "validate_users_exist not yet implemented in AppState - use domain service directly"
        .to_string(),
    ))
  }

  /// INTERFACE MAPPING: Not implemented in AppState
  async fn find_users_by_workspace(
    &self,
    _workspace_id: i64,
  ) -> Result<Vec<UserProfileView>, AppError> {
    Err(AppError::Internal(
      "find_users_by_workspace not yet implemented in AppState - use domain service directly"
        .to_string(),
    ))
  }
}

// ============================================================================
// NOTIFICATION SERVICE ADAPTER - Pure Interface Translation
// ============================================================================

/// Notification Service Interface
#[async_trait]
pub trait NotificationServiceTrait: Send + Sync {
  async fn send_mention_notification(
    &self,
    mentioned_user_ids: Vec<i64>,
    mentioned_by_name: &str,
    message_preview: &str,
    chat_id: i64,
    message_id: i64,
  ) -> Result<Vec<i64>, AppError>;

  async fn send_direct_message_notification(
    &self,
    recipient_id: i64,
    sender_id: i64,
    sender_name: &str,
    message_preview: &str,
    chat_id: i64,
    message_id: i64,
  ) -> Result<Option<i64>, AppError>;

  async fn mark_notification_as_read(
    &self,
    notification_id: i64,
    user_id: i64,
  ) -> Result<(), AppError>;
}

/// Pure Notification Service Adapter
#[derive(Clone)]
pub struct AppStateNotificationServiceAdapter {
  state: AppState,
}

impl AppStateNotificationServiceAdapter {
  pub fn new(state: AppState) -> Self {
    Self { state }
  }
}

#[async_trait]
impl NotificationServiceTrait for AppStateNotificationServiceAdapter {
  /// INTERFACE MAPPING: Not implemented in AppState
  async fn send_mention_notification(
    &self,
    _mentioned_user_ids: Vec<i64>,
    _mentioned_by_name: &str,
    _message_preview: &str,
    _chat_id: i64,
    _message_id: i64,
  ) -> Result<Vec<i64>, AppError> {
    Err(AppError::Internal(
      "send_mention_notification not yet implemented in AppState".to_string(),
    ))
  }

  /// INTERFACE MAPPING: Not implemented in AppState
  async fn send_direct_message_notification(
    &self,
    _recipient_id: i64,
    _sender_id: i64,
    _sender_name: &str,
    _message_preview: &str,
    _chat_id: i64,
    _message_id: i64,
  ) -> Result<Option<i64>, AppError> {
    Err(AppError::Internal(
      "send_direct_message_notification not yet implemented in AppState".to_string(),
    ))
  }

  /// INTERFACE MAPPING: Not implemented in AppState
  async fn mark_notification_as_read(
    &self,
    _notification_id: i64,
    _user_id: i64,
  ) -> Result<(), AppError> {
    Err(AppError::Internal(
      "mark_notification_as_read not yet implemented in AppState".to_string(),
    ))
  }
}

// ============================================================================
// EXTENSION TRAITS - Pure Domain Delegation
// ============================================================================

/// Chat Membership Extensions - Pure domain delegation
#[async_trait]
pub trait ChatMembershipExtensions {
  async fn ensure_user_is_chat_member(&self, user_id: i64, chat_id: i64) -> Result<(), AppError>;
}

#[async_trait]
impl ChatMembershipExtensions for AppState {
  async fn ensure_user_is_chat_member(&self, user_id: i64, chat_id: i64) -> Result<(), AppError> {
    // PURE DELEGATION: No business logic, just repository access
    let chat_member_repo = crate::domains::chat::chat_member_repository::ChatMemberRepository::new(
      std::sync::Arc::new(self.pool().clone()),
    );

    chat_member_repo
      .ensure_user_is_chat_member(chat_id, user_id)
      .await
      .map_err(map_core_error_to_app_error)
  }
}

/// Workspace Extensions - Pure domain delegation
#[async_trait]
pub trait WorkspaceExtensions {
  async fn find_by_id_with_pool(
    &self,
    workspace_id: fechatter_core::WorkspaceId,
  ) -> Result<Option<fechatter_core::Workspace>, AppError>;

  async fn create_workspace_with_pool(
    &self,
    name: &str,
    owner_id: i64,
  ) -> Result<fechatter_core::Workspace, AppError>;
}

#[async_trait]
impl WorkspaceExtensions for AppState {
  async fn find_by_id_with_pool(
    &self,
    workspace_id: fechatter_core::WorkspaceId,
  ) -> Result<Option<fechatter_core::Workspace>, AppError> {
    // PURE DELEGATION: No business logic, just repository access
    let workspace_repo = crate::domains::workspace::repository::WorkspaceRepositoryImpl::new(
      std::sync::Arc::new(self.pool().clone()),
    );

    workspace_repo
      .find_by_id(workspace_id)
      .await
      .map_err(map_core_error_to_app_error)
  }

  async fn create_workspace_with_pool(
    &self,
    name: &str,
    owner_id: i64,
  ) -> Result<fechatter_core::Workspace, AppError> {
    // PURE DELEGATION: No business logic, just repository access
    let workspace_repo = crate::domains::workspace::repository::WorkspaceRepositoryImpl::new(
      std::sync::Arc::new(self.pool().clone()),
    );

    let workspace = workspace_repo
      .find_or_create_by_name(name)
      .await
      .map_err(map_core_error_to_app_error)?;

    // Update owner if needed
    if workspace.owner_id.0 == 0 {
      workspace_repo
        .update_owner(
          fechatter_core::WorkspaceId(workspace.id.0),
          fechatter_core::UserId(owner_id),
        )
        .await
        .map_err(map_core_error_to_app_error)
    } else {
      Ok(workspace)
    }
  }
}
