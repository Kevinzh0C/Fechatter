mod chat;
mod chat_member;
mod message;
mod user;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub use chat::*;
pub use chat_member::*;
// pub use message::*;
pub use user::{AuthUser, CreateUser, SigninUser};

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "user_status", rename_all = "lowercase")]
pub enum UserStatus {
  Suspended,
  Active,
}

#[derive(Debug, Serialize, Deserialize, FromRow, PartialEq, Eq, Clone)]
pub struct User {
  pub id: i64,
  pub fullname: String,
  pub email: String,
  #[sqlx(default)]
  #[serde(skip)]
  pub password_hash: Option<String>,
  pub status: UserStatus,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, sqlx::Type)]
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

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Chat {
  pub id: i64,
  pub name: String,
  pub chat_type: ChatType,
  pub chat_members: Vec<i64>,
  #[sqlx(default)]
  pub description: String,
  pub created_by: i64,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
  pub id: i64,
  pub content: String,
  pub sender_id: i64,
  pub chat_id: i64,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub is_deleted: bool,
  pub is_edited: bool,
  pub is_read: bool,
  pub is_pinned: bool,
  pub is_starred: bool,
  pub is_forwarded: bool,
  pub is_reply: bool,
  pub is_reply_to: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMember {
  pub chat_id: i64,
  pub user_id: i64,
  pub joined_at: DateTime<Utc>,
}
