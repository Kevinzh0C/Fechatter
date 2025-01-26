use crate::utils::refresh_token::auth_context_matches;
use async_trait::async_trait;
use chrono::Utc;
use fechatter_core::{
  AuthContext, AuthServiceTrait, AuthTokens, CoreError, CreateUser, LogoutService,
  RefreshTokenData, RefreshTokenRepository, RefreshTokenService, ReplaceTokenPayload,
  SigninService, SigninUser, SignupService, TokenService, UserClaims, UserRepository, UserStatus,
  error::TokenValidationError,
};
use std::sync::Arc;
use tracing;

#[derive(Clone)]
pub struct AuthService {
  pub(crate) user_repository: Arc<Box<dyn UserRepository + Send + Sync + 'static>>,
  pub(crate) token_service: Arc<Box<dyn TokenService + Send + Sync + 'static>>,
  pub(crate) refresh_token_repository: Arc<Box<dyn RefreshTokenRepository + Send + Sync + 'static>>,
}

#[async_trait]
impl RefreshTokenService for AuthService {
  async fn refresh_token(
    &self,
    refresh_token: &str,
    request_context: Option<AuthContext>,
  ) -> Result<AuthTokens, CoreError> {
    tracing::debug!(target: "auth_service_refresh", ?refresh_token, ?request_context, "Attempting token refresh");
    println!(
      "!! Debug refresh_token method START - token: {}, context: {:?}",
      refresh_token, request_context
    );

    // Find the token in the database (once)
    let find_result = self
      .refresh_token_repository
      .find_by_token(refresh_token)
      .await;
    println!("!! Debug find_token result: {:?}", find_result);

    // Extract stored_token from find_result instead of making a second database call
    let stored_token =
      find_result?.ok_or(CoreError::InvalidToken(TokenValidationError::NotFound))?;

    tracing::debug!(target: "auth_service_refresh", ?stored_token, "Found stored token");
    println!("!! Debug - found token in DB: {:?}", stored_token);

    // Only validate user agent if both token has one AND auth_context is provided
    if let Some(saved_agent) = &stored_token.user_agent {
      if let Some(ctx) = &request_context {
        if let Some(current) = &ctx.user_agent {
          // User agent present in both token and request, verify they match
          if current != saved_agent {
            tracing::warn!(
              "Token refresh failed: User agent mismatch. Expected: {}, Got: {}",
              saved_agent,
              current
            );
            println!(
              "!! Debug - User agent mismatch: token has '{}', request has '{}'",
              saved_agent, current
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
      .find_by_id(stored_token.user_id)
      .await?
    {
      Some(user) => user,
      None => {
        tracing::warn!(
          "Token refresh failed: User not found for token. User ID: {}",
          stored_token.user_id
        );
        println!("!! Debug - User not found: {}", stored_token.user_id);
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
      println!(
        "!! Debug - User account is disabled: id={}, status={:?}",
        user.id, user.status
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
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
    };

    // Generate new refresh token
    let new_token = uuid::Uuid::new_v4().to_string();
    println!("!! Debug - Generated new refresh token: {}", new_token);

    // Get auth context details
    let (req_user_agent, req_ip_address) = request_context
      .map(|ctx| (ctx.user_agent, ctx.ip_address))
      .unwrap_or((None, None));

    tracing::debug!(target: "auth_service_refresh", ua = ?req_user_agent, ip = ?req_ip_address, "Extracted request context for matching");

    if !auth_context_matches(
      stored_token.user_agent.as_deref(),
      stored_token.ip_address.as_deref(),
      req_user_agent.as_deref(),
      req_ip_address.as_deref(),
    ) {
      tracing::warn!(target: "auth_service_refresh", token_id = stored_token.id, "Auth context mismatch for refresh token");
      println!(
        "!! Debug - Auth context mismatch for token ID: {}",
        stored_token.id
      );
      return Err(CoreError::Unauthorized(
        "Refresh token context does not match request context".to_string(),
      ));
    }

    tracing::debug!(target: "auth_service_refresh", token_id = stored_token.id, "Auth context matched successfully");
    println!(
      "!! Debug - Auth context matched successfully for token ID: {}",
      stored_token.id
    );

    // Replace the old token with the new one
    let now = Utc::now();
    let expires_at = now + chrono::Duration::days(14); // 14 days
    let absolute_expires_at = now + chrono::Duration::days(30); // 30 days

    let replace_payload = ReplaceTokenPayload {
      old_token_id: stored_token.id,
      new_raw_token: new_token.clone(),
      new_expires_at: expires_at,
      new_absolute_expires_at: absolute_expires_at,
      user_agent: req_user_agent.clone(),
      ip_address: req_ip_address.clone(),
    };

    println!(
      "!! Debug - Calling replace on refresh token repository with token ID: {}",
      stored_token.id
    );
    let new_token_record = match self.refresh_token_repository.replace(replace_payload).await {
      Ok(record) => {
        println!(
          "!! Debug - Replace successful, new token record: {:?}",
          record
        );
        record
      }
      Err(e) => {
        tracing::warn!("Token refresh failed: Error replacing token: {:?}", e);
        println!("!! Debug - Replace failed with error: {:?}", e);
        return Err(CoreError::InvalidToken(
          fechatter_core::error::TokenValidationError::SecurityMismatch,
        ));
      }
    };

    // Generate access token
    let access_token = match self.token_service.generate_token(&user_claims) {
      Ok(token) => token,
      Err(e) => {
        tracing::warn!("Failed to generate access token: {:?}", e);
        println!("!! Debug - Failed to generate access token: {:?}", e);
        return Err(e);
      }
    };

    // Create AuthTokens with the new tokens
    let auth_tokens = AuthTokens {
      access_token,
      refresh_token: RefreshTokenData {
        token: new_token,
        expires_at: new_token_record.expires_at,
        absolute_expires_at: new_token_record.absolute_expires_at,
      },
    };

    println!("!! Debug refresh_token method END - success");
    // Return both tokens
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

  async fn logout_all(&self, user_id: fechatter_core::UserId) -> Result<(), CoreError> {
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
}
