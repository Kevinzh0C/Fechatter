//! Workspace Integration Tests
//!
//! Tests workspace-related functionality

use crate::common::{TestAssertions, TestEnvironment};
use anyhow::Result;
use fechatter_core::{ChatType, UserId, WorkspaceId};
use log::info;

/// Test workspace creation
#[tokio::test]
async fn test_workspace_creation() -> Result<()> {
  let mut env = TestEnvironment::new().await?;

  // Create test user
  let user = env.create_test_user("workspace_test").await?;

  // Verify user's workspace was created
  assert!(user.workspace_id > fechatter_core::WorkspaceId(0));

  // Get workspace info via API
  let workspace = env
    .app_state
    .find_by_id_with_pool(user.workspace_id.into())
    .await?
    .ok_or_else(|| anyhow::anyhow!("Workspace not found"))?;

  assert_eq!(workspace.owner_id, user.id);
  assert!(!workspace.name.is_empty());

  info!("Workspace creation test passed");
  Ok(())
}

/// Test workspace member management
#[tokio::test]
async fn test_workspace_members() -> Result<()> {
  let mut env = TestEnvironment::new().await?;

  // Create multiple users in the same workspace (using same workspace name)
  let shared_workspace = format!(
    "SharedWS_{}",
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs()
  );

  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();

  let mut users = Vec::new();
  for i in 0..3 {
    let user_data = fechatter_core::CreateUser {
      email: format!("member_test_{}_{}@test.com", i, timestamp),
      fullname: format!("Member Test User {}", i),
      password: "password123".to_string(),
      workspace: shared_workspace.clone(),
    };
    let user = env.app_state.create_user(&user_data, None).await?;
    users.push(user);
  }

  // Verify all users are in the same workspace
  let workspace_id = users[0].workspace_id;
  for user in &users {
    assert_eq!(user.workspace_id, workspace_id);
  }

  // Get workspace member list
  let pool = env.pool().clone();
  let members = env
    .app_state
    .fetch_workspace_users(workspace_id.into(), &pool)
    .await?;

  assert!(members.len() >= 3);

  info!("Workspace members test passed");
  Ok(())
}

/// Test workspace isolation
#[tokio::test]
async fn test_workspace_isolation() -> Result<()> {
  let mut env = TestEnvironment::new().await?;

  // Create two separate workspaces
  let user1 = env.create_test_user("workspace1").await?;
  let workspace1_id: WorkspaceId = user1.workspace_id;

  // Create third user in same workspace (to meet minimum 3 users requirement for group chat)
  let user1_workspace_name = format!(
    "WS1_{}",
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs()
  );

  let user3_data = fechatter_core::CreateUser {
    email: format!(
      "user3_{}@test.com",
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
    ),
    fullname: "User 3".to_string(),
    password: "password123".to_string(),
    workspace: user1_workspace_name.clone(),
  };
  let user3 = env.app_state.create_user(&user3_data, None).await?;

  // Create fourth user in same workspace
  let user4_data = fechatter_core::CreateUser {
    email: format!(
      "user4_{}@test.com",
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
    ),
    fullname: "User 4".to_string(),
    password: "password123".to_string(),
    workspace: user1_workspace_name,
  };
  let user4 = env.app_state.create_user(&user4_data, None).await?;

  // Generate short unique identifier to stay within 32 char limit
  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs();

  // Create new workspace by creating new user (simulating different company)
  let user2_data = fechatter_core::CreateUser {
    email: format!("user2_{}@test.com", timestamp),
    fullname: "User 2".to_string(),
    password: "password123".to_string(),
    workspace: format!("WS2_{}", timestamp), // Keep under 32 chars
  };

  let user2 = env.app_state.create_user(&user2_data, None).await?;
  let workspace2_id: WorkspaceId = user2.workspace_id;

  // Verify the two workspaces are different
  assert_ne!(workspace1_id, workspace2_id);

  // Create chat in workspace1 (containing user1, user3 and user4, all in same workspace)
  let chat_timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_nanos();

  let chat = env
    .app_state
    .create_new_chat(
      user1.id.into(),
      &format!("Isolated Chat {}", chat_timestamp),
      ChatType::Group,
      Some(vec![user1.id.into(), user3.id.into(), user4.id.into()]), // 3 users in same workspace
      Some("Testing workspace isolation"),
      workspace1_id.into(),
    )
    .await?;

  // Verify user2 cannot access workspace1's chat
  let chats = env.app_state.list_chats_of_user(user2.id.into()).await?;

  assert!(!chats.iter().any(|c| c.id == chat.id));

  info!("Workspace isolation test passed");
  Ok(())
}

/// Test cross-workspace operation restrictions
#[tokio::test]
async fn test_cross_workspace_restrictions() -> Result<()> {
  let mut env = TestEnvironment::new().await?;

  // Create users in two different workspaces
  let user1 = env.create_test_user("workspace_a").await?;

  // Generate short unique identifier to stay within 32 char limit
  let timestamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs();

  let user2_data = fechatter_core::CreateUser {
    email: format!("cross_workspace_{}@test.com", timestamp),
    fullname: "Cross Workspace User".to_string(),
    password: "password123".to_string(),
    workspace: format!("WSB_{}", timestamp), // Keep under 32 chars
  };
  let user2 = env.app_state.create_user(&user2_data, None).await?;

  // Try to create chat in workspace1 containing workspace2 user (should fail)
  let result = env
    .app_state
    .create_new_chat(
      user1.id.into(),
      "Cross Workspace Chat",
      ChatType::Group,
      Some(vec![user1.id.into(), user2.id.into()]),
      Some("Should fail"),
      user1.workspace_id.into(),
    )
    .await;

  assert!(result.is_err(), "Cross-workspace chat creation should fail");

  info!("Cross-workspace restrictions test passed");
  Ok(())
}
