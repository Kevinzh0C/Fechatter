use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::sync::Arc;

use fechatter_core::{
    contracts::ChatRepository as CoreChatRepository,
    error::CoreError,
    models::{Chat, ChatId, ChatSidebar, CreateChat, UpdateChat, UserId},
};

pub struct ChatRepository {
    pool: Arc<PgPool>,
}

impl ChatRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Create a new chat (implementation for both trait and direct use)
    async fn create_impl(
        &self,
        input: &CreateChat,
        created_by: UserId,
        workspace_id: Option<i64>,
    ) -> Result<Chat, CoreError> {
        // Validate chat name
        fechatter_core::models::chat::validate_chat_name(&input.name)?;

        // Process members based on chat type
        let members = fechatter_core::models::chat::process_chat_members(
            &input.chat_type,
            created_by,
            input.members.as_ref(),
        )?;

        // Start transaction to ensure both chat and member records are created atomically
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        // Create the chat
        let chat = sqlx::query_as::<_, Chat>(
      r#"INSERT INTO chats (chat_name, type, description, created_by, workspace_id, chat_members)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, workspace_id, chat_name as name, 
                         type as chat_type, chat_members, description, 
                         created_by, created_at, updated_at"#,
    )
    .bind(&input.name)
    .bind(&input.chat_type as &fechatter_core::ChatType)
    .bind(&input.description)
    .bind(i64::from(created_by))
    .bind(workspace_id)
    .bind(
      &members
        .iter()
        .map(|&id| i64::from(id))
        .collect::<Vec<i64>>(),
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| CoreError::Database(e.to_string()))?;

        // Insert all members into chat_members table with appropriate roles
        let chat_id = i64::from(chat.id);
        let created_by_id = i64::from(created_by);

        for &member_id in &members {
            let member_id_val = i64::from(member_id);
            let role = if member_id_val == created_by_id {
                "owner" // Creator gets owner role
            } else {
                "member" // Others get member role
            };

            sqlx::query(
                r#"INSERT INTO chat_members (chat_id, user_id, role) 
           VALUES ($1, $2, $3::chat_member_role)
           ON CONFLICT (chat_id, user_id) DO NOTHING"#,
            )
            .bind(chat_id)
            .bind(member_id_val)
            .bind(role)
            .execute(&mut *tx)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;
        }

        // Commit transaction
        tx.commit()
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(chat)
    }

    /// Get sidebar chats for user (implementation for both trait and direct use)
    async fn get_sidebar_impl(&self, user_id: UserId) -> Result<Vec<ChatSidebar>, CoreError> {
        let user_id_i64 = i64::from(user_id);

        let rows = sqlx::query(
        r#"WITH LastMessages AS (
            SELECT 
                chat_id,
                id AS last_message_id,
                content AS last_message_content,
                sender_id,
                created_at AS last_message_created_at,
                files,
                ROW_NUMBER() OVER(PARTITION BY chat_id ORDER BY created_at DESC) as rn
            FROM messages
        )
        SELECT
            c.id,
            CASE
                WHEN c.type = 'Single' THEN COALESCE(other_user.fullname, c.chat_name)
                ELSE c.chat_name
            END AS name,
            c.type::text AS chat_type,
            c.created_by,
            c.chat_members,
            lm.last_message_id,
            lm.last_message_content,
            sender.fullname AS last_sender_name,
            lm.last_message_created_at,
            CASE WHEN lm.files IS NOT NULL AND array_length(lm.files, 1) > 0 THEN true ELSE false END AS has_files
        FROM chats c
        LEFT JOIN LastMessages lm ON c.id = lm.chat_id AND lm.rn = 1
        LEFT JOIN users sender ON lm.sender_id = sender.id
        LEFT JOIN users other_user ON c.type = 'Single' AND other_user.id = (
            SELECT mem_id FROM unnest(c.chat_members) AS mem_id WHERE mem_id != $1 LIMIT 1
        )
        WHERE $1 = ANY(c.chat_members)
        ORDER BY COALESCE(lm.last_message_created_at, c.updated_at) DESC
        "#,
    )
    .bind(user_id_i64)
    .fetch_all(&*self.pool)
    .await
    .map_err(|e| CoreError::Database(e.to_string()))?;

        let mut chats = Vec::new();
        for row in rows {
            let chat_type_str: String = row
                .try_get("chat_type")
                .map_err(|e| CoreError::Database(e.to_string()))?;
            let chat_type = chat_type_str
                .parse::<fechatter_core::ChatType>()
                .map_err(|e| CoreError::Internal(format!("Invalid chat type: {}", e)))?;

            let created_by: i64 = row
                .try_get("created_by")
                .map_err(|e| CoreError::Database(e.to_string()))?;
            let is_creator = created_by == user_id_i64;

            let last_message =
                if let Ok(message_id) = row.try_get::<Option<i64>, _>("last_message_id") {
                    if let Some(id) = message_id {
                        Some(fechatter_core::models::chat::ChatLastMessage {
                            id: id.into(),
                            content: row
                                .try_get("last_message_content")
                                .map_err(|e| CoreError::Database(e.to_string()))?,
                            sender_name: row
                                .try_get("last_sender_name")
                                .map_err(|e| CoreError::Database(e.to_string()))?,
                            created_at: row
                                .try_get("last_message_created_at")
                                .map_err(|e| CoreError::Database(e.to_string()))?,
                            has_files: row
                                .try_get("has_files")
                                .map_err(|e| CoreError::Database(e.to_string()))?,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                };

            let member_ids_i64: Vec<i64> = row
                .try_get("chat_members")
                .map_err(|e| CoreError::Database(e.to_string()))?;
            let chat_members: Vec<UserId> = member_ids_i64.into_iter().map(UserId).collect();

            chats.push(ChatSidebar {
                id: row
                    .try_get::<i64, _>("id")
                    .map_err(|e| CoreError::Database(e.to_string()))?
                    .into(),
                name: row
                    .try_get("name")
                    .map_err(|e| CoreError::Database(e.to_string()))?,
                chat_type,
                last_message,
                is_creator,
                chat_members,
            });
        }

        Ok(chats)
    }

    /// Find chat by ID (implementation)
    async fn find_by_id_impl(&self, id: ChatId) -> Result<Option<Chat>, CoreError> {
        let chat_id = i64::from(id);

        let chat = sqlx::query_as::<_, Chat>(
            r#"SELECT id, workspace_id, chat_name as name, 
                type as chat_type, chat_members, description, 
                created_by, created_at, updated_at
               FROM chats WHERE id = $1"#,
        )
        .bind(chat_id)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(chat)
    }
}

// Implement the core ChatRepository trait
#[async_trait]
impl CoreChatRepository for ChatRepository {
    async fn create(&self, chat_data: &CreateChat) -> Result<Chat, CoreError> {
        // For now, use default created_by and workspace_id until we fix the core trait
        let created_by = UserId(1); // TODO: This should come from the trait signature
        self.create_impl(chat_data, created_by, None).await
    }

    async fn find_by_id(&self, id: ChatId) -> Result<Option<Chat>, CoreError> {
        self.find_by_id_impl(id).await
    }

    async fn get_user_chats(&self, user_id: UserId) -> Result<Vec<ChatSidebar>, CoreError> {
        self.get_sidebar_impl(user_id).await
    }

    async fn update(&self, id: ChatId, chat_data: &UpdateChat) -> Result<Chat, CoreError> {
        let chat_id = i64::from(id);

        let mut updates = Vec::new();
        let mut binds = Vec::new();

        if let Some(name) = &chat_data.name {
            updates.push("chat_name = $");
            binds.push(name.as_str());
        }

        if let Some(description) = &chat_data.description {
            updates.push("description = $");
            binds.push(description.as_str());
        }

        if updates.is_empty() {
            return self
                .find_by_id_impl(id)
                .await?
                .ok_or_else(|| CoreError::NotFound("Chat not found".to_string()));
        }

        let update_clause = updates
            .iter()
            .enumerate()
            .map(|(i, field)| format!("{}{}", field, i + 2))
            .collect::<Vec<_>>()
            .join(", ");

        let query = format!(
            r#"UPDATE chats SET {}, updated_at = NOW()
         WHERE id = $1
         RETURNING id, workspace_id, chat_name as name, 
                   type as chat_type, chat_members, description, 
                   created_by, created_at, updated_at"#,
            update_clause
        );

        let mut query_builder = sqlx::query_as::<_, Chat>(&query).bind(chat_id);

        for bind in binds {
            query_builder = query_builder.bind(bind);
        }

        let chat = query_builder
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(chat)
    }

    async fn delete(&self, id: ChatId) -> Result<bool, CoreError> {
        let chat_id = i64::from(id);

        let result = sqlx::query("DELETE FROM chats WHERE id = $1")
            .bind(chat_id)
            .execute(&*self.pool)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }
}

// Convenience methods for server internal use
impl ChatRepository {
    /// Create a new chat (convenience method for server use)
    pub async fn create_chat(
        &self,
        input: CreateChat,
        created_by: i64,
        workspace_id: Option<i64>,
    ) -> Result<Chat, CoreError> {
        self.create_impl(&input, UserId(created_by), workspace_id)
            .await
    }

    /// Get sidebar chats for user (convenience method for server use)
    pub async fn get_sidebar_chats(&self, user_id: i64) -> Result<Vec<ChatSidebar>, CoreError> {
        self.get_sidebar_impl(UserId(user_id)).await
    }

    /// Find chat by ID (convenience method for server use)
    pub async fn find_chat_by_id(&self, id: i64) -> Result<Option<Chat>, CoreError> {
        self.find_by_id_impl(ChatId(id)).await
    }

    /// Update chat name
    pub async fn update_chat_name(
        &self,
        chat_id: i64,
        user_id: i64,
        new_name: &str,
    ) -> Result<Chat, CoreError> {
        let chat_id = i64::from(chat_id);
        let user_id = i64::from(user_id);

        let chat = sqlx::query_as::<_, Chat>(
            r#"UPDATE chats SET chat_name = $1, updated_at = NOW()
                 WHERE id = $2 AND (created_by = $3 OR $3 = ANY(chat_members))
                 RETURNING id, workspace_id, chat_name as name, 
                           type as chat_type, chat_members, description, 
                           created_by, created_at, updated_at"#,
        )
        .bind(new_name)
        .bind(chat_id)
        .bind(user_id)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(chat)
    }

    /// Update chat description
    pub async fn update_chat_description(
        &self,
        chat_id: i64,
        user_id: i64,
        new_description: &str,
    ) -> Result<Chat, CoreError> {
        let chat_id = i64::from(chat_id);
        let user_id = i64::from(user_id);

        let chat = sqlx::query_as::<_, Chat>(
            r#"UPDATE chats SET description = $1, updated_at = NOW()
                 WHERE id = $2 AND (created_by = $3 OR $3 = ANY(chat_members))
                 RETURNING id, workspace_id, chat_name as name, 
                           type as chat_type, chat_members, description, 
                           created_by, created_at, updated_at"#,
        )
        .bind(new_description)
        .bind(chat_id)
        .bind(user_id)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(chat)
    }

    /// Delete chat
    pub async fn delete_chat(&self, chat_id: i64, user_id: i64) -> Result<(), CoreError> {
        let chat_id = i64::from(chat_id);
        let user_id = i64::from(user_id);

        let result = sqlx::query("DELETE FROM chats WHERE id = $1 AND created_by = $2")
            .bind(chat_id)
            .bind(user_id)
            .execute(&*self.pool)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(())
    }
}
