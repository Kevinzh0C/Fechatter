//! Database Integration Tests
//!
//! Tests database operations for integrity and consistency

use crate::common::*;
use anyhow::Result;
use sqlx::Row;
use tracing::info;

/// Database connection test
#[tokio::test]
async fn test_database_connection() -> Result<()> {
  let mut env = TestEnvironment::new().await?;

  // Test basic database connection
  let pool = env.fechatter_state.pool();
  let result = sqlx::query("SELECT 1 as test_value")
    .fetch_one(pool)
    .await?;

  let test_value: i32 = result.get("test_value");
  assert_eq!(test_value, 1);

  // Cleanup explicitly
  env.cleanup().await?;

  info!("✅ Database connection test passed");
  Ok(())
}

/// Database transaction test
#[tokio::test]
async fn test_database_transactions() -> Result<()> {
  let mut env = TestEnvironment::new().await?;

  // Create test user
  let user = env.create_test_user("tx_test").await?;
  let user_id = user.id;
  let workspace_id = user.workspace_id;

  // Generate unique chat name to avoid conflicts
  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let unique_chat_name = format!("Transaction Test Chat {}", timestamp);

  // Get reference to avoid borrow conflicts
  let pool = env.fechatter_state.pool();
  let mut tx = pool.begin().await?;

  // Create chat within transaction
  let _result = sqlx::query(
    "INSERT INTO chats (chat_name, created_by, type, chat_members, workspace_id) VALUES ($1, $2, $3::chat_type, $4, $5)",
  )
  .bind(&unique_chat_name)
  .bind(user_id)
  .bind("group")
  .bind(vec![user_id])
  .bind(workspace_id)
  .execute(&mut *tx)
  .await?;

  // Rollback transaction
  tx.rollback().await?;

  // Verify chat was not created
  let chat_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM chats WHERE chat_name = $1")
    .bind(&unique_chat_name)
    .fetch_one(pool)
    .await?;

  assert_eq!(chat_count, 0);

  // Cleanup explicitly
  env.cleanup().await?;

  info!("✅ Database transaction rollback test passed");
  Ok(())
}

/// Database constraint test
#[tokio::test]
async fn test_database_constraints() -> Result<()> {
  let mut env = TestEnvironment::new().await?;

  // Create test user
  let user = env.create_test_user("constraint_test").await?;
  let user_id = user.id;

  // Get reference to avoid borrow conflicts
  let pool = env.fechatter_state.pool();

  // Try to create message referencing non-existent chat
  let invalid_chat_id = 999999;
  let result =
    sqlx::query("INSERT INTO messages (chat_id, sender_id, content) VALUES ($1, $2, $3)")
      .bind(invalid_chat_id)
      .bind(user_id)
      .bind("This should fail")
      .execute(pool)
      .await;

  assert!(result.is_err());

  // Cleanup explicitly
  env.cleanup().await?;

  info!("✅ Foreign key constraint test passed");
  Ok(())
}

/// Data integrity test
#[tokio::test]
async fn test_data_integrity() -> Result<()> {
  let mut env = TestEnvironment::new().await?;

  // Create test users
  let users = env.create_test_users(3).await?;
  let user1_id = users[0].id;
  let user1_workspace_id = users[0].workspace_id;
  let user_ids: Vec<i64> = users.iter().map(|u| u.id).collect();

  // Generate unique chat name to avoid conflicts
  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let unique_chat_name = format!("Integrity Test Chat {}", timestamp);

  // Create chat and verify data integrity
  let chat = env
    .fechatter_state
    .create_new_chat(
      user1_id,
      &unique_chat_name,
      fechatter_core::ChatType::Group,
      Some(user_ids),
      Some("Testing data integrity"),
      user1_workspace_id,
    )
    .await?;

  // Get reference to avoid borrow conflicts
  let pool = env.fechatter_state.pool();

  // Check record in chats table
  let chat_record: (i64, String, i64) =
    sqlx::query_as("SELECT id, chat_name, created_by FROM chats WHERE id = $1")
      .bind(chat.id)
      .fetch_one(pool)
      .await?;

  assert_eq!(chat_record.0, chat.id);
  assert_eq!(chat_record.1, unique_chat_name);
  assert_eq!(chat_record.2, user1_id);

  // Check records in chat_members table
  let member_count: i64 =
    sqlx::query_scalar("SELECT COUNT(*) FROM chat_members WHERE chat_id = $1")
      .bind(chat.id)
      .fetch_one(pool)
      .await?;

  assert_eq!(member_count, 3);

  // Cleanup explicitly
  env.cleanup().await?;

  info!("✅ Data integrity test passed");
  Ok(())
}
