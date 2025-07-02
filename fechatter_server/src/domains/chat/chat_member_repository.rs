use sqlx::{PgPool, Row};
use std::future::Future;
use std::sync::Arc;
use tracing::{info, warn};

use fechatter_core::{
    error::CoreError,
    models::ChatMemberRepository as CoreChatMemberRepository,
    models::{ChatId, ChatMember, ChatType, UserId},
};

/// Production-grade chat membership status - Comprehensive error classification
#[derive(Debug, Clone, PartialEq)]
pub enum ChatMembershipStatus {
    /// User is active member of the chat
    ActiveMember {
        chat_id: i64,
        user_id: i64,
        role: String,
        joined_at: chrono::DateTime<chrono::Utc>,
    },
    /// Chat does not exist in database
    ChatNotFound { chat_id: i64 },
    /// User is not a member (never joined)
    NotMember { chat_id: i64, user_id: i64 },
    /// User was member but left the chat
    UserLeftChat {
        chat_id: i64,
        user_id: i64,
        left_at: chrono::DateTime<chrono::Utc>,
    },
    /// Data inconsistency detected (e.g., creator not in members table)
    DataInconsistency {
        chat_id: i64,
        user_id: i64,
        issue: String,
    },
}

impl ChatMembershipStatus {
    /// Get appropriate HTTP status code for this membership status
    pub fn http_status_code(&self) -> u16 {
        match self {
            Self::ActiveMember { .. } => 200,      // OK
            Self::ChatNotFound { .. } => 404,      // Not Found
            Self::NotMember { .. } => 403,         // Forbidden
            Self::UserLeftChat { .. } => 403,      // Forbidden
            Self::DataInconsistency { .. } => 500, // Internal Server Error
        }
    }

    /// Get user-friendly error message
    pub fn error_message(&self) -> String {
        match self {
            Self::ActiveMember { .. } => "User is active member".to_string(),
            Self::ChatNotFound { chat_id } => format!("Chat {} not found", chat_id),
            Self::NotMember { chat_id, user_id } => {
                format!("User {} is not a member of chat {}", user_id, chat_id)
            }
            Self::UserLeftChat {
                chat_id,
                user_id,
                left_at,
            } => {
                format!(
                    "User {} left chat {} at {}",
                    user_id,
                    chat_id,
                    left_at.format("%Y-%m-%d %H:%M:%S")
                )
            }
            Self::DataInconsistency {
                chat_id,
                user_id,
                issue,
            } => {
                format!(
                    "Data inconsistency for user {} in chat {}: {}",
                    user_id, chat_id, issue
                )
            }
        }
    }

    /// Check if user has access to the chat
    pub fn has_access(&self) -> bool {
        matches!(self, Self::ActiveMember { .. })
    }
}

pub struct ChatMemberRepository {
    pool: Arc<PgPool>,
}

