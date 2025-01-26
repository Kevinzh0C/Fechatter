pub mod chat;
pub mod chat_member;
pub mod ids;
pub mod jwt;
pub mod message;
pub mod user;
pub mod workspace;

pub use crate::error::CoreError;
pub use chat::*;
pub use chat_member::*;
pub use ids::*;
pub use jwt::*;
pub use message::*;
pub use user::*;
pub use workspace::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq, Eq, ToSchema)]
#[sqlx(type_name = "user_status", rename_all = "lowercase")]
pub enum UserStatus {
  Suspended,
  Active,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, PartialEq, Eq, Clone, ToSchema)]
pub struct User {
  pub id: UserId,
  pub fullname: String,
  pub email: String,
  #[sqlx(default)]
  #[serde(skip)]
  pub password_hash: Option<String>,
  pub status: UserStatus,
  pub created_at: DateTime<Utc>,
  pub workspace_id: WorkspaceId,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, PartialEq, Eq, Clone, ToSchema)]
pub struct ChatUser {
  pub id: UserId,
  pub fullname: String,
  pub email: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, PartialEq, Eq, Clone, ToSchema)]
pub struct Workspace {
  pub id: WorkspaceId,
  pub name: String,
  pub owner_id: UserId,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, sqlx::Type, ToSchema)]
#[sqlx(type_name = "chat_type", rename_all = "lowercase")]
pub enum ChatType {
  Single,
  Group,
  #[sqlx(rename = "private_channel")]
  PrivateChannel,
  #[sqlx(rename = "public_channel")]
  PublicChannel,
}

impl std::fmt::Display for ChatType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ChatType::Single => write!(f, "single"),
      ChatType::Group => write!(f, "group"),
      ChatType::PrivateChannel => write!(f, "private_channel"),
      ChatType::PublicChannel => write!(f, "public_channel"),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Chat {
  pub id: ChatId,
  pub workspace_id: WorkspaceId,
  pub name: String,
  pub chat_type: ChatType,
  pub chat_members: Vec<UserId>,
  #[sqlx(default)]
  pub description: String,
  pub created_by: UserId,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone, PartialEq, Eq, ToSchema)]
pub struct Message {
  pub id: MessageId,
  pub chat_id: ChatId,
  pub sender_id: UserId,
  pub content: String,
  #[sqlx(default)] // Handle NULL in DB
  pub files: Option<Vec<String>>,
  #[sqlx(default)] // Handle potential NULL or default timestamp
  pub created_at: DateTime<Utc>,
  #[sqlx(default)] // idempotency_key may be NULL, especially for older records
  #[schema(value_type = Option<String>, format = "uuid", example = "01834abd-8c37-7d82-9206-54b2f6b4f7c4")]
  pub idempotency_key: Option<uuid::Uuid>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatMember {
  pub chat_id: ChatId,
  pub user_id: UserId,
  pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateUser {
  pub fullname: String,
  pub email: String,
  pub workspace: String,
  pub password: String,
}

impl CreateUser {
  pub fn new(fullname: &str, email: &str, workspace: &str, password: &str) -> Self {
    Self {
      fullname: fullname.to_string(),
      email: email.to_string(),
      workspace: workspace.to_string(),
      password: password.to_string(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SigninUser {
  pub email: String,
  pub password: String,
}

impl SigninUser {
  pub fn new(email: &str, password: &str) -> Self {
    Self {
      email: email.to_string(),
      password: password.to_string(),
    }
  }
}
