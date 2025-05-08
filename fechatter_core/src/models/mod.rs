pub mod chat;
pub mod chat_member;
pub mod jwt;
pub mod message;
pub mod user;
pub mod workspace;

pub use crate::error::CoreError;
pub use chat::*;
pub use chat_member::*;
pub use jwt::*;
pub use message::*;
pub use user::*;
pub use workspace::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "user_status", rename_all = "lowercase")]
pub enum UserStatus {
  Suspended,
  Active,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, PartialEq, Eq, Clone)]
pub struct User {
  pub id: i64,
  pub fullname: String,
  pub email: String,
  #[sqlx(default)]
  #[serde(skip)]
  pub password_hash: Option<String>,
  pub status: UserStatus,
  pub created_at: DateTime<Utc>,
  pub workspace_id: i64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, PartialEq, Eq, Clone)]
pub struct ChatUser {
  pub id: i64,
  pub fullname: String,
  pub email: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, PartialEq, Eq, Clone)]
pub struct Workspace {
  pub id: i64,
  pub name: String,
  pub owner_id: i64,
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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Chat {
  pub id: i64,
  pub workspace_id: i64,
  pub name: String,
  pub chat_type: ChatType,
  pub chat_members: Vec<i64>,
  #[sqlx(default)]
  pub description: String,
  pub created_by: i64,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone, PartialEq, Eq)]
pub struct Message {
  pub id: i64,
  pub chat_id: i64,
  pub sender_id: i64,
  pub content: String,
  #[sqlx(default)] // Handle NULL in DB
  pub files: Option<Vec<String>>,
  #[sqlx(default)] // Handle potential NULL or default timestamp
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMember {
  pub chat_id: i64,
  pub user_id: i64,
  pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