impl ChatMemberRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Check if user is a member of the chat, return error if not
    pub async fn ensure_user_is_chat_member(
        &self,
        chat_id: i64,
        user_id: i64,
    ) -> Result<(), CoreError> {
        let is_member = self.is_user_member(chat_id, user_id).await?;
        if !is_member {
            return Err(CoreError::NotFound(format!(
                "User {} is not a member of chat {}",
                user_id, chat_id
            )));
        }
        Ok(())
    }

    /// Add members implementation - FIXED: Use proper error mapping for foreign key constraints
    async fn add_members_impl(
        &self,
        chat_id: ChatId,
        user_id: UserId,
        member_ids: Vec<UserId>,
    ) -> Result<Vec<ChatMember>, CoreError> {
        let chat_id = i64::from(chat_id);
        let user_id = i64::from(user_id);

        // Check if user has permission to add members
        let is_creator = self
            .is_creator_impl(ChatId(chat_id), UserId(user_id))
            .await?;
        if !is_creator {
            return Err(CoreError::Unauthorized(
                "Only chat creator can add members".to_string(),
            ));
        }

        // Start a transaction to ensure both updates succeed
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

        // Add members to the chat
        for &member_id in &member_ids {
            let member_id = i64::from(member_id);

            // 1. Update the array in chats table
            sqlx::query(
                r#"UPDATE chats SET chat_members = array_append(chat_members, $1)
                   WHERE id = $2 AND NOT $1 = ANY(chat_members)"#,
            )
            .bind(member_id)
            .bind(chat_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

            // 2. Insert into chat_members table with proper role, or reactivate if previously left
            sqlx::query(
                r#"INSERT INTO chat_members (chat_id, user_id, role) 
           VALUES ($1, $2, 'member'::chat_member_role)
           ON CONFLICT (chat_id, user_id) 
           DO UPDATE SET left_at = NULL, role = 'member'::chat_member_role"#,
            )
            .bind(chat_id)
            .bind(member_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| CoreError::from_database_error(e))?;
        }

        // Commit the transaction
        tx.commit()
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

        // Return the current member list
        self.list_members_impl(ChatId(chat_id)).await
    }

    /// Remove members implementation - FIXED: Use proper error mapping
    async fn remove_members_impl(
        &self,
        chat_id: ChatId,
        user_id: UserId,
        member_ids: Vec<UserId>,
    ) -> Result<bool, CoreError> {
        let chat_id = i64::from(chat_id);
        let user_id = i64::from(user_id);

        // Check if user has permission to remove members
        let is_creator = self
            .is_creator_impl(ChatId(chat_id), UserId(user_id))
            .await?;
        if !is_creator {
            return Err(CoreError::Unauthorized(
                "Only chat creator can remove members".to_string(),
            ));
        }

        // Start a transaction to ensure both updates succeed
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

        // Remove members from the chat
        for &member_id in &member_ids {
            let member_id = i64::from(member_id);

            // Don't allow removing the creator
            if member_id == user_id {
                continue;
            }

            // 1. Update the array in chats table
            sqlx::query(
                r#"UPDATE chats SET chat_members = array_remove(chat_members, $1)
                   WHERE id = $2"#,
            )
            .bind(member_id)
            .bind(chat_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

            // 2. Mark as left in chat_members table instead of deleting
            sqlx::query(
                r#"UPDATE chat_members 
           SET left_at = NOW()
           WHERE chat_id = $1 AND user_id = $2 AND left_at IS NULL"#,
            )
            .bind(chat_id)
            .bind(member_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| CoreError::from_database_error(e))?;
        }

        // Commit the transaction
        tx.commit()
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

        Ok(true)
    }

    /// List members implementation - FIXED: Use proper error mapping
    async fn list_members_impl(&self, chat_id: ChatId) -> Result<Vec<ChatMember>, CoreError> {
        let chat_id = i64::from(chat_id);

        let rows = sqlx::query(
            r#"SELECT u.id, c.created_at
         FROM chats c
         JOIN users u ON u.id = ANY(c.chat_members)
         WHERE c.id = $1
         ORDER BY c.created_by = u.id DESC, u.id"#,
        )
        .bind(chat_id)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| CoreError::from_database_error(e))?;

        let mut members = Vec::new();
        for row in rows {
            members.push(ChatMember {
                chat_id: ChatId(chat_id),
                user_id: row
                    .try_get::<i64, _>("id")
                    .map_err(|e| CoreError::from_database_error(e))?
                    .into(),
                joined_at: row
                    .try_get("created_at")
                    .map_err(|e| CoreError::from_database_error(e))?,
            });
        }

        Ok(members)
    }

    /// Check if user is member implementation - FIXED: Use proper error mapping
    async fn is_member_impl(&self, chat_id: ChatId, user_id: UserId) -> Result<bool, CoreError> {
        let chat_id = i64::from(chat_id);
        let user_id = i64::from(user_id);

        let count: i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM chat_members WHERE chat_id = $1 AND user_id = $2 AND left_at IS NULL",
    )
    .bind(chat_id)
    .bind(user_id)
    .fetch_one(&*self.pool)
    .await
    .map_err(|e| CoreError::from_database_error(e))?;

        Ok(count > 0)
    }

    /// Check if user is creator implementation - FIXED: Use proper error mapping
    async fn is_creator_impl(&self, chat_id: ChatId, user_id: UserId) -> Result<bool, CoreError> {
        let chat_id = i64::from(chat_id);
        let user_id = i64::from(user_id);

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM chats WHERE id = $1 AND created_by = $2")
                .bind(chat_id)
                .bind(user_id)
                .fetch_one(&*self.pool)
                .await
                .map_err(|e| CoreError::from_database_error(e))?;

        Ok(count > 0)
    }

    /// Count members implementation - FIXED: Use proper error mapping
    async fn count_members_impl(&self, chat_id: ChatId) -> Result<i64, CoreError> {
        let chat_id = i64::from(chat_id);

        let count: i64 =
            sqlx::query_scalar("SELECT array_length(chat_members, 1) FROM chats WHERE id = $1")
                .bind(chat_id)
                .fetch_one(&*self.pool)
                .await
                .map_err(|e| CoreError::from_database_error(e))?;

        Ok(count)
    }

    /// Get chat type implementation - FIXED: Use proper error mapping
    async fn get_chat_type_impl(&self, chat_id: ChatId) -> Result<ChatType, CoreError> {
        let chat_id = i64::from(chat_id);

        let chat_type_str: String =
            sqlx::query_scalar("SELECT type::text FROM chats WHERE id = $1")
                .bind(chat_id)
                .fetch_one(&*self.pool)
                .await
                .map_err(|e| CoreError::from_database_error(e))?;

        chat_type_str
            .parse::<ChatType>()
            .map_err(|e| CoreError::Internal(format!("Invalid chat type: {}", e)))
    }

    /// Get member count for a chat
    pub async fn get_member_count(&self, chat_id: i64) -> Result<i64, CoreError> {
        self.count_members_impl(ChatId(chat_id)).await
    }

    /// Add members to a chat (convenience method)
    pub async fn add_members(&self, chat_id: i64, member_ids: &[i64]) -> Result<(), CoreError> {
        let member_user_ids: Vec<UserId> = member_ids.iter().map(|&id| UserId(id)).collect();
        self.add_members_impl(ChatId(chat_id), UserId(0), member_user_ids)
            .await?;
        Ok(())
    }

    /// Remove members from a chat (convenience method)
    pub async fn remove_members(&self, chat_id: i64, member_ids: &[i64]) -> Result<(), CoreError> {
        let member_user_ids: Vec<UserId> = member_ids.iter().map(|&id| UserId(id)).collect();
        self.remove_members_impl(ChatId(chat_id), UserId(0), member_user_ids)
            .await?;
        Ok(())
    }

    /// List members of a chat (convenience method)
    pub async fn list_members(&self, chat_id: i64) -> Result<Vec<ChatMember>, CoreError> {
        self.list_members_impl(ChatId(chat_id)).await
    }

    /// Transfer ownership (convenience method)
    pub async fn transfer_ownership(
        &self,
        chat_id: i64,
        from_user_id: i64,
        to_user_id: i64,
    ) -> Result<bool, CoreError> {
        use fechatter_core::models::ChatMemberRepository as CoreChatMemberRepository;

        CoreChatMemberRepository::transfer_ownership(
            self,
            ChatId(chat_id),
            UserId(from_user_id),
            UserId(to_user_id),
        )
        .await
    }
}

