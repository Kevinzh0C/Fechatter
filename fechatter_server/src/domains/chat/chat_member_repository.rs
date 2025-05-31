use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::future::Future;
use std::sync::Arc;

use fechatter_core::{
  error::CoreError,
  models::ChatMemberRepository as CoreChatMemberRepository,
  models::{ChatId, ChatMember, ChatType, UserId},
};

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

  /// Add members implementation
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

    // Add members to the chat
    for &member_id in &member_ids {
      let member_id = i64::from(member_id);

      // Check if member is already in the chat
      let is_member = self
        .is_member_impl(ChatId(chat_id), UserId(member_id))
        .await?;
      if !is_member {
        sqlx::query(
          r#"UPDATE chats SET chat_members = array_append(chat_members, $1)
                       WHERE id = $2 AND NOT $1 = ANY(chat_members)"#,
        )
        .bind(member_id)
        .bind(chat_id)
        .execute(&*self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;
      }
    }

    // Return the current member list
    self.list_members_impl(ChatId(chat_id)).await
  }

  /// Remove members implementation
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

    // Remove members from the chat
    for &member_id in &member_ids {
      let member_id = i64::from(member_id);

      // Don't allow removing the creator
      if member_id == user_id {
        continue;
      }

      sqlx::query(
        r#"UPDATE chats SET chat_members = array_remove(chat_members, $1)
                   WHERE id = $2"#,
      )
      .bind(member_id)
      .bind(chat_id)
      .execute(&*self.pool)
      .await
      .map_err(|e| CoreError::Database(e.to_string()))?;
    }

    Ok(true)
  }

  /// List members implementation
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
    .map_err(|e| CoreError::Database(e.to_string()))?;

    let mut members = Vec::new();
    for row in rows {
      members.push(ChatMember {
        chat_id: ChatId(chat_id),
        user_id: row
          .try_get::<i64, _>("id")
          .map_err(|e| CoreError::Database(e.to_string()))?
          .into(),
        joined_at: row
          .try_get("created_at")
          .map_err(|e| CoreError::Database(e.to_string()))?,
      });
    }

    Ok(members)
  }

  /// Check if user is member implementation
  async fn is_member_impl(&self, chat_id: ChatId, user_id: UserId) -> Result<bool, CoreError> {
    let chat_id = i64::from(chat_id);
    let user_id = i64::from(user_id);

    let count: i64 =
      sqlx::query_scalar("SELECT COUNT(*) FROM chats WHERE id = $1 AND $2 = ANY(chat_members)")
        .bind(chat_id)
        .bind(user_id)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

    Ok(count > 0)
  }

  /// Check if user is creator implementation
  async fn is_creator_impl(&self, chat_id: ChatId, user_id: UserId) -> Result<bool, CoreError> {
    let chat_id = i64::from(chat_id);
    let user_id = i64::from(user_id);

    let count: i64 =
      sqlx::query_scalar("SELECT COUNT(*) FROM chats WHERE id = $1 AND created_by = $2")
        .bind(chat_id)
        .bind(user_id)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

    Ok(count > 0)
  }

  /// Count members implementation
  async fn count_members_impl(&self, chat_id: ChatId) -> Result<i64, CoreError> {
    let chat_id = i64::from(chat_id);

    let count: i64 =
      sqlx::query_scalar("SELECT array_length(chat_members, 1) FROM chats WHERE id = $1")
        .bind(chat_id)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

    Ok(count)
  }

  /// Get chat type implementation
  async fn get_chat_type_impl(&self, chat_id: ChatId) -> Result<ChatType, CoreError> {
    let chat_id = i64::from(chat_id);

    let chat_type_str: String = sqlx::query_scalar("SELECT type::text FROM chats WHERE id = $1")
      .bind(chat_id)
      .fetch_one(&*self.pool)
      .await
      .map_err(|e| CoreError::Database(e.to_string()))?;

    chat_type_str
      .parse::<ChatType>()
      .map_err(|e| CoreError::Internal(format!("Invalid chat type: {}", e)))
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
          .map_err(|e| CoreError::Database(e.to_string()))?;

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
          .map_err(|e| CoreError::Database(e.to_string()))?;
      }

      // Return the current member list
      let rows = sqlx::query(r#"SELECT u.id, c.created_at FROM chats c JOIN users u ON u.id = ANY(c.chat_members) WHERE c.id = $1 ORDER BY c.created_by = u.id DESC, u.id"#)
        .bind(chat_id_val)
        .fetch_all(&*pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

      let mut members = Vec::new();
      for row in rows {
        members.push(ChatMember {
          chat_id,
          user_id: row
            .try_get::<i64, _>("id")
            .map_err(|e| CoreError::Database(e.to_string()))?
            .into(),
          joined_at: row
            .try_get("created_at")
            .map_err(|e| CoreError::Database(e.to_string()))?,
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
          .map_err(|e| CoreError::Database(e.to_string()))?;

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
        .map_err(|e| CoreError::Database(e.to_string()))?;
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
        .map_err(|e| CoreError::Database(e.to_string()))?;

      let mut members = Vec::new();
      for row in rows {
        members.push(ChatMember {
          chat_id,
          user_id: row
            .try_get::<i64, _>("id")
            .map_err(|e| CoreError::Database(e.to_string()))?
            .into(),
          joined_at: row
            .try_get("created_at")
            .map_err(|e| CoreError::Database(e.to_string()))?,
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
      let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM chats WHERE id = $1 AND $2 = ANY(chat_members)")
          .bind(i64::from(chat_id))
          .bind(i64::from(user_id))
          .fetch_one(&*pool)
          .await
          .map_err(|e| CoreError::Database(e.to_string()))?;
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
          .map_err(|e| CoreError::Database(e.to_string()))?;
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
          .map_err(|e| CoreError::Database(e.to_string()))?;
      Ok(count)
    })
  }

  fn get_chat_type(
    &self,
    chat_id: ChatId,
  ) -> std::pin::Pin<Box<dyn Future<Output = Result<ChatType, CoreError>> + Send>> {
    let pool = self.pool.clone();
    Box::pin(async move {
      let chat_type_str: String = sqlx::query_scalar("SELECT type::text FROM chats WHERE id = $1")
        .bind(i64::from(chat_id))
        .fetch_one(&*pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;
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
          .map_err(|e| CoreError::Database(e.to_string()))?;

      if is_creator == 0 {
        return Err(CoreError::Unauthorized(
          "Only current creator can transfer ownership".to_string(),
        ));
      }

      // Check if to_user is a member
      let is_member: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM chats WHERE id = $1 AND $2 = ANY(chat_members)")
          .bind(chat_id_val)
          .bind(to_user_id_val)
          .fetch_one(&*pool)
          .await
          .map_err(|e| CoreError::Database(e.to_string()))?;

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
          .map_err(|e| CoreError::Database(e.to_string()))?;

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
    self
      .add_members_impl(ChatId(chat_id), UserId(user_id), member_user_ids)
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
    self
      .remove_members_impl(ChatId(chat_id), UserId(user_id), member_user_ids)
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
}
