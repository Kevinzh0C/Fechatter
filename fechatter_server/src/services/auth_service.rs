use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use tracing::{error, info, warn};

use crate::{
  AppError,
  models::{CreateUser, SigninUser, User},
  utils::jwt::{AuthTokens, RefreshToken, RefreshTokenData, TokenManager, generate_refresh_token},
};

pub struct AuthService<'a> {
  pool: &'a PgPool,
  token_manager: &'a TokenManager,
}

impl<'a> AuthService<'a> {
  pub fn new(pool: &'a PgPool, token_manager: &'a TokenManager) -> Self {
    Self {
      pool,
      token_manager,
    }
  }

  pub async fn signup(
    &self,
    payload: &CreateUser,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> Result<AuthTokens, AppError> {
    let user = User::create(payload, self.pool).await?;

    let tokens = self
      .token_manager
      .generate_auth_tokens(&user, user_agent, ip_address, self.pool)
      .await?;

    info!("User signed up successfully: {}", user.email);
    Ok(tokens)
  }

  /// Handle user login, validate user, generate tokens
  pub async fn signin(
    &self,
    payload: &SigninUser,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> Result<Option<AuthTokens>, AppError> {
    match User::authenticate(payload, self.pool).await? {
      Some(user) => {
        let tokens = self
          .token_manager
          .generate_auth_tokens(&user, user_agent.clone(), ip_address.clone(), self.pool)
          .await?;

        info!(
          "User signed in successfully: {} from {} with agent {}",
          user.email,
          ip_address.as_deref().unwrap_or("unknown IP"),
          user_agent.as_deref().unwrap_or("unknown agent")
        );
        Ok(Some(tokens))
      }
      None => {
        info!("Sign in failed for email: {}", payload.email);
        Ok(None)
      }
    }
  }

  /// Handle token refresh, validate old token, generate new token
  pub async fn refresh_token(
    &self,
    refresh_token_str: &str,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> Result<AuthTokens, AppError> {
    // Find and validate refresh token
    let token_record = self.validate_refresh_token(refresh_token_str).await?;

    if let Some(saved) = &token_record.user_agent {
      // If token has saved user agent, require user agent in request
      match &user_agent {
        None => {
          warn!(
            "Security check failed: token has user agent but current request has none for user_id: {}",
            token_record.user_id
          );
          return Err(AppError::InvalidInput(
            "Security validation failed for token".to_string(),
          ));
        }
        Some(current) if current != saved => {
          warn!(
            "Possible token theft attempt: token user_agent doesn't match current request for user_id: {}",
            token_record.user_id
          );
          return Err(AppError::InvalidInput(
            "Security validation failed for token".to_string(),
          ));
        }
        _ => {} // User agent matches, continue
      }
    }

    // Find associated user
    let user = self.find_user_by_token(&token_record).await?;

    // Generate new tokens and replace old token
    let (access_token, refresh_token_data) = self
      .rotate_tokens(&token_record, &user, user_agent, ip_address)
      .await?;

    info!("Token refreshed successfully for user: {}", user.email);
    Ok(AuthTokens {
      access_token,
      refresh_token: refresh_token_data,
    })
  }

  /// Handle user logout, revoke refresh token
  pub async fn logout(&self, refresh_token_str: &str) -> Result<(), AppError> {
    if let Some(token_record) = RefreshToken::find_by_token(refresh_token_str, self.pool).await? {
      token_record.revoke(self.pool).await?;
      info!("Refresh token revoked for user: {}", token_record.user_id);
    } else {
      info!("Logout attempt with non-existent or expired refresh token.");
    }
    Ok(())
  }

  /// logout all sessions by user_id
  pub async fn logout_all(&self, user_id: i64) -> Result<(), AppError> {
    RefreshToken::revoke_all_for_user(user_id, self.pool).await?;
    info!("All sessions revoked for user: {}", user_id);
    Ok(())
  }

  async fn validate_refresh_token(&self, token_str: &str) -> Result<RefreshToken, AppError> {
    let token_record = RefreshToken::find_by_token(token_str, self.pool)
      .await?
      .ok_or_else(|| {
        info!("Refresh token not found or expired");
        AppError::InvalidInput("Invalid or expired refresh token".to_string())
      })?;

    // Check if token has expired
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

    // Check if token has been revoked
    if token_record.revoked {
      warn!(
        "Attempt to use revoked refresh token for user: {}",
        token_record.user_id
      );
      return Err(AppError::InvalidInput(
        "Invalid or revoked refresh token".to_string(),
      ));
    }

    Ok(token_record)
  }

  /// Find user by token
  async fn find_user_by_token(&self, token: &RefreshToken) -> Result<User, AppError> {
    User::find_by_id(token.user_id, self.pool)
      .await?
      .ok_or_else(|| {
        error!(
          "User not found for valid refresh token: user_id {}",
          token.user_id
        );
        AppError::NotFound(vec!["User linked to refresh token not found".to_string()])
      })
  }

  /// Rotate tokens
  async fn rotate_tokens(
    &self,
    old_token: &RefreshToken,
    user: &User,
    current_user_agent: Option<String>,
    current_ip_address: Option<String>,
  ) -> Result<(String, RefreshTokenData), AppError> {
    // Generate new refresh and access tokens
    let new_refresh_token_str = generate_refresh_token();

    // Replace old token - Keep original user agent and IP if not provided in current request
    let user_agent = current_user_agent.or_else(|| old_token.user_agent.clone());
    let ip_address = current_ip_address.or_else(|| old_token.ip_address.clone());

    // Replace old token
    let new_token_db_record = old_token
      .replace(&new_refresh_token_str, user_agent, ip_address, self.pool)
      .await?;

    // Generate new access token
    let access_token = self.token_manager.generate_token(user)?;

    // Create refresh token data
    let refresh_token_data = RefreshTokenData {
      token: new_refresh_token_str,
      expires_at: new_token_db_record.expires_at,
      absolute_expires_at: new_token_db_record.absolute_expires_at,
    };

    Ok((access_token, refresh_token_data))
  }

  /// Generate authentication tokens for a user
  #[allow(unused)]
  pub async fn generate_auth_tokens(
    &self,
    user: &User,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> Result<AuthTokens, AppError> {
    self
      .token_manager
      .generate_auth_tokens(user, user_agent, ip_address, self.pool)
      .await
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use super::*;
  use crate::setup_test_users;
  use anyhow::Result;
  use tokio::time::sleep;

  #[tokio::test]
  async fn refresh_token_service_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];
    let auth_service = AuthService::new(&state.pool, &state.token_manager);

    let initial_tokens = auth_service
      .token_manager
      .generate_auth_tokens(user, None, None, &state.pool)
      .await?;

    sleep(Duration::from_secs(1)).await;

    let refresh_result = auth_service
      .refresh_token(&initial_tokens.refresh_token.token, None, None)
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
    let auth_service = AuthService::new(&state.pool, &state.token_manager);

    let result = auth_service
      .refresh_token("invalid_token", None, None)
      .await;

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
    let auth_service = AuthService::new(&state.pool, &state.token_manager);

    let tokens = auth_service
      .token_manager
      .generate_auth_tokens(user, None, None, &state.pool)
      .await?;

    auth_service.logout(&tokens.refresh_token.token).await?;

    let refresh_result = auth_service
      .refresh_token(&tokens.refresh_token.token, None, None)
      .await;

    assert!(refresh_result.is_err());

    Ok(())
  }

  #[tokio::test]
  async fn refresh_token_can_only_be_used_once() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];
    let auth_service = AuthService::new(&state.pool, &state.token_manager);

    let initial_tokens = auth_service
      .token_manager
      .generate_auth_tokens(user, None, None, &state.pool)
      .await?;

    let new_tokens = auth_service
      .refresh_token(&initial_tokens.refresh_token.token, None, None)
      .await?;

    let result = auth_service
      .refresh_token(&initial_tokens.refresh_token.token, None, None)
      .await;

    assert!(result.is_err());

    let another_refresh = auth_service
      .refresh_token(&new_tokens.refresh_token.token, None, None)
      .await;

    assert!(another_refresh.is_ok());

    Ok(())
  }

  #[tokio::test]
  async fn signup_service_should_create_user_and_tokens() -> Result<()> {
    let (_tdb, state, _) = setup_test_users!(0).await; // Setup DB, no initial users needed
    let auth_service = AuthService::new(&state.pool, &state.token_manager);

    let create_user_payload = CreateUser {
      workspace: "Acme".to_string(), // Use workspace (String) instead of workspace_id
      fullname: "New User".to_string(),
      email: "newuser@test.com".to_string(),
      password: "password123".to_string(),
    };

    let result = auth_service.signup(&create_user_payload, None, None).await;

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
    let auth_service = AuthService::new(&state.pool, &state.token_manager);

    let signin_payload = SigninUser {
      email: user.email.clone(),
      password: "password".to_string(), // Default password from setup_test_users
    };

    let result = auth_service.signin(&signin_payload, None, None).await?;

    assert!(result.is_some());
    let tokens = result.unwrap();
    assert!(!tokens.access_token.is_empty());
    assert!(!tokens.refresh_token.token.is_empty());

    Ok(())
  }

  #[tokio::test]
  async fn test_user_agent_consistency_check() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];
    let auth_service = AuthService::new(&state.pool, &state.token_manager);