// Implement the core ChatMemberRepository trait
impl CoreChatMemberRepository for ChatMemberRepository {
    fn add_members(
        &self,
        chat_id: ChatId,
        user_id: UserId,
        member_ids: Vec<UserId>,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<ChatMember>, CoreError>> + Send>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let chat_id_val = i64::from(chat_id);
            let user_id_val = i64::from(user_id);

            // Check if user has permission to add members
            let is_creator: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM chats WHERE id = $1 AND created_by = $2")
                    .bind(chat_id_val)
                    .bind(user_id_val)
                    .fetch_one(&*pool)
                    .await
                    .map_err(|e| CoreError::from_database_error(e))?;

            if is_creator == 0 {
                return Err(CoreError::Unauthorized(
                    "Only chat creator can add members".to_string(),
                ));
            }

            // Add members to the chat
            for &member_id in &member_ids {
                let member_id_val = i64::from(member_id);
                sqlx::query(r#"UPDATE chats SET chat_members = array_append(chat_members, $1) WHERE id = $2 AND NOT $1 = ANY(chat_members)"#)
          .bind(member_id_val)
          .bind(chat_id_val)
          .execute(&*pool)
          .await
          .map_err(|e| CoreError::from_database_error(e))?;
            }

            // Return the current member list
            let rows = sqlx::query(r#"SELECT u.id, c.created_at FROM chats c JOIN users u ON u.id = ANY(c.chat_members) WHERE c.id = $1 ORDER BY c.created_by = u.id DESC, u.id"#)
        .bind(chat_id_val)
        .fetch_all(&*pool)
        .await
        .map_err(|e| CoreError::from_database_error(e))?;

            let mut members = Vec::new();
            for row in rows {
                members.push(ChatMember {
                    chat_id,
                    user_id: row
                        .try_get::<i64, _>("id")
                        .map_err(|e| CoreError::from_database_error(e))?
                        .into(),
                    joined_at: row
                        .try_get("created_at")
                        .map_err(|e| CoreError::from_database_error(e))?,
                });
            }
            Ok(members)
        })
    }

