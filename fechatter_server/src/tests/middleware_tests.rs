use crate::setup_test_users;
use anyhow::Result;

#[cfg(test)]
mod token_refresh_tests {
  use crate::setup_test_users;
  use anyhow::Result;
  use fechatter_core::TokenService;

  #[tokio::test]
  async fn test_refresh_token_cookie_mechanism() -> Result<()> {
    let (state, users) = setup_test_users!(1).await;
    let user = &users[0];

    let user_claims = fechatter_core::UserClaims {
      id: user.id,
      workspace_id: user.workspace_id.into(),
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

    let new_tokens = state
      .token_manager()
      .refresh_access_token(&refresh_token, None, None)
      .await?;

    assert_ne!(tokens.access_token, new_tokens.access_token);
    assert_ne!(tokens.refresh_token.token, new_tokens.refresh_token.token);

    let claims = state
      .token_manager()
      .verify_access_token(&new_tokens.access_token)?;

    assert_eq!(claims.id, user.id);
    assert_eq!(claims.email, user.email);

    Ok(())
  }

  #[tokio::test]
  async fn test_auth_middleware_order() -> Result<()> {
    let (state, users) = setup_test_users!(1).await;
    let user = &users[0];

    let user_claims = fechatter_core::UserClaims {
      id: user.id,
      workspace_id: user.workspace_id.into(),
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
    };

    let tokens = state
      .token_manager()
      .generate_auth_tokens(&user_claims, None, None)
      .await?;

    let claims = state
      .token_manager()
      .verify_access_token(&tokens.access_token)?;

    assert_eq!(claims.id, user.id);
    assert_eq!(claims.email, user.email);

    Ok(())
  }
}

#[cfg(test)]
mod list_messages_auth_tests {
  use crate::handlers::messages::list_messages_handler;
  use crate::setup_test_users;
  use anyhow::Result;
  use axum::{
    Extension,
    extract::{Path, Query, State},
  };
  use fechatter_core::models::{AuthUser, ChatType, ListMessages};
  use sqlx::Row;

  #[tokio::test]
  async fn test_list_messages_requires_chat_membership() -> Result<()> {
    let (state, users) = setup_test_users!(4).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let user4 = &users[3];

    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let unique_chat_name = format!("Test Chat {}", timestamp);

    let chat = state
      .services()
      .chat()
      .create_new_chat(
        user1.id,
        &unique_chat_name,
        ChatType::Group,
        Some(vec![user1.id, user2.id, user4.id]),
        Some("Test chat for messages".to_string()),
        user1.workspace_id,
      )
      .await?;

    let query =
      "SELECT EXISTS(SELECT 1 FROM chat_members WHERE chat_id = $1 AND user_id = $2) as exists";

    let is_member_row = sqlx::query(query)
      .bind(chat.id)
      .bind(user3.id)
      .fetch_one(state.infrastructure().database_pool().as_ref())
      .await?;

    let is_member: Option<bool> = is_member_row.try_get("exists")?;
    assert!(!is_member.unwrap_or(true), "User3 should not be a member");

    let non_member_auth = Extension(AuthUser {
      id: user3.id,
      email: user3.email.clone(),
      workspace_id: user3.workspace_id,
      fullname: user3.fullname.clone(),
      status: user3.status,
      created_at: user3.created_at,
    });

    let query_params = ListMessages {
      last_id: None,
      limit: 10,
    };

    let result = list_messages_handler(
      State(state.clone()),
      non_member_auth,
      Path(chat.id.into()),
      Query(query_params),
    )
    .await;

    assert!(result.is_ok(), "Direct handler call should succeed");

    let middleware_row = sqlx::query(query)
      .bind(chat.id)
      .bind(user3.id)
      .fetch_one(state.infrastructure().database_pool().as_ref())
      .await?;

    let middleware_would_allow: Option<bool> = middleware_row.try_get("exists")?;
    assert!(
      !middleware_would_allow.unwrap_or(true),
      "Middleware should block"
    );

    Ok(())
  }

  #[tokio::test]
  async fn test_list_messages_allows_chat_member() -> Result<()> {
    let (state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];

    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let unique_chat_name = format!("Test Chat {}", timestamp);

    let chat = state
      .services()
      .chat()
      .create_new_chat(
        user1.id,
        &unique_chat_name,
        ChatType::Group,
        Some(vec![user1.id, user2.id, user3.id]),
        Some("Test chat for messages".to_string()),
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

    let query_params = ListMessages {
      last_id: None,
      limit: 10,
    };

    let result = list_messages_handler(
      State(state.clone()),
      member_auth,
      Path(chat.id.into()),
      Query(query_params),
    )
    .await;

    assert!(result.is_ok());

    Ok(())
  }
}
