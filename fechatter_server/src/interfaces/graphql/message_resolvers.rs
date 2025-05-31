// Message resolvers
use crate::interfaces::graphql::types::{Message, MessageWithSender, User};
use crate::{application::queries::GetChatMessagesQuery, AppState};
use async_graphql::{Error, Result};

pub async fn get_messages(
  state: &AppState,
  user_id: i64,
  chat_id: i64,
  limit: i32,
  offset: i32,
) -> Result<Vec<MessageWithSender>> {
  // Use CQRS query
  let query = GetChatMessagesQuery {
    chat_id,
    user_id,
    limit,
    offset,
  };

  let messages = state
    .execute_query(query)
    .await
    .map_err(|e| Error::new(e.to_string()))?;

  // Convert to GraphQL types with sender info
  // TODO: Implement batch loading of users to avoid N+1
  let mut messages_with_senders = Vec::new();

  for msg in messages {
    // TODO: Use DataLoader to batch load users
    let sender = User {
      id: msg.sender_id,
      email: format!("user{}@example.com", msg.sender_id),
      fullname: format!("User {}", msg.sender_id),
      created_at: chrono::Utc::now(),
      workspace_id: 1,
    };

    messages_with_senders.push(MessageWithSender {
      message: Message {
        id: msg.id,
        chat_id: msg.chat_id,
        sender_id: msg.sender_id,
        content: msg.content,
        created_at: msg.created_at,
        files: msg.files,
      },
      sender,
    });
  }

  Ok(messages_with_senders)
}
