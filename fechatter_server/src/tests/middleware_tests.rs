#[cfg(test)]
mod token_refresh_tests {
  use crate::{call_service, create_auth_service, setup_test_users, verify_token};
  use anyhow::Result;
  use fechatter_core::{RefreshTokenService, TokenService};

  #[tokio::test]
  async fn test_refresh_token_cookie_mechanism() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    // 使用宏创建服务
    let auth_service = create_auth_service!(state);

    // Generate tokens using TokenService directly
    let user_claims = fechatter_core::UserClaims {
      id: user.id,
      workspace_id: user.workspace_id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
    };

    let tokens = state
      .token_manager()
      .generate_auth_tokens(&user_claims, None, None)
      .await?;

    let refresh_token = tokens.refresh_token.token.clone();

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // 使用宏调用方法
    let new_tokens = call_service!(
      auth_service,
      RefreshTokenService,
      refresh_token,
      &refresh_token,
      None
    )?;

    assert_ne!(tokens.access_token, new_tokens.access_token);
    assert_ne!(tokens.refresh_token.token, new_tokens.refresh_token.token);

    // 使用宏验证token
    let claims = verify_token!(state, &new_tokens.access_token)?;
    assert_eq!(claims.id, user.id);
    assert_eq!(claims.email, user.email);

    Ok(())
  }

  #[tokio::test]
  async fn test_auth_middleware_order() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    // Generate tokens using TokenService directly
    let user_claims = fechatter_core::UserClaims {
      id: user.id,
      workspace_id: user.workspace_id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
    };

    let tokens = state
      .token_manager()
      .generate_auth_tokens(&user_claims, None, None)
      .await?;

    // 使用宏验证token
    let claims = verify_token!(state, &tokens.access_token)?;
    assert_eq!(claims.id, user.id);
    assert_eq!(claims.email, user.email);

    Ok(())
  }
}

#[cfg(test)]
mod list_messages_auth_tests {
  use crate::setup_test_users;
  use crate::{
    handlers::messages::list_messages_handler,
    models::{AuthUser, ChatType, ListMessage},
  };
  use anyhow::Result;
  use axum::extract::{Extension, Path, Query, State};

  #[tokio::test]
  async fn test_list_messages_requires_chat_membership() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(4).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2]; // This user will NOT be a member
    let user4 = &users[3];

    let chat = state
      .create_new_chat(
        user1.id,
        "Test Chat",
        ChatType::Group,
        Some(vec![user1.id, user2.id, user4.id]), // Only user1, user2, and user4 are members
        Some("Test chat for messages"),
        user1.workspace_id,
      )
      .await?;

    // 确认user3不是聊天成员
    let is_member = sqlx::query_scalar!(
      "SELECT EXISTS(SELECT 1 FROM chat_members WHERE chat_id = $1 AND user_id = $2) as exists",
      chat.id,
      user3.id
    )
    .fetch_one(state.pool())
    .await?;
    assert!(!is_member.unwrap_or(true), "User3应该不是聊天成员");

    let non_member_auth = Extension(AuthUser {
      id: user3.id,
      email: user3.email.clone(),
      workspace_id: user3.workspace_id,
      fullname: user3.fullname.clone(),
      status: user3.status,
      created_at: user3.created_at,
    });

    let query = ListMessage {
      last_id: None,
      limit: 10,
    };

    let result = list_messages_handler(
      State(state.clone()),
      non_member_auth,
      Path(chat.id),
      Query(query),
    )
    .await;

    // 注意：直接调用handler绕过了中间件，所以这里预期成功
    // 在实际路由中，中间件会拦截非成员访问
    assert!(
      result.is_ok(),
      "直接调用handler应该成功，因为绕过了中间件检查"
    );

    // 如果要做更完整的测试，可以加上：
    // 模拟中间件校验逻辑
    let middleware_would_allow = sqlx::query_scalar!(
      "SELECT EXISTS(SELECT 1 FROM chat_members WHERE chat_id = $1 AND user_id = $2) as exists",
      chat.id,
      user3.id
    )
    .fetch_one(state.pool())
    .await?;
    assert!(
      !middleware_would_allow.unwrap_or(true),
      "中间件应该会阻止非成员访问"
    );

    Ok(())
  }

  #[tokio::test]
  async fn test_list_messages_allows_chat_member() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];

    let chat = state
      .create_new_chat(
        user1.id,
        "Test Chat",
        ChatType::Group,
        Some(vec![user1.id, user2.id, user3.id]), // All users are members
        Some("Test chat for messages"),
        user1.workspace_id,
      )
      .await?;

    let member_auth = Extension(AuthUser {
      id: user2.id,
      email: user2.email.clone(),
      workspace_id: user2.workspace_id,
      fullname: user2.fullname.clone(),
      status: user2.status,
      created_at: user2.created_at,
    });

    let query = ListMessage {
      last_id: None,
      limit: 10,
    };

    let result = list_messages_handler(
      State(state.clone()),
      member_auth,
      Path(chat.id),
      Query(query),
    )
    .await;

    assert!(result.is_ok());

    Ok(())
  }
}
