// Chat resolvers
use crate::interfaces::graphql::types::{Chat, ChatType, ChatWithDetails, Message, User};
use crate::{application::queries::ListUserChatsQuery, AppState};
use async_graphql::{Error, Result};

pub async fn get_chat(state: &AppState, chat_id: i64) -> Result<ChatWithDetails> {
  // TODO: Implement using CQRS query
  // For now, return mock data
  Ok(ChatWithDetails {
    chat: Chat {
      id: chat_id,
      name: "Test Chat".to_string(),
      chat_type: ChatType::Group,
      created_by: 1,
      created_at: chrono::Utc::now(),
      updated_at: chrono::Utc::now(),
      workspace_id: 1,
      member_ids: vec![1, 2, 3],
    },
    last_message: None,
    unread_count: 0,
    members: vec![],
  })
}

pub async fn list_chats(state: &AppState, user_id: i64) -> Result<Vec<ChatWithDetails>> {
  // Use CQRS query
  let query = ListUserChatsQuery {
    user_id,
    workspace_id: 1, // TODO: Get from context
  };

  let chats = state
    .execute_query(query)
    .await
    .map_err(|e| Error::new(e.to_string()))?;

  // Convert to GraphQL types
  // TODO: Implement proper conversion with last message and unread count
  let chat_details: Vec<ChatWithDetails> = chats
    .into_iter()
    .map(|chat| ChatWithDetails {
      chat: Chat {
        id: chat.id,
        name: chat.name,
        chat_type: match chat.chat_type {
          crate::domain::chat::entities::ChatType::Single => ChatType::Single,
          crate::domain::chat::entities::ChatType::Group => ChatType::Group,
          crate::domain::chat::entities::ChatType::PrivateChannel => ChatType::PrivateChannel,
          crate::domain::chat::entities::ChatType::PublicChannel => ChatType::PublicChannel,
        },
        created_by: chat.created_by,
        created_at: chat.created_at,
        updated_at: chat.updated_at,
        workspace_id: chat.workspace_id,
        member_ids: chat.chat_members,
      },
      last_message: None,
      unread_count: 0,
      members: vec![],
    })
    .collect();

  Ok(chat_details)
}
