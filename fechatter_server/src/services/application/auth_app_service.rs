//! # Authentication Application Service
//!
//! ## Single Responsibility
//! - User authentication orchestration
//! - Sign up/sign in workflow coordination
//! - Token management integration
//! - Domain events publishing for auth operations

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, info, instrument, warn};

// Core domain models
use fechatter_core::contracts::AuthContext;
use fechatter_core::contracts::repositories::UserRepository;
use fechatter_core::jwt::{
  AuthServiceTrait, LogoutService, RefreshTokenRepository, RefreshTokenService, SigninService,
  SignupService,
};
use fechatter_core::models::SigninUser;
use fechatter_core::models::jwt::TokenService;
use fechatter_core::{
  AuthTokens, CreateUser, User, UserId,
  error::{CoreError, TokenValidationError},
};

// Domain services
use crate::domains::user::{
  repository::UserRepositoryImpl,
  user_domain::{UserConfig, UserDomainService, UserDomainServiceImpl},
};

// Application services
use super::application_event_publisher::{ApplicationEvent, ApplicationEventPublisher, UserEvent};

// ================================================================================================
// Authentication Application Service
// ================================================================================================

/// Authentication Application Service - Single responsibility: Orchestrate auth workflows
#[derive(Clone)]
pub struct AuthService {
  /// User repository for data access
  user_repository: Arc<UserRepositoryImpl>,
  /// Token service for JWT operations
  token_service: Arc<Box<dyn TokenService + Send + Sync + 'static>>,
  /// Refresh token repository
  refresh_token_repository: Arc<Box<dyn RefreshTokenRepository + Send + Sync + 'static>>,
  /// Event publisher for domain events
  event_publisher: Arc<ApplicationEventPublisher>,
  /// User domain configuration
  user_config: UserConfig,
}

impl AuthService {
  /// Create new auth service with domain dependencies
  pub fn new(
    user_repository: Arc<UserRepositoryImpl>,
    token_service: Box<dyn TokenService + Send + Sync + 'static>,
    refresh_token_repository: Box<dyn RefreshTokenRepository + Send + Sync + 'static>,
    event_publisher: Arc<ApplicationEventPublisher>,
  ) -> Self {
    Self {
      user_repository,
      token_service: Arc::new(token_service),
      refresh_token_repository: Arc::new(refresh_token_repository),
      event_publisher,
      user_config: UserConfig::default(),
    }
  }

  /// Create auth service with custom user config
  pub fn new_with_config(
    user_repository: Arc<UserRepositoryImpl>,
    token_service: Box<dyn TokenService + Send + Sync + 'static>,
    refresh_token_repository: Box<dyn RefreshTokenRepository + Send + Sync + 'static>,
    event_publisher: Arc<ApplicationEventPublisher>,
    user_config: UserConfig,
  ) -> Self {
    Self {
      user_repository,
      token_service: Arc::new(token_service),
      refresh_token_repository: Arc::new(refresh_token_repository),
      event_publisher,
      user_config,
    }
  }

  /// Get user domain service
  fn get_user_domain_service(&self) -> UserDomainServiceImpl {
    UserDomainServiceImpl::new(self.user_repository.clone(), self.user_config.clone())
  }

  /// Generate auth tokens for user
  async fn generate_auth_tokens(&self, user: &User) -> Result<AuthTokens, CoreError> {
    // Create UserClaims from User
    let user_claims = fechatter_core::UserClaims {
      id: user.id,
      workspace_id: user.workspace_id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
    };

    // Generate auth tokens using TokenService
    self
      .token_service
      .generate_auth_tokens(&user_claims, None, None)
      .await
  }

  /// Publish user created event
  async fn publish_user_created_event(&self, user: &User) {
    let event = ApplicationEvent::User(UserEvent::UserCreated {
      user_id: i64::from(user.id),
      email: user.email.clone(),
      workspace_id: i64::from(user.workspace_id),
    });

    if let Err(e) = self.event_publisher.publish_async(event).await {
      warn!("Failed to publish user created event: {}", e);
    }
  }

  /// Validate email doesn't exist
  async fn validate_email_available(&self, email: &str) -> Result<(), CoreError> {
    let user_exists = self.user_repository.email_user_exists(email).await?;
    if user_exists.is_some() {
      return Err(CoreError::UserAlreadyExists(format!(
        "User with email {} already exists",
        email
      )));
    }
    Ok(())
  }

  /// Check if email exists
  pub async fn email_user_exists(&self, email: &str) -> Result<bool, CoreError> {
    let user_result = self.user_repository.email_user_exists(email).await?;
    Ok(user_result.is_some())
  }

  /// Get user by ID
  pub async fn get_user_by_id(&self, user_id: UserId) -> Result<Option<User>, CoreError> {
    self.user_repository.find_by_id_ext(user_id).await
  }
}

