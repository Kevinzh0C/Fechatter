use async_trait::async_trait;
use chrono::Utc;
use fechatter_core::{
  AuthContext, AuthServiceTrait, AuthTokens, CoreError, CreateUser, LogoutService, RefreshToken,
  RefreshTokenData, RefreshTokenRepository, RefreshTokenService, ReplaceTokenPayload,
  SigninService, SigninUser, SignupService, TokenService, UserClaims, UserRepository, UserStatus,
};
use std::sync::Arc;
use tracing;

#[derive(Clone)]
pub struct AuthService {
  pub(crate) user_repository: Arc<Box<dyn UserRepository + Send + Sync + 'static>>,
  pub(crate) token_service: Arc<Box<dyn TokenService + Send + Sync + 'static>>,
  pub(crate) refresh_token_repository: Arc<Box<dyn RefreshTokenRepository + Send + Sync + 'static>>,
}

// Define RefreshTokenInfo as an alias for RefreshToken
type RefreshTokenInfo = RefreshToken;

#[async_trait]
impl RefreshTokenService for AuthService {
  async fn refresh_token(
    &self,
    refresh_token_str: &str,
    auth_context: Option<AuthContext>,
  ) -> Result<AuthTokens, CoreError> {
    // Find and validate refresh token
    let token_record = self.validate_refresh_token(refresh_token_str).await?;

    // Only validate user agent if both token has one AND auth_context is provided
    if let Some(saved_agent) = &token_record.user_agent {
      if let Some(ctx) = &auth_context {
        if let Some(current) = &ctx.user_agent {
          // User agent present in both token and request, verify they match
          if current != saved_agent {
            tracing::warn!(
              "Token refresh failed: User agent mismatch. Expected: {}, Got: {}",
              saved_agent,
              current
            );
            return Err(CoreError::InvalidToken(
              fechatter_core::error::TokenValidationError::SecurityMismatch,
            ));
          }
        }
        // If auth_context exists but has no user_agent, allow the refresh
        // This makes the API more flexible for different client types
      }
      // If no auth_context, also allow refresh for compatibility with different clients
      // This is more permissive but necessary for mobile apps and other clients
    }

    // Find associated user
    let user = match self
      .user_repository
      .find_by_id(token_record.user_id)
      .await?
    {
      Some(user) => user,
      None => {
        tracing::warn!(
          "Token refresh failed: User not found for token. User ID: {}",
          token_record.user_id
        );
        return Err(CoreError::InvalidToken(
          fechatter_core::error::TokenValidationError::NotFound,
        ));
      }
    };

    // 重要：检查用户状态，禁止已禁用用户刷新令牌
    if user.status != UserStatus::Active {
      tracing::warn!(
        "Token refresh failed: User account is disabled. User ID: {}, Status: {:?}",
        user.id,
        user.status
      );
      return Err(CoreError::Unauthorized(format!(
        "User account is disabled. Current status: {:?}",
        user.status
      )));
    }

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
    let new_token = uuid::Uuid::new_v4().to_string();

    // Get auth context details
    let user_agent = auth_context.as_ref().and_then(|ctx| ctx.user_agent.clone());
    let ip_address = auth_context.as_ref().and_then(|ctx| ctx.ip_address.clone());

    // Replace the old token with the new one
    let now = Utc::now();
    let expires_at = now + chrono::Duration::days(14); // 14 days
    let absolute_expires_at = now + chrono::Duration::days(30); // 30 days

    let replace_payload = ReplaceTokenPayload {
      old_token_id: token_record.id,
      new_raw_token: new_token.clone(),
      new_expires_at: expires_at,
      new_absolute_expires_at: absolute_expires_at,
      user_agent,
      ip_address,
    };

    let new_token_record = match self.refresh_token_repository.replace(replace_payload).await {
      Ok(record) => record,
      Err(e) => {
        tracing::warn!("Token refresh failed: Error replacing token: {:?}", e);
        return Err(CoreError::InvalidToken(
          fechatter_core::error::TokenValidationError::SecurityMismatch,
        ));
      }
    };

    // Generate new access token - 使用已有的auth_context而不尝试unwrap_or_default
    let tokens = match self
      .token_service
      .generate_auth_tokens(
        &user_claims,
        auth_context.as_ref().and_then(|ctx| ctx.user_agent.clone()),
        auth_context.as_ref().and_then(|ctx| ctx.ip_address.clone()),
      )
      .await
    {
      Ok(tokens) => tokens,
      Err(e) => {
        tracing::warn!("Token refresh failed: Error generating new tokens: {:?}", e);
        return Err(CoreError::InvalidToken(
          fechatter_core::error::TokenValidationError::SecurityMismatch,
        ));
      }
    };

    // Create auth tokens response with new refresh token data but keeping tokens.access_token
    let auth_tokens = AuthTokens {
      access_token: tokens.access_token,
      refresh_token: RefreshTokenData {
        token: new_token,
        expires_at: new_token_record.expires_at,
        absolute_expires_at: new_token_record.absolute_expires_at,
      },
    };

    Ok(auth_tokens)
  }
}

