use async_trait::async_trait;
use chrono::Utc;
use sqlx::Acquire;
use tracing::{error, info, warn};

use crate::{
  AppError,
  models::{CreateUser, SigninUser, User},
  services::{
    AuthServiceTrait,
    service_provider::{ServiceFactory, ServiceProvider},
  },
  utils::jwt::{AuthTokens, RefreshToken, RefreshTokenData, generate_refresh_token},
};

pub struct AuthServiceMarker;

pub struct AuthService<'a> {
  provider: &'a ServiceProvider,
}

impl ServiceFactory for AuthServiceMarker {
  type Service = AuthService<'static>;

  fn create(provider: &ServiceProvider) -> Self::Service {
    let provider_static = unsafe { std::mem::transmute(provider) };
    AuthService {
      provider: provider_static,
    }
  }
}

impl<'a> AuthService<'a> {
  pub fn new(provider: &'a ServiceProvider) -> Self {
    Self { provider }
  }
}

#[async_trait]
impl<'a> AuthServiceTrait for AuthService<'a> {
  async fn signup(
    &self,
    payload: &CreateUser,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> Result<AuthTokens, AppError> {
    let user = self.create_user(payload).await?;

    let tokens = self
      .generate_auth_tokens(&user, user_agent, ip_address)
      .await?;

    info!("User signed up successfully: {}", user.email);
    Ok(tokens)
  }

  async fn signin(
    &self,
    payload: &SigninUser,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> Result<Option<AuthTokens>, AppError> {
    match self.authenticate(payload).await? {
      Some(user) => {
        let tokens = self
          .generate_auth_tokens(&user, user_agent.clone(), ip_address.clone())
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

  async fn refresh_token(
    &self,
    refresh_token_str: &str,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> Result<AuthTokens, AppError> {
    let token_record = self.validate_refresh_token(refresh_token_str).await?;

    if let Some(saved) = &token_record.user_agent {
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

    let user = self.find_user_by_token(&token_record).await?;

    if !user.is_active() {
      warn!("Attempt to refresh token for disabled user: {}", user.id);
      return Err(AppError::InvalidInput(
        "User account is disabled".to_string(),
      ));
    }

    let (access_token, refresh_token_data) = self
      .rotate_tokens(&token_record, &user, user_agent, ip_address)
      .await?;

    info!("Token refreshed successfully for user: {}", user.email);
    Ok(AuthTokens {
      access_token,
      refresh_token: refresh_token_data,
    })
  }

  async fn logout(&self, refresh_token_str: &str) -> Result<(), AppError> {
    if let Some(token_record) =
      RefreshToken::find_by_token(refresh_token_str, self.provider.pool()).await?
    {
      token_record.revoke(self.provider.pool()).await?;
      info!("Refresh token revoked for user: {}", token_record.user_id);
    } else {
      info!("Logout attempt with non-existent or expired refresh token.");
    }
    Ok(())
  }

  async fn logout_all(&self, user_id: i64) -> Result<(), AppError> {
    RefreshToken::revoke_all_for_user(user_id, self.provider.pool()).await?;
    info!("All sessions revoked for user: {}", user_id);
    Ok(())
  }

  async fn find_user_by_token(&self, token: &RefreshToken) -> Result<User, AppError> {
    User::find_by_id(token.user_id, self.provider.pool())
      .await?
      .ok_or_else(|| {
        error!(
          "User not found for valid refresh token: user_id {}",
          token.user_id
        );
        AppError::NotFound(vec!["User linked to refresh token not found".to_string()])
      })
  }

  async fn validate_refresh_token(&self, token_str: &str) -> Result<RefreshToken, AppError> {
    let token_record = RefreshToken::find_by_token(token_str, self.provider.pool())
      .await?
      .ok_or_else(|| {
        info!("Refresh token not found or expired");
        AppError::InvalidInput("Invalid or expired refresh token".to_string())
      })?;

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

  async fn generate_auth_tokens(
    &self,
    user: &User,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> Result<AuthTokens, AppError> {
    self
      .provider
      .token_manager()
      .generate_auth_tokens(user, user_agent, ip_address, self.provider.pool())
      .await
  }

  async fn rotate_tokens(
    &self,
    old_token: &RefreshToken,
    user: &User,
    current_user_agent: Option<String>,
    current_ip_address: Option<String>,
  ) -> Result<(String, RefreshTokenData), AppError> {
    let new_refresh_token_str = generate_refresh_token();

    let user_agent = current_user_agent.or_else(|| old_token.user_agent.clone());
    let ip_address = current_ip_address.or_else(|| old_token.ip_address.clone());

    let new_token_db_record = old_token
      .replace(
        &new_refresh_token_str,
        user_agent,
        ip_address,
        self.provider.pool(),
      )
      .await?;

    let access_token = self.provider.token_manager().generate_token(user)?;

    let refresh_token_data = RefreshTokenData {
      token: new_refresh_token_str,
      expires_at: new_token_db_record.expires_at,
      absolute_expires_at: new_token_db_record.absolute_expires_at,
    };

    Ok((access_token, refresh_token_data))
  }


  async fn create_user(&self, payload: &CreateUser) -> Result<User, AppError> {
    let mut tx = self.provider.pool().begin().await?;

    let conn = tx.acquire().await?;

    let mut is_new_workspace = false;
    let workspace =
      match crate::models::Workspace::find_by_name(&payload.workspace, &mut *conn).await? {
        Some(workspace) => {
          if workspace.owner_id == 0 {
            is_new_workspace = true;
          }
          workspace
        }
        None => {
          is_new_workspace = true;
          sqlx::query_as::<_, crate::models::Workspace>(
            r#"
          INSERT INTO workspaces (name, owner_id)
          VALUES ($1, 0)
          RETURNING id, name, owner_id, created_at
          "#,
          )
          .bind(&payload.workspace)
          .fetch_one(&mut *conn)
          .await?
        }
      };

    let password_to_hash = payload.password.clone();
    let password_hash = tokio::task::spawn_blocking(move || {
      use argon2::{
        Argon2,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
      };

      let salt = SaltString::generate(&mut OsRng);
      let argon2 = Argon2::default();

      argon2
        .hash_password(password_to_hash.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| anyhow::anyhow!("Password hashing error: {}", e))
    })
    .await
    .map_err(|e| AppError::AnyError(anyhow::anyhow!("Task join error: {}", e)))?
    .map_err(|e| AppError::AnyError(e))?;

    let user = sqlx::query_as::<_, User>(
      r#"
      INSERT INTO users (workspace_id, email, fullname, password_hash) 
      VALUES ($1, $2, $3, $4) 
      RETURNING id, fullname, email, status, created_at, workspace_id, password_hash
      "#,
    )
    .bind(workspace.id)
    .bind(&payload.email)
    .bind(&payload.fullname)
    .bind(password_hash)
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| {
      if let Some(db_err) = e.as_database_error() {
        if db_err.is_unique_violation() {
          AppError::UserAlreadyExists(payload.email.clone())
        } else {
          e.into()
        }
      } else {
        e.into()
      }
    })?;

    if is_new_workspace {
      let res = sqlx::query("UPDATE workspaces SET owner_id = $1 WHERE id = $2")
        .bind(user.id)
        .bind(workspace.id)
        .execute(&mut *conn)
        .await?;

      if res.rows_affected() == 0 {
        return Err(AppError::NotFound(vec![payload.workspace.clone()]));
      }
    }

    tx.commit().await?;

    Ok(user)
  }

  async fn authenticate(&self, payload: &SigninUser) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
      "SELECT id, fullname, email, password_hash, status, created_at, workspace_id FROM users WHERE email = $1",
    )
    .bind(&payload.email)
    .fetch_optional(self.provider.pool())
    .await?;

    match user {
      Some(mut user) => {
        let password_hash = match std::mem::take(&mut user.password_hash) {
          Some(h) => h,
          None => return Ok(None), // User has no password hash, so it's not authenticated
        };

        let password_to_check = payload.password.clone();

        let is_valid = tokio::task::spawn_blocking(move || {
          use argon2::{Argon2, PasswordHash, PasswordVerifier};

          let argon2 = Argon2::default();
          let parsed_hash = match PasswordHash::new(&password_hash) {
            Ok(hash) => hash,
            Err(_) => return false,
          };

          argon2
            .verify_password(password_to_check.as_bytes(), &parsed_hash)
            .is_ok()
        })
        .await
        .map_err(|e| AppError::AnyError(anyhow::anyhow!("Task join error: {}", e)))?;

        if is_valid {
          Ok(Some(user))
        } else {
          Ok(None) // Password is invalid, so it's not authenticated
        }
      }
      None => Ok(None), // User not found, so it's not authenticated
    }
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
    let auth_service = AuthService::new(&state.service_provider);

    let initial_tokens = auth_service.generate_auth_tokens(user, None, None).await?;

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
    let auth_service = AuthService::new(&state.service_provider);

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
    let auth_service = AuthService::new(&state.service_provider);

    let tokens = auth_service.generate_auth_tokens(user, None, None).await?;

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
    let auth_service = AuthService::new(&state.service_provider);

    let initial_tokens = auth_service.generate_auth_tokens(user, None, None).await?;

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
    let auth_service = AuthService::new(&state.service_provider);

    let create_user_payload = CreateUser {
      workspace: "Acme".to_string(), // Use workspace (String) instead of workspace_id
      fullname: "New User".to_string(),
      email: "newuser@test.com".to_string(),
      password: "password123".to_string(),
    };

    let result = auth_service.signup(&create_user_payload, None, None).await;

    if let Err(ref e) = result {
      println!("Signup error: {:?}", e);
    }

    assert!(result.is_ok());
    let tokens = result.unwrap();
    assert!(!tokens.access_token.is_empty());
    assert!(!tokens.refresh_token.token.is_empty());

    Ok(())
  }

  #[tokio::test]
  async fn signin_service_should_return_tokens_for_valid_credentials() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await; // Setup with one user
    let user = &users[0];
    let auth_service = AuthService::new(&state.service_provider);

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
    let auth_service = AuthService::new(&state.service_provider);

    let original_agent = Some("Chrome Browser".to_string());

    let tokens = auth_service
      .generate_auth_tokens(user, original_agent.clone(), None)
      .await?;

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
    let auth_service = AuthService::new(&state.service_provider);

    let original_agent = Some("Chrome Browser".to_string());

    let tokens = auth_service
      .generate_auth_tokens(user, original_agent, None)
      .await?;

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
    let auth_service = AuthService::new(&state.service_provider);

    let token1 = auth_service.generate_auth_tokens(user, None, None).await?;
    let token2 = auth_service.generate_auth_tokens(user, None, None).await?;

    auth_service.logout_all(user.id).await?;

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
