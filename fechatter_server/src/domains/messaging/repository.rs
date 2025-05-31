use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use fechatter_core::{
  ChatId, Message, UserId, error::CoreError, models::CreateMessage, models::ListMessages,
};

pub struct MessageRepository {
  pool: Arc<PgPool>,
}

impl MessageRepository {
  pub fn new(pool: Arc<PgPool>) -> Self {
    Self { pool }
  }

  /// Static create message method for use in async move blocks
  async fn create_message_static(
    input: &CreateMessage,
    chat_id: i64,
    user_id: i64,
    pool: Arc<PgPool>,
  ) -> Result<Message, CoreError> {
    // Check if both content is empty and no files are attached
    if input.content.is_empty() && input.files.is_empty() {
      return Err(CoreError::Validation(
        "Message must contain either text content or attachments".to_string(),
      ));
    }

    // Check for existing message with same idempotency_key
    let existing_message = sqlx::query_as::<_, Message>(
            r#"SELECT id as "id: _", chat_id as "chat_id: _", sender_id as "sender_id: _", content, files, 
                      created_at, idempotency_key
               FROM messages 
               WHERE chat_id = $1 AND sender_id = $2 AND idempotency_key = $3"#,
        )
        .bind(chat_id)
        .bind(user_id)
        .bind(input.idempotency_key)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

    if let Some(existing) = existing_message {
      // TODO: Publish duplicate message attempt event
      return Ok(existing);
    }

    // Create new message
    let message = sqlx::query_as::<_, Message>(
            r#"INSERT INTO messages (chat_id, sender_id, content, files, idempotency_key)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING id as "id: _", chat_id as "chat_id: _", sender_id as "sender_id: _", content, files, 
                         created_at, idempotency_key"#,
        )
        .bind(chat_id)
        .bind(user_id)
        .bind(&input.content)
        .bind(&input.files)
        .bind(input.idempotency_key)
        .fetch_one(&*pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

    Ok(message)
  }

  /// Static list messages method for use in async move blocks
  async fn list_messages_static(
    input: &ListMessages,
    chat_id: i64,
    pool: Arc<PgPool>,
  ) -> Result<Vec<Message>, CoreError> {
    let mut query_builder = sqlx::QueryBuilder::new(
      r#"SELECT id as "id: _", chat_id as "chat_id: _", sender_id as "sender_id: _", content, files,
                      created_at, idempotency_key
               FROM messages WHERE chat_id = "#,
    );

    query_builder.push_bind(chat_id);

    // Use last_id from core ListMessages (this means "get messages before this ID")
    if let Some(last_id) = input.last_id {
      query_builder.push(" AND id < ").push_bind(last_id);
    }

    query_builder
      .push(" ORDER BY created_at DESC LIMIT ")
      .push_bind(input.limit);

    let messages = query_builder
      .build_query_as::<Message>()
      .fetch_all(&*pool)
      .await
      .map_err(|e| CoreError::Database(e.to_string()))?;

    Ok(messages)
  }

  /// Create a new message (implementation for both trait and direct use)
  async fn create_message_impl(
    &self,
    input: &CreateMessage,
    chat_id: i64,
    user_id: i64,
  ) -> Result<Message, CoreError> {
    Self::create_message_static(input, chat_id, user_id, self.pool.clone()).await
  }

  /// List messages implementation for both trait and direct use
  async fn list_messages_impl(
    &self,
    input: &ListMessages,
    chat_id: i64,
  ) -> Result<Vec<Message>, CoreError> {
    Self::list_messages_static(input, chat_id, self.pool.clone()).await
  }
}

// Implement the core MessageRepository trait
#[async_trait]
impl fechatter_core::models::MessageRepository for MessageRepository {
  fn create_message(
    &self,
    input: &CreateMessage,
    chat_id: ChatId,
    user_id: UserId,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Message, CoreError>> + Send>> {
    let chat_id = i64::from(chat_id);
    let user_id = i64::from(user_id);
    let input = input.clone();
    let pool = self.pool.clone();

    Box::pin(async move { Self::create_message_static(&input, chat_id, user_id, pool).await })
  }

  fn list_messages(
    &self,
    input: &ListMessages,
    chat_id: ChatId,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Message>, CoreError>> + Send>>
  {
    let chat_id = i64::from(chat_id);
    let input = input.clone();
    let pool = self.pool.clone();

    Box::pin(async move { Self::list_messages_static(&input, chat_id, pool).await })
  }
}

// Convenience methods for server internal use (using i64 types)
impl MessageRepository {
  /// Create a new message (convenience method for server use)
  pub async fn create_message(
    &self,
    input: CreateMessage,
    chat_id: i64,
    user_id: i64,
  ) -> Result<Message, CoreError> {
    self.create_message_impl(&input, chat_id, user_id).await
  }

  /// List messages for a chat (convenience method for server use)
  pub async fn list_messages(
    &self,
    input: ListMessages,
    chat_id: i64,
  ) -> Result<Vec<Message>, CoreError> {
    self.list_messages_impl(&input, chat_id).await
  }

  /// Get a message by ID
  pub async fn get_message_by_id(&self, message_id: i64) -> Result<Option<Message>, CoreError> {
    let message = sqlx::query_as::<_, Message>(
      r#"SELECT id as "id: _", chat_id as "chat_id: _", sender_id as "sender_id: _", content, files,
                      created_at, idempotency_key
               FROM messages WHERE id = $1"#,
    )
    .bind(message_id)
    .fetch_optional(&*self.pool)
    .await
    .map_err(|e| CoreError::Database(e.to_string()))?;

    Ok(message)
  }

  /// Update message content
  pub async fn update_message(
    &self,
    message_id: i64,
    new_content: String,
    editor_id: i64,
  ) -> Result<Message, CoreError> {
    let message = sqlx::query_as::<_, Message>(
            r#"UPDATE messages SET content = $1 WHERE id = $2 AND sender_id = $3
               RETURNING id as "id: _", chat_id as "chat_id: _", sender_id as "sender_id: _", content, files,
                         created_at, idempotency_key"#,
        )
        .bind(new_content)
        .bind(message_id)
        .bind(editor_id)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

    Ok(message)
  }

  /// Delete a message
  pub async fn delete_message(&self, message_id: i64, user_id: i64) -> Result<(), CoreError> {
    let result = sqlx::query("DELETE FROM messages WHERE id = $1 AND sender_id = $2")
      .bind(message_id)
      .bind(user_id)
      .execute(&*self.pool)
      .await
      .map_err(|e| CoreError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
      return Err(CoreError::NotFound(format!(
        "Message {} not found or permission denied",
        message_id
      )));
    }

    Ok(())
  }

  /// Get messages count for a chat
  pub async fn get_messages_count(&self, chat_id: i64) -> Result<i64, CoreError> {
    let count = sqlx::query_scalar("SELECT COUNT(*) FROM messages WHERE chat_id = $1")
      .bind(chat_id)
      .fetch_one(&*self.pool)
      .await
      .map_err(|e| CoreError::Database(e.to_string()))?;

    Ok(count)
  }
}
