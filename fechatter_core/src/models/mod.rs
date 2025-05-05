pub mod chat;
pub mod chat_member;
pub mod file;
pub mod message;
pub mod user;
pub mod workspace;

use std::future::Future;

use crate::error::CoreError;
use crate::state::WithDbPool;

pub trait DatabaseModel: Sized {
  type CreateType;
  type UpdateType;
  type IdType;

  fn create<S: WithDbPool + Sync>(
    input: &Self::CreateType,
    state: &S,
  ) -> impl Future<Output = Result<Self, CoreError>> + Send;

  fn find_by_id<S: WithDbPool + Sync>(
    id: Self::IdType,
    state: &S,
  ) -> impl Future<Output = Result<Option<Self>, CoreError>> + Send;

  fn update<S: WithDbPool + Sync>(
    id: Self::IdType,
    input: &Self::UpdateType,
    state: &S,
  ) -> impl Future<Output = Result<Self, CoreError>> + Send;
}

pub use chat::*;
pub use chat_member::*;
pub use message::*;
pub use user::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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
  pub workspace_id: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow, PartialEq, Eq, Clone)]
pub struct ChatUser {
  pub id: i64,
  pub fullname: String,
  pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
  pub id: i64,
  pub fullname: String,
  pub email: String,
  pub status: UserStatus,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub workspace_id: i64,
}

use crate::utils::jwt::UserClaims;

impl From<UserClaims> for AuthUser {
  fn from(claims: UserClaims) -> Self {
    Self {
      id: claims.id,
      fullname: claims.fullname,
      email: claims.email,
      status: claims.status,
      created_at: claims.created_at,
      workspace_id: claims.workspace_id,
    }
  }
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

#[derive(Debug, Serialize, Deserialize, FromRow, PartialEq, Eq, Clone)]
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

#[derive(Debug, Serialize, Deserialize, FromRow)]
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

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, PartialEq, Eq)]
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
pub struct ChatFile {
  pub workspace_id: i64,
  pub ext: String, // extract from the uploaded filename
  pub hash: String,
}
