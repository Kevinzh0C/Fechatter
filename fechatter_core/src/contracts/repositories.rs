use crate::{
  error::CoreError,
  models::{Chat, ChatId, CreateUser, Message, MessageId, SigninUser, User, UserId},
};
use async_trait::async_trait;

/// 用户仓储接口契约
#[async_trait]
pub trait UserRepository: Send + Sync {
  /// 创建新用户
  async fn create(&self, user_data: &CreateUser) -> Result<User, CoreError>;

  /// 根据ID查找用户
  async fn find_by_id(&self, id: UserId) -> Result<Option<User>, CoreError>;

  /// 根据邮箱查找用户
  async fn find_by_email(&self, email: &str) -> Result<Option<User>, CoreError>;

  /// 验证用户凭据
  async fn authenticate(&self, signin_data: &SigninUser) -> Result<Option<User>, CoreError>;

  /// 更新用户信息
  async fn update(&self, id: UserId, user_data: &User) -> Result<User, CoreError>;

  /// 检查用户是否存在
  async fn exists_by_email(&self, email: &str) -> Result<bool, CoreError>;
}

/// 聊天仓储接口契约
#[async_trait]
pub trait ChatRepository: Send + Sync {
  /// 创建聊天
  async fn create(&self, chat_data: &crate::models::CreateChat) -> Result<Chat, CoreError>;

  /// 根据ID查找聊天
  async fn find_by_id(&self, id: ChatId) -> Result<Option<Chat>, CoreError>;

  /// 获取用户聊天列表
  async fn get_user_chats(
    &self,
    user_id: UserId,
  ) -> Result<Vec<crate::models::ChatSidebar>, CoreError>;

  /// 更新聊天信息
  async fn update(
    &self,
    id: ChatId,
    chat_data: &crate::models::UpdateChat,
  ) -> Result<Chat, CoreError>;

  /// 删除聊天
  async fn delete(&self, id: ChatId) -> Result<bool, CoreError>;
}

/// 消息仓储接口契约
#[async_trait]
pub trait MessageRepository: Send + Sync {
  /// 创建消息
  async fn create(&self, message_data: &crate::models::CreateMessage)
  -> Result<Message, CoreError>;

  /// 根据ID查找消息
  async fn find_by_id(&self, id: MessageId) -> Result<Option<Message>, CoreError>;

  /// 获取聊天消息列表
  async fn get_chat_messages(
    &self,
    chat_id: ChatId,
    params: &crate::models::ListMessages,
  ) -> Result<Vec<Message>, CoreError>;

  /// 更新消息
  async fn update(&self, id: MessageId, content: &str) -> Result<Message, CoreError>;

  /// 删除消息
  async fn delete(&self, id: MessageId) -> Result<bool, CoreError>;
}
