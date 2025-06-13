use super::ChatFile;
use sqlx::Row;
use crate::AppError;
use crate::AppState;
use crate::services::indexer_sync_service::ChatInfo;

use fechatter_core::{Message, error::CoreError, models::CreateMessage, models::ListMessages};
use std::str::FromStr;
use tracing::{error, info, warn};

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

    // 如果已经存在相同的消息，发布重复消息尝试事件并返回现有消息
    if let Some(message) = existing_message {
      info!(
        "Duplicate message detected: idempotency_key={}",
        input.idempotency_key
      );

      // 发布重复消息尝试事件（如果启用了NATS）
      if let Some(event_publisher) = self.event_publisher() {
        if let Err(e) = event_publisher
          .publish_duplicate_message_attempted(input.idempotency_key, chat_id, user_id)
          .await
        {
          error!("Failed to publish duplicate message attempted event: {}", e);
          // 不阻止消息返回，只记录错误
        }
      }

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

    info!(
      "Message created successfully: message_id={}, chat_id={}, sender_id={}",
      message.id, chat_id, user_id
    );

    // 发布消息创建事件（如果启用了NATS）
    if let Some(event_publisher) = self.event_publisher() {
      match self.get_chat_members(chat_id).await {
        Ok(chat_members) => {
          if let Err(e) = event_publisher
            .publish_message_created(&message, chat_members)
            .await
          {
            error!("Failed to publish message created event: {}", e);
            // 不阻止消息创建，只记录错误
          }
        }
        Err(e) => {
          error!("Failed to get chat members for event publishing: {}", e);
          // 不阻止消息创建，只记录错误
        }
      }

      // 发布异步搜索索引事件（如果启用了搜索）
      if self.config.search.enabled && self.config.search.async_indexing.enabled {
        match self.get_chat_and_sender_info(chat_id, user_id).await {
          Ok((chat_name, sender_name, chat_type, workspace_id)) => {
            let chat_info = ChatInfo {
              chat_name,
              sender_name,
              chat_type,
              workspace_id,
            };

            // 发布异步索引事件
            if let Err(e) = event_publisher
              .publish_search_index_event(&message, &chat_info)
              .await
            {
              warn!("Failed to publish search index event: {}", e);
              // 不阻止消息创建，只记录警告
            }
          }
          Err(e) => {
            warn!("Failed to get chat info for async indexing: {}", e);
          }
        }
      }
    }

    Ok(message)
  }

  /// 获取聊天成员列表
  async fn get_chat_members(&self, chat_id: i64) -> Result<Vec<i64>, AppError> {
    let members =
      sqlx::query_scalar::<_, i64>("SELECT user_id FROM chat_members WHERE chat_id = $1")
        .bind(chat_id)
        .fetch_all(self.pool())
        .await?;

    Ok(members)
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

  /// Get chat and sender information for search indexing
  async fn get_chat_and_sender_info(
    &self,
    chat_id: i64,
    user_id: i64,
  ) -> Result<(String, String, String, i64), AppError> {
    // Get chat information
    let chat_query = sqlx::query(
      r#"SELECT chat_name as name, type::text as chat_type, workspace_id 
         FROM chats WHERE id = $1"#,
    )
    .bind(chat_id)
    .fetch_optional(self.pool())
    .await?;

    let (chat_name, chat_type, workspace_id) = match chat_query {
      Some(row) => {
        let name: String = row.try_get("name")?;
        let chat_type: String = row.try_get("chat_type")?;
        let workspace_id: i64 = row.try_get("workspace_id")?;
        (name, chat_type, workspace_id)
      }
      None => {
        return Err(AppError::NotFound(vec![format!(
          "Chat with id {} not found",
          chat_id
        )]));
      }
    };

    // Get sender information
    let sender_query = sqlx::query("SELECT fullname FROM users WHERE id = $1")
      .bind(user_id)
      .fetch_optional(self.pool())
      .await?;

    let sender_name = match sender_query {
      Some(row) => {
        let fullname: String = row.try_get("fullname")?;
        fullname
      }
      None => {
        return Err(AppError::NotFound(vec![format!(
          "User with id {} not found",
          user_id
        )]));
      }
    };

    Ok((chat_name, sender_name, chat_type, workspace_id))
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

    // Generate unique chat name to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let unique_chat_name = format!("Test Chat {}", timestamp);

    // Create a chat first
    let chat = state
      .create_new_chat(user1.id.into(),
        &unique_chat_name,
        crate::models::ChatType::Group,
        Some(vec![user1.id.into(), user2.id.into(), user3.id.into()]),
        Some("Test chat for messages"),
        user1.workspace_id.into(),
    )
      .await?;

    let message_payload1 = CreateMessage {
      content: "test".to_string(),
      files: vec![],
      idempotency_key: uuid::Uuid::now_v7(),
    };

    let message1 = state
      .create_message(message_payload1, chat.id.into(), user1.id.into())
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
      .create_message(message_payload2, chat.id.into(), user2.id.into())
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
      .create_message(message_payload3, chat.id.into(), user3.id.into())
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
      .create_message(message_payload4.clone(), chat.id.into(), user1.id.into())
      .await
      .expect("Failed to create message");

    // Send the exact same message again
    let message5 = state
      .create_message(message_payload4, chat.id.into(), user1.id.into())
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

    // Generate unique chat name to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let unique_chat_name = format!("Test Chat {}", timestamp);

    // Create a chat first
    let chat = state
      .create_new_chat(user1.id.into(),
        &unique_chat_name,
        crate::models::ChatType::Group,
        Some(users.iter().map(|u| u.id.into()).collect()),
        Some("Test chat for messages"),
        user1.workspace_id.into(),
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
        .create_message(messages_payload[i].clone(), chat.id.into(), users[i].id.into())
        .await
        .expect("Failed to create message");
      message_ids.push(m.id);
    }

    // Use the highest message ID + 1 as last_id to ensure we get all messages
    let input = ListMessages {
      last_id: Some(i64::from(*message_ids.iter().max().unwrap()) + 1),
      limit: 10,
    };

    let messages = state
      .list_messages(input, chat.id.into())
      .await
      .expect("Failed to list messages");
    assert_eq!(messages.len(), 10);

    Ok(())
  }
}
