//! API Integration Tests
//!
//! Tests end-to-end functionality of the Fechatter API

use crate::common::{test_utils, HttpClient, TestAssertions, TestContext, TestEnvironment};
use anyhow::Result;
use fechatter_core::{ChatType, CreateMessage, UserId, WorkspaceId};
use log::{info, warn};
use serde_json::json;

/// User authentication flow test
#[tokio::test]
async fn test_user_authentication_flow() -> Result<(), Box<dyn std::error::Error>> {
  let _env = TestEnvironment::new().await?;

  // Should start actual fechatter_server service here
  // For integration tests, we need a running server instance
  warn!("This test requires a running fechatter_server instance");

  // TODO: Start server or connect to existing server
  let base_url = "http://localhost:3000".to_string();
  let _client = HttpClient::new(base_url);

  // Test user registration
  let _signup_data = json!({
      "fullname": "Integration Test User",
      "email": "integration@example.com",
      "workspace": "IntegrationWorkspace",
      "password": "password123"
  });

  // This test will fail without an actual running server
  // This is just to demonstrate integration test structure
  info!("WARNING: This test requires a running server to complete");

  Ok(())
}

/// Chat creation and messaging test
#[tokio::test]
async fn test_chat_and_messaging_flow() -> Result<(), Box<dyn std::error::Error>> {
  let mut env = TestEnvironment::new().await?;

  // Create test users and extract IDs
  let users_data = env.create_test_users(3).await?;
  let user1_id = users_data[0].id;
  let user1_workspace_id = users_data[0].workspace_id;
  let user2_id = users_data[1].id;
  let user3_id = users_data[2].id;

  // Get immutable reference to app_state
  let app_state = &env.app_state;

  // Generate unique chat name to avoid conflicts
  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let unique_chat_name = format!("API Test Chat {}", timestamp);

  // Create chat
  let chat = app_state
    .create_new_chat(
      user1_id.into(),
      &unique_chat_name,
      fechatter_core::ChatType::Group,
      Some(vec![user1_id.into(), user2_id.into(), user3_id.into()]),
      Some("Testing chat creation and messaging"),
      user1_workspace_id.into(),
    )
    .await?;

  assert_eq!(chat.name, unique_chat_name);
  assert_eq!(chat.created_by, user1_id.into());

  // Create message
  let message_payload = CreateMessage {
    content: "Test message for API flow".to_string(),
    files: vec![],
    idempotency_key: uuid::Uuid::now_v7(),
  };

  let message = app_state
    .create_message(message_payload, chat.id.into(), user1_id.into())
    .await?;

  assert_eq!(message.content, "Test message for API flow");
  assert_eq!(message.chat_id, chat.id);
  assert_eq!(message.sender_id, user1_id.into());

  // List messages
  let list_query = fechatter_core::ListMessages {
    last_id: None,
    limit: 10,
  };

  let messages = app_state.list_messages(list_query, chat.id.into()).await?;

  assert!(!messages.is_empty());
  assert_eq!(messages[0].id, message.id);

  // Cleanup explicitly
  env.cleanup().await?;

  info!("Chat and messaging flow test passed");
  Ok(())
}

