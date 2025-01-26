use crate::{
  error::CoreError,
  models::{ChatId, ChatMember, ChatType, UserId},
};
use serde::{Deserialize, Serialize};
use std::future::Future;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateChatMember {
  pub chat_id: ChatId,
  pub user_id: UserId,
}

pub trait ChatMemberRepository: Send + Sync {
  fn add_members(
    &self,
    chat_id: ChatId,
    user_id: UserId,
    member_ids: Vec<UserId>,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<ChatMember>, CoreError>> + Send>>;

  fn remove_members(
    &self,
    chat_id: ChatId,
    user_id: UserId,
    member_ids: Vec<UserId>,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<bool, CoreError>> + Send>>;

  fn list_members(
    &self,
    chat_id: ChatId,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<ChatMember>, CoreError>> + Send>>;

  fn is_member(
    &self,
    chat_id: ChatId,
    user_id: UserId,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<bool, CoreError>> + Send>>;

  fn is_creator(
    &self,
    chat_id: ChatId,
    user_id: UserId,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<bool, CoreError>> + Send>>;

  fn count_members(
    &self,
    chat_id: ChatId,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<i64, CoreError>> + Send>>;

  fn get_chat_type(
    &self,
    chat_id: ChatId,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<ChatType, CoreError>> + Send>>;

  fn transfer_ownership(
    &self,
    chat_id: ChatId,
    from_user_id: UserId,
    to_user_id: UserId,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<bool, CoreError>> + Send>>;
}
