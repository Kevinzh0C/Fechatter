//! # User Profile Application Service
//!
//! ## Responsibilities
//! - Orchestrate user profile operations
//! - Business logic for profile management
//! - Data transformation between layers
//! - Validation and error handling
//! - Cache invalidation on profile updates

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, info, instrument, warn};
use validator::Validate;

use crate::domains::user::repository::UserRepositoryImpl;
use crate::dtos::models::{
  requests::user::UpdateUserProfileRequest,
  responses::user::{ProfileUpdateResponse, UserProfileResponse},
};
use crate::error::AppError;
use crate::services::infrastructure::cache::{DistributedLockCacheInvalidator, UnifiedCacheService};
use fechatter_core::models::{AuthUser, User, UserId};

/// User Profile Application Service Trait
#[async_trait]
pub trait UserProfileServiceTrait: Send + Sync {
  /// Get user profile by ID
  async fn get_user_profile(&self, user_id: UserId) -> Result<UserProfileResponse, AppError>;

  /// Update user profile
  async fn update_user_profile(
    &self,
    user_id: UserId,
    request: UpdateUserProfileRequest,
  ) -> Result<ProfileUpdateResponse, AppError>;
}

/// User Profile Application Service Implementation with Cache Invalidation
pub struct UserProfileService {
  user_repository: Arc<UserRepositoryImpl>,
  cache_invalidator: Option<Arc<DistributedLockCacheInvalidator>>,
}

impl UserProfileService {
  pub fn new(user_repository: Arc<UserRepositoryImpl>) -> Self {
    Self { 
      user_repository,
      cache_invalidator: None,
    }
  }

  /// Create new service with cache invalidation support
  pub fn with_cache_invalidation(
    user_repository: Arc<UserRepositoryImpl>,
    cache_service: Arc<UnifiedCacheService>,
  ) -> Self {
    let cache_invalidator = Arc::new(DistributedLockCacheInvalidator::new(cache_service));
    Self { 
      user_repository,
      cache_invalidator: Some(cache_invalidator),
    }
  }
}

#[async_trait]
impl UserProfileServiceTrait for UserProfileService {
  #[instrument(skip(self), fields(user_id = %user_id))]
  async fn get_user_profile(&self, user_id: UserId) -> Result<UserProfileResponse, AppError> {
    info!(user_id = %user_id, "Getting user profile");

    // Get user from repository
    let user = self
      .user_repository
      .get_user_profile(user_id)
      .await
      .map_err(|e| {
        error!(user_id = %user_id, error = %e, "Failed to get user profile from repository");
        AppError::from(e)
      })?
      .ok_or_else(|| {
        error!(user_id = %user_id, "User not found");
        AppError::NotFound(vec!["User not found".to_string()])
      })?;

    // Get user settings
    let settings = self
      .user_repository
      .get_user_settings(user_id)
      .await
      .map_err(|e| {
        error!(user_id = %user_id, error = %e, "Failed to get user settings");
        AppError::from(e)
      })?;

    // Convert to response DTO
    let response = UserProfileResponse {
      id: i64::from(user.id),
      fullname: user.fullname,
      email: user.email,
      status: user.status,
      created_at: user.created_at,
      workspace_id: i64::from(user.workspace_id),
      phone: user.phone,
      title: user.title,
      department: user.department,
      avatar_url: user.avatar_url,
      bio: user.bio,
      timezone: user.timezone,
      language: user.language,
      last_active_at: user.last_active_at,
      settings: settings.map(
        |s| crate::dtos::models::responses::user::UserSettingsResponse {
          email_notifications: s.email_notifications,
          push_notifications: s.push_notifications,
          desktop_notifications: s.desktop_notifications,
          notification_sound: s.notification_sound,
          show_online_status: s.show_online_status,
          auto_away: s.auto_away,
          auto_away_minutes: s.auto_away_minutes,
          theme: s.theme,
          message_display: s.message_display,
        },
      ),
    };

    info!(user_id = %user_id, "User profile retrieved successfully");
    Ok(response)
  }