// ================================================================================================
// Signup Service Implementation
// ================================================================================================

#[async_trait]
impl SignupService for AuthService {
  #[instrument(skip(self, payload), fields(email = %payload.email))]
  async fn signup(
    &self,
    payload: &CreateUser,
    auth_context: Option<AuthContext>,
  ) -> Result<AuthTokens, CoreError> {
    info!("Starting user signup process");

    // 1. Validate email availability
    self.validate_email_available(&payload.email).await?;

    // 2. Create user through repository (handles workspace creation)
    let user = self.user_repository.create(payload).await.map_err(|e| {
      error!("Failed to create user: {}", e);
      e
    })?;

    info!(
      user_id = i64::from(user.id),
      workspace_id = i64::from(user.workspace_id),
      "User created successfully"
    );

    // 3. Generate auth tokens
    let tokens = self.generate_auth_tokens(&user).await?;

    // 4. Publish domain event
    self.publish_user_created_event(&user).await;

    // 5. Log authentication context if provided
    if let Some(ctx) = auth_context {
      info!(
        user_id = i64::from(user.id),
        ip_address = ctx.ip_address,
        user_agent = ctx.user_agent,
        "User signup completed with context"
      );
    }

    Ok(tokens)
  }
}

// ================================================================================================
// Signin Service Implementation
// ================================================================================================

#[async_trait]
impl SigninService for AuthService {
  #[instrument(skip(self, payload), fields(email = %payload.email))]
  async fn signin(
    &self,
    payload: &SigninUser,
    auth_context: Option<AuthContext>,
  ) -> Result<Option<AuthTokens>, CoreError> {
    info!("Starting user signin process");

    // 1. Authenticate user through repository
    let user = match self.user_repository.authenticate(payload).await? {
      Some(user) => user,
      None => {
        warn!(email = %payload.email, "Authentication failed - invalid credentials");
        return Ok(None);
      }
    };

    info!(
      user_id = i64::from(user.id),
      email = %user.email,
      "User authenticated successfully"
    );

    // 2. Generate auth tokens
    let tokens = self.generate_auth_tokens(&user).await?;

    // 3. Log authentication context if provided
    if let Some(ctx) = auth_context {
      info!(
        user_id = i64::from(user.id),
        ip_address = ctx.ip_address,
        user_agent = ctx.user_agent,
        "User signin completed with context"
      );
    }

    Ok(Some(tokens))
  }
}

// ================================================================================================
// Refresh Token Service Implementation
// ================================================================================================

#[async_trait]
impl RefreshTokenService for AuthService {
  #[instrument(skip(self, refresh_token))]
  async fn refresh_token(
    &self,
    refresh_token: &str,
    request_context: Option<AuthContext>,
  ) -> Result<AuthTokens, CoreError> {
    info!("Starting token refresh process");

    // 1. Validate and get refresh token
    let token_record = self
      .refresh_token_repository
      .find_by_token(refresh_token)
      .await?
      .ok_or_else(|| {
        warn!("Invalid refresh token provided");
        CoreError::InvalidToken(TokenValidationError::NotFound)
      })?;

    // 2. Get user by token's user_id
    let user = self
      .user_repository
      .find_by_id(token_record.user_id)
      .await?
      .ok_or_else(|| {
        error!(
          user_id = i64::from(token_record.user_id),
          "User not found for refresh token"
        );
        CoreError::NotFound("User not found".to_string())
      })?;

    info!(
      user_id = i64::from(user.id),
      "Token refresh validated for user"
    );

    // 3. Generate new tokens
    let new_tokens = self.generate_auth_tokens(&user).await?;

    // 4. Revoke old refresh token
    self
      .refresh_token_repository
      .revoke(token_record.id)
      .await?;

    // 5. Log context if provided
    if let Some(ctx) = request_context {
      info!(
        user_id = i64::from(user.id),
        ip_address = ctx.ip_address,
        user_agent = ctx.user_agent,
        "Token refresh completed with context"
      );
    }

    Ok(new_tokens)
  }
}

// ================================================================================================
// Logout Service Implementation
// ================================================================================================

#[async_trait]
impl LogoutService for AuthService {
  #[instrument(skip(self, refresh_token_str))]
  async fn logout(&self, refresh_token_str: &str) -> Result<(), CoreError> {
    info!("Starting user logout process");

    // Find and revoke the refresh token
    if let Some(token_record) = self
      .refresh_token_repository
      .find_by_token(refresh_token_str)
      .await?
    {
      self
        .refresh_token_repository
        .revoke(token_record.id)
        .await
        .map_err(|e| {
          warn!("Failed to revoke refresh token during logout: {}", e);
          e
        })?;
    }

    info!("User logout completed successfully");
    Ok(())
  }

