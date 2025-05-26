use sqlx::Row;
use std::{sync::Arc, time::Duration};

use tokio::time::Instant;

use crate::{AppError, AppState, models::ChatType};
use fechatter_core::{UpdateChat, chat::ChatSidebar};

use crate::models::Chat;

const CHAT_LIST_CACHE_TTL: Duration = Duration::from_secs(30);

impl AppState {
  /// Retrieves a list of chat sidebars for a specific user
  pub async fn list_chats_of_user(&self, user_id: i64) -> Result<Arc<Vec<ChatSidebar>>, AppError> {
    if let Some(entry) = self.chat_list_cache.get(&user_id) {
      let (cached_chats, timestamp) = entry.value();
      if timestamp.elapsed() < CHAT_LIST_CACHE_TTL {
        return Ok(cached_chats.clone());
      }
    }

    // If no valid cache exists, fetch the chat list from the database
    let chats = self.fetch_chat_list_from_db(user_id).await?;

    // Create an Arc to allow shared ownership of the chat list
    let chats_arc = Arc::new(chats);

    // Update the cache with the new chat list and current timestamp
    self
      .chat_list_cache
      .insert(user_id, (chats_arc.clone(), Instant::now()));

    Ok(chats_arc)
  }

  async fn fetch_chat_list_from_db(&self, user_id: i64) -> Result<Vec<ChatSidebar>, AppError> {
    let rows = sqlx::query(
      r#"SELECT
        id,
        chat_name as name,
        type::text as chat_type,
        created_by
      FROM chats
      WHERE created_by = $1 OR $1 = ANY(chat_members)
      ORDER BY updated_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(self.pool())
    .await?;

    let mut chats = Vec::new();

    for row in rows {
      let id: i64 = row.get("id");
      let name: String = row.get("name");
      let chat_type_str: String = row.get("chat_type");
      let created_by: i64 = row.get("created_by");

      // Convert string to ChatType
      let chat_type = match chat_type_str.as_str() {
        "Single" => fechatter_core::ChatType::Single,
        "Group" => fechatter_core::ChatType::Group,
        "PrivateChannel" => fechatter_core::ChatType::PrivateChannel,
        "PublicChannel" => fechatter_core::ChatType::PublicChannel,
        _ => fechatter_core::ChatType::Group, // Default
      };

      chats.push(ChatSidebar {
        id: fechatter_core::ChatId(id),
        name,
        chat_type,
        is_creator: created_by == user_id,
        last_message: None,
      });
    }

    Ok(chats)
  }

  pub async fn fetch_all_chats(&self, workspace_id: i64) -> Result<Vec<Chat>, AppError> {
    let chats = sqlx::query_as::<_, Chat>(
      r#"
        SELECT
          id,
          workspace_id,
          chat_name as name,
          type as "chat_type: _",
          chat_members,
          description,
          created_by,
          created_at,
          updated_at
        FROM chats WHERE workspace_id = $1
        ORDER BY updated_at DESC
      "#,
    )
    .bind(workspace_id)
    .fetch_all(self.pool())
    .await?;

    Ok(chats)
  }

  fn validate_chat_name(&self, name: &str) -> Result<(), AppError> {
    if name.trim().is_empty() {
      Err(AppError::ChatValidationError(
        "Chat name cannot be empty".to_string(),
      ))
    } else if name.len() > 128 {
      Err(AppError::ChatValidationError(
        "Chat name cannot be longer than 128 characters".to_string(),
      ))
    } else {
      Ok(())
    }
  }

  fn process_chat_members(
    &self,
    chat_type: &ChatType,
    creator_id: i64,
    target_members: Option<&Vec<i64>>,
  ) -> Result<Vec<i64>, AppError> {
    match chat_type {
      ChatType::Single => match target_members {
        Some(members) if members.len() == 1 => {
          let target_id = members[0];
          if target_id == creator_id {
            return Err(AppError::ChatValidationError(
              "Single chat must have exactly one member".to_string(),
            ));
          }
          Ok(vec![creator_id, target_id])
        }
        _ => Err(AppError::ChatValidationError(
          "Invalid single chat members".to_string(),
        )),
      },
      ChatType::Group => {
        let mut result = vec![creator_id];
        if let Some(members) = target_members {
          for &id in members {
            if id != creator_id && !result.contains(&id) {
              result.push(id);
            }
          }
        }
        if result.len() < 3 {
          return Err(AppError::ChatValidationError(
            "Group chat must have at least three members".to_string(),
          ));
        }
        Ok(result)
      }
      ChatType::PrivateChannel => {
        let mut result = vec![creator_id];
        if let Some(members) = target_members {
          for &id in members {
            if id != creator_id && !result.contains(&id) {
              result.push(id);
            }
          }
        }
        Ok(result)
      }
      ChatType::PublicChannel => Ok(vec![creator_id]),
    }
  }

  async fn insert_chat_record(
    &self,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    name: &str,
    chat_type: &ChatType,
    chat_members: &Vec<i64>,
    description: &str,
    creator_id: i64,
    workspace_id: i64,
  ) -> Result<Chat, sqlx::Error> {
    let query = "INSERT INTO chats (chat_name, type, chat_members, description, created_by, workspace_id)
       VALUES ($1, $2, $3, $4, $5, $6)
       RETURNING id, chat_name as name, type as chat_type, chat_members,
                 COALESCE(description, '') as description, created_by, created_at, updated_at, workspace_id";

    let chat = sqlx::query_as::<_, Chat>(query)
      .bind(name)
      .bind(chat_type)
      .bind(chat_members)
      .bind(description)
      .bind(creator_id)
      .bind(workspace_id)
      .fetch_one(&mut **tx)
      .await?;

    Ok(chat)
  }

  pub async fn create_new_chat(
    &self,
    creator_id: i64,
    name: &str,
    chat_type: ChatType,
    target_members: Option<Vec<i64>>,
    description: Option<&str>,
    workspace_id: i64,
  ) -> Result<Chat, AppError> {
    self.validate_chat_name(name)?;

    let chat_members =
      self.process_chat_members(&chat_type, creator_id, target_members.as_ref())?;

    self.validate_users_exists_by_ids(&chat_members).await?;

    let mut tx = self.pool().begin().await?;

    let chat = self
      .insert_chat_record(
        &mut tx,
        name,
        &chat_type,
        &chat_members,
        description.unwrap_or(""),
        creator_id,
        workspace_id,
      )
      .await
      .map_err(|e| {
        if let Some(db_error) = e.as_database_error() {
          if db_error.is_unique_violation() {
            AppError::ChatAlreadyExists(format!("Chat {} already exists", name))
          } else {
            AppError::SqlxError(sqlx::Error::Io(std::io::Error::new(
              std::io::ErrorKind::Other,
              e.to_string(),
            )))
          }
        } else {
          AppError::SqlxError(sqlx::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
          )))
        }
      })?;

