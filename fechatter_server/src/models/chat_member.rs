use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::Postgres;
use sqlx::Row;
use sqlx::Transaction;
use sqlx::postgres::PgRow;
use tracing::{error, info, warn};

use super::ChatType;
use crate::{AppError, AppState};
use fechatter_core::{Chat, ChatMember};

pub fn chat_member_from_row(row: &PgRow) -> Result<ChatMember, AppError> {
  Ok(ChatMember {
    chat_id: row.try_get("chat_id").map_err(|e| AppError::SqlxError(e))?,
    user_id: row.try_get("user_id").map_err(|e| AppError::SqlxError(e))?,
    joined_at: row
      .try_get("joined_at")
      .map_err(|e| AppError::SqlxError(e))?,
  })
}

fn member_insert_query(with_conflict_handling: bool) -> &'static str {
  if with_conflict_handling {
    "INSERT INTO chat_members_relation (chat_id, user_id)
       VALUES ($1, $2)
       ON CONFLICT (chat_id, user_id) DO NOTHING
       RETURNING chat_id, user_id, joined_at"
  } else {
    "INSERT INTO chat_members_relation (chat_id, user_id)
       VALUES ($1, $2)
       RETURNING chat_id, user_id, joined_at"
  }
}

pub async fn execute_member_insert(
  member: &CreateChatMember,
  tx: &mut Transaction<'_, Postgres>,
  with_conflict_handling: bool,
) -> Result<Option<ChatMember>, AppError> {
  let query = member_insert_query(with_conflict_handling);

  let row_opt = sqlx::query(query)
    .bind(member.chat_id)
    .bind(member.user_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e: sqlx::Error| {
      error!(
        "Database error inserting member (chat_id={}, user_id={}): {:?}",
        member.chat_id, member.user_id, e
      );
      e
    })?;

  if let Some(row) = row_opt {
    let member = chat_member_from_row(&row)?;
    Ok(Some(member))
  } else {
    Ok(None)
  }
}

#[allow(unused)]
pub async fn add_single_member(
  state: &AppState,
  chat_id: i64,
  user_id: i64,
  member_id: i64,
) -> Result<ChatMember, AppError> {
  let added_members = add_chat_members(state, chat_id, user_id, vec![member_id]).await?;

  added_members.into_iter().next().ok_or_else(|| {
    AppError::ChatValidationError(format!(
      "User {} might already be a member of chat {} or could not be added.",
      member_id, chat_id
    ))
  })
}

async fn ensure_user_is_chat_creator(
  pool: &PgPool,
  chat_id: i64,
  user_id: i64,
) -> Result<(), AppError> {
  let creator_check = CreateChatMember { chat_id, user_id };
  if !is_creator_in_chat(pool, &creator_check).await? {
    // Check if the chat exists before returning permission error
    sqlx::query("SELECT 1 FROM chats WHERE id = $1")
      .bind(chat_id)
      .fetch_optional(pool)
      .await?
      .ok_or(AppError::NotFound(vec![chat_id.to_string()]))?;

    return Err(AppError::ChatPermissionError(format!(
      "User {} is not the creator of chat {} and cannot perform this action", // If chat exists but user is not creator, return PermissionError
      user_id, chat_id
    )));
  }
  Ok(())
}

pub async fn add_chat_members(
  state: &AppState,
  chat_id: i64,
  user_id: i64,
  member_ids: Vec<i64>,
) -> Result<Vec<ChatMember>, AppError> {
  if member_ids.is_empty() {
    info!("No members to add to chat_id {}", chat_id);
    return Ok(Vec::new());
  }

  ensure_user_is_chat_creator(&state.pool, chat_id, user_id).await?;

  let added_members = sqlx::query_as!(
    ChatMember,
    r#"
    INSERT INTO chat_members_relation (chat_id, user_id)
    SELECT $1, user_id_to_add
    FROM UNNEST($2::bigint[]) AS t(user_id_to_add)
    ON CONFLICT (chat_id, user_id) DO NOTHING
    RETURNING chat_id, user_id, joined_at
    "#,
    chat_id,
    &member_ids
  )
  .fetch_all(&state.pool)
  .await
  .map_err(|e: sqlx::Error| {
    error!(
      "Database error batch inserting members into chat_id {}: {:?}",
      chat_id, e
    );
    e
  })?;

  if !added_members.is_empty() {
    for member in &added_members {
      state.chat_list_cache.remove(&member.user_id);
      info!(
        "Invalidated chat list cache for added user {}",
        member.user_id
      );
    }
  }

  Ok(added_members)
}

