use super::ChatFile;
use super::Message;
use crate::error::CoreError;
use crate::state::{WithCache, WithDbPool};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessage {
  pub content: String,
  #[serde(default)]
  pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMessage {
  #[serde(default)]
  pub last_id: Option<i64>,
  #[serde(default)]
  pub limit: i64,
}

pub async fn create_message<S>(
  state: &S,
  input: CreateMessage,
  chat_id: i64,
  user_id: i64,
) -> Result<Message, CoreError>
where
  S: WithDbPool + Sync,
{
  // Check if both content is empty and no files are attached
  if input.content.is_empty() && input.files.is_empty() {
    return Err(CoreError::Validation(
      "Message must contain either text content or attachments".to_string(),
    ));
  }

  // Validate files exist
  for s in &input.files {
    let chat_file = ChatFile::from_str(s)?;
  }

  let message = sqlx::query_as::<_, Message>(
    r#"INSERT INTO messages (chat_id, sender_id, content, files) 
      VALUES ($1, $2, $3, $4) 
      RETURNING id, chat_id, sender_id, content, files, created_at::timestamptz"#,
  )
  .bind(chat_id)
  .bind(user_id)
  .bind(input.content)
  .bind(&input.files)
  .fetch_one(state.db_pool())
  .await?;

  Ok(message)
}

pub async fn list_messages<S>(
  state: &S,
  input: ListMessage,
  chat_id: i64,
) -> Result<Vec<Message>, CoreError>
where
  S: WithDbPool + Sync,
{
  let last_id = input.last_id.unwrap_or(i64::MAX);

  let limit = match input.limit {
    0 => i64::MAX,
    1..=100 => input.limit as _,
    _ => 100,
  };

  let messages: Vec<Message> = sqlx::query_as(
    r#"
      SELECT id, chat_id, sender_id, content, files, created_at::timestamptz
      FROM messages
      WHERE chat_id = $1
      AND id < $2
      ORDER BY created_at DESC
      LIMIT $3
    "#,
  )
  .bind(chat_id)
  .bind(last_id)
  .bind(limit)
  .fetch_all(state.db_pool())
  .await?;

  Ok(messages)
}

mod tests {
  use super::*;
  use crate::models::DatabaseModel;
  use anyhow::Result;

  #[allow(unused)]
  async fn upload_dummy_file<S>(state: &S) -> Result<String>
  where
    S: std::fmt::Debug,
  {
    let url = "/files/1/aa/bb/cc/test.txt";
    Ok(url.to_string())
  }

  #[tokio::test]
  async fn create_message_should_work() -> Result<()> {
    let (_tdb, state, users) = crate::setup_test_users!(3);
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];

    // Create a chat first
    let chat = crate::models::create_new_chat(
      &state,
      user1.id,
      "Test Chat",
      crate::models::ChatType::Group,
      Some(vec![user1.id, user2.id, user3.id]),
      Some("Test chat for messages"),
      user1.workspace_id,
    )
    .await?;

    let message_payload1 = CreateMessage {
      content: "test".to_string(),
      files: vec![],
    };

    let message1 = create_message(&state, message_payload1, chat.id, user1.id)
      .await
      .expect("Failed to create message");

    assert_eq!(message1.content, "test");
    assert!(
      message1
        .files
        .as_ref()
        .map_or(false, |files| files.is_empty())
    );

    // Create test file in workspace 1
    let url = upload_dummy_file(&state).await?;

    let message_payload2 = CreateMessage {
      content: "test".to_string(),
      files: vec![url],
    };

    let message2 = create_message(&state, message_payload2, chat.id, user2.id)
      .await
      .expect("Failed to create message");

    assert_eq!(message2.content, "test");

    // Test file-only message (with empty content)
    let url = upload_dummy_file(&state).await?;
    let message_payload3 = CreateMessage {
      content: "".to_string(),
      files: vec![url],
    };

    let message3 = create_message(&state, message_payload3, chat.id, user3.id)
      .await
      .expect("Failed to create file-only message");

    assert!(message3.content.is_empty());
    assert!(
      message3
        .files
        .as_ref()
        .map_or(false, |files| !files.is_empty())
    );
    Ok(())
  }

  #[tokio::test]
  async fn list_messages_should_work() -> Result<()> {
    let (_tdb, state, users) = crate::setup_test_users!(10);
    let user1 = &users[0];

    // Create a chat first
    let chat = crate::models::create_new_chat(
      &state,
      user1.id,
      "Test Chat",
      crate::models::ChatType::Group,
      Some(users.iter().map(|u| u.id).collect()),
      Some("Test chat for messages"),
      user1.workspace_id,
    )
    .await?;

    let mut messages_payload = Vec::with_capacity(10);
    for _i in 0..10 {
      let m = CreateMessage {
        content: "test".to_string(),
        files: vec![],
      };
      messages_payload.push(m);
    }

    // Create messages and collect their IDs
    let mut message_ids = Vec::new();
    for i in 0..10 {
      let m: Message = create_message(&state, messages_payload[i].clone(), chat.id, users[i].id)
        .await
        .expect("Failed to create message");
      message_ids.push(m.id);
    }

    // Use the highest message ID + 1 as last_id to ensure we get all messages
    let input = ListMessage {
      last_id: Some(message_ids.iter().max().unwrap() + 1),
      limit: 10,
    };

    let messages = list_messages(&state, input, chat.id)
      .await
      .expect("Failed to list messages");
    assert_eq!(messages.len(), 10);

    Ok(())
  }
}