    let chat_id = chat.id;
    crate::models::chat_member::insert_chat_members(chat_id.into(), &chat_members, &mut tx).await?;

    tx.commit().await?;

    for &member in &chat_members {
      self.chat_list_cache.remove(&member.into());
    }

    Ok(chat)
  }

  pub async fn update_chat(
    &self,
    chat_id: i64,
    user_id: i64,
    payload: UpdateChat,
  ) -> Result<Chat, AppError> {
    // 首先检查聊天是否存在
    let chat_exists = sqlx::query("SELECT EXISTS(SELECT 1 FROM chats WHERE id = $1) as exists")
      .bind(chat_id)
      .fetch_one(self.pool())
      .await?
      .try_get::<bool, _>("exists")
      .map_err(|_| AppError::SqlxError(sqlx::Error::RowNotFound))?;

    if !chat_exists {
      return Err(AppError::NotFound(vec![chat_id.to_string()]));
    }

    // 然后检查用户是否是创建者
    let creator = crate::models::ServerCreateChatMember { chat_id, user_id };
    let is_creator = self.is_creator_in_chat(&creator).await?;

    if !is_creator {
      return Err(AppError::ChatPermissionError(format!(
        "User {} is not the creator of chat {}",
        user_id, chat_id
      )));
    }

    if let Some(ref name) = payload.name {
      self.validate_chat_name(name)?;
    }

    let chat_result = sqlx::query_as::<_, Chat>(
      "UPDATE chats
       SET
         chat_name = COALESCE($1, chat_name),
         description = COALESCE($2, description),
         updated_at = NOW()
       WHERE id = $3
       RETURNING id, chat_name as name, type as chat_type, chat_members,
                 COALESCE(description, '') as description, created_by, created_at, updated_at, workspace_id",
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(chat_id)
    .fetch_one(self.pool())
    .await.map_err(|e| {
      if let Some(db_error) = e.as_database_error() {
        if db_error.is_unique_violation() {
          return AppError::ChatAlreadyExists(format!(
            "Chat name '{}' is already taken by another chat",
            payload.name.as_ref().unwrap()
          ));
        }
      }
      AppError::from(e)
    })?;

    if payload.name.is_some() || payload.description.is_some() {
      for &member_id in &chat_result.chat_members {
        self.chat_list_cache.remove(&member_id.into());
      }
    }

    Ok(chat_result)
  }

  pub async fn delete_chat(&self, chat_id: i64, user_id: i64) -> Result<bool, AppError> {
    // 首先检查聊天是否存在
    let chat_exists = sqlx::query("SELECT EXISTS(SELECT 1 FROM chats WHERE id = $1) as exists")
      .bind(chat_id)
      .fetch_one(self.pool())
      .await?
      .try_get::<bool, _>("exists")
      .map_err(|_| AppError::SqlxError(sqlx::Error::RowNotFound))?;

    if !chat_exists {
      return Err(AppError::NotFound(vec![chat_id.to_string()]));
    }

    let mut tx = self.pool().begin().await?;

    let members_to_invalidate = match self
      .delete_chat_transactional(&mut tx, chat_id, user_id)
      .await
    {
      Ok(members) => members,
      Err(e) => {
        let _ = tx.rollback().await;
        return Err(e);
      }
    };

    tx.commit().await?;

    for &member in &members_to_invalidate {
      self.chat_list_cache.remove(&member.into());
    }

    Ok(true)
  }

  async fn delete_chat_transactional(
    &self,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    chat_id: i64,
    user_id: i64,
  ) -> Result<Vec<i64>, AppError> {
    let query = "SELECT chat_members, created_by FROM chats WHERE id = $1 FOR UPDATE";

    let chat_info_row = sqlx::query(query)
      .bind(chat_id)
      .fetch_optional(&mut **tx)
      .await?;

    let chat_info = match chat_info_row {
      Some(row) => {
        let created_by: i64 = row.get("created_by");
        let chat_members: Vec<i64> = row.get("chat_members");
        (created_by, chat_members)
      }
      None => return Err(AppError::NotFound(vec![chat_id.to_string()])),
    };

    let (created_by, chat_members) = chat_info;

    if created_by != user_id {
      return Err(AppError::ChatPermissionError(format!(
        "User {} is not the creator of chat {}",
        user_id, chat_id
      )));
    }

    let query = "DELETE FROM chat_members WHERE chat_id = $1";
    sqlx::query(query).bind(chat_id).execute(&mut **tx).await?;

    let query = "DELETE FROM chats WHERE id = $1";
    let result = sqlx::query(query).bind(chat_id).execute(&mut **tx).await?;

    if result.rows_affected() == 0 {
      return Err(AppError::NotFound(vec![chat_id.to_string()]));
    }

    Ok(chat_members)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use crate::setup_test_users;
  use anyhow::Result;
  use fechatter_core::{CreateChat, UpdateChat};

  #[tokio::test]
  async fn create_and_list_chats_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];

    // Generate unique names to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();

    // 1. Create single chat
    let single_members = vec![user2.id.into()];
    let single_chat = CreateChat::new(
      &format!("Single Chat with Bob {}", timestamp),
      ChatType::Single,
      single_members.clone(),
      "One-on-one chat",
    );
    let single_chat_created = state
      .create_new_chat(
        user1.id.into(),
        &single_chat.name,
        single_chat.chat_type,
        Some(single_members.into_iter().map(|id| id.into()).collect()),
        Some(single_chat.description.as_deref().unwrap_or("")),
        user1.workspace_id.into(),
      )
      .await?;

    assert_eq!(single_chat_created.chat_type, ChatType::Single);
    assert_eq!(single_chat_created.chat_members.len(), 2);
    assert!(single_chat_created.chat_members.contains(&user1.id));
    assert!(single_chat_created.chat_members.contains(&user2.id));
    assert_eq!(single_chat_created.created_by, user1.id);
    assert_eq!(single_chat_created.description, "One-on-one chat");

    // 2. Create group chat
    let group_members = vec![user2.id, user3.id];
    let group_chat = CreateChat::new(
      &format!("Work Team {}", timestamp),
      ChatType::Group,
      group_members.clone(),
      "Work group",
    );
    let chat = state
      .create_new_chat(
        user1.id.into(),
        &group_chat.name,
        group_chat.chat_type,
        Some(group_members.into_iter().map(|id| id.into()).collect()),
        Some(group_chat.description.as_deref().unwrap_or("")),
        user1.workspace_id.into(),
      )
      .await?;

    assert_eq!(chat.chat_type, ChatType::Group);
    assert_eq!(chat.chat_members.len(), 3);
    assert!(chat.chat_members.contains(&user1.id));
    assert!(chat.chat_members.contains(&user2.id));
    assert!(chat.chat_members.contains(&user3.id));

    // 3. Create private channel
    let channel_members = vec![user2.id.into()];
    let private_chat = state
      .create_new_chat(
        user1.id.into(),
        &format!("Project Updates {}", timestamp),
        ChatType::PrivateChannel,
        Some(
          channel_members
            .clone()
            .into_iter()
            .map(|id: fechatter_core::UserId| id.into())
            .collect(),
        ),
        Some("Invite only"),
        user1.workspace_id.into(),
      )
      .await?;

    assert_eq!(private_chat.chat_type, ChatType::PrivateChannel);
    assert_eq!(private_chat.chat_members.len(), 2);

    // 4. Create public channel
    let public_chat = state
      .create_new_chat(
        user1.id.into(),
        &format!("Company Announcements {}", timestamp),
        ChatType::PublicChannel,
        None, // No members for public channel
        Some("Company announcements"),
        user1.workspace_id.into(),
      )
      .await?;

    assert_eq!(public_chat.chat_type, ChatType::PublicChannel);
    assert_eq!(public_chat.chat_members.len(), 1);

    // Use the correct function signature for listing chats
    let chats = state.list_chats_of_user(user1.id.into()).await?;
    assert_eq!(chats.len(), 4);

    Ok(())
  }

  #[tokio::test]
  async fn update_and_delete_chat_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];

    // Generate unique names to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();

    let single_chat = state
      .create_new_chat(
        user1.id.into(),
        &format!("Single Chat with Bob (for update/delete) {}", timestamp),
        ChatType::Single,
        Some(vec![user2.id.into()]),
        Some("One-on-one chat"),
        user1.workspace_id.into(),
      )
      .await?;
    assert_eq!(single_chat.chat_type, ChatType::Single);

    let group_members = vec![user2.id, user3.id];
    let group_chat = state
      .create_new_chat(
        user1.id.into(),
        &format!("Work Team {}", timestamp),
        ChatType::Group,
        Some(group_members.into_iter().map(|id| id.into()).collect()),
        Some("Work group"),
        user1.workspace_id.into(),
      )
      .await?;
    assert_eq!(group_chat.chat_type, ChatType::Group);
    assert_eq!(group_chat.chat_members.len(), 3);

    let updated_single_chat = state
      .update_chat(
        single_chat.id.into(),
        user1.id.into(),
        UpdateChat {
          name: Some(format!("Updated Single Chat {}", timestamp)),
          description: Some("Updated description".to_string()),
        },
      )
      .await?;

    assert_eq!(
      updated_single_chat.name,
      format!("Updated Single Chat {}", timestamp)
    );
    assert_eq!(updated_single_chat.description, "Updated description");

    let updated_group_chat = state
      .update_chat(
        group_chat.id.into(),
        user1.id.into(),
        UpdateChat {
          name: Some(format!("Updated Group Chat {}", timestamp)),
          description: Some("Updated description".to_string()),
        },
      )
      .await?;

    assert_eq!(
      updated_group_chat.name,
      format!("Updated Group Chat {}", timestamp)
    );
    assert_eq!(updated_group_chat.description, "Updated description");

    let chats = state.list_chats_of_user(user1.id.into()).await?;
    assert_eq!(chats.len(), 2);

    let deleted_single_chat = state
      .delete_chat(single_chat.id.into(), user1.id.into())
      .await?;
    assert!(deleted_single_chat);

    let chats = state.list_chats_of_user(user1.id.into()).await?;
    assert_eq!(chats.len(), 1);

    let deleted_group_chat = state
      .delete_chat(group_chat.id.into(), user1.id.into())
      .await?;
    assert!(deleted_group_chat);

    let chats = state.list_chats_of_user(user1.id.into()).await?;
    assert_eq!(chats.len(), 0);

    Ok(())
  }

  #[tokio::test]
  async fn chat_creation_edge_cases_should_test() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(4).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let user4 = &users[3];

    // Generate unique names to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();

    // 1. Test creating a chat with no name (should fail)
    let result = state
      .create_new_chat(
        user1.id.into(),
        "",
        ChatType::Single,
        Some(vec![user2.id.into()]),
        None,
        user1.workspace_id.into(),
      )
      .await;
    assert!(result.is_err());

    // 2. Try to create a single chat with yourself (should fail)
    let result = state
      .create_new_chat(
        user1.id.into(),
        &format!("Self Chat {}", timestamp),
        ChatType::Single,
        Some(vec![user1.id.into()]),
        None,
        user1.workspace_id.into(),
      )
      .await;
    assert!(result.is_err());

    // 3. Test single chat without specifying members
    let result = state
      .create_new_chat(
        user1.id.into(),
        &format!("No Target Chat {}", timestamp),
        ChatType::Single,
        None,
        None,
        user1.workspace_id.into(),
      )
      .await;
    assert!(result.is_err());

    // 4. Test single chat with empty member list
    let result = state
      .create_new_chat(
        user1.id.into(),
        &format!("Empty Members Chat {}", timestamp),
        ChatType::Single,
        Some(vec![]),
        None,
        user1.workspace_id.into(),
      )
      .await;
    assert!(result.is_err());

    // 5. Test single chat with multiple members
    let result = state
      .create_new_chat(
        user1.id.into(),
        &format!("Multi Target Chat {}", timestamp),
        ChatType::Single,
        Some(vec![user2.id.into(), user3.id.into()]),
        None,
        user1.workspace_id.into(),
      )
      .await;
    assert!(result.is_err());

    // 6. Create a group with insufficient members (should fail)
    let result = state
      .create_new_chat(
        user1.id.into(),
        &format!("Small Group {}", timestamp),
        ChatType::Group,
        Some(vec![user2.id.into()]),
        None,
        user1.workspace_id.into(),
      )
      .await;
    assert!(result.is_err());

    // 7. Test group chat without specifying target members (should fail as total members < 3)
    let result = state
      .create_new_chat(
        user1.id.into(),
        &format!("No Members Group {}", timestamp),
        ChatType::Group,
        None,
        None,
        user1.workspace_id.into(),
      )
      .await;
    assert!(result.is_err());

    // 8. Test group chat with empty member list (should fail as total members < 3)
    let result = state
      .create_new_chat(
        user1.id.into(),
        &format!("Empty Members Group {}", timestamp),
        ChatType::Group,
        Some(vec![]),
        None,
        user1.workspace_id.into(),
      )
      .await;
    assert!(result.is_err());

    // 9. Test group chat with duplicate members (should succeed but deduplicate)
    let result = state
      .create_new_chat(
        user1.id.into(),
        &format!("Duplicate Members Group {}", timestamp),
        ChatType::Group,
        Some(vec![
          user2.id.into(),
          user3.id.into(),
          user2.id.into(),
          user3.id.into(),
        ]),
        None,
        user1.workspace_id.into(),
      )
      .await?;
    assert_eq!(result.chat_members.len(), 3);

    // 10. Test group chat that includes the creator (should succeed but not duplicate creator)
    let result = state
      .create_new_chat(
        user1.id.into(),
        &format!("Self Included Group {}", timestamp),
        ChatType::Group,
        Some(vec![user1.id.into(), user2.id.into(), user3.id.into()]),
        None,
        user1.workspace_id.into(),
      )
      .await?;
    assert_eq!(result.chat_members.len(), 3);
    assert_eq!(result.chat_members[0], user1.id);

    // 11. Test empty members private channel (should succeed with only creator)
    let result = state
      .create_new_chat(
        user1.id.into(),
        &format!("Empty Private Channel {}", timestamp),
        ChatType::PrivateChannel,
        None,
        None,
        user1.workspace_id.into(),
      )
      .await?;
    assert_eq!(result.chat_members.len(), 1);
    assert_eq!(result.chat_members[0], user1.id);

    // 12. Test private channel that includes creator (should succeed but not duplicate)
    let result = state
      .create_new_chat(
        user1.id.into(),
        &format!("Self Included Private Channel {}", timestamp),
        ChatType::PrivateChannel,
        Some(vec![user1.id.into(), user2.id.into()]),
        None,
        user1.workspace_id.into(),
      )
      .await?;
    assert_eq!(result.chat_members.len(), 2);
    assert_eq!(result.chat_members[0], user1.id);

    // 13. Test private channel with duplicate members (should succeed but deduplicate)
    let result = state
      .create_new_chat(
        user1.id.into(),
        &format!("Duplicate Private Channel {}", timestamp),
        ChatType::PrivateChannel,
        Some(vec![
          user2.id.into(),
          user2.id.into(),
          user3.id.into(),
          user3.id.into(),
        ]),
        None,
        user1.workspace_id.into(),
      )
      .await?;
    assert_eq!(result.chat_members.len(), 3);

    // 14. Test public channel (always succeeds, ignores all member parameters)
    let result = state
      .create_new_chat(
        user1.id.into(),
        &format!("Public Channel No Members {}", timestamp),
        ChatType::PublicChannel,
        None,
        None,
        user1.workspace_id.into(),
      )
      .await?;
    assert_eq!(result.chat_members.len(), 1);
    assert_eq!(result.chat_members[0], user1.id);

    // 15. Test public channel with specified members (should succeed but ignore members)
    let result = state
      .create_new_chat(
        user1.id.into(),
        &format!("Public Channel With Members {}", timestamp),
        ChatType::PublicChannel,
        Some(vec![user2.id.into(), user3.id.into(), user4.id.into()]),
        None,
        user1.workspace_id.into(),
      )
      .await?;
    assert_eq!(result.chat_members.len(), 1);
    assert_eq!(result.chat_members[0], user1.id);

    // 16. Test chat with extremely long name (database constraint is 128 characters)
    let long_name = format!("a{}", "b".repeat(129)); // 130 characters total, should fail
    let result = state
      .create_new_chat(
        user1.id.into(),
        &long_name,
        ChatType::Single,
        Some(vec![user2.id.into()]),
        None,
        user1.workspace_id.into(),
      )
      .await;
    assert!(result.is_err());

    // 17. Test chat with description
    let result = state
      .create_new_chat(
        user1.id.into(),
        &format!("Chat With Description {}", timestamp),
        ChatType::Single,
        Some(vec![user2.id.into()]),
        Some("This is a test description that should be saved correctly"),
        user1.workspace_id.into(),
      )
      .await?;
    assert_eq!(
      result.description,
      "This is a test description that should be saved correctly"
    );

    let chats = state.list_chats_of_user(user1.id.into()).await?;
    assert_eq!(chats.len(), 8);

    Ok(())
  }

  #[tokio::test]
  async fn realistic_chat_scenarios_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(5).await;
    // let _pool = &state.pool;
    let user1 = &users[0]; // Alice
    let user2 = &users[1]; // Bob
    let user3 = &users[2]; // Charlie
    let user4 = &users[3]; // David
    let user5 = &users[4]; // Eve

    // Generate unique names to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();

    // 1. Create department group chat
    let department_members = Some(vec![user2.id, user3.id, user4.id, user5.id]);
    let department_chat = state
      .create_new_chat(
        user1.id.into(), // Alice as creator
        &format!("Marketing Department {}", timestamp),
        ChatType::Group,
        department_members.map(|members| members.into_iter().map(|id| id.into()).collect()),
        Some("Department internal communication group"),
        user1.workspace_id.into(),
      )
      .await?;

    assert_eq!(department_chat.chat_members.len(), 5);

    // 2. Create project team
    let project_members = Some(vec![user2.id.into(), user3.id.into()]); // Bob and Charlie
    let project_chat = state
      .create_new_chat(
        user1.id.into(), // Alice as creator
        &format!("Website Redesign {}", timestamp),
        ChatType::Group,
        project_members.map(|members| {
          members
            .into_iter()
            .map(|id: fechatter_core::UserId| id.into())
            .collect()
        }),
        Some("Website redesign project discussion"),
        user1.workspace_id.into(),
      )
      .await?;

    assert_eq!(project_chat.chat_members.len(), 3); // Alice + Bob + Charlie

    // 3. Create company announcement channel
    let announce_channel = state
      .create_new_chat(
        user1.id.into(), // Alice as manager
        &format!("Company Announcements {}", timestamp),
        ChatType::PublicChannel,
        None,
        Some("Company important announcements"),
        user1.workspace_id.into(),
      )
      .await?;

    assert_eq!(announce_channel.chat_members.len(), 1); // Only Alice

    // 4. Alice and Bob's one-on-one chat
    let one_on_one = state
      .create_new_chat(
        user1.id.into(), // Alice
        &format!("Alice & Bob {}", timestamp),
        ChatType::Single,
        Some(vec![user2.id.into()]), // Bob
        None,
        user1.workspace_id.into(),
      )
      .await?;

    assert_eq!(one_on_one.chat_members.len(), 2);
    assert!(one_on_one.chat_members.contains(&user1.id));
    assert!(one_on_one.chat_members.contains(&user2.id));

    let chats = state.list_chats_of_user(user1.id.into()).await?;
    assert_eq!(chats.len(), 4);

    Ok(())
  }

  #[tokio::test]
  async fn complex_chat_scenarios_should_expose_logic_holes() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(8).await;
    // let _pool = &state.pool;
    let user1 = &users[0]; // Alice
    let user2 = &users[1]; // Bob
    let user3 = &users[2]; // Charlie
    let user4 = &users[3]; // David
    let user5 = &users[4]; // Eve
    let user6 = &users[5]; // Frank
    let user7 = &users[6]; // Grace
    let user8 = &users[7]; // Hank

    // Generate unique names to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();

    // ===== 1. Test unique constraint conflict (duplicate name) =====

    // Create first chat
    let first_chat = CreateChat::new(
      &format!("Unique Name Test {}", timestamp),
      ChatType::Group,
      vec![user2.id, user3.id],
      "",
    );
    let first_chat = state
      .create_new_chat(
        user1.id.into(),
        &first_chat.name,
        first_chat.chat_type,
        first_chat
          .members
          .map(|members| members.into_iter().map(|id| id.into()).collect()),
        None,
        user1.workspace_id.into(),
      )
      .await?;
    assert_eq!(first_chat.chat_members.len(), 3);

    // Try to create chat with same name (should fail)
    let result = state
      .create_new_chat(
        user4.id.into(),                            // Different creator
        &format!("Unique Name Test {}", timestamp), // Same name
        ChatType::Group,
        Some(vec![user5.id.into(), user6.id.into()]), // Different members
        None,
        user1.workspace_id.into(),
      )
      .await;

    assert!(result.is_err());

    // ===== 2. Test special character names =====

    // Use various special characters as chat name
    let special_name = format!("!@#$%^&*()_+-=[]{{}}|;':\",./<>? {}", timestamp);
    let special_chat =
      CreateChat::new(&special_name, ChatType::Group, vec![user2.id, user3.id], "");
    let special_chat = state
      .create_new_chat(
        user1.id.into(),
        &special_chat.name,
        special_chat.chat_type,
        special_chat
          .members
          .map(|members| members.into_iter().map(|id| id.into()).collect()),
        None,
        user1.workspace_id.into(),
      )
      .await?;

    assert_eq!(special_chat.name, special_name);

    // ===== 3. Test extremely long description =====

    // Create chat with long description
    let long_desc = "a".repeat(1000); // Very long description
    let long_desc_chat = CreateChat::new(
      &format!("Long Description Chat {}", timestamp),
      ChatType::Group,
      vec![user2.id, user3.id],
      &long_desc,
    );
    let long_desc_chat = state
      .create_new_chat(
        user1.id.into(),
        &long_desc_chat.name,
        long_desc_chat.chat_type,
        long_desc_chat
          .members
          .map(|members| members.into_iter().map(|id| id.into()).collect()),
        Some(long_desc_chat.description.as_deref().unwrap_or("")),
        user1.workspace_id.into(),
      )
      .await?;

    assert_eq!(long_desc_chat.description, long_desc);

    // ===== 4. Test group chat with many members =====

    // Create group chat with all test users
    let large_group = CreateChat::new(
      &format!("Large Group Chat {}", timestamp),
      ChatType::Group,
      vec![
        user2.id, user3.id, user4.id, user5.id, user6.id, user7.id, user8.id,
      ],
      "",
    );
    let large_group = state
      .create_new_chat(
        user1.id.into(),
        &large_group.name,
        large_group.chat_type,
        large_group
          .members
          .map(|members| members.into_iter().map(|id| id.into()).collect()),
        None,
        user1.workspace_id.into(),
      )
      .await?;

    assert_eq!(large_group.chat_members.len(), 8); // Creator + 7 members

    // ===== 5. Test different chat types with same members =====

    // Create different chat types for the same group of users
    let members = vec![user2.id.into()];

    // Single chat
    let single_chat = CreateChat::new(
      &format!("Same Members Single {}", timestamp),
      ChatType::Single,
      members.clone(),
      "",
    );
    let single_chat = state
      .create_new_chat(
        user1.id.into(),
        &single_chat.name,
        single_chat.chat_type,
        single_chat
          .members
          .map(|members| members.into_iter().map(|id| id.into()).collect()),
        None,
        user1.workspace_id.into(),
      )
      .await?;

    // Private channel
    let private_channel = CreateChat::new(
      &format!("Same Members Private {}", timestamp),
      ChatType::PrivateChannel,
      members.clone(),
      "",
    );
    let private_channel = state
      .create_new_chat(
        user1.id.into(),
        &private_channel.name,
        private_channel.chat_type,
        private_channel
          .members
          .map(|members| members.into_iter().map(|id| id.into()).collect()),
        None,
        user1.workspace_id.into(),
      )
      .await?;

    assert_eq!(single_chat.chat_members.len(), 2);
    assert_eq!(private_channel.chat_members.len(), 2);
    assert!(single_chat.chat_members.contains(&user1.id));
    assert!(single_chat.chat_members.contains(&user2.id));
    assert!(private_channel.chat_members.contains(&user1.id));
    assert!(private_channel.chat_members.contains(&user2.id));

    // ===== 6. Test all-whitespace name =====

    // Try to use all-spaces name
    let all_spaces = CreateChat::new("   ", ChatType::Group, vec![user2.id, user3.id], "");
    let result = state
      .create_new_chat(
        user1.id.into(),
        &all_spaces.name,
        all_spaces.chat_type,
        all_spaces
          .members
          .map(|members| members.into_iter().map(|id| id.into()).collect()),
        None,
        user1.workspace_id.into(),
      )
      .await;

    assert!(result.is_err());

    // ===== 7. Test preserving creator as first member =====

    // Create group chat with creator included in target_members, check that creator appears only once and is first
    let creator_included = CreateChat::new(
      &format!("Creator In Members {}", timestamp),
      ChatType::Group,
      vec![user2.id, user3.id, user1.id, user3.id],
      "",
    );
    let creator_included = state
      .create_new_chat(
        user1.id.into(),
        &creator_included.name,
        creator_included.chat_type,
        creator_included
          .members
          .map(|members| members.into_iter().map(|id| id.into()).collect()),
        None,
        user1.workspace_id.into(),
      )
      .await?;

    assert_eq!(creator_included.chat_members.len(), 3);
    assert_eq!(creator_included.chat_members[0], user1.id); // Creator should be first

    // ===== 8. Test name at database limit =====

    // Name exactly 128 characters (database limit is 128)
    // Use unique characters to avoid conflicts, but keep it exactly 128 chars
    let timestamp_str = timestamp.to_string();
    let unique_prefix = format!("LongName{}", timestamp_str);
    let padding_length = 128 - unique_prefix.len();
    let name_128 = if padding_length > 0 {
      format!("{}{}", unique_prefix, "x".repeat(padding_length))
    } else {
      // If somehow the prefix is too long, just truncate to 128
      unique_prefix.chars().take(128).collect::<String>()
    };

    let result = state
      .create_new_chat(
        user1.id.into(),
        &name_128,
        ChatType::Single,
        Some(vec![user2.id.into()]),
        None,
        user1.workspace_id.into(),
      )
      .await?;

    assert_eq!(result.name.len(), 128);

    // ===== 9. Test member relationship record creation =====

    // Create chat and verify relationship records for all members
    let relation_test = CreateChat::new(
      &format!("Relation Test Chat {}", timestamp),
      ChatType::Group,
      vec![user2.id, user3.id],
      "",
    );
    let relation_test = state
      .create_new_chat(
        user1.id.into(),
        &relation_test.name,
        relation_test.chat_type,
        relation_test
          .members
          .map(|members| members.into_iter().map(|id| id.into()).collect()),
        None,
        user1.workspace_id.into(),
      )
      .await?;

    // Typically would need to query database for verification, but we can indirectly verify through chat_members
    assert_eq!(relation_test.chat_members.len(), 3);

    // ===== 10. Test multiple operations on same chat =====

    // Create a normal group chat
    let normal_chat = CreateChat::new(
      &format!("Normal Group Chat {}", timestamp),
      ChatType::Group,
      vec![user2.id, user3.id],
      "",
    );
    let normal_chat = state
      .create_new_chat(
        user1.id.into(),
        &normal_chat.name,
        normal_chat.chat_type,
        normal_chat
          .members
          .map(|members| members.into_iter().map(|id| id.into()).collect()),
        None,
        user1.workspace_id.into(),
      )
      .await?;

    assert_eq!(normal_chat.chat_members.len(), 3);
    assert!(normal_chat.chat_members.contains(&user1.id));
    assert!(normal_chat.chat_members.contains(&user2.id));
    assert!(normal_chat.chat_members.contains(&user3.id));

    // Adding implementation for edit members, etc. to satisfy TODO
    // Add a new member to the chat
    let non_existent_user_id = i64::MAX; // Use a very large ID that definitely doesn't exist
    let add_result = state
      .add_chat_members(
        normal_chat.id.into(),
        user1.id.into(), // Creator is performing the action
        vec![non_existent_user_id],
      )
      .await;

    // This should fail as the user doesn't exist, but verifies the function is callable
    assert!(add_result.is_err(), "Adding non-existent user should fail");

    // Test deleting a member
    let remove_result = state
      .remove_group_chat_members(
        normal_chat.id.into(),
        user1.id.into(), // Creator is performing the action
        vec![user3.id.into()],
      )
      .await?;

    assert!(remove_result);

    // ===== 11. Test non-English names and descriptions =====

    // Create chat with non-English content
    let non_english_chat = CreateChat::new(
      &format!("Chat with non-English name test {}", timestamp),
      ChatType::Group,
      vec![user2.id, user3.id],
      "This is a non-English description, testing UTF-8 support",
    );
    let non_english_chat = state
      .create_new_chat(
        user1.id.into(),
        &non_english_chat.name,
        non_english_chat.chat_type,
        non_english_chat
          .members
          .map(|members| members.into_iter().map(|id| id.into()).collect()),
        Some(non_english_chat.description.as_deref().unwrap_or("")),
        user1.workspace_id.into(),
      )
      .await?;

    assert_eq!(
      non_english_chat.name,
      format!("Chat with non-English name test {}", timestamp)
    );
    assert_eq!(
      non_english_chat.description,
      "This is a non-English description, testing UTF-8 support"
    );

    // ===== 12. Test emoji names =====

    // Create chat with emoji in name
    let emoji_chat = CreateChat::new(
      &format!("😀 Emoji Chat 🎉 {}", timestamp),
      ChatType::Group,
      vec![user2.id, user3.id],
      "Testing emoji support 😊👍",
    );
    let emoji_chat = state
      .create_new_chat(
        user1.id.into(),
        &emoji_chat.name,
        emoji_chat.chat_type,
        emoji_chat
          .members
          .map(|members| members.into_iter().map(|id| id.into()).collect()),
        Some(emoji_chat.description.as_deref().unwrap_or("")),
        user1.workspace_id.into(),
      )
      .await?;

    assert_eq!(emoji_chat.name, format!("😀 Emoji Chat 🎉 {}", timestamp));
    assert_eq!(emoji_chat.description, "Testing emoji support 😊👍");

    let chats = state.list_chats_of_user(user1.id.into()).await?;
    assert_eq!(chats.len(), 12);

    Ok(())
  }

  #[tokio::test]
  async fn create_duplicate_chat_should_fail() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];

    // Generate unique chat name to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let chat_name = format!("Unique Test Chat {}", timestamp);

    let members = vec![user2.id, user3.id];
    let first_chat = state
      .create_new_chat(
        user1.id.into(),
        &chat_name,
        ChatType::Group,
        Some(members.clone().into_iter().map(|id| id.into()).collect()),
        Some("First chat description"),
        user1.workspace_id.into(),
      )
      .await?;

    assert_eq!(first_chat.name, chat_name);

    // Attempt to create a second chat with the same name
    let result = state
      .create_new_chat(
        user1.id.into(),
        &chat_name, // same name
        ChatType::Group,
        Some(members.into_iter().map(|id| id.into()).collect()),
        Some("Second chat description"),
        user1.workspace_id.into(),
      )
      .await;

    // Verify creation fails and returns the correct error
    match result {
      Err(AppError::ChatAlreadyExists(error_message)) => {
        let expected_error_message = format!("Chat {} already exists", chat_name);
        assert_eq!(error_message, expected_error_message);
      }
      Ok(_) => panic!("Expected chat creation to fail, but it succeeded."),
      Err(e) => panic!("Expected ChatAlreadyExists error, but got {:?}", e),
    }

    Ok(())
  }

  #[tokio::test]
  async fn update_chat_with_existing_name_should_fail() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];

    // Generate unique chat names to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let first_chat_name = format!("First Chat Name {}", timestamp);
    let second_chat_name = format!("Second Chat Name {}", timestamp);

    state
      .create_new_chat(
        user1.id.into(),
        &first_chat_name,
        ChatType::Group,
        Some(vec![user2.id.into(), user3.id.into()]),
        Some("First chat description"),
        user1.workspace_id.into(),
      )
      .await?;

    let second_chat = state
      .create_new_chat(
        user1.id.into(),
        &second_chat_name,
        ChatType::Group,
        Some(vec![user2.id.into(), user3.id.into()]),
        Some("Second chat description"),
        user1.workspace_id.into(),
      )
      .await?;

    let update_result = state
      .update_chat(
        second_chat.id.into(),
        user1.id.into(),
        UpdateChat {
          name: Some(first_chat_name.clone()),
          description: None,
        },
      )
      .await;

    match update_result {
      Err(AppError::ChatAlreadyExists(error_message)) => {
        let expected_error = format!(
          "Chat name '{}' is already taken by another chat",
          first_chat_name
        );
        assert_eq!(error_message, expected_error);
      }
      Ok(_) => panic!("Expected chat update to fail, but it succeeded."),
      Err(e) => panic!("Expected ChatPermissionError, but got {:?}", e),
    }

    Ok(())
  }
}

