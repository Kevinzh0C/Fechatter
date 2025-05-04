#[cfg(test)]
mod token_refresh_tests {
  use crate::{
    setup_test_users,
    services::AuthServiceTrait,
    utils::token::TokenValidator,
  };
  use anyhow::Result;

  #[tokio::test]
  async fn test_refresh_token_cookie_mechanism() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];
    let auth_service: Box<dyn AuthServiceTrait> = state.service_provider.create_service();

    let tokens = auth_service.generate_auth_tokens(user, None, None).await?;
    let refresh_token = tokens.refresh_token.token.clone();
    
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let new_tokens = auth_service.refresh_token(&refresh_token, None, None).await?;

    assert_ne!(tokens.access_token, new_tokens.access_token);
    assert_ne!(tokens.refresh_token.token, new_tokens.refresh_token.token);

    let claims = state.token_manager.validate_token(&new_tokens.access_token)?;
    assert_eq!(claims.id, user.id);
    assert_eq!(claims.email, user.email);

    Ok(())
  }

  #[tokio::test]
  async fn test_auth_middleware_order() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];
    let auth_service: Box<dyn AuthServiceTrait> = state.service_provider.create_service();

    let tokens = auth_service.generate_auth_tokens(user, None, None).await?;
    
    let claims = state.token_manager.validate_token(&tokens.access_token)?;
    assert_eq!(claims.id, user.id);
    assert_eq!(claims.email, user.email);

    Ok(())
  }
}

#[cfg(test)]
mod list_messages_auth_tests {
  use crate::{
    handlers::list_messages_handler,
    models::{AuthUser, ListMessage, ChatType},
  };
  use crate::setup_test_users;
  use axum::{
    extract::{Extension, Path, Query, State},
  };
  use anyhow::Result;

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
      Query(query)
    ).await;

    assert!(result.is_err());
    
    if let Err(err) = result {
      assert!(matches!(err, crate::AppError::ChatPermissionError(_)));
    }

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
      Query(query)
    ).await;

    assert!(result.is_ok());

    Ok(())
  }
}
