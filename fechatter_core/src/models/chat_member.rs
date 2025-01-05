use crate::{
  error::CoreError,
  models::{ChatMember, ChatType},
};
use serde::{Deserialize, Serialize};
use std::future::Future;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatMember {
  pub chat_id: i64,
  pub user_id: i64,
}

pub trait ChatMemberRepository: Send + Sync {
  fn add_members(
    &self,
    chat_id: i64,
    user_id: i64,
    member_ids: Vec<i64>,
  ) -> impl Future<Output = Result<Vec<ChatMember>, CoreError>> + Send;

  fn remove_members(
    &self,
    chat_id: i64,
    user_id: i64,
    member_ids: Vec<i64>,
  ) -> impl Future<Output = Result<bool, CoreError>> + Send;

  fn list_members(
    &self,
    chat_id: i64,
  ) -> impl Future<Output = Result<Vec<ChatMember>, CoreError>> + Send;

  fn is_member(
    &self,
    chat_id: i64,
    user_id: i64,
  ) -> impl Future<Output = Result<bool, CoreError>> + Send;

  fn is_creator(
    &self,
    chat_id: i64,
    user_id: i64,
  ) -> impl Future<Output = Result<bool, CoreError>> + Send;

  fn count_members(&self, chat_id: i64) -> impl Future<Output = Result<i64, CoreError>> + Send;

  fn get_chat_type(&self, chat_id: i64)
  -> impl Future<Output = Result<ChatType, CoreError>> + Send;

  fn transfer_ownership(
    &self,
    chat_id: i64,
    from_user_id: i64,
    to_user_id: i64,
  ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}