#[async_trait]
impl SignupService for AuthService {
  async fn signup(
    &self,
    payload: &CreateUser,
    auth_context: Option<AuthContext>,
  ) -> Result<AuthTokens, CoreError> {
    // Create user
    let user = match self.user_repository.create(payload).await {
      Ok(user) => user,
      Err(e) => {
        tracing::warn!("Signup failed: Error creating user: {:?}", e);
        return Err(e);
      }
    };

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
    match self
      .token_service
      .generate_auth_tokens(
        &user_claims,
        auth_context.as_ref().and_then(|ctx| ctx.user_agent.clone()),
        auth_context.as_ref().and_then(|ctx| ctx.ip_address.clone()),
      )
      .await
    {
      Ok(tokens) => Ok(tokens),
      Err(e) => {
        tracing::warn!("Signup failed: Error generating tokens: {:?}", e);
        Err(e)
      }
    }
  }
}

#[async_trait]
impl SigninService for AuthService {
  async fn signin(
    &self,
    payload: &SigninUser,
    auth_context: Option<AuthContext>,
  ) -> Result<Option<AuthTokens>, CoreError> {
    match self.user_repository.authenticate(payload).await {
      Ok(Some(user)) => {
        let user_claims = UserClaims {
          id: user.id,
          workspace_id: user.workspace_id,
          fullname: user.fullname,
          email: user.email,
          status: user.status,
          created_at: user.created_at,
        };

        match self
          .token_service
          .generate_auth_tokens(
            &user_claims,
            auth_context.as_ref().and_then(|ctx| ctx.user_agent.clone()),
            auth_context.as_ref().and_then(|ctx| ctx.ip_address.clone()),
          )
          .await
        {
          Ok(tokens) => Ok(Some(tokens)),
          Err(e) => {
            tracing::warn!("Signin failed: Error generating tokens: {:?}", e);
            Err(e)
          }
        }
      }
      Ok(None) => {
        tracing::info!(
          "Signin failed: Invalid credentials for email: {}",
          payload.email
        );
        Ok(None)
      }
      Err(e) => {
        tracing::warn!("Signin failed: Error authenticating user: {:?}", e);
        Err(e)
      }
    }
  }
}

#[async_trait]
impl LogoutService for AuthService {
  async fn logout(&self, refresh_token_str: &str) -> Result<(), CoreError> {
    if let Some(token_record) = self
      .refresh_token_repository
      .find_by_token(refresh_token_str)
      .await?
    {
      match self.refresh_token_repository.revoke(token_record.id).await {
        Ok(_) => Ok(()),
        Err(e) => {
          tracing::warn!("Logout failed: Error revoking token: {:?}", e);
          Err(CoreError::Internal(format!(
            "Failed to revoke token: {}",
            e
          )))
        }
      }
    } else {
      tracing::info!("Logout: Token not found or already expired");
      Ok(())
    }
  }

  async fn logout_all(&self, user_id: i64) -> Result<(), CoreError> {
    match self
      .refresh_token_repository
      .revoke_all_for_user(user_id)
      .await
    {
      Ok(_) => Ok(()),
      Err(e) => {
        tracing::warn!("Logout all failed for user {}: {:?}", user_id, e);
        Err(CoreError::Internal(format!(
          "Failed to revoke all tokens for user {}: {}",
          user_id, e
        )))
      }
    }
  }
}

#[async_trait]
impl AuthServiceTrait for AuthService {}

impl AuthService {
  pub fn new(
    user_repository: Box<dyn UserRepository + Send + Sync + 'static>,
    token_service: Box<dyn TokenService + Send + Sync + 'static>,
    refresh_token_repository: Box<dyn RefreshTokenRepository + Send + Sync + 'static>,
  ) -> Self {
    Self {
      user_repository: Arc::new(user_repository),
      token_service: Arc::new(token_service),
      refresh_token_repository: Arc::new(refresh_token_repository),
    }
  }

  // 添加方法从Arc<AuthService>创建AuthService
  pub fn from_arc(arc_service: Arc<Self>) -> Self {
    Self {
      user_repository: Arc::clone(&arc_service.user_repository),
      token_service: Arc::clone(&arc_service.token_service),
      refresh_token_repository: Arc::clone(&arc_service.refresh_token_repository),
    }
  }

  async fn validate_refresh_token(
    &self,
    refresh_token_str: &str,
  ) -> Result<RefreshTokenInfo, CoreError> {
    let token_record = match self
      .refresh_token_repository
      .find_by_token(refresh_token_str)
      .await
    {
      Ok(Some(record)) => record,
      Ok(None) => {
        tracing::warn!("Token validation failed: Token not found");
        return Err(CoreError::InvalidToken(
          fechatter_core::error::TokenValidationError::NotFound,
        ));
      }
      Err(e) => {
        tracing::warn!("Token validation failed: Error finding token: {:?}", e);
        return Err(CoreError::InvalidToken(
          fechatter_core::error::TokenValidationError::NotFound,
        ));
      }
    };

    let now = chrono::Utc::now();

    // 检查令牌是否过期 - 同时检查常规过期时间和绝对过期时间，防止时钟回拨问题
    if token_record.expires_at < now || token_record.absolute_expires_at < now {
      tracing::warn!(
        "Token validation failed: Token expired. Expires at: {}, Absolute expires at: {}, Now: {}",
        token_record.expires_at,
        token_record.absolute_expires_at,
        now
      );
      return Err(CoreError::InvalidToken(
        fechatter_core::error::TokenValidationError::Expired,
      ));
    }

    // Check if token has been revoked
    if token_record.revoked {
      tracing::warn!("Token validation failed: Token has been revoked");
      return Err(CoreError::InvalidToken(
        fechatter_core::error::TokenValidationError::Revoked,
      ));
    }

    Ok(token_record)
  }
}