    fn remove_members(
        &self,
        chat_id: ChatId,
        user_id: UserId,
        member_ids: Vec<UserId>,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<bool, CoreError>> + Send>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let chat_id_val = i64::from(chat_id);
            let user_id_val = i64::from(user_id);

            // Check if user has permission to remove members
            let is_creator: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM chats WHERE id = $1 AND created_by = $2")
                    .bind(chat_id_val)
                    .bind(user_id_val)
                    .fetch_one(&*pool)
                    .await
                    .map_err(|e| CoreError::from_database_error(e))?;

            if is_creator == 0 {
                return Err(CoreError::Unauthorized(
                    "Only chat creator can remove members".to_string(),
                ));
            }

            // Remove members from the chat
            for &member_id in &member_ids {
                let member_id_val = i64::from(member_id);
                if member_id_val == user_id_val {
                    continue;
                } // Don't remove creator

                sqlx::query(
          r#"UPDATE chats SET chat_members = array_remove(chat_members, $1) WHERE id = $2"#,
        )
        .bind(member_id_val)
        .bind(chat_id_val)
        .execute(&*pool)
        .await
        .map_err(|e| CoreError::from_database_error(e))?;

                // FIX: Mark as left in chat_members table instead of deleting
                sqlx::query(
                    r#"UPDATE chat_members 
             SET left_at = NOW()
             WHERE chat_id = $1 AND user_id = $2 AND left_at IS NULL"#,
                )
                .bind(chat_id_val)
                .bind(member_id_val)
                .execute(&*pool)
                .await
                .map_err(|e| CoreError::from_database_error(e))?;
            }
            Ok(true)
        })
    }

    fn list_members(
        &self,
        chat_id: ChatId,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<ChatMember>, CoreError>> + Send>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let chat_id_val = i64::from(chat_id);
            let rows = sqlx::query(r#"SELECT u.id, c.created_at FROM chats c JOIN users u ON u.id = ANY(c.chat_members) WHERE c.id = $1 ORDER BY c.created_by = u.id DESC, u.id"#)
        .bind(chat_id_val)
        .fetch_all(&*pool)
        .await
        .map_err(|e| CoreError::from_database_error(e))?;

            let mut members = Vec::new();
            for row in rows {
                members.push(ChatMember {
                    chat_id,
                    user_id: row
                        .try_get::<i64, _>("id")
                        .map_err(|e| CoreError::from_database_error(e))?
                        .into(),
                    joined_at: row
                        .try_get("created_at")
                        .map_err(|e| CoreError::from_database_error(e))?,
                });
            }
            Ok(members)
        })
    }

    fn is_member(
        &self,
        chat_id: ChatId,
        user_id: UserId,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<bool, CoreError>> + Send>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM chat_members WHERE chat_id = $1 AND user_id = $2 AND left_at IS NULL",
      )
      .bind(i64::from(chat_id))
      .bind(i64::from(user_id))
      .fetch_one(&*pool)
      .await
      .map_err(|e| CoreError::from_database_error(e))?;

            Ok(count > 0)
        })
    }

    fn is_creator(
        &self,
        chat_id: ChatId,
        user_id: UserId,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<bool, CoreError>> + Send>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM chats WHERE id = $1 AND created_by = $2")
                    .bind(i64::from(chat_id))
                    .bind(i64::from(user_id))
                    .fetch_one(&*pool)
                    .await
                    .map_err(|e| CoreError::from_database_error(e))?;
            Ok(count > 0)
        })
    }

    fn count_members(
        &self,
        chat_id: ChatId,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<i64, CoreError>> + Send>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let count: i64 =
                sqlx::query_scalar("SELECT array_length(chat_members, 1) FROM chats WHERE id = $1")
                    .bind(i64::from(chat_id))
                    .fetch_one(&*pool)
                    .await
                    .map_err(|e| CoreError::from_database_error(e))?;
            Ok(count)
        })
    }

    fn get_chat_type(
        &self,
        chat_id: ChatId,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<ChatType, CoreError>> + Send>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let chat_type_str: String =
                sqlx::query_scalar("SELECT type::text FROM chats WHERE id = $1")
                    .bind(i64::from(chat_id))
                    .fetch_one(&*pool)
                    .await
                    .map_err(|e| CoreError::from_database_error(e))?;
            chat_type_str
                .parse::<ChatType>()
                .map_err(|e| CoreError::Internal(format!("Invalid chat type: {}", e)))
        })
    }

    fn transfer_ownership(
        &self,
        chat_id: ChatId,
        from_user_id: UserId,
        to_user_id: UserId,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<bool, CoreError>> + Send>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let chat_id_val = i64::from(chat_id);
            let from_user_id_val = i64::from(from_user_id);
            let to_user_id_val = i64::from(to_user_id);

            // Check if from_user is the current creator
            let is_creator: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM chats WHERE id = $1 AND created_by = $2")
                    .bind(chat_id_val)
                    .bind(from_user_id_val)
                    .fetch_one(&*pool)
                    .await
                    .map_err(|e| CoreError::from_database_error(e))?;

            if is_creator == 0 {
                return Err(CoreError::Unauthorized(
                    "Only current creator can transfer ownership".to_string(),
                ));
            }

            // Check if to_user is a member
            let is_member: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM chats WHERE id = $1 AND $2 = ANY(chat_members)",
            )
            .bind(chat_id_val)
            .bind(to_user_id_val)
            .fetch_one(&*pool)
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

            if is_member == 0 {
                return Err(CoreError::Unauthorized(
                    "Target user must be a member of the chat".to_string(),
                ));
            }

            // Transfer ownership
            let result =
                sqlx::query("UPDATE chats SET created_by = $1 WHERE id = $2 AND created_by = $3")
                    .bind(to_user_id_val)
                    .bind(chat_id_val)
                    .bind(from_user_id_val)
                    .execute(&*pool)
                    .await
                    .map_err(|e| CoreError::from_database_error(e))?;

            Ok(result.rows_affected() > 0)
        })
    }
}

