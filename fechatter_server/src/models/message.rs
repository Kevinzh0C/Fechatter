use super::ChatFile;
use crate::AppError;
use crate::AppState;

use fechatter_core::{Message, error::CoreError, models::CreateMessage, models::ListMessages};
use sqlx::Row;
use std::str::FromStr;
use uuid;

impl AppState {
  pub async fn create_message(
    &self,
    input: CreateMessage,
    chat_id: i64,
    user_id: i64,
  ) -> Result<Message, AppError> {
    let base_dir = &self.config.server.base_dir;

    // Check if both content is empty and no files are attached
    if input.content.is_empty() && input.files.is_empty() {
      return Err(AppError::ChatFileError(
        "Message must contain either text content or attachments".to_string(),
      ));
    }

    // Validate files exist
    for s in &input.files {
      let chat_file = ChatFile::from_str(s)?;
      if !chat_file.from_path(base_dir).exists() {
        return Err(AppError::ChatFileError(format!(
          "File {} does not exist",
          s
        )));
      }
    }

    // 检查是否已经存在相同idempotency_key的消息
    let existing_message = sqlx::query_as::<_, Message>(
      r#"SELECT id, chat_id, sender_id, content, files, created_at::timestamptz, idempotency_key
         FROM messages 
         WHERE chat_id = $1 AND sender_id = $2 AND idempotency_key = $3"#,
    )
    .bind(chat_id)
    .bind(user_id)
    .bind(input.idempotency_key)
    .fetch_optional(self.pool())
    .await?;

    // 如果已经存在相同的消息，则直接返回
    if let Some(message) = existing_message {
      return Ok(message);
    }

    // 不存在则创建新消息
    let message = sqlx::query_as::<_, Message>(
      r#"INSERT INTO messages (chat_id, sender_id, content, files, idempotency_key)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, chat_id, sender_id, content, files, created_at::timestamptz, idempotency_key"#,
    )
    .bind(chat_id)
    .bind(user_id)
    .bind(input.content)
    .bind(&input.files)
    .bind(input.idempotency_key)
    .fetch_one(self.pool())
    .await?;

    Ok(message)
  }

  pub async fn list_messages(
    &self,
    input: ListMessages,
    chat_id: i64,
  ) -> Result<Vec<Message>, AppError> {
    let last_id = input.last_id.unwrap_or(i64::MAX);

    let limit = match input.limit {
      0 => i64::MAX,
      1..=100 => input.limit as _,
      _ => 100,
    };

    let messages: Vec<Message> = sqlx::query_as::<_, Message>(
      r#"
        SELECT id, chat_id, sender_id, content, files, created_at::timestamptz, idempotency_key
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
    .fetch_all(self.pool())
    .await?;

    Ok(messages)
  }
}

#[allow(unused)]
pub fn validate_message(input: &CreateMessage) -> Result<(), CoreError> {
  // Check if both content is empty and no files are attached
  if input.content.is_empty() && input.files.is_empty() {
    return Err(CoreError::Validation(
      "Message must contain either text content or attachments".to_string(),
    ));
  }

  // Validate files exist
  for s in &input.files {
    match ChatFile::from_str(s) {
      Ok(_) => {}
      Err(e) => return Err(CoreError::Validation(format!("Invalid file URL: {}", e))),
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::models::ChatFile;
  use crate::setup_test_users;
  use anyhow::Result;

  #[allow(unused)]
  async fn upload_dummy_file(state: &AppState) -> Result<String> {
    let file = ChatFile::new(1, "test.txt", b"hello world");
    let path = file.from_path(&state.config.server.base_dir);
    std::fs::create_dir_all(path.parent().expect("file path parent should exists"))?;
    std::fs::write(&path, b"hello world")?;

    // Instead of using file.url(), construct a URL that ChatFile::from_str can parse
    // Format: /files/workspace_id/part1/part2/part3.ext
    let hash_parts = file.hash_to_path(); // format: "{workspace_id}/{part1}/{part2}/{part3}.{ext}"
    let url = format!("/files/{}", hash_parts);

    Ok(url)
  }

  #[tokio::test]
  async fn create_message_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];

    // Create a chat first
    let chat = state
      .create_new_chat(
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
      idempotency_key: uuid::Uuid::now_v7(),
    };

    let message1 = state
      .create_message(message_payload1, chat.id, user1.id)
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
      idempotency_key: uuid::Uuid::now_v7(),
    };

    let message2 = state
      .create_message(message_payload2, chat.id, user2.id)
      .await
      .expect("Failed to create message");

    assert_eq!(message2.content, "test");

    // Test file-only message (with empty content)
    let url = upload_dummy_file(&state).await?;
    let message_payload3 = CreateMessage {
      content: "".to_string(),
      files: vec![url],
      idempotency_key: uuid::Uuid::now_v7(),
    };

    let message3 = state
      .create_message(message_payload3, chat.id, user3.id)
      .await
      .expect("Failed to create file-only message");

    assert!(message3.content.is_empty());
    assert!(
      message3
        .files
        .as_ref()
        .map_or(false, |files| !files.is_empty())
    );

    // Test idempotency with the same key
    let idempotency_key = uuid::Uuid::now_v7();
    let message_payload4 = CreateMessage {
      content: "idempotency test".to_string(),
      files: vec![],
      idempotency_key,
    };

    let message4 = state
      .create_message(message_payload4.clone(), chat.id, user1.id)
      .await
      .expect("Failed to create message");

    // Send the exact same message again
    let message5 = state
      .create_message(message_payload4, chat.id, user1.id)
      .await
      .expect("Failed to create duplicate message");

    // Should return the same message
    assert_eq!(message4.id, message5.id);
    assert_eq!(message4.content, message5.content);
    assert_eq!(message4.created_at, message5.created_at);

    Ok(())
  }

  #[tokio::test]
  async fn list_messages_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(10).await;
    let user1 = &users[0];

    // Create a chat first
    let chat = state
      .create_new_chat(
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
        idempotency_key: uuid::Uuid::now_v7(),
      };
      messages_payload.push(m);
    }

    // Create messages and collect their IDs
    let mut message_ids = Vec::new();
    for i in 0..10 {
      let m: Message = state
        .create_message(messages_payload[i].clone(), chat.id, users[i].id)
        .await
        .expect("Failed to create message");
      message_ids.push(m.id);
    }

    // Use the highest message ID + 1 as last_id to ensure we get all messages
    let input = ListMessages {
      last_id: Some(message_ids.iter().max().unwrap() + 1),
      limit: 10,
    };

    let messages = state
      .list_messages(input, chat.id)
      .await
      .expect("Failed to list messages");
    assert_eq!(messages.len(), 10);

    Ok(())
  }
}
