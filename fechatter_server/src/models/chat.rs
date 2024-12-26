use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, prelude::FromRow};
use tokio::time::Instant;

use crate::{AppError, AppState, models::ChatType};

use super::{Chat, CreateChatMember, User, insert_chat_members_relation, is_creator_in_chat};

const CHAT_LIST_CACHE_TTL: Duration = Duration::from_secs(30);

/// Retrieves a list of chat sidebars for a specific user
pub async fn list_chats_of_user(
  state: &AppState,
  user_id: i64,
) -> Result<Arc<Vec<ChatSidebar>>, AppError> {
  if let Some(entry) = state.chat_list_cache.get(&user_id) {
    let (cached_chats, timestamp) = entry.value();
    if timestamp.elapsed() < CHAT_LIST_CACHE_TTL {
      return Ok(cached_chats.clone());
    }
  }

  // If no valid cache exists, fetch the chat list from the database
  let chats = fetch_chat_list_from_db(&state.pool, user_id).await?;

  // Create an Arc to allow shared ownership of the chat list
  let chats_arc = Arc::new(chats);

  // Update the cache with the new chat list and current timestamp
  state
    .chat_list_cache
    .insert(user_id, (chats_arc.clone(), Instant::now()));

  Ok(chats_arc)
}

async fn fetch_chat_list_from_db(
  pool: &PgPool,
  user_id: i64,
) -> Result<Vec<ChatSidebar>, AppError> {
  let chats = sqlx::query_as!(
    ChatSidebar,
    r#"SELECT
      id,
      chat_name as name,
      type as "chat_type: _"
    FROM chats
    WHERE created_by = $1 OR $1 = ANY(chat_members)
    ORDER BY updated_at DESC"#,
    user_id
  )
  .fetch_all(pool)
  .await
  .map(|rows| {
    rows
      .into_iter()
      .map(|row: ChatSidebar| ChatSidebar {
        id: row.id,
        name: row.name,
        chat_type: row.chat_type,
      })
      .collect()
  })?;

  Ok(chats)
}

fn validate_chat_name(name: &str) -> Result<(), AppError> {
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

#[allow(dead_code)]
pub async fn fetch_all_chats(workspace_id: i64, pool: &PgPool) -> Result<Vec<Chat>, AppError> {
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
  .fetch_all(pool)
  .await?;

  Ok(chats)
}

pub async fn create_new_chat(
  state: &AppState,
  creator_id: i64,
  name: &str,
  chat_type: ChatType,
  target_members: Option<Vec<i64>>,
  description: Option<&str>,
  workspace_id: i64,
) -> Result<Chat, AppError> {
  validate_chat_name(name)?;

  let chat_members = process_chat_members(&chat_type, creator_id, target_members.as_ref())?;

  User::validate_users_exists_by_ids(&chat_members, &state.pool).await?;

  let mut tx = state.pool.begin().await?;

  let chat = insert_chat_record(
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
        e.into()
      }
    } else {
      e.into()
    }
  })?;

  let chat_id = chat.id;
  insert_chat_members_relation(chat_id, &chat_members, &mut tx).await?;

  tx.commit().await?;

  for &member in &chat_members {
    state.chat_list_cache.remove(&member);
  }

  Ok(chat)
}

