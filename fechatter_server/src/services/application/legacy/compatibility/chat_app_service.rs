//! # Chat Application Service
//!
//! **职责**: 聊天应用服务的兼容性模块
//! **原则**: 提供兼容性类型和服务接口

use crate::AppError;
use async_trait::async_trait;
use fechatter_core::models::{ChatSidebar, UpdateChat};
use fechatter_core::{ChatId, UserId};
use serde::{Deserialize, Serialize};

/// 创建聊天输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatInput {
  pub name: String,
  pub description: Option<String>,
  pub chat_type: String,
  pub member_ids: Vec<i64>,
}

/// 聊天详情视图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatDetailView {
  pub id: i64,
  pub name: String,
  pub description: Option<String>,
  pub chat_type: String,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
  pub member_count: i64,
  pub workspace_id: i64,
}

/// 聊天服务特征
#[async_trait]
pub trait ChatServiceTrait: Send + Sync {
  async fn create_chat(&self, input: CreateChatInput) -> Result<ChatDetailView, AppError>;
  async fn get_chat(&self, id: i64) -> Result<Option<ChatDetailView>, AppError>;
  async fn list_user_chats(&self, user_id: i64) -> Result<Vec<ChatSidebar>, AppError>;
  async fn update_chat(
    &self,
    chat_id: ChatId,
    user_id: UserId,
    payload: UpdateChat,
  ) -> Result<ChatDetailView, AppError>;
  async fn delete_chat(&self, chat_id: ChatId, user_id: UserId) -> Result<bool, AppError>;
  async fn list_chat_members(
    &self,
    chat_id: i64,
  ) -> Result<Vec<fechatter_core::models::ChatMember>, AppError>;
  async fn add_members(
    &self,
    chat_id: i64,
    user_id: i64,
    member_ids: Vec<i64>,
  ) -> Result<(), AppError>;
  async fn remove_members(
    &self,
    chat_id: i64,
    user_id: i64,
    member_ids: Vec<i64>,
  ) -> Result<(), AppError>;
  async fn transfer_chat_ownership(
    &self,
    chat_id: i64,
    current_owner_id: i64,
    new_owner_id: i64,
  ) -> Result<bool, AppError>;
}