// Insertion for chat creation
pub(crate) async fn insert_chat_members_relation(
  chat_id: i64,
  chat_members: &[i64],
  tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), AppError> {
  for &user_id in chat_members {
    let member = CreateChatMember { chat_id, user_id };
    execute_member_insert(&member, tx, false).await?;
  }
  Ok(())
}

pub async fn remove_group_chat_members(
  state: &AppState,
  chat_id: i64,
  user_id: i64,
  target_member_ids: Vec<i64>,
) -> Result<bool, AppError> {
  if target_member_ids.is_empty() {
    info!("No members specified for removal from chat_id {}", chat_id);
    return Ok(false);
  }

  // --- Fetch Chat Info (Separate Queries) ---

  // 1. Fetch creator_id and chat_type
  let chat_info = sqlx::query!(
    r#"SELECT created_by, type AS "chat_type: ChatType" FROM chats WHERE id = $1"#,
    chat_id
  )
  .fetch_optional(&state.pool)
  .await?;

  let (created_by, chat_type) = match chat_info {
    Some(info) => (info.created_by, info.chat_type),
    None => return Err(AppError::NotFound(vec![chat_id.to_string()])), // Chat not found
  };

  // 2. Fetch current member count
  let current_member_count = sqlx::query_scalar!(
    "SELECT COUNT(*) FROM chat_members_relation WHERE chat_id = $1",
    chat_id
  )
  .fetch_one(&state.pool)
  .await?
  .unwrap_or(0); // COUNT should always return a row, handle Option just in case

  // --- Type Check ---
  if chat_type != ChatType::Group {
    return Err(AppError::ChatValidationError(
      "This operation is only valid for group chats.".to_string(),
    ));
  }

  // --- Permission Check: Only Creator ---
  if created_by != user_id {
    return Err(AppError::ChatPermissionError(format!(
      "User {} is not the creator of group chat {} and cannot remove members",
      user_id, chat_id
    )));
  }

  // --- Minimum Member Check (Potentially combined with deletion later if using CTEs) ---
  // Fetching count separately for now to maintain clarity without complex SQL yet.
  let actual_members_to_remove = sqlx::query_scalar!(
    "SELECT user_id FROM chat_members_relation WHERE chat_id = $1 AND user_id = ANY($2)",
    chat_id,
    &target_member_ids
  )
  .fetch_all(&state.pool)
  .await?;

  if actual_members_to_remove.is_empty() {
    info!(
      "None of the specified users are members of chat {}",
      chat_id
    );
    return Ok(false);
  }

  if current_member_count > 0 && (current_member_count - actual_members_to_remove.len() as i64) < 1
  {
    warn!(
      "Attempt to remove {} members from chat {} (current: {}) would leave it empty.",
      actual_members_to_remove.len(),
      chat_id,
      current_member_count
    );
    return Err(AppError::ChatValidationError(
      "Cannot remove the last member(s) of a group chat.".to_string(), // More specific error
    ));
  }

  // --- Execute Batch Deletion and Get Actually Deleted IDs ---
  let deleted_ids = sqlx::query_scalar!(
    r#"
    DELETE FROM chat_members_relation
    WHERE chat_id = $1 AND user_id = ANY($2::bigint[])
    RETURNING user_id
    "#,
    chat_id,
    &actual_members_to_remove // Use the filtered list of actual members
  )
  .fetch_all(&state.pool) // Fetch all returned user_ids
  .await
  .map_err(|e: sqlx::Error| {
    error!("Database error batch deleting members from chat {}: {:?}", chat_id, e);
    e
  })?;

  let rows_affected = deleted_ids.len() as u64;

  // --- Cache Invalidation (More Precise) ---
  if rows_affected > 0 {
    for &removed_user_id in &deleted_ids {
      // Iterate only over actually deleted IDs
      state.chat_list_cache.remove(&removed_user_id);
      info!(
        "Invalidated chat list cache for removed user {}",
        removed_user_id
      );
    }
  }

  info!(
    "User {} removed {} members from group chat {}",
    user_id, rows_affected, chat_id
  );

  Ok(rows_affected > 0) // <--- Return the actual count
}