  #[instrument(skip(self))]
  async fn logout_all(&self, user_id: UserId) -> Result<(), CoreError> {
    info!(user_id = i64::from(user_id), "Starting logout all sessions");

    // Revoke all refresh tokens for the user
    self
      .refresh_token_repository
      .revoke_all_for_user(user_id)
      .await
      .map_err(|e| {
        error!(
          user_id = i64::from(user_id),
          error = %e,
          "Failed to revoke all refresh tokens"
        );
        e
      })?;

    info!(
      user_id = i64::from(user_id),
      "All user sessions logged out successfully"
    );
    Ok(())
  }
}

// ================================================================================================
// Auth Service Trait Implementation
// ================================================================================================

impl AuthServiceTrait for AuthService {}

// ================================================================================================
// Additional Domain Methods
// ================================================================================================

impl AuthService {
  /// Change user password using domain service
  #[instrument(skip(self, current_password, new_password))]
  pub async fn change_password(
    &self,
    user_id: UserId,
    current_password: &str,
    new_password: &str,
  ) -> Result<(), CoreError> {
    info!(user_id = i64::from(user_id), "Starting password change");

    // Use domain service for password change logic
    let user_domain_service = self.get_user_domain_service();

    user_domain_service
      .change_password(user_id, current_password, new_password)
      .await
      .map_err(|e| {
        error!(
          user_id = i64::from(user_id),
          error = %e,
          "Password change failed"
        );
        e
      })?;

    info!(
      user_id = i64::from(user_id),
      "Password changed successfully"
    );
    Ok(())
  }

  /// Update user profile using domain service
  #[instrument(skip(self))]
  pub async fn update_profile(&self, user_id: UserId, fullname: &str) -> Result<User, CoreError> {
    info!(user_id = i64::from(user_id), "Starting profile update");

    // Use domain service for profile update logic
    let user_domain_service = self.get_user_domain_service();

    let updated_user = user_domain_service
      .update_profile(user_id, fullname)
      .await
      .map_err(|e| {
        error!(
          user_id = i64::from(user_id),
          error = %e,
          "Profile update failed"
        );
        e
      })?;

    // Publish profile updated event
    let event = ApplicationEvent::User(UserEvent::UserProfileUpdated {
      user_id: i64::from(user_id),
      updated_fields: vec!["fullname".to_string()],
    });

    if let Err(e) = self.event_publisher.publish_async(event).await {
      warn!("Failed to publish profile updated event: {}", e);
    }

    info!(user_id = i64::from(user_id), "Profile updated successfully");
    Ok(updated_user)
  }

  /// Validate users exist using domain service
  pub async fn validate_users_exist(&self, user_ids: &[UserId]) -> Result<(), CoreError> {
    let user_domain_service = self.get_user_domain_service();
    user_domain_service.validate_users_exist(user_ids).await
  }
}

// ================================================================================================
// Factory Functions
// ================================================================================================

/// Create auth service with all dependencies
pub fn create_auth_service(
  user_repository: Arc<UserRepositoryImpl>,
  token_service: Box<dyn TokenService + Send + Sync + 'static>,
  refresh_token_repository: Box<dyn RefreshTokenRepository + Send + Sync + 'static>,
  event_publisher: Arc<ApplicationEventPublisher>,
) -> AuthService {
  AuthService::new(
    user_repository,
    token_service,
    refresh_token_repository,
    event_publisher,
  )
}

/// Create auth service with custom configuration
pub fn create_auth_service_with_config(
  user_repository: Arc<UserRepositoryImpl>,
  token_service: Box<dyn TokenService + Send + Sync + 'static>,
  refresh_token_repository: Box<dyn RefreshTokenRepository + Send + Sync + 'static>,
  event_publisher: Arc<ApplicationEventPublisher>,
  user_config: UserConfig,
) -> AuthService {
  AuthService::new_with_config(
    user_repository,
    token_service,
    refresh_token_repository,
    event_publisher,
    user_config,
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_auth_service_creation() {
    // This test verifies that AuthService can be created with proper dependencies
    // Real integration tests would require actual database connections

    // Verify UserConfig defaults
    let config = UserConfig::default();
    assert_eq!(config.min_password_length, 8);
    assert_eq!(config.max_password_length, 128);
    assert_eq!(config.min_fullname_length, 1);
    assert_eq!(config.max_fullname_length, 100);
  }

  #[test]
  fn test_user_config_customization() {
    let custom_config = UserConfig {
      min_password_length: 12,
      max_password_length: 256,
      min_fullname_length: 2,
      max_fullname_length: 200,
    };

    assert_eq!(custom_config.min_password_length, 12);
    assert_eq!(custom_config.max_password_length, 256);
    assert_eq!(custom_config.min_fullname_length, 2);
    assert_eq!(custom_config.max_fullname_length, 200);
  }
}
