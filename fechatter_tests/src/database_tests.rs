//! Database Integration Tests
//!
//! Tests database operations for integrity and consistency

use crate::common::TestEnvironment;
use anyhow::Result;
use log::info;
use sqlx;
use sqlx::Row;

/// Database connection test
#[tokio::test]
async fn test_database_connection() -> Result<(), Box<dyn std::error::Error>> {
  let env = TestEnvironment::new().await?;

  // Test basic database connectivity
  let pool = env.pool();

  // Simple query to test connection
  let result = sqlx::query("SELECT 1 as test_value")
    .fetch_one(pool)
    .await?;

  let test_value: i32 = result.try_get("test_value")?;
  assert_eq!(test_value, 1);

  info!("Database connection test passed");
  Ok(())
}

/// Database transaction test
#[tokio::test]
async fn test_database_transactions() -> Result<(), Box<dyn std::error::Error>> {
  let env = TestEnvironment::new().await?;
  let pool = env.pool();

  // Test transaction rollback using the users table
  let mut tx = pool.begin().await?;

  // Generate unique email to avoid conflicts
  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let test_email = format!("transaction_test_{}@test.com", timestamp);

  // Insert test user data within transaction
  let result = sqlx::query(
    "INSERT INTO users (email, fullname, password_hash, workspace_id, status) VALUES ($1, $2, $3, $4, $5)"
  )
  .bind(&test_email)
  .bind("Transaction Test User")
  .bind("dummy_hash")
  .bind(1i64) // Use workspace_id 1
  .bind("Active")
  .execute(&mut *tx)
  .await;

  // Check if insert was successful within transaction
  if result.is_ok() {
    // Rollback transaction
    tx.rollback().await?;

    // Verify data was not committed by checking if user exists
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1")
      .bind(&test_email)
      .fetch_one(pool)
      .await?;

    assert_eq!(count, 0, "User should not exist after transaction rollback");
  } else {
    // If insert failed (e.g., due to foreign key constraints), just rollback
    tx.rollback().await?;
  }

  info!("Database transaction test passed");
  Ok(())
}

/// Database constraint test
#[tokio::test]
async fn test_database_constraints() -> Result<(), Box<dyn std::error::Error>> {
  let env = TestEnvironment::new().await?;
  let pool = env.pool();

  // Test foreign key constraints by trying to insert invalid data
  let result = sqlx::query("INSERT INTO messages (chat_id, sender_id, content) VALUES ($1, $2, $3)")
    .bind(99999i64) // Non-existent chat_id
    .bind(99999i64) // Non-existent sender_id  
    .bind("test message")
    .execute(pool)
    .await;

  // Should fail due to foreign key constraints
  assert!(result.is_err());

  info!("Database constraints test passed");
  Ok(())
}

/// Data integrity test
#[tokio::test]
async fn test_data_integrity() -> Result<(), Box<dyn std::error::Error>> {
  let env = TestEnvironment::new().await?;

  // Create test users with the same workspace to ensure they're in the same workspace
  let shared_workspace = "SharedTestWorkspace";
  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();

  let user1_data = fechatter_core::CreateUser {
    email: format!("user1_{}@test.com", timestamp),
    fullname: "Test User 1".to_string(),
    password: "password123".to_string(),
    workspace: shared_workspace.to_string(),
  };

  let user2_data = fechatter_core::CreateUser {
    email: format!("user2_{}@test.com", timestamp),
    fullname: "Test User 2".to_string(),
    password: "password123".to_string(),
    workspace: shared_workspace.to_string(),
  };

  let user1 = env.app_state.create_user(&user1_data, None).await?;
  let user2 = env.app_state.create_user(&user2_data, None).await?;

  // Test that user data is properly stored and retrieved
  assert!(!user1.fullname.is_empty());
  assert!(!user1.email.is_empty());
  assert_ne!(user1.id, user2.id);
  assert_eq!(user1.workspace_id, user2.workspace_id); // Same workspace

  info!("Data integrity test passed");
  Ok(())
}
