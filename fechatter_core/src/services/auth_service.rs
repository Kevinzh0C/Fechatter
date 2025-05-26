use crate::{
  error::CoreError,
  error::TokenValidationError,
  jwt::{AuthTokens, RefreshTokenData, RefreshTokenRepository, ReplaceTokenPayload, UserClaims},
  models::{CreateUser, SigninUser, UserId, user::UserRepository},
  services::AuthContext,
};
use chrono::Utc;

use uuid::Uuid;

// Function to generate a refresh token
fn generate_refresh_token() -> String {
  Uuid::new_v4().to_string()
}

// Define interfaces for the dependencies
pub trait TokenService: Send + Sync {
  fn generate_token(&self, user_claims: &UserClaims) -> Result<String, CoreError>;
  fn verify_token(&self, token: &str) -> Result<UserClaims, CoreError>;
  async fn generate_auth_tokens(
    &self,
    user_claims: &UserClaims,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> Result<AuthTokens, CoreError>;
}

// Core service that orchestrates the business logic without DB operations
pub struct CoreAuthService<U, T, R> {
  user_repository: U,
  token_service: T,
  refresh_token_repository: R,
}

impl<U, T, R> CoreAuthService<U, T, R>
where
  U: UserRepository + Sync,
  T: TokenService + Sync,
  R: RefreshTokenRepository + Sync,
{
  pub fn new(user_repository: U, token_service: T, refresh_token_repository: R) -> Self {
    Self {
      user_repository,
      token_service,
      refresh_token_repository,
    }
  }

  pub async fn signup(
    &self,
    payload: &CreateUser,
    auth_context: Option<AuthContext>,
  ) -> Result<AuthTokens, CoreError> {
    let user = self.user_repository.create(payload).await?;
    let user_claims = UserClaims {
      id: user.id,
      workspace_id: user.workspace_id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
    };
    let auth_context_data = auth_context.clone().unwrap_or_default();
    let auth_tokens = self
      .token_service
      .generate_auth_tokens(
        &user_claims,
        auth_context_data.user_agent,
        auth_context_data.ip_address,
      )
      .await?;
    Ok(auth_tokens)
  }

  pub async fn signin(
    &self,
    payload: &SigninUser,
    auth_context: Option<AuthContext>,
  ) -> Result<Option<AuthTokens>, CoreError> {
    match self.user_repository.authenticate(payload).await? {
      Some(user) => {
        let user_claims = UserClaims {
          id: user.id,
          workspace_id: user.workspace_id,
          fullname: user.fullname.clone(),
          email: user.email.clone(),
          status: user.status,
          created_at: user.created_at,
        };
        let auth_context_data = auth_context.clone().unwrap_or_default();
        let auth_tokens = self
          .token_service
          .generate_auth_tokens(
            &user_claims,
            auth_context_data.user_agent,
            auth_context_data.ip_address,
          )
          .await?;
        Ok(Some(auth_tokens))
      }
      None => Ok(None),
    }
  }

  pub async fn refresh_token(
    &self,
    refresh_token_str: &str,
    auth_context: Option<AuthContext>,
  ) -> Result<AuthTokens, CoreError> {
    let refresh_token = self
      .refresh_token_repository
      .find_by_token(refresh_token_str)
      .await?
      .ok_or_else(|| CoreError::InvalidToken(TokenValidationError::NotFound))?;

    if refresh_token.expires_at < Utc::now() {
      return Err(CoreError::InvalidToken(TokenValidationError::Expired));
    }

    if refresh_token.revoked {
      return Err(CoreError::InvalidToken(TokenValidationError::Revoked));
    }

    if refresh_token.absolute_expires_at < Utc::now() {
      return Err(CoreError::InvalidToken(TokenValidationError::Expired));
    }

    if let Some(ctx) = &auth_context {
      if let Some(token_user_agent) = &refresh_token.user_agent {
        if let Some(request_user_agent) = &ctx.user_agent {
          if token_user_agent != request_user_agent {
            return Err(CoreError::InvalidToken(
              TokenValidationError::SecurityMismatch,
            ));
          }
        }
      }
    }

    let user = self
      .user_repository
      .find_by_id(refresh_token.user_id)
      .await?
      .ok_or_else(|| CoreError::NotFound("User not found".to_string()))?;

    let user_claims = UserClaims {
      id: user.id,
      workspace_id: user.workspace_id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
    };

    let new_access_token = self.token_service.generate_token(&user_claims)?;
    let new_raw_refresh_token = generate_refresh_token();
    let now = Utc::now();
    let new_expires_at =
      now + chrono::Duration::seconds(crate::models::jwt::REFRESH_TOKEN_EXPIRATION as i64);
    let new_absolute_expires_at = refresh_token.absolute_expires_at;

    let auth_context_data = auth_context.unwrap_or_default();
    let replace_payload = ReplaceTokenPayload {
      old_token_id: refresh_token.id,
      new_raw_token: new_raw_refresh_token.clone(),
      new_expires_at,
      new_absolute_expires_at,
      user_agent: auth_context_data.user_agent,
      ip_address: auth_context_data.ip_address,
    };

    let new_refresh_token_record = self
      .refresh_token_repository
      .replace(replace_payload)
      .await?;

    let refresh_token_data = RefreshTokenData {
      token: new_raw_refresh_token,
      expires_at: new_refresh_token_record.expires_at,
      absolute_expires_at: new_refresh_token_record.absolute_expires_at,
    };

    Ok(AuthTokens {
      access_token: new_access_token,
      refresh_token: refresh_token_data,
    })
  }

  pub async fn logout(&self, refresh_token_str: &str) -> Result<(), CoreError> {
    let refresh_token = self
      .refresh_token_repository
      .find_by_token(refresh_token_str)
      .await?
      .ok_or_else(|| CoreError::InvalidToken(TokenValidationError::NotFound))?;

    self
      .refresh_token_repository
      .revoke(refresh_token.id)
      .await?;

    Ok(())
  }

  pub async fn logout_all(&self, user_id: UserId) -> Result<(), CoreError> {
    self
      .refresh_token_repository
      .revoke_all_for_user(user_id)
      .await?;
    Ok(())
  }
}
