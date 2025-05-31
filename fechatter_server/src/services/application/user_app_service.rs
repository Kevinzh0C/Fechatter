//! # User Application Service
//!
//! ## Single Responsibility Principle Application
//! - Each function focuses on a single use case
//! - Business rule validation is independent
//! - Cache strategy is delegated to a dedicated service
//! - Event publishing is delegated to an event publisher
//! - Clear layering between upper and lower levels

use crate::AppError;
use crate::AppState;
use crate::services::application::{
  ApplicationEvent, ApplicationEventPublisher, CacheStrategyService, UserEvent,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use async_trait::async_trait;
use fechatter_core::contracts::UserRepository;
use fechatter_core::{User, UserId, UserStatus, error::CoreError};
use std::sync::Arc;
use tracing::{info, instrument, warn};

//==============================================================================
// BUSINESS ERROR DEFINITIONS
//==============================================================================

/// User business errors - Single responsibility: User domain error handling
#[derive(Debug, thiserror::Error)]
pub enum UserError {
  #[error("Repository error: {0}")]
  Repository(String),

  #[error("User not found: {0}")]
  NotFound(i64),

  #[error("Email already exists: {0}")]
  EmailAlreadyExists(String),

  #[error("Invalid user data: {0}")]
  InvalidUserData(String),

  #[error("Permission denied: {0}")]
  PermissionDenied(String),

  #[error("Business rule violation: {0}")]
  BusinessRuleViolation(String),
}

impl From<UserError> for AppError {
  fn from(err: UserError) -> Self {
    match err {
      UserError::Repository(msg) => AppError::InvalidInput(msg),
      UserError::NotFound(id) => AppError::NotFound(vec![format!("User {} not found", id)]),
      UserError::EmailAlreadyExists(email) => {
        AppError::InvalidInput(format!("Email {} already exists", email))
      }
      UserError::InvalidUserData(msg) => AppError::InvalidInput(msg),
      UserError::PermissionDenied(msg) => AppError::Unauthorized(msg),
      UserError::BusinessRuleViolation(msg) => AppError::InvalidInput(msg),
    }
  }
}

impl From<CoreError> for UserError {
  fn from(err: CoreError) -> Self {
    match err {
      CoreError::NotFound(msg) => UserError::Repository(msg),
      CoreError::ValidationError(msg) => UserError::InvalidUserData(msg),
      _ => UserError::Repository(err.to_string()),
    }
  }
}

//==============================================================================
// APPLICATION SERVICE TRAIT
//==============================================================================

/// User application service contract - Defines use case boundaries
#[async_trait]
pub trait UserServiceTrait: Send + Sync {
  /// Use case: Get user profile
  async fn get_user_profile(&self, user_id: i64) -> Result<UserProfileView, AppError>;

  /// Use case: Update user profile
  async fn update_user_profile(
    &self,
    user_id: i64,
    input: UpdateUserInput,
  ) -> Result<UserProfileView, AppError>;

  /// Use case: Change user password
  async fn change_password(&self, user_id: i64, input: ChangePasswordInput)
  -> Result<(), AppError>;

  /// Use case: Update user status
  async fn update_user_status(&self, user_id: i64, status: UserStatus) -> Result<(), AppError>;

  /// Use case: Validate users exist
  async fn validate_users_exist(&self, user_ids: Vec<i64>) -> Result<(), AppError>;

  /// Use case: Find users by workspace
  async fn find_users_by_workspace(
    &self,
    workspace_id: i64,
  ) -> Result<Vec<UserProfileView>, AppError>;
}

//==============================================================================
// BUSINESS RULES VALIDATOR - Single Responsibility
//==============================================================================

/// User business rules validator - Single responsibility: Business rule validation
pub struct UserBusinessRules;

impl UserBusinessRules {
  // 200DAU platform business rule constants
  const MIN_PASSWORD_LENGTH: usize = 8;
  const MAX_FULLNAME_LENGTH: usize = 100;
  const MAX_EMAIL_LENGTH: usize = 255;

  /// Validate user profile input - Single responsibility: User profile validation
  pub fn validate_user_profile(input: &UpdateUserInput) -> Result<(), UserError> {
    if let Some(ref fullname) = input.fullname {
      Self::validate_fullname(fullname)?;
    }

    if let Some(ref email) = input.email {
      Self::validate_email(email)?;
    }

    Ok(())
  }

  /// Validate password change input - Single responsibility: Password validation
  pub fn validate_password_change(input: &ChangePasswordInput) -> Result<(), UserError> {
    Self::validate_password(&input.new_password)?;

    if input.new_password == input.current_password {
      return Err(UserError::InvalidUserData(
        "New password must be different from current password".to_string(),
      ));
    }

    Ok(())
  }

  /// Validate full name - Single responsibility: Full name validation
  fn validate_fullname(fullname: &str) -> Result<(), UserError> {
    if fullname.trim().is_empty() {
      return Err(UserError::InvalidUserData(
        "Full name cannot be empty".to_string(),
      ));
    }

    if fullname.len() > Self::MAX_FULLNAME_LENGTH {
      return Err(UserError::InvalidUserData(format!(
        "Full name cannot exceed {} characters",
        Self::MAX_FULLNAME_LENGTH
      )));
    }

    Ok(())
  }

  /// Validate email - Single responsibility: Email validation
  fn validate_email(email: &str) -> Result<(), UserError> {
    if email.trim().is_empty() {
      return Err(UserError::InvalidUserData(
        "Email cannot be empty".to_string(),
      ));
    }

    if email.len() > Self::MAX_EMAIL_LENGTH {
      return Err(UserError::InvalidUserData(format!(
        "Email cannot exceed {} characters",
        Self::MAX_EMAIL_LENGTH
      )));
    }

    // Basic email format validation
    if !email.contains('@') || !email.contains('.') {
      return Err(UserError::InvalidUserData(
        "Invalid email format".to_string(),
      ));
    }

    Ok(())
  }

  /// Validate password - Single responsibility: Password validation
  fn validate_password(password: &str) -> Result<(), UserError> {
    if password.len() < Self::MIN_PASSWORD_LENGTH {
      return Err(UserError::InvalidUserData(format!(
        "Password must be at least {} characters",
        Self::MIN_PASSWORD_LENGTH
      )));
    }

    Ok(())
  }
}

//==============================================================================
// APPLICATION SERVICE IMPLEMENTATION
//==============================================================================

/// User application service implementation - Single responsibility: Use case orchestration and coordination
pub struct UserAppService {
  repository: Arc<dyn UserRepository>,
  cache_strategy: Arc<CacheStrategyService>,
  event_publisher: Arc<ApplicationEventPublisher>,
}

impl UserAppService {
  /// Constructor - Dependency injection
  pub fn new(
    repository: Arc<dyn UserRepository>,
    cache_strategy: Arc<CacheStrategyService>,
    event_publisher: Arc<ApplicationEventPublisher>,
  ) -> Self {
    Self {
      repository,
      cache_strategy,
      event_publisher,
    }
  }
}

#[async_trait]
impl UserServiceTrait for UserAppService {
  /// Use case: Get user profile - Single responsibility: User profile query
  #[instrument(skip(self))]
  async fn get_user_profile(&self, user_id: i64) -> Result<UserProfileView, AppError> {
    // 1. Try to get from cache
    let cached_result = self.get_user_profile_from_cache(user_id).await;
    if cached_result.is_ok() {
      return cached_result;
    }

    // 2. Get from repository
    let user = self
      .repository
      .find_by_id(UserId::new(user_id))
      .await?
      .ok_or(UserError::NotFound(user_id))?;

    // 3. Build view
    let profile_view = UserProfileView::from_user(user);

    // 4. Update cache
    self.cache_user_profile(&profile_view).await;

    Ok(profile_view)
  }

  /// Use case: Update user profile - Single responsibility: User profile update
  #[instrument(skip(self, input))]
  async fn update_user_profile(
    &self,
    user_id: i64,
    input: UpdateUserInput,
  ) -> Result<UserProfileView, AppError> {
    // 1. Business rule validation
    UserBusinessRules::validate_user_profile(&input)?;

    // 2. Check email uniqueness (if updating email)
    if let Some(ref email) = input.email {
      self.validate_email_uniqueness(user_id, email).await?;
    }

    // 3. Get current user data
    let mut user = self
      .repository
      .find_by_id(UserId::new(user_id))
      .await?
      .ok_or(UserError::NotFound(user_id))?;

    // 4. Apply updates
    let updated_fields = self.apply_user_updates(&mut user, &input);

    // 5. Save updates
    let updated_user = self.repository.update(UserId::new(user_id), &user).await?;

    // 6. Build return view
    let profile_view = UserProfileView::from_user(updated_user);

    // 7. Publish domain event
    if !updated_fields.is_empty() {
      let event = ApplicationEvent::User(UserEvent::UserProfileUpdated {
        user_id,
        updated_fields,
      });

      if let Err(e) = self.event_publisher.publish_async(event).await {
        warn!("Failed to publish user profile updated event: {}", e);
      }
    }

    info!(user_id = %user_id, "User profile updated successfully");
    Ok(profile_view)
  }

  /// Use case: Change password - Single responsibility: Password change
  #[instrument(skip(self, input))]
  async fn change_password(
    &self,
    user_id: i64,
    input: ChangePasswordInput,
  ) -> Result<(), AppError> {
    // 1. Business rule validation
    UserBusinessRules::validate_password_change(&input)?;

    // 2. Get current user and validate password
    let user = self
      .repository
      .find_by_id(UserId::new(user_id))
      .await?
      .ok_or(UserError::NotFound(user_id))?;

    if !self.verify_password(&input.current_password, &user.password_hash) {
      return Err(UserError::PermissionDenied("Invalid current password".to_string()).into());
    }

    // 3. Hash new password
    let new_password_hash = self.hash_password(&input.new_password)?;

    // 4. Update password
    let mut updated_user = user;
    updated_user.password_hash = Some(new_password_hash);
    self
      .repository
      .update(UserId::new(user_id), &updated_user)
      .await?;

    // 5. Clear user cache (security consideration)
    self.cache_strategy.invalidate_user_caches(user_id).await;

    info!(user_id = %user_id, "Password changed successfully");
    Ok(())
  }

  /// Use case: Update user status - Single responsibility: Status update
  #[instrument(skip(self))]
  async fn update_user_status(&self, user_id: i64, status: UserStatus) -> Result<(), AppError> {
    // 1. Get current user
    let user = self
      .repository
      .find_by_id(UserId::new(user_id))
      .await?
      .ok_or(UserError::NotFound(user_id))?;

    let old_status = format!("{:?}", user.status);
    let new_status = format!("{:?}", status);

    // 2. If status is the same, no need to update
    if user.status == status {
      return Ok(());
    }

    // 3. Update status
    let mut updated_user = user;
    updated_user.status = status;
    self
      .repository
      .update(UserId::new(user_id), &updated_user)
      .await?;

    // 4. Publish status change event
    let event = ApplicationEvent::User(UserEvent::UserStatusChanged {
      user_id,
      old_status,
      new_status,
    });

    if let Err(e) = self.event_publisher.publish_async(event).await {
      warn!("Failed to publish user status changed event: {}", e);
    }

    info!(user_id = %user_id, new_status = ?status, "User status updated successfully");
    Ok(())
  }

  /// Use case: Validate users exist - Single responsibility: User existence validation
  #[instrument(skip(self))]
  async fn validate_users_exist(&self, user_ids: Vec<i64>) -> Result<(), AppError> {
    for user_id in user_ids {
      if self
        .repository
        .find_by_id(UserId::new(user_id))
        .await?
        .is_none()
      {
        return Err(UserError::NotFound(user_id).into());
      }
    }
    Ok(())
  }

  /// Use case: Find users by workspace - Single responsibility: Workspace user query
  #[instrument(skip(self))]
  async fn find_users_by_workspace(
    &self,
    workspace_id: i64,
  ) -> Result<Vec<UserProfileView>, AppError> {
    // TODO: Need to add find by workspace method in core layer
    // Temporary implementation - return empty list
    warn!("find_users_by_workspace not yet implemented in repository layer");
    Ok(vec![])
  }
}

//==============================================================================
// PRIVATE HELPER METHODS - Single Responsibility
//==============================================================================

impl UserAppService {
  /// Get user profile from cache - Single responsibility: Cache query
  async fn get_user_profile_from_cache(&self, user_id: i64) -> Result<UserProfileView, AppError> {
    let cache_key = CacheStrategyService::user_profile_key(user_id);

    match self.cache_strategy.get::<UserProfileView>(&cache_key).await {
      Ok(Some(cached_data)) => Ok(cached_data),
      Ok(None) => Err(AppError::NotFound(vec!["Cache miss".to_string()])), // Use error to indicate cache miss
      Err(e) => {
        warn!("Cache error for user profile {}: {}", user_id, e);
        Err(AppError::InvalidInput(e.to_string()))
      }
    }
  }

  /// Cache user profile - Single responsibility: Cache update
  async fn cache_user_profile(&self, profile_view: &UserProfileView) {
    let cache_key = CacheStrategyService::user_profile_key(profile_view.id);

    if let Err(e) = self
      .cache_strategy
      .set(
        &cache_key,
        profile_view,
        CacheStrategyService::USER_PROFILE_TTL,
      )
      .await
    {
      warn!("Failed to cache user profile {}: {}", profile_view.id, e);
    }
  }

  /// Validate email uniqueness - Single responsibility: Email uniqueness check
  async fn validate_email_uniqueness(&self, user_id: i64, email: &str) -> Result<(), UserError> {
    // TODO: Need to implement find by email method in repository
    // Temporary implementation - skip validation
    warn!("Email uniqueness validation not yet implemented");
    Ok(())
  }

  /// Apply user updates - Single responsibility: User data update
  fn apply_user_updates(&self, user: &mut User, input: &UpdateUserInput) -> Vec<String> {
    let mut updated_fields = Vec::new();

    if let Some(ref fullname) = input.fullname {
      if user.fullname != *fullname {
        user.fullname = fullname.clone();
        updated_fields.push("fullname".to_string());
      }
    }

    if let Some(ref email) = input.email {
      if user.email != *email {
        user.email = email.clone();
        updated_fields.push("email".to_string());
      }
    }

    updated_fields
  }

  /// Verify password - Single responsibility: Password verification
  fn verify_password(&self, password: &str, hash: &Option<String>) -> bool {
    // Implement actual password verification logic using argon2
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    match hash {
      Some(h) => match PasswordHash::new(h) {
        Ok(parsed_hash) => Argon2::default()
          .verify_password(password.as_bytes(), &parsed_hash)
          .is_ok(),
        Err(_) => false,
      },
      None => false,
    }
  }

  /// Hash password - Single responsibility: Password hashing
  fn hash_password(&self, password: &str) -> Result<String, UserError> {
    // Implement actual password hashing logic using argon2
    use argon2::{
      Argon2,
      password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
      .hash_password(password.as_bytes(), &salt)
      .map(|hash| hash.to_string())
      .map_err(|e| UserError::InvalidUserData(format!("Password hashing failed: {}", e)))
  }
}

//==============================================================================
// VIEW MODELS AND DTOS
//==============================================================================

/// User profile view - Service layer DTO
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserProfileView {
  pub id: i64,
  pub fullname: String,
  pub email: String,
  pub status: String,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub updated_at: chrono::DateTime<chrono::Utc>,
  pub workspace_id: Option<i64>,
}

impl UserProfileView {
  /// Build view from User entity - Single responsibility: View construction
  pub fn from_user(user: User) -> Self {
    Self {
      id: i64::from(user.id),
      fullname: user.fullname,
      email: user.email,
      status: format!("{:?}", user.status),
      created_at: user.created_at,
      updated_at: user.created_at,
      workspace_id: Some(i64::from(user.workspace_id)),
    }
  }
}

/// Update user input
#[derive(Debug, Clone)]
pub struct UpdateUserInput {
  pub fullname: Option<String>,
  pub email: Option<String>,
}

/// Change password input
#[derive(Debug, Clone)]
pub struct ChangePasswordInput {
  pub current_password: String,
  pub new_password: String,
}

//==============================================================================
// UNIT TESTS
//==============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_business_rules_fullname_validation() {
    // Test empty full name
    let input = UpdateUserInput {
      fullname: Some("".to_string()),
      email: None,
    };
    assert!(UserBusinessRules::validate_user_profile(&input).is_err());

    // Test too long full name
    let long_name = "a".repeat(101);
    let input = UpdateUserInput {
      fullname: Some(long_name),
      email: None,
    };
    assert!(UserBusinessRules::validate_user_profile(&input).is_err());

    // Test valid full name
    let input = UpdateUserInput {
      fullname: Some("John Doe".to_string()),
      email: None,
    };
    assert!(UserBusinessRules::validate_user_profile(&input).is_ok());
  }

  #[test]
  fn test_business_rules_password_validation() {
    // Test too short password
    let input = ChangePasswordInput {
      current_password: "old123".to_string(),
      new_password: "123".to_string(),
    };
    assert!(UserBusinessRules::validate_password_change(&input).is_err());

    // Test new password same as old
    let input = ChangePasswordInput {
      current_password: "password123".to_string(),
      new_password: "password123".to_string(),
    };
    assert!(UserBusinessRules::validate_password_change(&input).is_err());

    // Test valid password
    let input = ChangePasswordInput {
      current_password: "oldpassword123".to_string(),
      new_password: "newpassword123".to_string(),
    };
    assert!(UserBusinessRules::validate_password_change(&input).is_ok());
  }

  #[test]
  fn test_user_status_to_string() {
    assert_eq!(format!("{:?}", UserStatus::Active), "Active");
    assert_eq!(format!("{:?}", UserStatus::Suspended), "Suspended");
  }
}

//==============================================================================
// SIMPLE USER APPLICATION SERVICE FOR APPSTATE INTEGRATION
//==============================================================================

/// Simple user application service for AppState integration
pub struct UserApplicationService {
  app_state: AppState,
}

impl UserApplicationService {
  pub fn new(app_state: AppState) -> Self {
    Self { app_state }
  }

  /// Find user by ID through repository
  pub async fn find_user_by_id(
    &self,
    user_id: i64,
  ) -> Result<Option<fechatter_core::User>, AppError> {
    let user_repo = crate::domains::user::repository::UserRepositoryImpl::new(Arc::new(
      self.app_state.pool().clone(),
    ));

    // Use the correct method from the repository
    user_repo
      .find_by_id(fechatter_core::UserId(user_id))
      .await
      .map_err(|e| AppError::InvalidInput(format!("Failed to find user: {}", e)))
  }

  /// Find user by ID with pool (compatibility method)
  pub async fn find_user_by_id_with_pool(
    &self,
    user_id: fechatter_core::UserId,
  ) -> Result<Option<fechatter_core::User>, AppError> {
    self.find_user_by_id(i64::from(user_id)).await
  }
}
