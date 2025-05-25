//! API Integration Tests
//!
//! Tests end-to-end functionality of the Fechatter API

use crate::common::*;
use anyhow::Result;
use fechatter_core::{CreateMessage, CreateUser, SigninUser};
use serde_json::json;
use tracing::{info, warn};

/// User authentication flow test
#[tokio::test]
async fn test_user_authentication_flow() -> Result<()> {
  let env = TestEnvironment::new().await?;

  // Should start actual fechatter_server service here
  // For integration tests, we need a running server instance
  warn!("This test requires a running fechatter_server instance");

  // TODO: Start server or connect to existing server
  let base_url = "http://localhost:3000".to_string();
  let client = HttpClient::new(base_url);

  // Test user registration
  let signup_data = json!({
      "fullname": "Integration Test User",
      "email": "integration@example.com",
      "workspace": "IntegrationWorkspace",
      "password": "password123"
  });

  // This test will fail without an actual running server
  // This is just to demonstrate integration test structure
  info!("⚠️ This test requires a running server to complete");

  Ok(())
}

/// Chat creation and messaging test
#[tokio::test]
async fn test_chat_and_messaging_flow() -> Result<()> {
  let mut env = TestEnvironment::new().await?;

  // Create test users and extract IDs
  let users_data = env.create_test_users(3).await?.to_vec();
  let user1_id = users_data[0].id;
  let user1_workspace_id = users_data[0].workspace_id;
  let user2_id = users_data[1].id;
  let user3_id = users_data[2].id;

  // Get immutable reference to fechatter_state
  let fechatter_state = &env.fechatter_state;

  // Generate unique chat name to avoid conflicts
  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let unique_chat_name = format!("API Test Chat {}", timestamp);

  // Create chat
  let chat = fechatter_state
    .create_new_chat(
      user1_id,
      &unique_chat_name,
      fechatter_core::ChatType::Group,
      Some(vec![user1_id, user2_id, user3_id]),
      Some("Testing chat creation and messaging"),
      user1_workspace_id,
    )
    .await?;

  assert_eq!(chat.name, unique_chat_name);
  assert_eq!(chat.created_by, user1_id);

  // Create message
  let message_payload = CreateMessage {
    content: "Test message for API flow".to_string(),
    files: vec![],
    idempotency_key: uuid::Uuid::now_v7(),
  };

  let message = fechatter_state
    .create_message(message_payload, chat.id, user1_id)
    .await?;

  assert_eq!(message.content, "Test message for API flow");
  assert_eq!(message.chat_id, chat.id);
  assert_eq!(message.sender_id, user1_id);

  // List messages
  let list_query = fechatter_core::ListMessages {
    last_id: None,
    limit: 10,
  };

  let messages = fechatter_state.list_messages(list_query, chat.id).await?;

  assert!(!messages.is_empty());
  assert_eq!(messages[0].id, message.id);

  // Cleanup explicitly
  env.cleanup().await?;

  info!("✅ Chat and messaging flow test passed");
  Ok(())
}

/// Chat member management test
#[tokio::test]
async fn test_chat_member_management() -> Result<()> {
  let mut env = TestEnvironment::new().await?;

  // Create test users
  let users = env.create_test_users(4).await?;
  let creator_id = users[0].id;
  let creator_workspace_id = users[0].workspace_id;
  let member1_id = users[1].id;
  let member2_id = users[2].id;
  let new_member_id = users[3].id;

  // Generate unique chat name to avoid conflicts
  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let unique_chat_name = format!("Chat Member Management Test {}", timestamp);

  // Create chat with initial members
  let chat = env
    .fechatter_state
    .create_new_chat(
      creator_id,
      &unique_chat_name,
      fechatter_core::ChatType::Group,
      Some(vec![creator_id, member1_id, member2_id]),
      Some("Testing member management"),
      creator_workspace_id,
    )
    .await?;

  // Verify initial members
  let initial_members = env.fechatter_state.list_chat_members(chat.id).await?;
  assert_eq!(initial_members.len(), 3);

  // Add new member
  env
    .fechatter_state
    .add_chat_members(chat.id, creator_id, vec![new_member_id])
    .await?;

  // Verify member was added
  let updated_members = env.fechatter_state.list_chat_members(chat.id).await?;
  assert_eq!(updated_members.len(), 4);

  // Remove member
  env
    .fechatter_state
    .remove_group_chat_members(chat.id, creator_id, vec![new_member_id])
    .await?;

  // Verify member was removed
  let final_members = env.fechatter_state.list_chat_members(chat.id).await?;
  assert_eq!(final_members.len(), 3);

  // Cleanup explicitly
  env.cleanup().await?;

  info!("✅ Chat member management test passed");
  Ok(())
}

/// File upload/download test
#[tokio::test]
async fn test_file_upload_download() -> Result<()> {
  let mut env = TestEnvironment::new().await?;

  // Create test user
  let users = env.create_test_users(1).await?;
  let user = &users[0];

  // This test requires actual file upload functionality
  // Since we're using AppState directly, we need to simulate file upload process

  warn!("File upload/download test requires HTTP server integration");

  // Test file creation (using ChatFile)
  let test_content = b"Test file content for integration testing";
  let filename = "integration_test.txt";

  let file_meta =
    fechatter_server::models::ChatFile::new(user.workspace_id, filename, test_content);

  info!("✅ Created file metadata: {}", file_meta.url());

  // TODO: Actual file upload/download requires HTTP client and running server

  Ok(())
}

