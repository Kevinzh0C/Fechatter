mod chat;
mod chat_member;
mod message;
mod user;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub use user::{CreateUser, SigninUser};

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

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "chat_type", rename_all = "lowercase")]
pub enum ChatType {
  Single,
  Group,
  PrivateChannel,
  PublicChannel,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chat {
  pub id: i64,
  pub name: String,
  pub chat_type: ChatType,
  pub chat_members: Vec<i64>,
  pub description: String,
  pub created_by: i64,
  pub created_at: DateTime<Utc>,
  pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
  pub id: i64,
  pub content: String,
  pub sender_id: i64,
  pub chat_id: i64,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
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
  pub id: i64,
  pub chat_id: i64,
  pub user_id: i64,
}
