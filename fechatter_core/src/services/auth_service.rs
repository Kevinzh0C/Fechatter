use crate::{
  error::CoreError,
  jwt::{AuthTokens, RefreshTokenRepository, UserClaims},
  models::{CreateUser, SigninUser},
  services::AuthContext,
};

// Define interfaces for the dependencies
pub trait TokenService: Send + Sync {
  fn generate_token(&self, user_claims: &UserClaims) -> Result<String, CoreError>;
  fn verify_token(&self, token: &str) -> Result<UserClaims, CoreError>;
  fn generate_auth_tokens(
    &self,
    user_claims: &UserClaims,
    auth_context: Option<AuthContext>,
  ) -> impl std::future::Future<Output = Result<AuthTokens, CoreError>> + Send;
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
    // Create user
    let user = self.user_repository.create(payload).await?;

    // Generate user claims
    let user_claims = UserClaims {
      id: user.id,
      workspace_id: user.workspace_id,
      fullname: user.fullname,
      email: user.email,
      status: user.status,
      created_at: user.created_at,
    };

    // Generate tokens
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
    // Find and validate refresh token
    let token_record = self.validate_refresh_token(refresh_token_str).await?;

    if let Some(saved_agent) = &token_record.user_agent {
      // If token has saved user agent, require user agent in request
      if let Some(ctx) = &auth_context {
        match &ctx.user_agent {
          None => {
            return Err(CoreError::Validation(
              "Security validation failed for token".to_string(),
            ));
          }
          Some(current) if current != saved_agent => {
            return Err(CoreError::Validation(
              "Security validation failed for token".to_string(),
            ));
          }
          _ => {} // User agent matches, continue
        }
      } else {
        return Err(CoreError::Validation(
          "Security validation failed for token".to_string(),
        ));
      }
    }

    // Find associated user
    let user = self
      .user_repository
      .find_by_id(token_record.user_id)
      .await?
      .ok_or_else(|| CoreError::NotFound(format!("User linked to refresh token not found")))?;

    // Create user claims
    let user_claims = UserClaims {
      id: user.id,
      workspace_id: user.workspace_id,
      fullname: user.fullname,
      email: user.email,
      status: user.status,
      created_at: user.created_at,
    };

    // Generate new refresh token
    let new_token = generate_refresh_token();

    // Get auth context details
    let user_agent = auth_context.as_ref().and_then(|ctx| ctx.user_agent.clone());
    let ip_address = auth_context.as_ref().and_then(|ctx| ctx.ip_address.clone());

    // Replace the old token with the new one
    let new_token_record = self
      .refresh_token_repository
      .replace(token_record.id, &new_token, user_agent, ip_address)
      .await?;

    // Generate new access token
    let access_token = self.token_service.generate_token(&user_claims)?;

    // Create auth tokens response
    let auth_tokens = AuthTokens {
      access_token,
      refresh_token: crate::utils::jwt::RefreshTokenData {
        token: new_token,
        expires_at: new_token_record.expires_at,
        absolute_expires_at: new_token_record.absolute_expires_at,
      },
    };

    Ok(auth_tokens)
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
      .ok_or_else(|| CoreError::Validation("Invalid or expired refresh token".to_string()))?;

    // Check if token has expired
    if token_record.expires_at < chrono::Utc::now() {
      return Err(CoreError::Validation(
        "Invalid or expired refresh token".to_string(),
      ));
    }

    // Check if token has been revoked
    if token_record.revoked {
      return Err(CoreError::Validation(
        "Invalid or revoked refresh token".to_string(),
      ));
    }

    Ok(token_record)
  }
}

use crate::models::user::UserRepository;
