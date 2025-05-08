#[cfg(test)]
mod refresh_token_tests {
  use crate::{
    models::{SigninUser, UserStatus},
    setup_test_users,
  };
  use anyhow::Result;
  use fechatter_core::{
    TokenService,
    middlewares::ActualAuthServiceProvider,
    models::jwt::{RefreshTokenService, SigninService, UserClaims},
  };
  use std::sync::Arc;
  use tokio::sync::Semaphore;

  #[tokio::test]
  async fn concurrent_token_refresh_should_not_violate_constraints() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    // Create UserClaims from user
    let user_claims = UserClaims {
      id: user.id,
      workspace_id: user.workspace_id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
    };

    // Call token_manager directly to generate the tokens
    let tokens = state
      .token_manager()
      .generate_auth_tokens(&user_claims, None, None)
      .await?;
    let refresh_token = tokens.refresh_token.token;

    let semaphore = Arc::new(Semaphore::new(0));
    let sem_clone = semaphore.clone();

    let mut handles = vec![];
    for _ in 0..5 {
      let service_provider = state.service_provider.clone();
      let token_clone = refresh_token.clone();
      let sem = sem_clone.clone();

      let handle = tokio::spawn(async move {
        let _permit = sem.acquire().await.unwrap();
        let service = service_provider.create_service();
        // Use the trait method from RefreshTokenService
        RefreshTokenService::refresh_token(&service, &token_clone, None).await
      });

      handles.push(handle);
    }

    semaphore.add_permits(5);

    let results = futures::future::join_all(handles).await;

    let success_count = results.iter().filter(|r| matches!(r, Ok(Ok(_)))).count();

    assert_eq!(success_count, 1);

    Ok(())
  }

  #[tokio::test]
  async fn disabled_user_should_not_get_refresh_token() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    // Create UserClaims from user
    let user_claims = UserClaims {
      id: user.id,
      workspace_id: user.workspace_id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
    };

    // Call token_manager directly
    let tokens = state
      .token_manager()
      .generate_auth_tokens(&user_claims, None, None)
      .await?;
    let refresh_token = tokens.refresh_token.token;

    sqlx::query("UPDATE users SET status = $1 WHERE id = $2")
      .bind(UserStatus::Suspended)
      .bind(user.id)
      .execute(state.pool())
      .await?;

    // Create the service only when we need it
    let auth_service = state.service_provider.create_service();
    // Use the trait method from RefreshTokenService
    let result = RefreshTokenService::refresh_token(&auth_service, &refresh_token, None).await;
    assert!(result.is_err());

    if let Err(err) = result {
      assert!(format!("{err:?}").contains("User account is disabled"));
    } else {
      panic!("Expected an error but got success");
    }

    Ok(())
  }

  #[tokio::test]
  async fn async_password_verification_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    let signin_user = SigninUser {
      email: user.email.clone(),
      password: "password".to_string(), // Default test password
    };

    let auth_service = state.service_provider.create_service();
    // Use the trait method from SigninService
    let result = SigninService::signin(&auth_service, &signin_user, None).await?;
    assert!(result.is_some());

    Ok(())
  }

  #[tokio::test]
  async fn token_validation_should_work_with_trait() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    // Create UserClaims from user
    let user_claims = UserClaims {
      id: user.id,
      workspace_id: user.workspace_id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
    };

    // Call token_manager directly
    let tokens = state
      .token_manager()
      .generate_auth_tokens(&user_claims, None, None)
      .await?;

    let token_manager = state.token_manager();
    let claims = token_manager.verify_token(&tokens.access_token)?;

    assert_eq!(claims.id, user.id);
    assert_eq!(claims.email, user.email);

    Ok(())
  }
}
