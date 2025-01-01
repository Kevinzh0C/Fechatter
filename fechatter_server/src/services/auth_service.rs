// fechatter_server/src/services/auth_service.rs
use anyhow::{Result, anyhow};
use chrono::Utc;
use sqlx::PgPool;
use tokio::time::{Duration, sleep};
use tracing::{error, info};

use crate::{
  AppError,
  models::{CreateUser, SigninUser, User},
  utils::jwt::{
    self, ACCESS_TOKEN_EXPIRATION, AuthTokens, RefreshToken, RefreshTokenData, TokenManager,
    generate_refresh_token,
  },
};

/// Handles user signup, creates user, generates tokens.
pub async fn signup(
  pool: &PgPool,
  token_manager: &TokenManager,
  payload: &CreateUser,
) -> Result<AuthTokens, AppError> {
  let user = User::create(payload, pool).await?;

  // TODO: Capture user_agent and ip_address from request in handler
  let user_agent = None;
  let ip_address = None;

  let tokens = token_manager
    .generate_auth_tokens(&user, user_agent, ip_address, pool)
    .await?;

  info!("User signed up successfully: {}", user.email);
  Ok(tokens)
}

/// Handles user signin, authenticates user, generates tokens.
pub async fn signin(
  pool: &PgPool,
  token_manager: &TokenManager,
  payload: &SigninUser,
) -> Result<Option<AuthTokens>, AppError> {
  match User::authenticate(payload, pool).await? {
    Some(user) => {
      let user_agent = None;
      let ip_address = None;

      let tokens = token_manager
        .generate_auth_tokens(&user, user_agent, ip_address, pool)
        .await?;
      info!("User signed in successfully: {}", user.email);
      Ok(Some(tokens))
    }
    None => {
      info!("Sign in failed for email: {}", payload.email);
      Ok(None) // Indicate invalid credentials (handler will map this to 403)
    }
  }
}

/// Handles token refresh, validates old token, generates new tokens.
pub async fn refresh_token(
  pool: &PgPool,
  token_manager: &TokenManager,
  refresh_token_str: &str,
) -> Result<AuthTokens, AppError> {
  let token_record = RefreshToken::find_by_token(refresh_token_str, pool)
    .await?
    .ok_or_else(|| {
      info!("Refresh token not found or expired");
      // Use a more generic error, the handler will decide the HTTP status
      AppError::InvalidInput("Invalid or expired refresh token".to_string())
    })?;

  // Check if token is expired or revoked explicitly
  if token_record.expires_at < Utc::now() {
    info!(
      "Refresh token expired for user: {} (Expiry: {}, Now: {})",
      token_record.user_id,
      token_record.expires_at,
      Utc::now()
    );
    return Err(AppError::InvalidInput(
      "Invalid or expired refresh token".to_string(),
    ));
  }

  let user = User::find_by_id(token_record.user_id, pool)
    .await?
    .ok_or_else(|| {
      error!(
        "User not found for valid refresh token: user_id {}",
        token_record.user_id
      );
      // Use a more specific internal error or a generic one if UserNotFound doesn't exist
      AppError::AnyError(anyhow!("User linked to refresh token not found"))
    })?;

  // TODO: Capture user_agent and ip_address from request in handler
  let user_agent = None;
  let ip_address = None;

  // Generate new tokens and replace the old refresh token
  let new_refresh_token_str = generate_refresh_token();
  let new_token_db_record = token_record
    .replace(&new_refresh_token_str, user_agent, ip_address, pool)
    .await?;

  let access_token = token_manager.generate_token(&user)?;

  info!("Token refreshed successfully for user: {}", user.email);
  // Construct RefreshTokenData from the *new* token string and the *new* db record
  let refresh_token_data = RefreshTokenData {
    token: new_refresh_token_str,
    expires_at: new_token_db_record.expires_at,
    absolute_expires_at: new_token_db_record.absolute_expires_at,
  };

  Ok(AuthTokens {
    access_token,
    refresh_token: refresh_token_data,
  })
}