pub async fn list_chat_members(pool: &PgPool, chat_id: i64) -> Result<Vec<ChatMember>, AppError> {
  let rows = sqlx::query!(
    r#"
    SELECT
      chat_id,
      user_id,
      joined_at
    FROM chat_members_relation
    WHERE chat_id = $1
    ORDER BY joined_at ASC
    "#,
    chat_id
  )
  .fetch_all(pool)
  .await
  .map_err(|e: sqlx::Error| {
    error!(
      "Database error listing chat members for chat_id {}: {:?}",
      chat_id, e
    );
    e
  })?;

  let members: Vec<ChatMember> = rows
    .into_iter()
    .map(|row| ChatMember {
      chat_id: row.chat_id,
      user_id: row.user_id,
      joined_at: row.joined_at,
    })
    .collect();

  Ok(members)
}

pub async fn member_exists_in_chat(
  pool: &PgPool,
  member: &CreateChatMember,
) -> Result<bool, AppError> {
  let result = sqlx::query!(
    r#"
    SELECT EXISTS(
      SELECT 1 FROM chat_members_relation
      WHERE user_id = $1 AND chat_id = $2
    ) as "exists!"
    "#,
    member.user_id,
    member.chat_id
  )
  .fetch_one(pool)
  .await
  .map_err(|e: sqlx::Error| {
    error!(
      "Database error checking if member exists (user_id: {}, chat_id: {}): {:?}",
      member.user_id, member.chat_id, e
    );
    e
  })?;

  let exists = result.exists;
  info!(
    "Member exists check for user_id {} in chat_id {}: {}",
    member.user_id, member.chat_id, exists
  );
  Ok(exists)
}

pub async fn is_creator_in_chat(
  pool: &PgPool,
  member: &CreateChatMember,
) -> Result<bool, AppError> {
  let result = sqlx::query!(
    r#"
    SELECT EXISTS(
      SELECT 1 FROM chats WHERE id = $1 AND created_by = $2
    ) as "exists!"
    "#,
    member.chat_id,
    member.user_id
  )
  .fetch_one(pool)
  .await?;

  Ok(result.exists)
}

#[allow(unused)]
pub async fn count_members(pool: &PgPool, chat_id: i64) -> Result<i64, AppError> {
  let result = sqlx::query!(
    r#"
    SELECT COUNT(*) as "count!"
    FROM chat_members_relation
    WHERE chat_id = $1
    "#,
    chat_id
  )
  .fetch_one(pool)
  .await?;

  Ok(result.count)
}

#[allow(unused)]
pub async fn get_chat_type(pool: &PgPool, chat_id: i64) -> Result<ChatType, AppError> {
  let result = sqlx::query_as::<_, Chat>("SELECT type as chat_type FROM chats WHERE id = $1")
    .bind(chat_id)
    .fetch_one(pool)
    .await?;

  Ok(result.chat_type)
}