  #[instrument(skip(self, request), fields(user_id = %user_id))]
  async fn update_user_profile(
    &self,
    user_id: UserId,
    request: UpdateUserProfileRequest,
  ) -> Result<ProfileUpdateResponse, AppError> {
    info!(user_id = %user_id, "Updating user profile");

    // Validate request
    request.validate().map_err(|e| {
      error!(user_id = %user_id, error = %e, "Invalid profile update request");
      AppError::InvalidInput(format!("Validation failed: {}", e))
    })?;

    // Track which fields are being updated
    let mut updated_fields = Vec::new();

    if request.fullname.is_some() {
      updated_fields.push("fullname".to_string());
    }
    if request.email.is_some() {
      updated_fields.push("email".to_string());
    }
    if request.phone.is_some() {
      updated_fields.push("phone".to_string());
    }
    if request.title.is_some() {
      updated_fields.push("title".to_string());
    }
    if request.department.is_some() {
      updated_fields.push("department".to_string());
    }
    if request.avatar_url.is_some() {
      updated_fields.push("avatar_url".to_string());
    }
    if request.bio.is_some() {
      updated_fields.push("bio".to_string());
    }
    if request.timezone.is_some() {
      updated_fields.push("timezone".to_string());
    }
    if request.language.is_some() {
      updated_fields.push("language".to_string());
    }

    // If no fields to update, return current profile
    if updated_fields.is_empty() {
      let current_profile = self.get_user_profile(user_id).await?;
      return Ok(ProfileUpdateResponse {
        success: true,
        message: "No changes requested".to_string(),
        updated_fields: Vec::new(),
        profile: current_profile,
      });
    }

    // Update user profile in repository
    let updated_user = self
      .user_repository
      .update_profile(
        user_id,
        request.fullname.as_deref(),
        request.email.as_deref(),
        request.phone.as_deref(),
        request.title.as_deref(),
        request.department.as_deref(),
        request.avatar_url.as_deref(),
        request.bio.as_deref(),
        request.timezone.as_deref(),
        request.language.as_deref(),
      )
      .await
      .map_err(|e| {
        error!(user_id = %user_id, error = %e, "Failed to update user profile in repository");
        AppError::from(e)
      })?;

    // Get updated user settings
    let settings = self
      .user_repository
      .get_user_settings(user_id)
      .await
      .map_err(|e| {
        error!(user_id = %user_id, error = %e, "Failed to get user settings after update");
        AppError::from(e)
      })?;

    // Convert to response
    let profile_response = UserProfileResponse {
      id: i64::from(updated_user.id),
      fullname: updated_user.fullname,
      email: updated_user.email,
      status: updated_user.status,
      created_at: updated_user.created_at,
      workspace_id: i64::from(updated_user.workspace_id),
      phone: updated_user.phone,
      title: updated_user.title,
      department: updated_user.department,
      avatar_url: updated_user.avatar_url,
      bio: updated_user.bio,
      timezone: updated_user.timezone,
      language: updated_user.language,
      last_active_at: updated_user.last_active_at,
      settings: settings.map(
        |s| crate::dtos::models::responses::user::UserSettingsResponse {
          email_notifications: s.email_notifications,
          push_notifications: s.push_notifications,
          desktop_notifications: s.desktop_notifications,
          notification_sound: s.notification_sound,
          show_online_status: s.show_online_status,
          auto_away: s.auto_away,
          auto_away_minutes: s.auto_away_minutes,
          theme: s.theme,
          message_display: s.message_display,
        },
      ),
    };

    // Trigger cache invalidation after successful profile update
    if let Some(ref cache_invalidator) = self.cache_invalidator {
      let user_id_i64 = i64::from(user_id);
      match cache_invalidator.handle_user_updated_with_lock(user_id_i64).await {
        Ok(_) => {
          info!(user_id = %user_id, "Cache invalidation triggered successfully after profile update");
        }
        Err(e) => {
          warn!(user_id = %user_id, error = %e, "Failed to trigger cache invalidation after profile update - continuing anyway");
        }
      }
    } else {
      warn!(user_id = %user_id, "Cache invalidator not configured - profile update completed without cache invalidation");
    }

    let response = ProfileUpdateResponse {
      success: true,
      message: format!(
        "Profile updated successfully. Updated fields: {}",
        updated_fields.join(", ")
      ),
      updated_fields,
      profile: profile_response,
    };

    info!(user_id = %user_id, updated_fields = ?response.updated_fields, "User profile updated successfully");
    Ok(response)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::domains::user::repository::UserRepositoryImpl;
  use crate::setup_test_users;
  use anyhow::Result;
  use fechatter_core::models::{UserStatus, WorkspaceId};

  #[tokio::test]
  async fn test_get_user_profile() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user_repo = Arc::new(UserRepositoryImpl::new(state.pool()));
    let profile_service = UserProfileService::new(user_repo);

    let user_id = users[0].id;
    let profile = profile_service.get_user_profile(user_id).await?;

    assert_eq!(profile.id, i64::from(user_id));
    assert_eq!(profile.fullname, users[0].fullname);
    assert_eq!(profile.email, users[0].email);

    Ok(())
  }

  #[tokio::test]
  async fn test_update_user_profile() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user_repo = Arc::new(UserRepositoryImpl::new(state.pool()));
    let profile_service = UserProfileService::new(user_repo);

    let user_id = users[0].id;
    let update_request = UpdateUserProfileRequest {
      fullname: Some("Updated Name".to_string()),
      email: Some("updated@example.com".to_string()),
      phone: Some("+1234567890".to_string()),
      title: Some("Senior Engineer".to_string()),
      department: Some("Engineering".to_string()),
      avatar_url: Some("https://example.com/avatar.jpg".to_string()),
      bio: Some("Updated bio".to_string()),
      timezone: Some("America/New_York".to_string()),
      language: Some("en".to_string()),
    };

    let response = profile_service
      .update_user_profile(user_id, update_request)
      .await?;

    assert!(response.success);
    assert_eq!(response.profile.fullname, "Updated Name");
    assert_eq!(response.profile.email, "updated@example.com");
    assert!(response.updated_fields.contains(&"fullname".to_string()));
    assert!(response.updated_fields.contains(&"email".to_string()));

    Ok(())
  }

  #[tokio::test]
  async fn test_update_user_profile_no_changes() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user_repo = Arc::new(UserRepositoryImpl::new(state.pool()));
    let profile_service = UserProfileService::new(user_repo);

    let user_id = users[0].id;
    let update_request = UpdateUserProfileRequest {
      fullname: None,
      email: None,
      phone: None,
      title: None,
      department: None,
      avatar_url: None,
      bio: None,
      timezone: None,
      language: None,
    };

    let response = profile_service
      .update_user_profile(user_id, update_request)
      .await?;

    assert!(response.success);
    assert!(response.updated_fields.is_empty());
    assert_eq!(response.message, "No changes requested");

    Ok(())
  }
}