/// Handles user logout, revokes the refresh token.
pub async fn logout(pool: &PgPool, refresh_token_str: &str) -> Result<(), AppError> {
  if let Some(token_record) = RefreshToken::find_by_token(refresh_token_str, pool).await? {
    token_record.revoke(pool).await?;
    info!("Refresh token revoked for user: {}", token_record.user_id);
  } else {
    info!("Logout attempt with non-existent or expired refresh token.");
    // Optionally return an error or just succeed silently
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{assert_handler_error, assert_handler_success, setup_test_users};
  use anyhow::Result;

  #[tokio::test]
  async fn refresh_token_service_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    let initial_tokens = state
      .token_manager
      .generate_auth_tokens(user, None, None, &state.pool)
      .await?;

    sleep(Duration::from_secs(1)).await;

    let refresh_result = refresh_token(
      &state.pool,
      &state.token_manager,
      &initial_tokens.refresh_token.token,
    )
    .await?;

    assert_ne!(refresh_result.access_token, initial_tokens.access_token);
    assert_ne!(
      refresh_result.refresh_token.token,
      initial_tokens.refresh_token.token
    );

    Ok(())
  }

  #[tokio::test]
  async fn refresh_token_service_should_fail_with_invalid_token() -> Result<()> {
    let (_tdb, state, _users) = setup_test_users!(1).await;

    let result = refresh_token(&state.pool, &state.token_manager, "invalid_token").await;

    assert!(result.is_err());
    if let Err(AppError::InvalidInput(_)) = result {
    } else {
      panic!("Expected InvalidInput error");
    }

    Ok(())
  }

  #[tokio::test]
  async fn logout_service_should_revoke_token() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    let tokens = state
      .token_manager
      .generate_auth_tokens(user, None, None, &state.pool)
      .await?;

    logout(&state.pool, &tokens.refresh_token.token).await?;

    let refresh_result = refresh_token(
      &state.pool,
      &state.token_manager,
      &tokens.refresh_token.token,
    )
    .await;

    assert!(refresh_result.is_err());

    Ok(())
  }

  #[tokio::test]
  async fn refresh_token_can_only_be_used_once() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    let initial_tokens = state
      .token_manager
      .generate_auth_tokens(user, None, None, &state.pool)
      .await?;

    let new_tokens = refresh_token(
      &state.pool,
      &state.token_manager,
      &initial_tokens.refresh_token.token,
    )
    .await?;

    let result = refresh_token(
      &state.pool,
      &state.token_manager,
      &initial_tokens.refresh_token.token,
    )
    .await;

    assert!(result.is_err());

    let another_refresh = refresh_token(
      &state.pool,
      &state.token_manager,
      &new_tokens.refresh_token.token,
    )
    .await;

    assert!(another_refresh.is_ok());

    Ok(())
  }

  #[tokio::test]
  async fn signup_service_should_create_user_and_tokens() -> Result<()> {
    let (_tdb, state, _) = setup_test_users!(0).await; // Setup DB, no initial users needed

    let create_user_payload = CreateUser {
      workspace: "Acme".to_string(), // Use workspace (String) instead of workspace_id
      fullname: "New User".to_string(),
      email: "newuser@test.com".to_string(),
      password: "password123".to_string(),
    };

    let result = signup(&state.pool, &state.token_manager, &create_user_payload).await;

    assert!(result.is_ok());
    let tokens = result.unwrap();
    assert!(!tokens.access_token.is_empty());
    assert!(!tokens.refresh_token.token.is_empty());

    // Optional: Verify user exists in DB
    let user_check = User::email_user_exists("newuser@test.com", &state.pool).await?;
    assert!(user_check.is_some());
    assert_eq!(user_check.unwrap().fullname, "New User");

    Ok(())
  }

  #[tokio::test]
  async fn signin_service_should_return_tokens_for_valid_credentials() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await; // Setup with one user
    let user = &users[0];

    let signin_payload = SigninUser {
      email: user.email.clone(),
      password: "password".to_string(), // Default password from setup_test_users
    };

    let result = signin(&state.pool, &state.token_manager, &signin_payload).await?;

    assert!(result.is_some());
    let tokens = result.unwrap();
    assert!(!tokens.access_token.is_empty());
    assert!(!tokens.refresh_token.token.is_empty());

    Ok(())
  }

  #[tokio::test]
  async fn signin_service_should_return_none_for_invalid_password() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    let signin_payload = SigninUser {
      email: user.email.clone(),
      password: "wrongpassword".to_string(), // Incorrect password
    };

    let result = signin(&state.pool, &state.token_manager, &signin_payload).await?;

    assert!(result.is_none());

    Ok(())
  }

  #[tokio::test]
  async fn signin_service_should_return_none_for_nonexistent_user() -> Result<()> {
    let (_tdb, state, _) = setup_test_users!(0).await; // No initial users

    let signin_payload = SigninUser {
      email: "nosuchuser@test.com".to_string(),
      password: "password123".to_string(),
    };

    let result = signin(&state.pool, &state.token_manager, &signin_payload).await?;

    assert!(result.is_none());

    Ok(())
  }

  // Test for refresh token expiration (conceptual - requires time manipulation or long waits)
  // #[tokio::test]
  // #[ignore] // Ignore because it might require time manipulation or be slow
  // async fn refresh_token_service_should_fail_if_token_expired() -> Result<()> {
  //   // 1. Setup user and generate initial tokens
  //   // 2. Manually find the refresh token record in the DB
  //   // 3. Update its `expires_at` to a time in the past
  //   // 4. Wait briefly or manipulate time if possible
  //   // 5. Call refresh_token with the expired token string
  //   // 6. Assert that the result is Err(AppError::InvalidInput)
  //   Ok(())
  // }

  // Test for user not found during refresh (conceptual - requires manual DB manipulation)
  // #[tokio::test]
  // #[ignore] // Ignore because it requires manual DB state manipulation mid-test
  // async fn refresh_token_service_should_fail_if_user_deleted() -> Result<()> {
  //   // 1. Setup user and generate initial tokens
  //   // 2. Get the user ID and refresh token string
  //   // 3. Manually delete the user from the DB (leave the token record)
  //   // 4. Call refresh_token with the valid token string
  //   // 5. Assert that the result is Err(AppError::AnyError) with the correct message
  //   Ok(())
  // }
}
