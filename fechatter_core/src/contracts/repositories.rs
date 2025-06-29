use crate::{
  error::CoreError,
  models::{Chat, ChatId, CreateUser, Message, MessageId, SigninUser, User, UserId},
};
use async_trait::async_trait;
use std::any::Any;

/// User repository interface contract
#[async_trait]
pub trait UserRepository: Send + Sync {
  /// Allow downcasting to concrete types
  fn as_any(&self) -> &dyn Any;
  /// Create new user
  async fn create(&self, user_data: &CreateUser) -> Result<User, CoreError>;

  /// Find user by ID
  async fn find_by_id(&self, id: UserId) -> Result<Option<User>, CoreError>;

  /// Find user by email
  async fn find_by_email(&self, email: &str) -> Result<Option<User>, CoreError>;

  /// Authenticate user credentials
  async fn authenticate(&self, signin_data: &SigninUser) -> Result<Option<User>, CoreError>;

  /// Update user information
  async fn update(&self, id: UserId, user_data: &User) -> Result<User, CoreError>;

  /// Check if user exists
  async fn exists_by_email(&self, email: &str) -> Result<bool, CoreError>;
}

/// Chat repository interface contract
#[async_trait]
pub trait ChatRepository: Send + Sync {
  /// Create chat
  async fn create(&self, chat_data: &crate::models::CreateChat) -> Result<Chat, CoreError>;

  /// Find chat by ID
  async fn find_by_id(&self, id: ChatId) -> Result<Option<Chat>, CoreError>;

  /// Get user chat list
  async fn get_user_chats(
    &self,
    user_id: UserId,
  ) -> Result<Vec<crate::models::ChatSidebar>, CoreError>;

  /// Update chat information
  async fn update(
    &self,
    id: ChatId,
    chat_data: &crate::models::UpdateChat,
  ) -> Result<Chat, CoreError>;

  /// Delete chat
  async fn delete(&self, id: ChatId) -> Result<bool, CoreError>;
}

/// Message repository interface contract
#[async_trait]
pub trait MessageRepository: Send + Sync {
  /// Create message
  async fn create(&self, message_data: &crate::models::CreateMessage)
    -> Result<Message, CoreError>;

  /// Find message by ID
  async fn find_by_id(&self, id: MessageId) -> Result<Option<Message>, CoreError>;

  /// Get chat message list
  async fn get_chat_messages(
    &self,
    chat_id: ChatId,
    params: &crate::models::ListMessages,
  ) -> Result<Vec<Message>, CoreError>;

  /// Update message
  async fn update(&self, id: MessageId, content: &str) -> Result<Message, CoreError>;

  /// Delete message
  async fn delete(&self, id: MessageId) -> Result<bool, CoreError>;
}