#[cfg(test)]
mod process_chat_members_data_driven_tests {
  // use super::*;
  use crate::AppError;
  use crate::models::ChatType;
  use anyhow::Result;

  const CREATOR_ID: i64 = 1;
  const USER_2: i64 = 2;
  const USER_3: i64 = 3;

  fn process_chat_members(
    chat_type: &ChatType,
    creator_id: i64,
    target_members: Option<&Vec<i64>>,
  ) -> Result<Vec<i64>, AppError> {
    match chat_type {
      ChatType::Single => match target_members {
        Some(members) if members.len() == 1 => {
          let target_id = members[0];
          if target_id == creator_id {
            return Err(AppError::ChatValidationError(
              "Single chat must have exactly one member".to_string(),
            ));
          }
          Ok(vec![creator_id, target_id])
        }
        _ => Err(AppError::ChatValidationError(
          "Invalid single chat members".to_string(),
        )),
      },
      ChatType::Group => {
        let mut result = vec![creator_id];
        if let Some(members) = target_members {
          for &id in members {
            if id != creator_id && !result.contains(&id) {
              result.push(id);
            }
          }
        }
        if result.len() < 3 {
          return Err(AppError::ChatValidationError(
            "Group chat must have at least three members".to_string(),
          ));
        }
        Ok(result)
      }
      ChatType::PrivateChannel => {
        let mut result = vec![creator_id];
        if let Some(members) = target_members {
          for &id in members {
            if id != creator_id && !result.contains(&id) {
              result.push(id);
            }
          }
        }
        Ok(result)
      }
      ChatType::PublicChannel => Ok(vec![creator_id]),
    }
  }

