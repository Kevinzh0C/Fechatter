pub mod chat;
pub mod chat_member;
pub mod ids;
pub mod jwt;
pub mod message;
pub mod time_management;
pub mod user;
pub mod vector_db;
pub mod workspace;

pub use crate::error::CoreError;
pub use chat::*;
pub use chat_member::*;
pub use ids::*;
pub use jwt::*;
pub use message::*;
pub use time_management::*;
pub use user::*;
pub use vector_db::*;
pub use workspace::*;

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, ToSchema)]
pub enum UserStatus {
  #[serde(rename = "Suspended")]
  Suspended,
  #[serde(rename = "Active")]
  Active,
}

// Implement conversions for database VARCHAR storage
impl sqlx::Type<sqlx::Postgres> for UserStatus {
  fn type_info() -> sqlx::postgres::PgTypeInfo {
    sqlx::postgres::PgTypeInfo::with_name("VARCHAR")
  }
}

impl sqlx::Encode<'_, sqlx::Postgres> for UserStatus {
  fn encode_by_ref(
    &self,
    buf: &mut sqlx::postgres::PgArgumentBuffer,
  ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
    let s = match self {
      UserStatus::Suspended => "Suspended",
      UserStatus::Active => "Active",
    };
    <&str as sqlx::Encode<sqlx::Postgres>>::encode(s, buf)
  }
}

impl sqlx::Decode<'_, sqlx::Postgres> for UserStatus {
  fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
    let s: &str = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
    match s {
      "Suspended" => Ok(UserStatus::Suspended),
      "Active" => Ok(UserStatus::Active),
      _ => Err(format!("Unknown UserStatus value: {}", s).into()),
    }
  }
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
  // Profile fields
  #[sqlx(default)]
  pub phone: Option<String>,
  #[sqlx(default)]
  pub title: Option<String>,
  #[sqlx(default)]
  pub department: Option<String>,
  #[sqlx(default)]
  pub avatar_url: Option<String>,
  #[sqlx(default)]
  pub bio: Option<String>,
  #[sqlx(default)]
  pub timezone: Option<String>,
  #[sqlx(default)]
  pub language: Option<String>,
  #[sqlx(default)]
  pub last_active_at: Option<DateTime<Utc>>,
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, ToSchema)]
pub enum ChatType {
  #[serde(rename = "Single")]
  Single,
  #[serde(rename = "Group")]
  Group,
  #[serde(rename = "PrivateChannel")]
  PrivateChannel,
  #[serde(rename = "PublicChannel")]
  PublicChannel,
}

// Implement conversions for database VARCHAR storage
impl sqlx::Type<sqlx::Postgres> for ChatType {
  fn type_info() -> sqlx::postgres::PgTypeInfo {
    sqlx::postgres::PgTypeInfo::with_name("VARCHAR")
  }
}

impl sqlx::Encode<'_, sqlx::Postgres> for ChatType {
  fn encode_by_ref(
    &self,
    buf: &mut sqlx::postgres::PgArgumentBuffer,
  ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
    let s = match self {
      ChatType::Single => "Single",
      ChatType::Group => "Group",
      ChatType::PrivateChannel => "PrivateChannel",
      ChatType::PublicChannel => "PublicChannel",
    };
    <&str as sqlx::Encode<sqlx::Postgres>>::encode(s, buf)
  }
}

impl sqlx::Decode<'_, sqlx::Postgres> for ChatType {
  fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
    let s: &str = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
    match s {
      "Single" => Ok(ChatType::Single),
      "Group" => Ok(ChatType::Group),
      "PrivateChannel" => Ok(ChatType::PrivateChannel),
      "PublicChannel" => Ok(ChatType::PublicChannel),
      _ => Err(format!("Unknown ChatType value: {}", s).into()),
    }
  }
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

impl std::str::FromStr for ChatType {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "single" => Ok(ChatType::Single),
      "group" => Ok(ChatType::Group),
      "private_channel" | "privatechannel" => Ok(ChatType::PrivateChannel),
      "public_channel" | "publicchannel" => Ok(ChatType::PublicChannel),
      _ => Err(format!("Invalid chat type: {}", s)),
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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, PartialEq, Clone, ToSchema)]
pub struct UserSettings {
  pub user_id: UserId,
  // Notification preferences
  pub email_notifications: bool,
  pub push_notifications: bool,
  pub desktop_notifications: bool,
  pub notification_sound: String,
  // Presence preferences
  pub show_online_status: bool,
  pub auto_away: bool,
  pub auto_away_minutes: i32,
  // UI preferences
  pub theme: String,
  pub message_display: String,
  // Privacy preferences
  pub profile_visibility: String,
  pub show_email: bool,
  pub show_phone: bool,
  // Advanced preferences
  #[sqlx(default)]
  pub custom_preferences: Option<serde_json::Value>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone, ToSchema)]
pub struct UserActivityLog {
  pub id: i64,
  pub user_id: UserId,
  pub activity_type: String,
  pub description: Option<String>,
  #[schema(value_type = Option<String>, format = "ip")]
  pub ip_address: Option<std::net::IpAddr>,
  pub user_agent: Option<String>,
  #[sqlx(default)]
  pub metadata: Option<serde_json::Value>,
  pub created_at: DateTime<Utc>,
}