// Convenience methods for server internal use
impl ChatMemberRepository {
    /// Add members (convenience method for server use)
    pub async fn add_chat_members(
        &self,
        chat_id: i64,
        user_id: i64,
        member_ids: Vec<i64>,
    ) -> Result<Vec<ChatMember>, CoreError> {
        let member_user_ids = member_ids.into_iter().map(UserId).collect();
        self.add_members_impl(ChatId(chat_id), UserId(user_id), member_user_ids)
            .await
    }

    /// Remove members (convenience method for server use)
    pub async fn remove_chat_members(
        &self,
        chat_id: i64,
        user_id: i64,
        member_ids: Vec<i64>,
    ) -> Result<bool, CoreError> {
        let member_user_ids = member_ids.into_iter().map(UserId).collect();
        self.remove_members_impl(ChatId(chat_id), UserId(user_id), member_user_ids)
            .await
    }

    /// List chat members (convenience method for server use)
    pub async fn list_chat_members(&self, chat_id: i64) -> Result<Vec<ChatMember>, CoreError> {
        self.list_members_impl(ChatId(chat_id)).await
    }

    /// Check if user is member (convenience method for server use)
    pub async fn is_user_member(&self, chat_id: i64, user_id: i64) -> Result<bool, CoreError> {
        self.is_member_impl(ChatId(chat_id), UserId(user_id)).await
    }