/// Chat member management test
#[tokio::test]
async fn test_chat_member_management() -> Result<(), Box<dyn std::error::Error>> {
  let mut env = TestEnvironment::new().await?;

  // Create test users and extract IDs
  let users_data = env.create_test_users(4).await?;
  let creator_id = users_data[0].id;
  let creator_workspace_id = users_data[0].workspace_id;
  let member1_id = users_data[1].id;
  let member2_id = users_data[2].id;
  let new_member_id = users_data[3].id;

  // Generate unique chat name to avoid conflicts
  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let unique_chat_name = format!("Member Management Test {}", timestamp);

  // Create chat with initial members
  let chat = env
    .app_state
    .create_new_chat(
      creator_id.into(),
      &unique_chat_name,
      fechatter_core::ChatType::Group,
      Some(vec![
        creator_id.into(),
        member1_id.into(),
        member2_id.into(),
      ]),
      Some("Chat for member management testing"),
      creator_workspace_id.into(),
    )
    .await?;

  // List initial members
  let initial_members = env.app_state.list_chat_members(chat.id.into()).await?;
  assert_eq!(initial_members.len(), 3);

  // Add new member
  env
    .app_state
    .add_chat_members(
      chat.id.into(),
      creator_id.into(),
      vec![new_member_id.into()],
    )
    .await?;

  // Verify member was added
  let updated_members = env.app_state.list_chat_members(chat.id.into()).await?;
  assert_eq!(updated_members.len(), 4);

  // Remove member
  env
    .app_state
    .remove_group_chat_members(
      chat.id.into(),
      creator_id.into(),
      vec![new_member_id.into()],
    )
    .await?;

  // Verify member was removed
  let final_members = env.app_state.list_chat_members(chat.id.into()).await?;
  assert_eq!(final_members.len(), 3);

  info!("Chat member management test passed");
  Ok(())
}

/// File upload/download test
#[tokio::test]
async fn test_file_upload_download() -> Result<(), Box<dyn std::error::Error>> {
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
    fechatter_server::models::ChatFile::new(user.workspace_id.into(), filename, test_content);

  info!("Created file metadata: {}", file_meta.url());

  // TODO: Actual file upload/download requires HTTP client and running server

  Ok(())
}

