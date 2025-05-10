use serde::{Deserialize, Serialize};
use std::future::Future;
use utoipa::ToSchema;

use crate::error::{ChatValidationError, CoreError};
use crate::models::{Chat, ChatType};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateChat {
  pub name: String,
  pub chat_type: ChatType,
  pub members: Option<Vec<i64>>,
  pub description: Option<String>,
}

impl CreateChat {
  pub fn new(name: &str, chat_type: ChatType, members: Vec<i64>, description: &str) -> Self {
    Self {
      name: name.to_string(),
      chat_type,
      members: Some(members),
      description: Some(description.to_string()),
    }
  }
}

pub trait ChatRepository: Send + Sync {
  fn create(
    &self,
    input: &CreateChat,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<Chat, CoreError>> + Send>>;

  fn find_by_id(
    &self,
    id: i64,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<Option<Chat>, CoreError>> + Send>>;

  fn get_sidebar_for_user(
    &self,
    user_id: i64,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<ChatSidebar>, CoreError>> + Send>>;

  fn update_chat_name(
    &self,
    chat_id: i64,
    user_id: i64,
    new_name: &str,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<Chat, CoreError>> + Send>>;

  fn update_chat_description(
    &self,
    chat_id: i64,
    user_id: i64,
    new_description: &str,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<Chat, CoreError>> + Send>>;

  fn delete_chat(
    &self,
    chat_id: i64,
    user_id: i64,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<bool, CoreError>> + Send>>;
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct ChatSidebar {
  pub id: i64,
  pub name: String,
  pub chat_type: ChatType,
  pub last_message: Option<ChatLastMessage>,
  pub is_creator: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct ChatLastMessage {
  pub id: i64,
  pub content: String,
  pub sender_name: String,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub has_files: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateChat {
  pub name: Option<String>,
  pub description: Option<String>,
}

pub fn validate_chat_name(name: &str) -> Result<(), CoreError> {
  if name.trim().is_empty() {
    Err(CoreError::ChatValidation(ChatValidationError::InvalidName(
      "Chat name cannot be empty".to_string(),
    )))
  } else if name.len() > 128 {
    Err(CoreError::ChatValidation(ChatValidationError::InvalidName(
      "Chat name cannot be longer than 128 characters".to_string(),
    )))
  } else {
    Ok(())
  }
}

pub fn process_chat_members(
  chat_type: &ChatType,
  creator_id: i64,
  target_members: Option<&Vec<i64>>,
) -> Result<Vec<i64>, CoreError> {
  match chat_type {
    ChatType::Single => match target_members {
      Some(members) if members.len() == 1 => {
        let target_id = members[0];
        if target_id == creator_id {
          return Err(CoreError::ChatValidation(
            ChatValidationError::InvalidMembers(
              "Single chat must have exactly one member".to_string(),
            ),
          ));
        }
        Ok(vec![creator_id, target_id])
      }
      _ => Err(CoreError::ChatValidation(
        ChatValidationError::InvalidMembers("Invalid single chat members".to_string()),
      )),
    },
    ChatType::Group => {
      let mut result = vec![creator_id];
      if let Some(members) = target_members {
        for &id in members {
          if id != creator_id && !result.contains(&id) {
            result.push(id);
          }
        }
      }
      if result.len() < 3 {
        return Err(CoreError::ChatValidation(
          ChatValidationError::InvalidMembers(
            "Group chat must have at least three members".to_string(),
          ),
        ));
      }
      Ok(result)
    }
    ChatType::PrivateChannel => {
      let mut result = vec![creator_id];
      if let Some(members) = target_members {
        for &id in members {
          if id != creator_id && !result.contains(&id) {
            result.push(id);
          }
        }
      }
      Ok(result)
    }
    ChatType::PublicChannel => Ok(vec![creator_id]),
  }
}