    /// Check if user is creator (convenience method for server use)
    pub async fn is_user_creator(&self, chat_id: i64, user_id: i64) -> Result<bool, CoreError> {
        self.is_creator_impl(ChatId(chat_id), UserId(user_id)).await
    }

    /// Get member's role in the chat
    pub async fn get_member_role(
        &self,
        chat_id: i64,
        user_id: i64,
    ) -> Result<Option<String>, CoreError> {
        let row = sqlx::query(
            "SELECT cm.role 
       FROM chat_members cm 
       WHERE cm.chat_id = $1 AND cm.user_id = $2",
        )
        .bind(chat_id)
        .bind(user_id)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| CoreError::from_database_error(e))?;

        Ok(row.map(|r| r.get("role")))
    }

    /// Enhanced chat existence and membership validation - Production-grade approach
    /// Returns detailed membership status instead of simple boolean
    pub async fn validate_chat_and_membership(
        &self,
        chat_id: i64,
        user_id: i64,
    ) -> Result<ChatMembershipStatus, CoreError> {
        info!("[CHAT_MEMBER_REPO] ========== Enhanced chat membership validation ==========",);
        info!(
            "[CHAT_MEMBER_REPO] Validating: user_id={}, chat_id={}",
            user_id, chat_id
        );

        // Step 1: Check if chat exists
        let chat_exists_query = "SELECT COUNT(*) FROM chats WHERE id = $1";
        let chat_exists: i64 = sqlx::query_scalar(chat_exists_query)
            .bind(chat_id)
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

        if chat_exists == 0 {
            warn!("[CHAT_MEMBER_REPO] ERROR: Chat {} does not exist", chat_id);
            return Ok(ChatMembershipStatus::ChatNotFound { chat_id });
        }

        info!("[CHAT_MEMBER_REPO] Chat {} exists", chat_id);

        // Step 2: Check detailed membership status
        let membership_query = r#"
      SELECT 
        cm.role,
        cm.joined_at,
        cm.left_at,
        c.created_by
      FROM chat_members cm
      INNER JOIN chats c ON c.id = cm.chat_id
      WHERE cm.chat_id = $1 AND cm.user_id = $2
      ORDER BY cm.joined_at DESC
      LIMIT 1
    "#;

        let membership_row = sqlx::query(membership_query)
            .bind(chat_id)
            .bind(user_id)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| CoreError::from_database_error(e))?;

        match membership_row {
            Some(row) => {
                let role: String = row
                    .try_get("role")
                    .map_err(|e| CoreError::from_database_error(e))?;
                let joined_at: chrono::DateTime<chrono::Utc> = row
                    .try_get("joined_at")
                    .map_err(|e| CoreError::from_database_error(e))?;
                let left_at: Option<chrono::DateTime<chrono::Utc>> = row
                    .try_get("left_at")
                    .map_err(|e| CoreError::from_database_error(e))?;

                match left_at {
                    None => {
                        info!(
                            "[CHAT_MEMBER_REPO] User {} is active member of chat {} with role '{}'",
                            user_id, chat_id, role
                        );
                        Ok(ChatMembershipStatus::ActiveMember {
                            chat_id,
                            user_id,
                            role,
                            joined_at,
                        })
                    }
                    Some(left_timestamp) => {
                        warn!(
                            "[CHAT_MEMBER_REPO] WARNING: User {} left chat {} on {}",
                            user_id, chat_id, left_timestamp
                        );
                        Ok(ChatMembershipStatus::UserLeftChat {
                            chat_id,
                            user_id,
                            left_at: left_timestamp,
                        })
                    }
                }
            }
            None => {
                // User was never a member
                warn!(
                    "[CHAT_MEMBER_REPO] ERROR: User {} is not a member of chat {}",
                    user_id, chat_id
                );
                Ok(ChatMembershipStatus::NotMember { chat_id, user_id })
            }
        }
    }

    /// Optimized membership check with caching consideration
    /// Returns true only if user is active member
    pub async fn is_user_member_enhanced(
        &self,
        chat_id: i64,
        user_id: i64,
    ) -> Result<bool, CoreError> {
        let status = self.validate_chat_and_membership(chat_id, user_id).await?;
        Ok(matches!(status, ChatMembershipStatus::ActiveMember { .. }))
    }
}
