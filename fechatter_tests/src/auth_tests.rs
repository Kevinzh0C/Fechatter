//! Authentication Integration Tests
//!
//! Tests authentication and authorization functionality

use crate::common::{TestEnvironment, TestFixtures};
use anyhow::Result;
use fechatter_core::{CreateUser, SigninUser};
use log::info;

/// 测试用户注册流程
#[tokio::test]
async fn test_user_registration() -> Result<()> {
  let env = TestEnvironment::new().await?;

  // 创建用户数据
  let user_data = TestFixtures::create_user("auth_test");

  // 注册用户
  let user = env.app_state.create_user(&user_data, None).await?;

  // 验证用户创建成功
  assert!(user.id > fechatter_core::UserId(0));
  assert_eq!(user.email, user_data.email);
  assert_eq!(user.fullname, user_data.fullname);
  assert!(user.workspace_id > fechatter_core::WorkspaceId(0));

  info!("✅ User registration test passed");
  Ok(())
}

/// 测试用户登录流程
#[tokio::test]
async fn test_user_signin() -> Result<()> {
  let mut env = TestEnvironment::new().await?;

  // 创建测试用户
  let password = "test_password123";
  let email = format!("signin_test_{}@test.com", uuid::Uuid::new_v4());
  let user_data = CreateUser {
    email: email.clone(),
    fullname: "Signin Test User".to_string(),
    password: password.to_string(),
    workspace: "TestWorkspace".to_string(),
  };

  let _user = env.app_state.create_user(&user_data, None).await?;

  // 测试登录
  let signin_payload = SigninUser {
    email,
    password: password.to_string(),
  };
  let tokens = env
    .app_state
    .signin(&signin_payload, None)
    .await?
    .ok_or_else(|| anyhow::anyhow!("Signin failed to return tokens"))?;

  // 验证令牌
  assert!(!tokens.access_token.is_empty());
  assert!(!tokens.refresh_token.token.is_empty());

  info!("✅ User signin test passed");
  Ok(())
}

/// 测试错误的登录凭据
#[tokio::test]
async fn test_invalid_signin() -> Result<()> {
  let env = TestEnvironment::new().await?;

  // 尝试使用不存在的用户登录
  let signin_payload_nonexistent = SigninUser {
    email: "nonexistent@test.com".to_string(),
    password: "password123".to_string(),
  };
  let result_nonexistent = env
    .app_state
    .signin(&signin_payload_nonexistent, None)
    .await?;

  assert!(
    result_nonexistent.is_none(),
    "Signin with non-existent user should return None or an error that leads to None"
  );

  // 创建用户然后使用错误密码登录
  let user_data = TestFixtures::create_user("invalid_signin");
  let _user = env.app_state.create_user(&user_data, None).await?;

  let signin_payload_wrong_password = SigninUser {
    email: user_data.email.clone(),
    password: "wrong_password".to_string(),
  };
  let result_wrong_password = env
    .app_state
    .signin(&signin_payload_wrong_password, None)
    .await?;

  assert!(
    result_wrong_password.is_none(),
    "Signin with wrong password should return None or an error that leads to None"
  );

  info!("✅ Invalid signin test passed");
  Ok(())
}

/// 测试令牌刷新
#[tokio::test]
async fn test_token_refresh() -> Result<()> {
  let env = TestEnvironment::new().await?;

  // 创建用户并登录
  let user_data = TestFixtures::create_user("refresh_test");
  let _user = env.app_state.create_user(&user_data, None).await?;

  let signin_payload = SigninUser {
    email: user_data.email.clone(),
    password: user_data.password.clone(),
  };
  let initial_tokens = env
    .app_state
    .signin(&signin_payload, None)
    .await?
    .ok_or_else(|| anyhow::anyhow!("Initial signin failed"))?;

  // 等待一小段时间确保令牌时间戳不同
  tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;

  // 刷新令牌
  let new_tokens = env
    .app_state
    .refresh_token(&initial_tokens.refresh_token.token, None)
    .await?;

  // 验证新令牌
  assert!(!new_tokens.access_token.is_empty());
  assert!(!new_tokens.refresh_token.token.is_empty());
  assert_ne!(new_tokens.access_token, initial_tokens.access_token);
  assert_ne!(
    new_tokens.refresh_token.token,
    initial_tokens.refresh_token.token
  );

  info!("✅ Token refresh test passed");
  Ok(())
}

