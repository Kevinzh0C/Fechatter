use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::sync::Arc;

use fechatter_core::{
    error::CoreError, models::CreateMessage, models::ListMessages, ChatId, Message, MessageId,
    UserId,
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
        // Check for duplicate message using idempotency key
        let existing_message = sqlx::query_as::<_, Message>(
            r#"SELECT id, chat_id, sender_id, content, files,
                      created_at, idempotency_key
               FROM messages WHERE idempotency_key = $1"#,
        )
        .bind(input.idempotency_key)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| CoreError::from_database_error(e))?;

        if let Some(existing) = existing_message {
            // TODO: Publish duplicate message attempt event
            return Ok(existing);
        }

        // Get next sequence number for this chat
        let sequence_number: i64 = sqlx::query_scalar("SELECT next_message_sequence($1)")
            .bind(chat_id)
            .fetch_one(&*pool)
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

        // Create new message with sequence number
        let message = sqlx::query_as::<_, Message>(
      r#"INSERT INTO messages (chat_id, sender_id, content, files, idempotency_key, sequence_number)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, chat_id, sender_id, content, files, 
                         created_at, idempotency_key"#,
    )
    .bind(chat_id)
    .bind(user_id)
    .bind(&input.content)
    .bind(&input.files)
    .bind(input.idempotency_key)
    .bind(sequence_number)
    .fetch_one(&*pool)
    .await
    .map_err(|e| CoreError::from_database_error(e))?;

        Ok(message)
    }

    /// Static list messages method for use in async move blocks
    async fn list_messages_static(
        input: &ListMessages,
        chat_id: i64,
        pool: Arc<PgPool>,
    ) -> Result<Vec<Message>, CoreError> {
        let mut query_builder = sqlx::QueryBuilder::new(
            r#"SELECT id, chat_id, sender_id, content, files,
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
            .map_err(|e| CoreError::from_database_error(e))?;

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
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Message, CoreError>> + Send>>
    {
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

    /// List messages with sender information for a chat
    pub async fn list_messages_with_senders(
        &self,
        input: ListMessages,
        chat_id: i64,
    ) -> Result<
        Vec<(
            Message,
            Option<(i64, String, Option<String>, Option<String>)>,
        )>,
        CoreError,
    > {
        let mut query_builder = sqlx::QueryBuilder::new(
            r#"SELECT m.id, m.chat_id, m.sender_id, m.content, m.files,
                m.created_at, m.idempotency_key,
                u.id as user_id, u.fullname, u.email
         FROM messages m
         LEFT JOIN users u ON m.sender_id = u.id
         WHERE m.chat_id = "#,
        );

        query_builder.push_bind(chat_id);

        if let Some(last_id) = input.last_id {
            query_builder.push(" AND m.id < ").push_bind(last_id);
        }

        query_builder
            .push(" ORDER BY m.created_at DESC LIMIT ")
            .push_bind(input.limit);

        #[derive(sqlx::FromRow)]
        struct MessageWithSender {
            // Message fields
            id: i64,
            chat_id: i64,
            sender_id: i64,
            content: String,
            files: Option<Vec<String>>,
            created_at: chrono::DateTime<chrono::Utc>,
            idempotency_key: Option<uuid::Uuid>,
            // User fields
            user_id: Option<i64>,
            fullname: Option<String>,
            email: Option<String>,
        }

        let rows = query_builder
            .build_query_as::<MessageWithSender>()
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

        let results = rows
            .into_iter()
            .map(|row| {
                let message = Message {
                    id: MessageId(row.id),
                    chat_id: ChatId(row.chat_id),
                    sender_id: UserId(row.sender_id),
                    content: row.content,
                    files: row.files,
                    created_at: row.created_at,
                    idempotency_key: row.idempotency_key,
                };

                // Include sender info if we have at least a user_id from the JOIN
                let sender_info = if row.user_id.is_some() {
                    Some((
                        row.sender_id,
                        row.fullname.unwrap_or_else(|| "Unknown User".to_string()),
                        None, // username doesn't exist in database
                        row.email,
                    ))
                } else {
                    None
                };

                (message, sender_info)
            })
            .collect();

        Ok(results)
    }

    /// Get a message by ID
    pub async fn get_message_by_id(&self, message_id: i64) -> Result<Option<Message>, CoreError> {
        let message = sqlx::query_as::<_, Message>(
            r#"SELECT id, chat_id, sender_id, content, files,
                      created_at, idempotency_key
               FROM messages WHERE id = $1"#,
        )
        .bind(message_id)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| CoreError::from_database_error(e))?;

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
               RETURNING id, chat_id, sender_id, content, files,
                         created_at, idempotency_key"#,
        )
        .bind(new_content)
        .bind(message_id)
        .bind(editor_id)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| CoreError::from_database_error(e))?;

        Ok(message)
    }

    /// Delete a message
    pub async fn delete_message(&self, message_id: i64, user_id: i64) -> Result<(), CoreError> {
        let result = sqlx::query("DELETE FROM messages WHERE id = $1 AND sender_id = $2")
            .bind(message_id)
            .bind(user_id)
            .execute(&*self.pool)
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

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
            .map_err(|e| CoreError::from_database_error(e))?;

        Ok(count)
    }

    /// Get chat members
    pub async fn get_chat_members(&self, chat_id: i64) -> Result<Vec<i64>, CoreError> {
        let members =
            sqlx::query_scalar::<_, i64>("SELECT user_id FROM chat_members WHERE chat_id = $1")
                .bind(chat_id)
                .fetch_all(&*self.pool)
                .await
                .map_err(|e| CoreError::from_database_error(e))?;

        Ok(members)
    }

    /// Get the next sequence number for a chat
    pub async fn get_next_sequence(&self, chat_id: i64) -> Result<i64, CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

        // Use PostgreSQL function to get next sequence atomically
        let sequence: i64 = sqlx::query_scalar("SELECT next_message_sequence($1)")
            .bind(chat_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

        tx.commit()
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

        Ok(sequence)
    }

    /// Create a message with sequence number
    pub async fn create_message_with_sequence(
        &self,
        input: CreateMessage,
        chat_id: i64,
        user_id: i64,
        sequence_number: i64,
    ) -> Result<Message, CoreError> {
        // Check for duplicate message using idempotency key
        if let Some(key) = input.idempotency_key {
            let existing_message = sqlx::query_as::<_, Message>(
                r#"SELECT id, chat_id, sender_id, content, files,
                        created_at, idempotency_key
                 FROM messages WHERE idempotency_key = $1"#,
            )
            .bind(key)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

            if let Some(existing) = existing_message {
                return Ok(existing);
            }
        }

        // Create new message with sequence number
        let message = sqlx::query_as::<_, Message>(
      r#"INSERT INTO messages (chat_id, sender_id, content, files, idempotency_key, sequence_number)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, chat_id, sender_id, content, files, 
                         created_at, idempotency_key"#,
    )
    .bind(chat_id)
    .bind(user_id)
    .bind(&input.content)
    .bind(&input.files)
    .bind(input.idempotency_key)
    .bind(sequence_number)
    .fetch_one(&*self.pool)
    .await
    .map_err(|e| CoreError::from_database_error(e))?;

        Ok(message)
    }

    /// Get messages after a specific sequence number
    pub async fn get_messages_after_sequence(
        &self,
        chat_id: i64,
        after_sequence: i64,
        limit: usize,
    ) -> Result<Vec<Message>, CoreError> {
        let messages = sqlx::query_as::<_, Message>(
            r#"SELECT id, chat_id, sender_id, content, files,
                created_at, idempotency_key
         FROM messages
         WHERE chat_id = $1 AND sequence_number > $2
         ORDER BY sequence_number ASC
         LIMIT $3"#,
        )
        .bind(chat_id)
        .bind(after_sequence)
        .bind(limit as i64)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| CoreError::from_database_error(e))?;

        Ok(messages)
    }

    /// Get recent messages for a user across all their chats
    pub async fn get_user_recent_messages(
        &self,
        user_id: i64,
        limit: usize,
    ) -> Result<Vec<Message>, CoreError> {
        let messages = sqlx::query_as::<_, Message>(
            r#"SELECT m.id, m.chat_id, m.sender_id, m.content, m.files,
                m.created_at, m.idempotency_key
         FROM messages m
         INNER JOIN chat_members cm ON cm.chat_id = m.chat_id
         WHERE cm.user_id = $1
         ORDER BY m.created_at DESC
         LIMIT $2"#,
        )
        .bind(user_id)
        .bind(limit as i64)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| CoreError::from_database_error(e))?;

        Ok(messages)
    }

    /// Get latest sequence number for a chat
    pub async fn get_latest_sequence(&self, chat_id: i64) -> Result<i64, CoreError> {
        let sequence: Option<i64> =
            sqlx::query_scalar("SELECT last_sequence FROM chat_sequences WHERE chat_id = $1")
                .bind(chat_id)
                .fetch_optional(&*self.pool)
                .await
                .map_err(|e| CoreError::from_database_error(e))?;

        Ok(sequence.unwrap_or(0))
    }

    /// Mark a message as delivered for a user
    pub async fn mark_message_delivered(
        &self,
        message_id: i64,
        user_id: i64,
    ) -> Result<(), CoreError> {
        // Insert receipt record
        sqlx::query(
            r#"INSERT INTO message_receipts (message_id, user_id, status, timestamp)
         VALUES ($1, $2, 'delivered', NOW())
         ON CONFLICT (message_id, user_id, status) DO NOTHING"#,
        )
        .bind(message_id)
        .bind(user_id)
        .execute(&*self.pool)
        .await
        .map_err(|e| CoreError::from_database_error(e))?;

        // Update message delivered_at if this is the first delivery
        sqlx::query(
            r#"UPDATE messages 
         SET delivered_at = COALESCE(delivered_at, NOW()),
             status = CASE 
               WHEN status = 'sent' THEN 'delivered'
               ELSE status
             END
         WHERE id = $1"#,
        )
        .bind(message_id)
        .execute(&*self.pool)
        .await
        .map_err(|e| CoreError::from_database_error(e))?;

        Ok(())
    }

    /// Mark a message as read by a user
    pub async fn mark_message_read(&self, message_id: i64, user_id: i64) -> Result<(), CoreError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

        // Insert read receipt
        sqlx::query(
            r#"INSERT INTO message_receipts (message_id, user_id, status, timestamp)
         VALUES ($1, $2, 'read', NOW())
         ON CONFLICT (message_id, user_id, status) DO NOTHING"#,
        )
        .bind(message_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| CoreError::from_database_error(e))?;

        // Update message read status
        sqlx::query(
            r#"UPDATE messages 
         SET read_at = COALESCE(read_at, NOW()),
             status = 'read',
             read_by_users = array_append(COALESCE(read_by_users, ARRAY[]::BIGINT[]), $2)
         WHERE id = $1 
         AND NOT ($2 = ANY(COALESCE(read_by_users, ARRAY[]::BIGINT[])))"#,
        )
        .bind(message_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| CoreError::from_database_error(e))?;

        tx.commit()
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

        Ok(())
    }

    /// Get unread message count for a user in a chat
    pub async fn get_unread_count(&self, chat_id: i64, user_id: i64) -> Result<i64, CoreError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"SELECT COUNT(*) 
         FROM messages m
         WHERE m.chat_id = $1 
         AND m.sender_id != $2
         AND NOT EXISTS (
           SELECT 1 FROM message_receipts mr 
           WHERE mr.message_id = m.id 
           AND mr.user_id = $2 
           AND mr.status = 'read'
         )"#,
        )
        .bind(chat_id)
        .bind(user_id)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| CoreError::from_database_error(e))?;

        Ok(count)
    }

    /// Get read status for messages (for private chat)
    pub async fn get_message_read_status(
        &self,
        message_ids: &[i64],
        chat_id: i64,
    ) -> Result<Vec<(i64, Vec<i64>)>, CoreError> {
        // For private chat, we need to know who read each message
        let results = sqlx::query_as::<_, (i64, Vec<i64>)>(
      r#"SELECT m.id, COALESCE(array_agg(DISTINCT mr.user_id) FILTER (WHERE mr.status = 'read'), ARRAY[]::BIGINT[]) as read_by
         FROM messages m
         LEFT JOIN message_receipts mr ON mr.message_id = m.id AND mr.status = 'read'
         WHERE m.id = ANY($1) AND m.chat_id = $2
         GROUP BY m.id"#,
    )
    .bind(message_ids)
    .bind(chat_id)
    .fetch_all(&*self.pool)
    .await
    .map_err(|e| CoreError::from_database_error(e))?;

        Ok(results)
    }

    /// Mark multiple messages as read (batch operation)
    pub async fn mark_messages_read_batch(
        &self,
        message_ids: &[i64],
        user_id: i64,
    ) -> Result<(), CoreError> {
        if message_ids.is_empty() {
            return Ok(());
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

        // Batch insert read receipts
        let values: Vec<String> = message_ids
            .iter()
            .map(|id| format!("({}, {}, 'read', NOW())", id, user_id))
            .collect();

        let query = format!(
            r#"INSERT INTO message_receipts (message_id, user_id, status, timestamp)
         VALUES {}
         ON CONFLICT (message_id, user_id, status) DO NOTHING"#,
            values.join(", ")
        );

        sqlx::query(&query)
            .execute(&mut *tx)
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

        // Update messages
        sqlx::query(
            r#"UPDATE messages 
         SET read_at = COALESCE(read_at, NOW()),
             status = 'read',
             read_by_users = array_append(COALESCE(read_by_users, ARRAY[]::BIGINT[]), $1)
         WHERE id = ANY($2)
         AND NOT ($1 = ANY(COALESCE(read_by_users, ARRAY[]::BIGINT[])))"#,
        )
        .bind(user_id)
        .bind(message_ids)
        .execute(&mut *tx)
        .await
        .map_err(|e| CoreError::from_database_error(e))?;

        tx.commit()
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

        Ok(())
    }

    // =============================================================================
    // MENTIONS MANAGEMENT
    // =============================================================================

    /// Get mentions for a specific message
    pub async fn get_message_mentions(
        &self,
        message_id: i64,
    ) -> Result<Vec<(i64, String, String, String)>, CoreError> {
        let rows = sqlx::query(
            r#"
      SELECT 
        mm.mentioned_user_id as user_id,
        u.username,
        u.fullname,
        mm.mention_type
      FROM message_mentions mm
      JOIN users u ON u.id = mm.mentioned_user_id
      WHERE mm.message_id = $1
      ORDER BY mm.created_at ASC
      "#,
        )
        .bind(message_id)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| CoreError::from_database_error(e))?;

        let mentions = rows
            .into_iter()
            .map(|row| {
                (
                    row.get("user_id"),
                    row.get("username"),
                    row.get("fullname"),
                    row.get("mention_type"),
                )
            })
            .collect();

        Ok(mentions)
    }

    /// Get unread mentions for a user across all chats
    pub async fn get_unread_mentions_for_user(
        &self,
        user_id: i64,
    ) -> Result<
        Vec<(
            i64,
            i64,
            String,
            String,
            chrono::DateTime<chrono::Utc>,
            String,
        )>,
        CoreError,
    > {
        let rows = sqlx::query(
            r#"
      SELECT DISTINCT
        m.chat_id,
        m.id as message_id,
        m.content,
        u.fullname as sender_name,
        m.created_at,
        mm.mention_type
      FROM message_mentions mm
      JOIN messages m ON m.id = mm.message_id
      JOIN users u ON u.id = m.sender_id
      JOIN chat_members cm ON cm.chat_id = m.chat_id AND cm.user_id = $1
      WHERE mm.mentioned_user_id = $1
      AND (cm.last_read_message_id IS NULL OR m.id > cm.last_read_message_id)
      AND cm.left_at IS NULL
      ORDER BY m.created_at DESC
      LIMIT 50
      "#,
        )
        .bind(user_id)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| CoreError::from_database_error(e))?;

        let mentions = rows
            .into_iter()
            .map(|row| {
                (
                    row.get("chat_id"),
                    row.get("message_id"),
                    row.get("content"),
                    row.get("sender_name"),
                    row.get("created_at"),
                    row.get("mention_type"),
                )
            })
            .collect();

        Ok(mentions)
    }

    // =============================================================================
    // DETAILED RECEIPTS MANAGEMENT
    // =============================================================================

    /// Get detailed read receipts for a message
    pub async fn get_detailed_message_receipts(
        &self,
        message_id: i64,
    ) -> Result<Vec<(i64, String, String, String, chrono::DateTime<chrono::Utc>)>, CoreError> {
        let rows = sqlx::query(
            r#"
      SELECT 
        mr.user_id,
        u.username,
        u.fullname,
        mr.status,
        mr.timestamp
      FROM message_receipts mr
      JOIN users u ON u.id = mr.user_id
      WHERE mr.message_id = $1
      ORDER BY mr.timestamp DESC, mr.status DESC
      "#,
        )
        .bind(message_id)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| CoreError::from_database_error(e))?;

        let receipts = rows
            .into_iter()
            .map(|row| {
                (
                    row.get("user_id"),
                    row.get("username"),
                    row.get("fullname"),
                    row.get("status"),
                    row.get("timestamp"),
                )
            })
            .collect();

        Ok(receipts)
    }

    // =============================================================================
    // ENHANCED READ TRACKING
    // =============================================================================

    /// Mark message as read with enhanced tracking (handles mentions)
    pub async fn mark_message_read_enhanced(
        &self,
        user_id: i64,
        chat_id: i64,
        message_id: i64,
    ) -> Result<(), CoreError> {
        sqlx::query("SELECT update_last_read_message_with_mentions($1, $2, $3)")
            .bind(user_id)
            .bind(chat_id)
            .bind(message_id)
            .execute(&*self.pool)
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

        Ok(())
    }
}
