use crate::{
  error::CoreError,
  error::TokenValidationError,
  jwt::{
    AuthTokens, RefreshToken, RefreshTokenData, RefreshTokenRepository, ReplaceTokenPayload,
    UserClaims,
  },
  models::{CreateUser, SigninUser, user::UserRepository},
  services::AuthContext,
};
use chrono::Utc;

use uuid::Uuid;

// Define RefreshTokenInfo as an alias for RefreshToken
type RefreshTokenInfo = RefreshToken;

// Function to generate a refresh token
fn generate_refresh_token() -> String {
  Uuid::new_v4().to_string()
}

// Define interfaces for the dependencies
pub trait TokenService: Send + Sync {
  fn generate_token(&self, user_claims: &UserClaims) -> Result<String, CoreError>;
  fn verify_token(&self, token: &str) -> Result<UserClaims, CoreError>;
  fn generate_auth_tokens(
    &self,
    user_claims: &UserClaims,
    auth_context: Option<AuthContext>,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<AuthTokens, CoreError>> + Send>>;
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
      fullname: user.fullname,
      email: user.email,
      status: user.status,
      created_at: user.created_at,
    };
    let tokens = self
      .token_service
      .generate_auth_tokens(&user_claims, auth_context)
      .await?;
    Ok(tokens)
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
          fullname: user.fullname,
          email: user.email,
          status: user.status,
          created_at: user.created_at,
        };
        let tokens = self
          .token_service
          .generate_auth_tokens(&user_claims, auth_context)
          .await?;
        Ok(Some(tokens))
      }
      None => Ok(None),
    }
  }

  pub async fn refresh_token(
    &self,
    refresh_token_str: &str,
    auth_context: Option<AuthContext>,
  ) -> Result<AuthTokens, CoreError> {
    let token_record = self.validate_refresh_token(refresh_token_str).await?;

    // Only validate user agent if the token has one AND auth_context is provided
    // This allows clients without user agent support to still refresh tokens
    if let Some(saved_agent) = &token_record.user_agent {
      if let Some(ctx) = &auth_context {
        // Auth context is provided, so we can validate the user agent
        if let Some(current_agent) = &ctx.user_agent {
          // Check if user agents match
          if current_agent != saved_agent {
            return Err(CoreError::InvalidToken(
              TokenValidationError::SecurityMismatch,
            ));
          }
        }
        // If auth_context has no user_agent, we'll still allow the refresh
        // This is more permissive than before but necessary for some clients
      }
      // If no auth_context provided but token has user_agent,
      // we'll also allow the refresh for better compatibility with different clients
    }

    let user = self
      .user_repository
      .find_by_id(token_record.user_id)
      .await?
      .ok_or_else(|| CoreError::NotFound(format!("User linked to refresh token not found")))?;

    let user_claims = UserClaims {
      id: user.id,
      workspace_id: user.workspace_id,
      fullname: user.fullname,
      email: user.email,
      status: user.status,
      created_at: user.created_at,
    };

    let new_raw_token = generate_refresh_token();

    let user_agent = auth_context.as_ref().and_then(|ctx| ctx.user_agent.clone());
    let ip_address = auth_context.as_ref().and_then(|ctx| ctx.ip_address.clone());

    let now = Utc::now();
    let new_expires_at =
      now + chrono::Duration::seconds(crate::models::jwt::REFRESH_TOKEN_EXPIRATION as i64);
    let new_absolute_expires_at =
      now + chrono::Duration::seconds(crate::models::jwt::REFRESH_TOKEN_MAX_LIFETIME as i64);

    let replace_payload = ReplaceTokenPayload {
      old_token_id: token_record.id,
      new_raw_token: new_raw_token.clone(),
      new_expires_at,
      new_absolute_expires_at,
      user_agent,
      ip_address,
    };

    let new_token_record = self
      .refresh_token_repository
      .replace(replace_payload)
      .await?;

    let access_token = self.token_service.generate_token(&user_claims)?;

    Ok(AuthTokens {
      access_token,
      refresh_token: RefreshTokenData {
        token: new_raw_token,
        expires_at: new_token_record.expires_at,
        absolute_expires_at: new_token_record.absolute_expires_at,
      },
    })
  }

  pub async fn logout(&self, refresh_token_str: &str) -> Result<(), CoreError> {
    if let Some(token_record) = self
      .refresh_token_repository
      .find_by_token(refresh_token_str)
      .await?
    {
      self
        .refresh_token_repository
        .revoke(token_record.id)
        .await?;
    }
    Ok(())
  }

  pub async fn logout_all(&self, user_id: i64) -> Result<(), CoreError> {
    self
      .refresh_token_repository
      .revoke_all_for_user(user_id)
      .await?;
    Ok(())
  }

  async fn validate_refresh_token(&self, token_str: &str) -> Result<RefreshTokenInfo, CoreError> {
    let token_record = self
      .refresh_token_repository
      .find_by_token(token_str)
      .await?
      .ok_or_else(|| CoreError::InvalidToken(TokenValidationError::NotFound))?;

    if token_record.expires_at < chrono::Utc::now() {
      return Err(CoreError::InvalidToken(TokenValidationError::Expired));
    }

    if token_record.revoked {
      return Err(CoreError::InvalidToken(TokenValidationError::Revoked));
    }

    Ok(token_record)
  }
}