pub async fn update_chat(
  state: &AppState,
  chat_id: i64,
  user_id: i64,
  payload: UpdateChat,
) -> Result<Chat, AppError> {
  let creator = CreateChatMember { chat_id, user_id };
  let is_creator = is_creator_in_chat(&state.pool, &creator).await?;

  if !is_creator {
    return Err(AppError::ChatPermissionError(format!(
      "User {} is not the creator of chat {}",
      user_id, chat_id
    )));
  }

  let chat = sqlx::query_as::<_, Chat>(
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
  .fetch_one(&state.pool)
  .await?;

  if payload.name.is_some() || payload.description.is_some() {
    for &member_id in &chat.chat_members {
      state.chat_list_cache.remove(&member_id);
    }
  }

  Ok(chat)
}

async fn delete_chat_transactional(
  tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
  chat_id: i64,
  user_id: i64,
) -> Result<Vec<i64>, AppError> {
  let chat_info = sqlx::query!(
    "SELECT chat_members, created_by FROM chats WHERE id = $1 FOR UPDATE",
    chat_id
  )
  .fetch_optional(&mut **tx)
  .await?
  .ok_or(AppError::ChatNotFound(chat_id))?;

  if chat_info.created_by != user_id {
    return Err(AppError::ChatPermissionError(format!(
      "User {} is not the creator of chat {}",
      user_id, chat_id
    )));
  }

  sqlx::query!(
    "DELETE FROM chat_members_relation WHERE chat_id = $1",
    chat_id
  )
  .execute(&mut **tx)
  .await?;

  let result = sqlx::query!("DELETE FROM chats WHERE id = $1", chat_id)
    .execute(&mut **tx)
    .await?;

  if result.rows_affected() == 0 {
    return Err(AppError::ChatNotFound(chat_id));
  }

  Ok(chat_info.chat_members)
}

pub async fn delete_chat(state: &AppState, chat_id: i64, user_id: i64) -> Result<bool, AppError> {
  let mut tx = state.pool.begin().await?;

  let members_to_invalidate = match delete_chat_transactional(&mut tx, chat_id, user_id).await {
    Ok(members) => members,
    Err(e) => {
      let _ = tx.rollback().await;
      return Err(e);
    }
  };

  tx.commit().await?;

  for &member in &members_to_invalidate {
    state.chat_list_cache.remove(&member);
  }

  Ok(true)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateChat {
  pub name: String,
  pub chat_type: ChatType,
  pub chat_members: Vec<i64>,
  pub description: String,
}

#[cfg(test)]
impl CreateChat {
  pub fn new(name: &str, chat_type: ChatType, chat_members: Vec<i64>, description: &str) -> Self {
    Self {
      name: name.to_string(),
      chat_type,
      chat_members,
      description: description.to_string(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ChatSidebar {
  pub id: i64,
  pub name: String,
  pub chat_type: ChatType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateChat {
  pub name: Option<String>,
  pub description: Option<String>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::models::{add_chat_members, remove_group_chat_members};
  use crate::setup_test_users;
  use anyhow::Result;

  #[tokio::test]
  async fn create_and_list_chats_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];

    // 1. Create single chat
    let single_members = vec![user2.id];
    let single_chat = CreateChat::new(
      "Single Chat with Bob",
      ChatType::Single,
      single_members.clone(),
      "One-on-one chat",
    );
    let single_chat_created = create_new_chat(
      &state,
      user1.id,
      &single_chat.name,
      single_chat.chat_type,
      Some(single_members),           // Pass Option<Vec<i64>> for members
      Some(&single_chat.description), // Pass Option<&str> for description
      user1.workspace_id,
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
      "Work Team",
      ChatType::Group,
      group_members.clone(),
      "Work group",
    );
    let chat = create_new_chat(
      &state,
      user1.id,
      &group_chat.name,
      group_chat.chat_type,
      Some(group_members),           // Pass Option<Vec<i64>> for members
      Some(&group_chat.description), // Pass Option<&str> for description
      user1.workspace_id,
    )
    .await?;

    assert_eq!(chat.chat_type, ChatType::Group);
    assert_eq!(chat.chat_members.len(), 3);
    assert!(chat.chat_members.contains(&user1.id));
    assert!(chat.chat_members.contains(&user2.id));
    assert!(chat.chat_members.contains(&user3.id));

    // 3. Create private channel
    let channel_members = vec![user2.id];
    let private_chat = create_new_chat(
      &state,
      user1.id,
      "Project Updates",
      ChatType::PrivateChannel,
      Some(channel_members.clone()),
      Some("Invite only"),
      user1.workspace_id,
    )
    .await?;

    assert_eq!(private_chat.chat_type, ChatType::PrivateChannel);
    assert_eq!(private_chat.chat_members.len(), 2);

    // 4. Create public channel
    let public_chat = create_new_chat(
      &state,
      user1.id,
      "Company Announcements",
      ChatType::PublicChannel,
      None, // No members for public channel
      Some("Company announcements"),
      user1.workspace_id,
    )
    .await?;

    assert_eq!(public_chat.chat_type, ChatType::PublicChannel);
    assert_eq!(public_chat.chat_members.len(), 1);

    // Use the correct function signature for listing chats
    let chats = list_chats_of_user(&state, user1.id).await?;
    assert_eq!(chats.len(), 4);

    Ok(())
  }

  #[tokio::test]
  async fn update_and_delete_chat_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];

    let single_chat = create_new_chat(
      &state,
      user1.id,
      "Single Chat with Bob (for update/delete)",
      ChatType::Single,
      Some(vec![user2.id]),
      Some("One-on-one chat"),
      user1.workspace_id,
    )
    .await?;
    assert_eq!(single_chat.chat_type, ChatType::Single);

    let group_members = vec![user2.id, user3.id];
    let group_chat = create_new_chat(
      &state,
      user1.id,
      "Work Team",
      ChatType::Group,
      Some(group_members),
      Some("Work group"),
      user1.workspace_id,
    )
    .await?;
    assert_eq!(group_chat.chat_type, ChatType::Group);
    assert_eq!(group_chat.chat_members.len(), 3);

    let updated_single_chat = update_chat(
      &state,
      single_chat.id,
      user1.id,
      UpdateChat {
        name: Some("Updated Single Chat".to_string()),
        description: Some("Updated description".to_string()),
      },
    )
    .await?;

    assert_eq!(updated_single_chat.name, "Updated Single Chat");
    assert_eq!(updated_single_chat.description, "Updated description");

    let updated_group_chat = update_chat(
      &state,
      group_chat.id,
      user1.id,
      UpdateChat {
        name: Some("Updated Group Chat".to_string()),
        description: Some("Updated description".to_string()),
      },
    )
    .await?;

    assert_eq!(updated_group_chat.name, "Updated Group Chat");
    assert_eq!(updated_group_chat.description, "Updated description");

    let chats = list_chats_of_user(&state, user1.id).await?;
    assert_eq!(chats.len(), 2);

    let deleted_single_chat = delete_chat(&state, single_chat.id, user1.id).await?;
    assert!(deleted_single_chat);

    let chats = list_chats_of_user(&state, user1.id).await?;
    assert_eq!(chats.len(), 1);

    let deleted_group_chat = delete_chat(&state, group_chat.id, user1.id).await?;
    assert!(deleted_group_chat);

    let chats = list_chats_of_user(&state, user1.id).await?;
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

    // 1. Test creating a chat with no name (should fail)
    let result = create_new_chat(
      &state,
      user1.id,
      "",
      ChatType::Single,
      Some(vec![user2.id]),
      None,
      user1.workspace_id,
    )
    .await;
    assert!(result.is_err());

    // 2. Try to create a single chat with yourself (should fail)
    let result = create_new_chat(
      &state,
      user1.id,
      "Self Chat",
      ChatType::Single,
      Some(vec![user1.id]),
      None,
      user1.workspace_id,
    )
    .await;
    assert!(result.is_err());

    // 3. Test single chat without specifying members
    let result = create_new_chat(
      &state,
      user1.id,
      "No Target Chat",
      ChatType::Single,
      None,
      None,
      user1.workspace_id,
    )
    .await;
    assert!(result.is_err());

    // 4. Test single chat with empty member list
    let result = create_new_chat(
      &state,
      user1.id,
      "Empty Members Chat",
      ChatType::Single,
      Some(vec![]),
      None,
      user1.workspace_id,
    )
    .await;
    assert!(result.is_err());

    // 5. Test single chat with multiple members
    let result = create_new_chat(
      &state,
      user1.id,
      "Multi Target Chat",
      ChatType::Single,
      Some(vec![user2.id, user3.id]),
      None,
      user1.workspace_id,
    )
    .await;
    assert!(result.is_err());

    // 6. Create a group with insufficient members (should fail)
    let result = create_new_chat(
      &state,
      user1.id,
      "Small Group",
      ChatType::Group,
      Some(vec![user2.id]),
      None,
      user1.workspace_id,
    )
    .await;
    assert!(result.is_err());

    // 7. Test group chat without specifying target members (should fail as total members < 3)
    let result = create_new_chat(
      &state,
      user1.id,
      "No Members Group",
      ChatType::Group,
      None,
      None,
      user1.workspace_id,
    )
    .await;
    assert!(result.is_err());

    // 8. Test group chat with empty member list (should fail as total members < 3)
    let result = create_new_chat(
      &state,
      user1.id,
      "Empty Members Group",
      ChatType::Group,
      Some(vec![]),
      None,
      user1.workspace_id,
    )
    .await;
    assert!(result.is_err());

    // 9. Test group chat with duplicate members (should succeed but deduplicate)
    let result = create_new_chat(
      &state,
      user1.id,
      "Duplicate Members Group",
      ChatType::Group,
      Some(vec![user2.id, user3.id, user2.id, user3.id]),
      None,
      user1.workspace_id,
    )
    .await?;
    assert_eq!(result.chat_members.len(), 3);

    // 10. Test group chat that includes the creator (should succeed but not duplicate creator)
    let result = create_new_chat(
      &state,
      user1.id,
      "Self Included Group",
      ChatType::Group,
      Some(vec![user1.id, user2.id, user3.id]),
      None,
      user1.workspace_id,
    )
    .await?;
    assert_eq!(result.chat_members.len(), 3);
    assert_eq!(result.chat_members[0], user1.id);

    // 11. Test empty members private channel (should succeed with only creator)
    let result = create_new_chat(
      &state,
      user1.id,
      "Empty Private Channel",
      ChatType::PrivateChannel,
      None,
      None,
      user1.workspace_id,
    )
    .await?;
    assert_eq!(result.chat_members.len(), 1);
    assert_eq!(result.chat_members[0], user1.id);

    // 12. Test private channel that includes creator (should succeed but not duplicate)
    let result = create_new_chat(
      &state,
      user1.id,
      "Self Included Private Channel",
      ChatType::PrivateChannel,
      Some(vec![user1.id, user2.id]),
      None,
      user1.workspace_id,
    )
    .await?;
    assert_eq!(result.chat_members.len(), 2);
    assert_eq!(result.chat_members[0], user1.id);

    // 13. Test private channel with duplicate members (should succeed but deduplicate)
    let result = create_new_chat(
      &state,
      user1.id,
      "Duplicate Private Channel",
      ChatType::PrivateChannel,
      Some(vec![user2.id, user2.id, user3.id, user3.id]),
      None,
      user1.workspace_id,
    )
    .await?;
    assert_eq!(result.chat_members.len(), 3);

    // 14. Test public channel (always succeeds, ignores all member parameters)
    let result = create_new_chat(
      &state,
      user1.id,
      "Public Channel No Members",
      ChatType::PublicChannel,
      None,
      None,
      user1.workspace_id,
    )
    .await?;
    assert_eq!(result.chat_members.len(), 1);
    assert_eq!(result.chat_members[0], user1.id);

    // 15. Test public channel with specified members (should succeed but ignore members)
    let result = create_new_chat(
      &state,
      user1.id,
      "Public Channel With Members",
      ChatType::PublicChannel,
      Some(vec![user2.id, user3.id, user4.id]),
      None,
      user1.workspace_id,
    )
    .await?;
    assert_eq!(result.chat_members.len(), 1);
    assert_eq!(result.chat_members[0], user1.id);

    // 16. Test chat with extremely long name (database constraint is 128 characters)
    let long_name = "a".repeat(130);
    let result = create_new_chat(
      &state,
      user1.id,
      &long_name,
      ChatType::Single,
      Some(vec![user2.id]),
      None,
      user1.workspace_id,
    )
    .await;
    assert!(result.is_err());

    // 17. Test chat with description
    let result = create_new_chat(
      &state,
      user1.id,
      "Chat With Description",
      ChatType::Single,
      Some(vec![user2.id]),
      Some("This is a test description that should be saved correctly"),
      user1.workspace_id,
    )
    .await?;
    assert_eq!(
      result.description,
      "This is a test description that should be saved correctly"
    );

    let chats = list_chats_of_user(&state, user1.id).await?;
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

    // 1. Create department group chat
    let department_members = Some(vec![user2.id, user3.id, user4.id, user5.id]);
    let department_chat = create_new_chat(
      &state,
      user1.id, // Alice as creator
      "Marketing Department",
      ChatType::Group,
      department_members,
      Some("Department internal communication group"),
      user1.workspace_id,
    )
    .await?;

    assert_eq!(department_chat.chat_members.len(), 5);

    // 2. Create project team
    let project_members = Some(vec![user2.id, user3.id]); // Bob and Charlie
    let project_chat = create_new_chat(
      &state,
      user1.id, // Alice as creator
      "Website Redesign",
      ChatType::Group,
      project_members,
      Some("Website redesign project discussion"),
      user1.workspace_id,
    )
    .await?;

    assert_eq!(project_chat.chat_members.len(), 3); // Alice + Bob + Charlie

    // 3. Create company announcement channel
    let announce_channel = create_new_chat(
      &state,
      user1.id, // Alice as manager
      "Company Announcements",
      ChatType::PublicChannel,
      None,
      Some("Company important announcements"),
      user1.workspace_id,
    )
    .await?;

    assert_eq!(announce_channel.chat_members.len(), 1); // Only Alice

    // 4. Alice and Bob's one-on-one chat
    let one_on_one = create_new_chat(
      &state,
      user1.id, // Alice
      "Alice & Bob",
      ChatType::Single,
      Some(vec![user2.id]), // Bob
      None,
      user1.workspace_id,
    )
    .await?;

    assert_eq!(one_on_one.chat_members.len(), 2);
    assert!(one_on_one.chat_members.contains(&user1.id));
    assert!(one_on_one.chat_members.contains(&user2.id));

    let chats = list_chats_of_user(&state, user1.id).await?;
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

    // ===== 1. Test unique constraint conflict (duplicate name) =====

    // Create first chat
    let first_chat = CreateChat::new(
      "Unique Name Test",
      ChatType::Group,
      vec![user2.id, user3.id],
      "",
    );
    let first_chat = create_new_chat(
      &state,
      user1.id,
      &first_chat.name,
      first_chat.chat_type,
      Some(first_chat.chat_members),
      None,
      user1.workspace_id,
    )
    .await?;
    assert_eq!(first_chat.chat_members.len(), 3);

    // Try to create chat with same name (should fail)
    let result = create_new_chat(
      &state,
      user4.id,           // Different creator
      "Unique Name Test", // Same name
      ChatType::Group,
      Some(vec![user5.id, user6.id]), // Different members
      None,
      user1.workspace_id,
    )
    .await;

    assert!(result.is_err());

    // ===== 2. Test special character names =====

    // Use various special characters as chat name
    let special_name = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
    let special_chat = CreateChat::new(special_name, ChatType::Group, vec![user2.id, user3.id], "");
    let special_chat = create_new_chat(
      &state,
      user1.id,
      &special_chat.name,
      special_chat.chat_type,
      Some(special_chat.chat_members),
      None,
      user1.workspace_id,
    )
    .await?;

    assert_eq!(special_chat.name, special_name);

    // ===== 3. Test extremely long description =====

    // Create chat with long description
    let long_desc = "a".repeat(1000); // Very long description
    let long_desc_chat = CreateChat::new(
      "Long Description Chat",
      ChatType::Group,
      vec![user2.id, user3.id],
      &long_desc,
    );
    let long_desc_chat = create_new_chat(
      &state,
      user1.id,
      &long_desc_chat.name,
      long_desc_chat.chat_type,
      Some(long_desc_chat.chat_members),
      Some(&long_desc_chat.description),
      user1.workspace_id,
    )
    .await?;

    assert_eq!(long_desc_chat.description, long_desc);

    // ===== 4. Test group chat with many members =====

    // Create group chat with all test users
    let large_group = CreateChat::new(
      "Large Group Chat",
      ChatType::Group,
      vec![
        user2.id, user3.id, user4.id, user5.id, user6.id, user7.id, user8.id,
      ],
      "",
    );
    let large_group = create_new_chat(
      &state,
      user1.id,
      &large_group.name,
      large_group.chat_type,
      Some(large_group.chat_members),
      None,
      user1.workspace_id,
    )
    .await?;

    assert_eq!(large_group.chat_members.len(), 8); // Creator + 7 members

    // ===== 5. Test different chat types with same members =====

    // Create different chat types for the same group of users
    let members = vec![user2.id];

    // Single chat
    let single_chat = CreateChat::new("Same Members Single", ChatType::Single, members.clone(), "");
    let single_chat = create_new_chat(
      &state,
      user1.id,
      &single_chat.name,
      single_chat.chat_type,
      Some(single_chat.chat_members),
      None,
      user1.workspace_id,
    )
    .await?;

    // Private channel
    let private_channel = CreateChat::new(
      "Same Members Private",
      ChatType::PrivateChannel,
      members.clone(),
      "",
    );
    let private_channel = create_new_chat(
      &state,
      user1.id,
      &private_channel.name,
      private_channel.chat_type,
      Some(private_channel.chat_members),
      None,
      user1.workspace_id,
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
    let result = create_new_chat(
      &state,
      user1.id,
      &all_spaces.name,
      all_spaces.chat_type,
      Some(all_spaces.chat_members),
      None,
      user1.workspace_id,
    )
    .await;

    assert!(result.is_err());

    // ===== 7. Test preserving creator as first member =====

    // Create group chat with creator included in target_members, check that creator appears only once and is first
    let creator_included = CreateChat::new(
      "Creator In Members",
      ChatType::Group,
      vec![user2.id, user3.id, user1.id, user3.id],
      "",
    );
    let creator_included = create_new_chat(
      &state,
      user1.id,
      &creator_included.name,
      creator_included.chat_type,
      Some(creator_included.chat_members),
      None,
      user1.workspace_id,
    )
    .await?;

    assert_eq!(creator_included.chat_members.len(), 3);
    assert_eq!(creator_included.chat_members[0], user1.id); // Creator should be first

    // ===== 8. Test name at database limit =====

    // Name exactly 128 characters (database limit is 128)
    let name_128 = "a".repeat(128);
    let result = create_new_chat(
      &state,
      user1.id,
      &name_128,
      ChatType::Single,
      Some(vec![user2.id]),
      None,
      user1.workspace_id,
    )
    .await?;

    assert_eq!(result.name.len(), 128);

    // ===== 9. Test member relationship record creation =====

    // Create chat and verify relationship records for all members
    let relation_test = CreateChat::new(
      "Relation Test Chat",
      ChatType::Group,
      vec![user2.id, user3.id],
      "",
    );
    let relation_test = create_new_chat(
      &state,
      user1.id,
      &relation_test.name,
      relation_test.chat_type,
      Some(relation_test.chat_members),
      None,
      user1.workspace_id,
    )
    .await?;

    // Typically would need to query database for verification, but we can indirectly verify through chat_members
    assert_eq!(relation_test.chat_members.len(), 3);

    // ===== 10. Test multiple operations on same chat =====

    // Create a normal group chat
    let normal_chat = CreateChat::new(
      "Normal Group Chat",
      ChatType::Group,
      vec![user2.id, user3.id],
      "",
    );
    let normal_chat = create_new_chat(
      &state,
      user1.id,
      &normal_chat.name,
      normal_chat.chat_type,
      Some(normal_chat.chat_members),
      None,
      user1.workspace_id,
    )
    .await?;

    assert_eq!(normal_chat.chat_members.len(), 3);
    assert!(normal_chat.chat_members.contains(&user1.id));
    assert!(normal_chat.chat_members.contains(&user2.id));
    assert!(normal_chat.chat_members.contains(&user3.id));

    // Adding implementation for edit members, etc. to satisfy TODO
    // Add a new member to the chat
    let user4_id = 999; // Simulating another user
    let add_result = add_chat_members(
      &state,
      normal_chat.id,
      user1.id, // Creator is performing the action
      vec![user4_id],
    )
    .await;

    // This should fail as user4 doesn't exist, but verifies the function is callable
    assert!(add_result.is_err());

    // Test deleting a member
    let remove_result = remove_group_chat_members(
      &state,
      normal_chat.id,
      user1.id, // Creator is performing the action
      vec![user3.id],
    )
    .await?;

    assert!(remove_result);

    // ===== 11. Test non-English names and descriptions =====

    // Create chat with non-English content
    let non_english_chat = CreateChat::new(
      "Chat with non-English name test",
      ChatType::Group,
      vec![user2.id, user3.id],
      "This is a non-English description, testing UTF-8 support",
    );
    let non_english_chat = create_new_chat(
      &state,
      user1.id,
      &non_english_chat.name,
      non_english_chat.chat_type,
      Some(non_english_chat.chat_members),
      Some(&non_english_chat.description),
      user1.workspace_id,
    )
    .await?;

    assert_eq!(non_english_chat.name, "Chat with non-English name test");
    assert_eq!(
      non_english_chat.description,
      "This is a non-English description, testing UTF-8 support"
    );

    // ===== 12. Test emoji names =====

    // Create chat with emoji in name
    let emoji_chat = CreateChat::new(
      "😀 Emoji Chat 🎉",
      ChatType::Group,
      vec![user2.id, user3.id],
      "Testing emoji support 😊👍",
    );
    let emoji_chat = create_new_chat(
      &state,
      user1.id,
      &emoji_chat.name,
      emoji_chat.chat_type,
      Some(emoji_chat.chat_members),
      Some(&emoji_chat.description),
      user1.workspace_id,
    )
    .await?;

    assert_eq!(emoji_chat.name, "😀 Emoji Chat 🎉");
    assert_eq!(emoji_chat.description, "Testing emoji support 😊👍");

    let chats = list_chats_of_user(&state, user1.id).await?;
    assert_eq!(chats.len(), 12);

    Ok(())
  }

  #[tokio::test]
  async fn create_duplicate_chat_should_fail() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];

    // 创建第一个聊天
    let chat_name = "Unique Test Chat";
    let members = vec![user2.id, user3.id];
    let first_chat = create_new_chat(
      &state,
      user1.id,
      chat_name,
      ChatType::Group,
      Some(members.clone()),
      Some("First chat description"),
      user1.workspace_id,
    )
    .await?;

    assert_eq!(first_chat.name, chat_name);

    // Attempt to create a second chat with the same name
    let result = create_new_chat(
      &state,
      user1.id,
      chat_name, // same name
      ChatType::Group,
      Some(members.clone()),
      Some("Second chat description"),
      user1.workspace_id,
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
}

#[cfg(test)]
mod process_chat_members_data_driven_test {
  use super::*;
  use crate::AppError;
  use crate::models::ChatType;
  use anyhow::Result;

  const CREATOR_ID: i64 = 1;
  const USER_2: i64 = 2;
  const USER_3: i64 = 3;

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

      // Call the function, note that .as_ref() converts Option<Vec> to Option<&Vec>
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