/// 测试重复注册
#[tokio::test]
async fn test_duplicate_registration() -> Result<()> {
  let env = TestEnvironment::new().await?;

  // 创建第一个用户
  let user_data = TestFixtures::create_user("duplicate_test");
  let _user1 = env.app_state.create_user(&user_data, None).await?;

  // 尝试使用相同邮箱注册
  let result = env.app_state.create_user(&user_data, None).await;

  assert!(result.is_err(), "Duplicate email registration should fail");

  info!("✅ Duplicate registration test passed");
  Ok(())
}

/// 测试密码哈希安全性
#[tokio::test]
async fn test_password_security() -> Result<()> {
  let env = TestEnvironment::new().await?;

  // 创建两个密码相同的用户
  let password = "same_password123";

  let user1_email = format!("pwd_test1_{}@test.com", uuid::Uuid::new_v4());
  let user1_data = CreateUser {
    email: user1_email.clone(),
    fullname: "Password Test 1".to_string(),
    password: password.to_string(),
    workspace: "TestWorkspace".to_string(),
  };

  let user2_email = format!("pwd_test2_{}@test.com", uuid::Uuid::new_v4());
  let user2_data = CreateUser {
    email: user2_email.clone(),
    fullname: "Password Test 2".to_string(),
    password: password.to_string(),
    workspace: "TestWorkspace".to_string(),
  };

  let _user1 = env.app_state.create_user(&user1_data, None).await?;
  let _user2 = env.app_state.create_user(&user2_data, None).await?;

  let signin1_payload = SigninUser {
    email: user1_email,
    password: password.to_string(),
  };
  let tokens1 = env
    .app_state
    .signin(&signin1_payload, None)
    .await?
    .ok_or_else(|| anyhow::anyhow!("Signin for user1 failed"))?;

  let signin2_payload = SigninUser {
    email: user2_email,
    password: password.to_string(),
  };
  let tokens2 = env
    .app_state
    .signin(&signin2_payload, None)
    .await?
    .ok_or_else(|| anyhow::anyhow!("Signin for user2 failed"))?;

  assert!(!tokens1.access_token.is_empty());
  assert!(!tokens2.access_token.is_empty());
  assert_ne!(tokens1.access_token, tokens2.access_token);

  info!("✅ Password security test passed");
  Ok(())
}

/// 测试会话管理
#[tokio::test]
async fn test_session_management() -> Result<()> {
  let env = TestEnvironment::new().await?;

  // 创建用户并登录
  let user_data = TestFixtures::create_user("session_test");
  let _user = env.app_state.create_user(&user_data, None).await?;

  // 从不同设备登录（模拟）
  let auth_context1 = Some(fechatter_core::services::AuthContext {
    user_agent: Some("Device 1".to_string()),
    ip_address: Some("192.168.1.1".to_string()),
  });

  let auth_context2 = Some(fechatter_core::services::AuthContext {
    user_agent: Some("Device 2".to_string()),
    ip_address: Some("192.168.1.2".to_string()),
  });

  let signin_payload = SigninUser {
    email: user_data.email.clone(),
    password: user_data.password.clone(),
  };

  let tokens1 = env
    .app_state
    .signin(&signin_payload, auth_context1)
    .await?
    .ok_or_else(|| anyhow::anyhow!("Signin for device 1 failed"))?;

  let tokens2 = env
    .app_state
    .signin(&signin_payload, auth_context2.clone())
    .await?
    .ok_or_else(|| anyhow::anyhow!("Signin for device 2 failed"))?;

  // 验证两个会话都有效
  assert!(!tokens1.access_token.is_empty());
  assert!(!tokens2.access_token.is_empty());
  assert_ne!(tokens1.refresh_token.token, tokens2.refresh_token.token);

  // 登出第一个会话
  env.app_state.logout(&tokens1.refresh_token.token).await?;

  // 验证第一个会话的令牌无法刷新
  let refresh_result = env
    .app_state
    .refresh_token(&tokens1.refresh_token.token, None)
    .await;

  assert!(
    refresh_result.is_err(),
    "Logged out token should not be refreshable"
  );

  // 验证第二个会话仍然有效
  let new_tokens2 = env
    .app_state
    .refresh_token(&tokens2.refresh_token.token, auth_context2)
    .await?;

  assert!(!new_tokens2.access_token.is_empty());

  info!("✅ Session management test passed");
  Ok(())
}
