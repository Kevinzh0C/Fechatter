// fechatter_server/src/services/auth_service.rs
use anyhow::{Result, anyhow};
use chrono::Utc;
use sqlx::PgPool;
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