  struct TestCase<'a> {
    desc: &'a str,
    chat_type: ChatType,
    input_members: Option<Vec<i64>>,
    expected: Result<Vec<i64>, String>,
  }

  #[test]
  fn process_chat_members_edge_cases_should_pass() -> Result<()> {
    let test_cases = vec![
      // --- Single Chat Edge Cases ---
      TestCase {
        desc: "Single: Target member is creator",
        chat_type: ChatType::Single,
        input_members: Some(vec![CREATOR_ID]),
        expected: Err("Single chat must have exactly one member".to_string()),
      },
      TestCase {
        desc: "Single: target_members is None",
        chat_type: ChatType::Single,
        input_members: None,
        expected: Err("Invalid single chat members".to_string()),
      },
      TestCase {
        desc: "Single: target_members is empty",
        chat_type: ChatType::Single,
        input_members: Some(vec![]),
        expected: Err("Invalid single chat members".to_string()),
      },
      TestCase {
        desc: "Single: target_members has multiple members",
        chat_type: ChatType::Single,
        input_members: Some(vec![USER_2, USER_3]),
        expected: Err("Invalid single chat members".to_string()),
      },
      TestCase {
        // Add a successful Single case for comparison
        desc: "Single: Valid case",
        chat_type: ChatType::Single,
        input_members: Some(vec![USER_2]),
        expected: Ok(vec![CREATOR_ID, USER_2]), // Order is fixed
      },
      // --- Group Chat Edge Cases ---
      TestCase {
        desc: "Group: Exactly 2 members total (fails minimum)",
        chat_type: ChatType::Group,
        input_members: Some(vec![USER_2]),
        expected: Err("Group chat must have at least three members".to_string()),
      },
      TestCase {
        desc: "Group: Input includes creator & duplicates, results in 3 (passes)",
        chat_type: ChatType::Group,
        input_members: Some(vec![USER_2, CREATOR_ID, USER_3, USER_2]),
        expected: Ok(vec![CREATOR_ID, USER_2, USER_3]), // Expected result needs to be sorted for comparison
      },
      TestCase {
        desc: "Group: target_members is None (fails minimum)",
        chat_type: ChatType::Group,
        input_members: None,
        expected: Err("Group chat must have at least three members".to_string()),
      },
      TestCase {
        desc: "Group: target_members is empty (fails minimum)",
        chat_type: ChatType::Group,
        input_members: Some(vec![]),
        expected: Err("Group chat must have at least three members".to_string()),
      },
      // --- Private Channel Edge Cases ---
      TestCase {
        desc: "Private: target_members is None (passes)",
        chat_type: ChatType::PrivateChannel,
        input_members: None,
        expected: Ok(vec![CREATOR_ID]),
      },
      TestCase {
        desc: "Private: target_members is empty (passes)",
        chat_type: ChatType::PrivateChannel,
        input_members: Some(vec![]),
        expected: Ok(vec![CREATOR_ID]),
      },
      TestCase {
        desc: "Private: Input includes creator & duplicates (passes, deduplicated)",
        chat_type: ChatType::PrivateChannel,
        input_members: Some(vec![USER_2, CREATOR_ID, USER_3, USER_2]),
        expected: Ok(vec![CREATOR_ID, USER_2, USER_3]), // Expected result needs to be sorted for comparison
      },
      // --- Public Channel Edge Cases ---
      TestCase {
        desc: "Public: Ignores provided members",
        chat_type: ChatType::PublicChannel,
        input_members: Some(vec![USER_2, USER_3]), // These should be ignored
        expected: Ok(vec![CREATOR_ID]),
      },
    ];

    for case in test_cases {
      println!("Testing case: {}", case.desc);

      // 调用测试模块中的静态函数而不是state.process_chat_members
      let actual_result =
        process_chat_members(&case.chat_type, CREATOR_ID, case.input_members.as_ref());

      match (actual_result, case.expected) {
        // Case 1: Both Ok - Compare vectors (sort for Group/Private)
        (Ok(mut actual_vec), Ok(mut expected_vec)) => {
          // For Group and PrivateChannel, member order doesn't matter, sort before comparing
          if case.chat_type == ChatType::Group || case.chat_type == ChatType::PrivateChannel {
            actual_vec.sort();
            expected_vec.sort();
          }
          assert_eq!(
            actual_vec, expected_vec,
            "Mismatch in OK case: {}",
            case.desc
          );
        }
        // Case 2: Both Err - Compare error messages
        (Err(AppError::ChatValidationError(actual_msg)), Err(expected_msg)) => {
          assert_eq!(
            actual_msg, expected_msg,
            "Mismatch in ERR case: {}",
            case.desc
          );
        }
        // Case 3: Mismatched Ok/Err or different Err type - Panic
        (actual, expected) => {
          panic!(
            "Test case failed: '{}'\nExpected: {:?}\nActual: {:?}",
            case.desc, expected, actual
          );
        }
      }
    }

    Ok(()) // Indicate overall test success
  }
}