#[allow(unused)]
pub async fn transfer_chat_ownership(
  state: &AppState,
  chat_id: i64,
  from_user_id: i64,
  to_user_id: i64,
) -> Result<bool, AppError> {
  let mut tx = state.pool.begin().await?;

  // Fetch chat details, including members for potential invalidation
  let chat = sqlx::query_as::<_, Chat>(
    "SELECT id, created_by, type as chat_type, chat_members,
     chat_name as name, COALESCE(description, '') as description,
     created_at, updated_at, workspace_id
     FROM chats
     WHERE id = $1
     FOR UPDATE",
  )
  .bind(chat_id)
  .fetch_optional(&mut *tx)
  .await?
  .ok_or(AppError::NotFound(vec![chat_id.to_string()]))?;

  if chat.chat_type != ChatType::Group {
    tx.rollback().await?;
    return Err(AppError::ChatValidationError(
      "Only group chats can be transferred".to_string(),
    ));
  }

  if chat.created_by != from_user_id {
    tx.rollback().await?;
    return Err(AppError::ChatPermissionError(
      "Only the creator can transfer ownership".to_string(),
    ));
  }

  // Check if the target user is already a member using the fetched members
  if !chat.chat_members.contains(&to_user_id) {
    tx.rollback().await?; // Rollback before returning error
    return Err(AppError::ChatValidationError(
      "Target user must be a chat member to receive ownership".to_string(),
    ));
  }

  // Update the creator
  let rows_affected = sqlx::query!(
    r#"
    UPDATE chats
    SET created_by = $1
    WHERE id = $2
    "#,
    to_user_id,
    chat_id
  )
  .execute(&mut *tx)
  .await?
  .rows_affected();

  if rows_affected == 0 {
    tx.rollback().await?;
    return Err(AppError::ChatPermissionError(format!(
      "Failed to update chat ownership for chat_id {}: 0 rows affected, possibly due to concurrent modification or deletion after lock acquisition.",
      chat_id
    )));
  }

  tx.commit().await?;

  // --- Cache Invalidation ---
  // Invalidate cache for all members of the chat, as ownership change might affect visibility/sorting
  for &member_id in &chat.chat_members {
    state.chat_list_cache.remove(&member_id);
    info!(
      "Invalidated chat list cache for user {} due to ownership transfer of chat {}",
      member_id, chat_id
    );
  }
  // --- Cache Invalidation End ---

  info!(
    "Chat ownership transferred: chat_id: {}, from user_id: {}, to user_id: {}",
    chat_id, from_user_id, to_user_id
  );

  Ok(true)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatMember {
  pub chat_id: i64,
  pub user_id: i64,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::setup_test_users;

  #[tokio::test]
  async fn transfer_chat_ownership_should_work() -> anyhow::Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];

    // Create a group chat with user1 as creator and user2, user3 as members
    let chat = state
      .create_new_chat(
        user1.id,
        "Test Chat",
        ChatType::Group,
        Some(vec![user2.id, user3.id]),
        None,
        user1.workspace_id,
      )
      .await?;

    // Verify initial creator
    let initial_is_creator = is_creator_in_chat(
      &state.pool,
      &CreateChatMember {
        chat_id: chat.id,
        user_id: user1.id,
      },
    )
    .await?;
    assert!(initial_is_creator, "User1 should be the initial creator");

    // Transfer ownership from user1 to user2
    let transfer_result = transfer_chat_ownership(&state, chat.id, user1.id, user2.id).await;
    assert!(transfer_result.is_ok(), "Ownership transfer should succeed");

    // Verify that user2 is now the creator
    let new_is_creator = is_creator_in_chat(
      &state.pool,
      &CreateChatMember {
        chat_id: chat.id,
        user_id: user2.id,
      },
    )
    .await?;
    assert!(new_is_creator, "User2 should now be the creator");

    // Verify that user1 is no longer the creator
    let old_is_creator = is_creator_in_chat(
      &state.pool,
      &CreateChatMember {
        chat_id: chat.id,
        user_id: user1.id,
      },
    )
    .await?;
    assert!(!old_is_creator, "User1 should no longer be the creator");

    Ok(())
  }

  #[tokio::test]
  async fn is_creator_in_chat_should_work() -> anyhow::Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await; // Use named guard
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];

    // Create a chat with the user as creator
    let chat = state.create_new_chat(
      user1.id,
      "Test Chat",
      ChatType::Group,
      Some(vec![user2.id, user3.id]),
      Some("Test Description"),
      user1.workspace_id,
    )
    .await?;

    // Check if the user is the creator
    let is_creator = is_creator_in_chat(
      &state.pool,
      &CreateChatMember {
        chat_id: chat.id,
        user_id: user1.id,
      },
    )
    .await?;
    assert!(is_creator);

    // Check with a non-existent user
    let non_existent_user_id = 9999;
    let not_creator = is_creator_in_chat(
      &state.pool,
      &CreateChatMember {
        chat_id: chat.id,
        user_id: non_existent_user_id,
      },
    )
    .await?;
    assert!(!not_creator, "Non-existent user should not be the creator");

    Ok(())
  }

  #[tokio::test]
  async fn member_exists_in_chat_should_work() -> anyhow::Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await; // Use named guard
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];

    // Create a chat with user1 as creator and user2, user3 as members
    let chat = state.create_new_chat( 
      user1.id,
      "Test Chat",
      ChatType::Group,
      Some(vec![user2.id, user3.id]),
      None,
      user1.workspace_id,
    )
    .await?;

    // Check if all users are members
    let is_member1 = member_exists_in_chat(
      &state.pool,
      &CreateChatMember {
        chat_id: chat.id,
        user_id: user1.id,
      },
    )
    .await?;
    let is_member2 = member_exists_in_chat(
      &state.pool,
      &CreateChatMember {
        chat_id: chat.id,
        user_id: user2.id,
      },
    )
    .await?;
    let is_member3 = member_exists_in_chat(
      &state.pool,
      &CreateChatMember {
        chat_id: chat.id,
        user_id: user3.id,
      },
    )
    .await?;

    assert!(is_member1, "User1 should be a member of the chat");
    assert!(is_member2, "User2 should be a member of the chat");
    assert!(is_member3, "User3 should be a member of the chat");

    // Check with a non-existent user
    let non_existent_user_id = 9999;
    let not_member = member_exists_in_chat(
      &state.pool,
      &CreateChatMember {
        chat_id: chat.id,
        user_id: non_existent_user_id,
      },
    )
    .await?;
    assert!(!not_member, "Non-existent user should not be a member");

    Ok(())
  }

  #[tokio::test]
  async fn add_and_remove_chat_members_should_work() -> anyhow::Result<()> {
    let (_tdb, state, users) = setup_test_users!(5).await; // Use named guard
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let user4 = &users[3];
    let user5 = &users[4];

    // Create a chat with user1 as creator and initial members
    let chat = state.create_new_chat(
      user1.id,
      "Test Chat",
      ChatType::Group,
      Some(vec![user2.id, user3.id]),
      None,
      user1.workspace_id,
    )
    .await?;

    // Test batch addition of members
    let members_to_add = vec![user4.id, user5.id];
    let added_members = add_chat_members(&state, chat.id, user1.id, members_to_add).await?;

    // Verify the number of added members
    assert_eq!(added_members.len(), 2);

    // Verify all members exist in chat
    for user_id in &[user1.id, user2.id, user3.id, user4.id, user5.id] {
      let exists = member_exists_in_chat(
        &state.pool,
        &CreateChatMember {
          chat_id: chat.id,
          user_id: *user_id,
        },
      )
      .await?;
      assert!(exists, "User {} should be a member of the chat", user_id);
    }

    // Count total members (should be 5: creator + initial 2 + added 2)
    let count = count_members(&state.pool, chat.id).await?;
    assert_eq!(count, 5, "Chat should have 5 members total");

    // Test duplicate addition (should be idempotent)
    let duplicate_members =
      add_chat_members(&state, chat.id, user1.id, vec![user4.id, user5.id]).await?;
    assert_eq!(
      duplicate_members.len(),
      0,
      "Should not add duplicate members"
    );

    // Count members again (should still be 5)
    let count_after_duplicates = count_members(&state.pool, chat.id).await?;
    assert_eq!(
      count_after_duplicates, 5,
      "Count should remain 5 after duplicate additions"
    );

    // Test removing a member
    remove_group_chat_members(&state, chat.id, user1.id, vec![user4.id]).await?;

    // Verify user4 is no longer a member
    let is_still_member = member_exists_in_chat(
      &state.pool,
      &CreateChatMember {
        chat_id: chat.id,
        user_id: user4.id,
      },
    )
    .await?;
    assert!(
      !is_still_member,
      "User4 should not be a member after removal"
    );

    // Count members one more time (should be 4)
    let final_count = count_members(&state.pool, chat.id).await?;
    assert_eq!(final_count, 4);

    Ok(())
  }

  #[tokio::test]
  async fn list_chat_members_should_work() -> anyhow::Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];

    // Create a chat with user1 as creator and user2, user3 as members
    let chat = state.create_new_chat(
      user1.id,
      "Test Chat",
      ChatType::Group,
      Some(vec![user2.id, user3.id]),
      None,
      user1.workspace_id,
    )
    .await?;

    // List members
    let members = list_chat_members(&state.pool, chat.id).await?;

    // Check count
    assert_eq!(members.len(), 3);

    // Check if all users are in the members list
    let member_ids: Vec<i64> = members.iter().map(|m| m.user_id).collect();
    assert!(member_ids.contains(&user1.id));
    assert!(member_ids.contains(&user2.id));
    assert!(member_ids.contains(&user3.id));

    Ok(())
  }
}
