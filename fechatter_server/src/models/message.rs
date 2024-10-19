// use chrono::{DateTime, Utc};
// use sqlx::PgPool;

// use super::Message;
// use crate::AppError;

// pub async fn create_message(
//   pool: &PgPool,
//   chat_id: i64,
//   sender_id: i64,
//   content: String,
// ) -> Result<Message, AppError> {
//   let row = sqlx::query!(
//     r#"
//     INSERT INTO messages (chat_id, sender_id, content)
//     VALUES ($1, $2, $3)
//     RETURNING id, chat_id, sender_id, content, created_at
//     "#,
//     chat_id,
//     sender_id,
//     content
//   )
//   .fetch_one(pool)
//   .await?;

//   Ok(Message {
//     id: row.id,
//     chat_id: row.chat_id,
//     sender_id: row.sender_id,
//     content: row.content,
//     created_at: row.created_at.unwrap_or_default(),
//     updated_at: row.created_at.unwrap_or_default(),
//     is_deleted: false,
//     is_edited: false,
//     is_read: false,
//     is_pinned: false,
//     is_starred: false,
//     is_forwarded: false,
//     is_reply: false,
//     is_reply_to: false,
//   })
// }

// pub async fn list_messages(pool: &PgPool, chat_id: i64) -> Result<Vec<Message>, AppError> {
//   let rows = sqlx::query!(
//     r#"
//     SELECT
//       id, chat_id, sender_id, content, created_at
//     FROM messages
//     WHERE chat_id = $1
//     ORDER BY created_at ASC
//     "#,
//     chat_id
//   )
//   .fetch_all(pool)
//   .await?;

//   let messages = rows
//     .into_iter()
//     .map(|row| Message {
//       id: row.id,
//       chat_id: row.chat_id,
//       sender_id: row.sender_id,
//       content: row.content,
//       created_at: row.created_at.unwrap_or_default(),
//       updated_at: row.created_at.unwrap_or_default(),
//       is_deleted: false,
//       is_edited: false,
//       is_read: false,
//       is_pinned: false,
//       is_starred: false,
//       is_forwarded: false,
//       is_reply: false,
//       is_reply_to: false,
//     })
//     .collect();

//   Ok(messages)
// }

// pub async fn get_message(pool: &PgPool, id: i64) -> Result<Message, AppError> {
//   let row = sqlx::query!(
//     r#"
//     SELECT id, chat_id, sender_id, content, created_at
//     FROM messages
//     WHERE id = $1
//     "#,
//     id
//   )
//   .fetch_one(pool)
//   .await?;

//   Ok(Message {
//     id: row.id,
//     chat_id: row.chat_id,
//     sender_id: row.sender_id,
//     content: row.content,
//     created_at: row.created_at.unwrap_or_default(),
//     updated_at: row.created_at.unwrap_or_default(),
//     is_deleted: false,
//     is_edited: false,
//     is_read: false,
//     is_pinned: false,
//     is_starred: false,
//     is_forwarded: false,
//     is_reply: false,
//     is_reply_to: false,
//   })
// }

// pub async fn update_message(pool: &PgPool, id: i64, content: String) -> Result<Message, AppError> {
//   let row = sqlx::query!(
//     r#"
//     UPDATE messages
//     SET content = $1
//     WHERE id = $2
//     RETURNING id, chat_id, sender_id, content, created_at
//     "#,
//     content,
//     id
//   )
//   .fetch_one(pool)
//   .await?;

//   Ok(Message {
//     id: row.id,
//     chat_id: row.chat_id,
//     sender_id: row.sender_id,
//     content: row.content,
//     created_at: row.created_at.unwrap_or_default(),
//     updated_at: row.created_at.unwrap_or_default(),
//     is_deleted: false,
//     is_edited: true,
//     is_read: false,
//     is_pinned: false,
//     is_starred: false,
//     is_forwarded: false,
//     is_reply: false,
//     is_reply_to: false,
//   })
// }

// pub async fn delete_message(pool: &PgPool, id: i64) -> Result<(), AppError> {
//   sqlx::query!(
//     r#"
//     DELETE FROM messages WHERE id = $1
//     "#,
//     id
//   )
//   .execute(pool)
//   .await?;

//   Ok(())
// }