/// Concurrent operations test
#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
  let mut env = TestEnvironment::new().await?;

  // Create test users
  let users = env.create_test_users(10).await?;
  let creator_id = users[0].id;
  let creator_workspace_id = users[0].workspace_id;
  let user_ids: Vec<i64> = users.iter().map(|u| u.id).collect();

  // Generate unique chat name to avoid conflicts
  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let unique_chat_name = format!("Concurrent Test Chat {}", timestamp);

  // Create chat
  let chat = env
    .fechatter_state
    .create_new_chat(
      creator_id,
      &unique_chat_name,
      fechatter_core::ChatType::Group,
      Some(user_ids.clone()),
      Some("Testing concurrent operations"),
      creator_workspace_id,
    )
    .await?;

  // Send concurrent messages
  let message_count = 10;
  let mut handles = Vec::new();

  for i in 0..message_count {
    let fechatter_state = env.fechatter_state.clone();
    let chat_id = chat.id;
    let sender_id = user_ids[i % user_ids.len()];

    let handle = tokio::spawn(async move {
      let message_payload = CreateMessage {
        content: format!("Concurrent message {}", i),
        files: vec![],
        idempotency_key: uuid::Uuid::now_v7(),
      };

      fechatter_state
        .create_message(message_payload, chat_id, sender_id)
        .await
    });

    handles.push(handle);
  }

  // Wait for all messages to complete
  let results = futures::future::join_all(handles).await;

  // Verify all messages were created
  let successful_messages = results
    .into_iter()
    .filter_map(|result| result.ok())
    .filter_map(|message_result| message_result.ok())
    .count();

  assert_eq!(successful_messages, message_count);

  // Cleanup explicitly
  env.cleanup().await?;

  info!("✅ Concurrent operations test passed");
  Ok(())
}

/// Error handling test
#[tokio::test]
async fn test_error_handling() -> Result<()> {
  let mut env = TestEnvironment::new().await?;

  // Create test users and extract IDs - need 3 users for group chat
  let users_data = env.create_test_users(3).await?.to_vec();
  let user1_id = users_data[0].id;
  let user1_workspace_id = users_data[0].workspace_id;
  let user2_id = users_data[1].id;
  let user3_id = users_data[2].id;

  // Generate unique chat name to avoid conflicts
  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let unique_chat_name = format!("Error Test Chat {}", timestamp);

  // Test accessing non-existent chat
  let non_existent_chat_id = 999999;
  let message_payload = CreateMessage {
    content: "This should fail".to_string(),
    files: vec![],
    idempotency_key: uuid::Uuid::now_v7(),
  };

  let result = env
    .fechatter_state
    .create_message(message_payload, non_existent_chat_id, user1_id)
    .await;

  assert!(result.is_err());

  // Create a valid chat for duplicate test - need at least 3 different members for group chat
  let chat = env
    .fechatter_state
    .create_new_chat(
      user1_id,
      &unique_chat_name,
      fechatter_core::ChatType::Group,
      Some(vec![user2_id, user3_id]), // user1 is creator, add user2 and user3 as members
      Some("Error test chat"),
      user1_workspace_id,
    )
    .await?;

  // Test duplicate chat name
  let duplicate_result = env
    .fechatter_state
    .create_new_chat(
      user1_id,
      &unique_chat_name, // Same name as previous chat
      fechatter_core::ChatType::Group,
      Some(vec![user1_id]),
      Some("Duplicate name test"),
      user1_workspace_id,
    )
    .await;

  assert!(duplicate_result.is_err());

  // Test creating message with empty content and no files (should fail)
  let empty_message_payload = CreateMessage {
    content: "".to_string(),
    files: vec![],
    idempotency_key: uuid::Uuid::now_v7(),
  };

  let empty_message_result = env
    .fechatter_state
    .create_message(empty_message_payload, chat.id, user1_id)
    .await;

  assert!(empty_message_result.is_err());

  // Cleanup explicitly
  env.cleanup().await?;

  info!("✅ Error handling test passed");
  Ok(())
}

/// Data consistency test
#[tokio::test]
async fn test_data_consistency() -> Result<()> {
  let mut env = TestEnvironment::new().await?;

  // Create test users and extract IDs
  let users_data = env.create_test_users(3).await?.to_vec();
  let user1_id = users_data[0].id;
  let user1_workspace_id = users_data[0].workspace_id;
  let user2_id = users_data[1].id;
  let user3_id = users_data[2].id;

  // Get immutable reference to fechatter_state
  let fechatter_state = &env.fechatter_state;

  // Generate unique chat name to avoid conflicts
  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let unique_chat_name = format!("Consistency Test Chat {}", timestamp);

  // Create chat
  let chat = fechatter_state
    .create_new_chat(
      user1_id,
      &unique_chat_name,
      fechatter_core::ChatType::Group,
      Some(vec![user1_id, user2_id, user3_id]),
      Some("Testing data consistency"),
      user1_workspace_id,
    )
    .await?;

  // Send multiple messages
  let message_count = 5;
  let mut message_ids = Vec::new();

  for i in 0..message_count {
    let message_payload = CreateMessage {
      content: format!("Consistency test message {}", i),
      files: vec![],
      idempotency_key: uuid::Uuid::now_v7(),
    };

    let message = fechatter_state
      .create_message(message_payload, chat.id, user1_id)
      .await?;

    message_ids.push(message.id);
  }

  // Retrieve messages and verify order
  let list_query = fechatter_core::ListMessages {
    last_id: None,
    limit: message_count as i64,
  };

  let retrieved_messages = fechatter_state.list_messages(list_query, chat.id).await?;

  assert_eq!(retrieved_messages.len(), message_count);

  // Verify messages are in correct order (newest first)
  for (i, message) in retrieved_messages.iter().enumerate() {
    let expected_content = format!("Consistency test message {}", message_count - 1 - i);
    assert_eq!(message.content, expected_content);
  }

  // Cleanup explicitly
  env.cleanup().await?;

  info!("✅ Data consistency test passed");
  Ok(())
}