    // Create token with specific user agent
    let original_agent = Some("Chrome Browser".to_string());

    let tokens = auth_service
      .token_manager
      .generate_auth_tokens(user, original_agent.clone(), None, &state.pool)
      .await?;

    // Try to refresh with different user agent should fail
    let different_agent = Some("Firefox Browser".to_string());
    let refresh_result = auth_service
      .refresh_token(&tokens.refresh_token.token, different_agent, None)
      .await;

    assert!(refresh_result.is_err());
    if let Err(AppError::InvalidInput(msg)) = refresh_result {
      assert!(msg.contains("Security validation failed"));
    } else {
      panic!("Expected security validation error");
    }

    // Try with same agent should work
    let refresh_success = auth_service
      .refresh_token(&tokens.refresh_token.token, original_agent, None)
      .await;

    assert!(refresh_success.is_ok());

    Ok(())
  }

  #[tokio::test]
  async fn test_missing_user_agent_fails_when_token_has_one() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];
    let auth_service = AuthService::new(&state.pool, &state.token_manager);

    // Create token with specific user agent
    let original_agent = Some("Chrome Browser".to_string());

    let tokens = auth_service
      .token_manager
      .generate_auth_tokens(user, original_agent, None, &state.pool)
      .await?;

    // Try to refresh without providing a user agent should fail
    let refresh_result = auth_service
      .refresh_token(&tokens.refresh_token.token, None, None)
      .await;

    assert!(refresh_result.is_err());
    if let Err(AppError::InvalidInput(msg)) = refresh_result {
      assert!(msg.contains("Security validation failed"));
    } else {
      panic!("Expected security validation error");
    }

    Ok(())
  }

  #[tokio::test]
  async fn logout_all_sessions_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];
    let auth_service = AuthService::new(&state.pool, &state.token_manager);

    // Create multiple tokens
    let token1 = auth_service.generate_auth_tokens(user, None, None).await?;
    let token2 = auth_service.generate_auth_tokens(user, None, None).await?;

    // Logout all sessions
    auth_service.logout_all(user.id).await?;

    // Both tokens should be invalid
    let refresh1 = auth_service
      .refresh_token(&token1.refresh_token.token, None, None)
      .await;
    let refresh2 = auth_service
      .refresh_token(&token2.refresh_token.token, None, None)
      .await;

    assert!(refresh1.is_err());
    assert!(refresh2.is_err());

    Ok(())
  }
}