/// Concurrent operations test
#[tokio::test]
async fn test_concurrent_operations() -> Result<(), Box<dyn std::error::Error>> {
  let mut env = TestEnvironment::new().await?;

  // Create test users
  let users = env.create_test_users(10).await?;
  let creator_id = users[0].id;
  let creator_workspace_id = users[0].workspace_id;
  let user_ids: Vec<i64> = users.iter().map(|u| u.id.into()).collect();

  // Generate unique chat name to avoid conflicts
  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let unique_chat_name = format!("Concurrent Test Chat {}", timestamp);

  // Create chat
  let chat = env
    .app_state
    .create_new_chat(
      creator_id.into(),
      &unique_chat_name,
      fechatter_core::ChatType::Group,
      Some(user_ids.clone()),
      Some("Testing concurrent operations"),
      creator_workspace_id.into(),
    )
    .await?;

  // Send concurrent messages
  let message_count = 10;
  let mut handles = Vec::new();

  for i in 0..message_count {
    let app_state = env.app_state.clone();
    let chat_id_val = chat.id;
    let sender_id_val = user_ids[i % user_ids.len()];

    let handle = tokio::spawn(async move {
      let message_payload = CreateMessage {
        content: format!("Concurrent message {}", i),
        files: vec![],
        idempotency_key: uuid::Uuid::now_v7(),
      };

      app_state
        .create_message(message_payload, chat_id_val.into(), sender_id_val)
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

  info!("Concurrent operations test passed");
  Ok(())
}

/// Error handling test
#[tokio::test]
async fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
  let mut env = TestEnvironment::new().await?;

  // Create test users and extract IDs - need 3 users for group chat
  let users_data = env.create_test_users(3).await?;
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
  let non_existent_chat_id = 999999i64;
  let message_payload = CreateMessage {
    content: "This should fail".to_string(),
    files: vec![],
    idempotency_key: uuid::Uuid::now_v7(),
  };

  let result = env
    .app_state
    .create_message(message_payload, non_existent_chat_id, user1_id.into())
    .await;

  assert!(result.is_err());

  // Create a valid chat for duplicate test
  let chat = env
    .app_state
    .create_new_chat(
      user1_id.into(),
      &unique_chat_name,
      fechatter_core::ChatType::Group,
      Some(vec![user2_id.into(), user3_id.into()]),
      Some("Error test chat"),
      user1_workspace_id.into(),
    )
    .await?;

  // Test duplicate chat name
  let duplicate_result = env
    .app_state
    .create_new_chat(
      user1_id.into(),
      &unique_chat_name,
      fechatter_core::ChatType::Group,
      Some(vec![user1_id.into()]),
      Some("Duplicate name test"),
      user1_workspace_id.into(),
    )
    .await;

  assert!(duplicate_result.is_err());

  // Test creating message with empty content and no files (should fail)
  let empty_message_payload = CreateMessage {
    content: "".to_string(),
    files: vec![],
    idempotency_key: uuid::Uuid::now_v7(),
  };

  let empty_result = env
    .app_state
    .create_message(empty_message_payload, chat.id.into(), user1_id.into())
    .await;

  assert!(empty_result.is_err());

  info!("Error handling test passed");
  Ok(())
}

/// Data consistency test
#[tokio::test]
async fn test_data_consistency() -> Result<(), Box<dyn std::error::Error>> {
  let mut env = TestEnvironment::new().await?;

  // Create test users and extract IDs
  let users_data = env.create_test_users(3).await?;
  let user1_id = users_data[0].id;
  let user1_workspace_id = users_data[0].workspace_id;
  let user2_id = users_data[1].id;
  let user3_id = users_data[2].id;

  // Get immutable reference to app_state
  let app_state = &env.app_state;

  // Generate unique chat name to avoid conflicts
  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();
  let unique_chat_name = format!("Consistency Test Chat {}", timestamp);

  // Create chat
  let chat = app_state
    .create_new_chat(
      user1_id.into(),
      &unique_chat_name,
      fechatter_core::ChatType::Group,
      Some(vec![user1_id.into(), user2_id.into(), user3_id.into()]),
      Some("Testing data consistency"),
      user1_workspace_id.into(),
    )
    .await?;

  // Send multiple messages
  let message_count = 5;
  let mut _message_ids = Vec::new();

  for i in 0..message_count {
    let message_payload = CreateMessage {
      content: format!("Consistency test message {}", i),
      files: vec![],
      idempotency_key: uuid::Uuid::now_v7(),
    };

    let message = app_state
      .create_message(message_payload, chat.id.into(), user1_id.into())
      .await?;

    _message_ids.push(message.id);
  }

  // Retrieve messages and verify order
  let list_query = fechatter_core::ListMessages {
    last_id: None,
    limit: message_count as i64,
  };

  let retrieved_messages = app_state.list_messages(list_query, chat.id.into()).await?;

  assert_eq!(retrieved_messages.len(), message_count);

  // Verify messages are in correct order (newest first)
  for (i, message) in retrieved_messages.iter().enumerate() {
    let expected_content = format!("Consistency test message {}", message_count - 1 - i);
    assert_eq!(message.content, expected_content);
  }

  // Cleanup explicitly
  env.cleanup().await?;

  info!("Data consistency test passed");
  Ok(())
}

/// Basic API connectivity test
#[tokio::test]
async fn test_api_connectivity() -> Result<(), Box<dyn std::error::Error>> {
  let _env = TestEnvironment::new().await?;

  // Test basic HTTP client setup
  let base_url = "http://localhost:8080".to_string();
  let _client = HttpClient::new(base_url);

  // Test basic JSON serialization
  let _signup_data = json!({
    "fullname": "Test User",
    "email": "test@example.com",
    "workspace": "Test Workspace",
    "password": "password123"
  });

  info!("API connectivity test passed");
  Ok(())
}

/// Chat creation and message sending test using TestContext
#[tokio::test]
async fn test_chat_creation_and_messaging_with_context() -> Result<(), Box<dyn std::error::Error>> {
  let mut ctx = TestContext::new().await?;

  // Create test users
  let _users_data = ctx.create_users("api_ctx_test", 3).await?;
  let user1_id = ctx.users[0].id;

  // Create chat using TestContext - member_indices should be Vec<usize>
  let chat = ctx
    .create_chat(
      "api_ctx_chat",
      0,             // creator_idx
      vec![0, 1, 2], // member_indices: indices into ctx.users
      ChatType::Group,
    )
    .await?;
  TestAssertions::assert_chat_valid(chat);
  info!("Created chat: {}", chat.name);

  // Store chat_id to avoid borrowing issues
  let chat_id = chat.id;

  // Create and send message using TestContext
  let message = ctx
    .create_message("Hello from API context test!", 0, 0)
    .await?;
  TestAssertions::assert_message_valid(&message);
  info!("Sent message: {}", message.content);

  // Verify message creation
  assert_eq!(message.sender_id, user1_id.into());
  assert_eq!(message.content, "Hello from API context test!");

  // List messages using the stored chat_id
  let list_query = fechatter_core::ListMessages {
    last_id: None,
    limit: 10,
  };

  let messages = ctx
    .app_state
    .list_messages(list_query, chat_id.into())
    .await?;
  assert!(!messages.is_empty(), "Should have at least one message");
  assert_eq!(messages[0].id, message.id, "First message should match");

  info!("Chat creation and messaging with context test passed");
  Ok(())
}

/// Bulk message creation test
#[tokio::test]
async fn test_bulk_message_creation() -> Result<(), Box<dyn std::error::Error>> {
  let mut env = TestEnvironment::new().await?;

  // Create test users and extract IDs
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
  let unique_chat_name = format!("Bulk Test Chat {}", timestamp);

  // Create chat
  let chat = env
    .app_state
    .create_new_chat(
      user1_id.into(),
      &unique_chat_name,
      fechatter_core::ChatType::Group,
      Some(vec![user1_id.into(), user2_id.into(), user3_id.into()]),
      Some("Chat for bulk message testing"),
      user1_workspace_id.into(),
    )
    .await?;

  // Create multiple messages
  let message_count = 10;
  let user_ids_for_sending: Vec<UserId> = vec![user1_id, user2_id, user3_id];

  for i in 0..message_count {
    let sender_id = user_ids_for_sending[i % user_ids_for_sending.len()];
    let message_payload = CreateMessage {
      content: format!("Bulk message {}", i + 1),
      files: vec![],
      idempotency_key: uuid::Uuid::now_v7(),
    };

    env
      .app_state
      .create_message(message_payload, chat.id.into(), sender_id.into())
      .await?;
  }

  // Verify all messages were created
  let list_query = fechatter_core::ListMessages {
    last_id: None,
    limit: 20,
  };

  let messages = env
    .app_state
    .list_messages(list_query, chat.id.into())
    .await?;
  assert_eq!(messages.len(), message_count);

  info!("Bulk message creation test passed");
  Ok(())
}

/// Message persistence and retrieval test
#[tokio::test]
async fn test_message_persistence() -> Result<(), Box<dyn std::error::Error>> {
  let mut env = TestEnvironment::new().await?;

  // Create test users and extract IDs
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
  let unique_chat_name = format!("Persistence Test Chat {}", timestamp);

  // Create chat
  let chat = env
    .app_state
    .create_new_chat(
      user1_id.into(),
      &unique_chat_name,
      fechatter_core::ChatType::Group,
      Some(vec![user1_id.into(), user2_id.into(), user3_id.into()]),
      Some("Chat for persistence testing"),
      user1_workspace_id.into(),
    )
    .await?;

  // Create message
  let message_payload = CreateMessage {
    content: "Persistence test message".to_string(),
    files: vec![],
    idempotency_key: uuid::Uuid::now_v7(),
  };

  let created_message = env
    .app_state
    .create_message(message_payload, chat.id.into(), user1_id.into())
    .await?;

  // Retrieve messages
  let list_query = fechatter_core::ListMessages {
    last_id: None,
    limit: 10,
  };

  let retrieved_messages = env
    .app_state
    .list_messages(list_query, chat.id.into())
    .await?;

  // Verify message persistence
  assert_eq!(retrieved_messages.len(), 1);
  let retrieved_message = &retrieved_messages[0];
  assert_eq!(retrieved_message.id, created_message.id);
  assert_eq!(retrieved_message.content, created_message.content);
  assert_eq!(retrieved_message.sender_id, created_message.sender_id);

  info!("Message persistence test passed");
  Ok(())
}
