//! # User Handlers
//!
//! ## Responsibilities
//! - HTTP request/response handling for user operations
//! - Parameter validation and extraction
//! - Response formatting and error handling
//! - Integration with user application services

use axum::{
  extract::{Extension, Path},
  http::{HeaderMap, StatusCode},
  response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument};
use utoipa::ToSchema;
use validator::Validate;

use crate::{
  dtos::{
    core::{ApiError, ApiResponse},
    models::{
      requests::{auth::ChangePasswordRequest, user::UpdateUserProfileRequest},
      responses::{
        auth::ChangePasswordResponse,
        user::{ProfileUpdateResponse, UserProfileResponse},
      },
    },
  },
  error::AppError,
  services::application::workers::profile::service::{UserProfileService, UserProfileServiceTrait},
  AppState,
};
use fechatter_core::{AuthUser, UserId};

// ================================================================================================
// User Profile Handlers
// ================================================================================================

/// Get current user profile
///
/// Returns the profile information for the currently authenticated user.
#[utoipa::path(
  get,
  path = "/api/users/profile",
  responses(
    (status = 200, description = "User profile retrieved successfully", body = UserProfileResponse),
    (status = 401, description = "Unauthorized"),
    (status = 404, description = "User not found"),
    (status = 500, description = "Internal server error")
  ),
  tag = "users",
  summary = "Get user profile",
  description = "Get the profile information for the currently authenticated user."
)]
#[instrument(skip(state), fields(user_id = %user.id))]
pub async fn get_user_profile(
  Extension(state): Extension<AppState>,
  Extension(user): Extension<AuthUser>,
) -> Result<Json<UserProfileResponse>, AppError> {
  info!(user_id = %user.id, "Getting user profile");

  // Create profile service
  let user_repo = std::sync::Arc::new(crate::domains::user::repository::UserRepositoryImpl::new(
    state.pool().clone(),
  ));
  let profile_service = UserProfileService::new(user_repo);

  // Get user profile
  let profile = profile_service.get_user_profile(user.id).await?;

  info!(user_id = %user.id, "User profile retrieved successfully");
  Ok(Json(profile))
}

/// Get user profile by ID
///
/// Returns the profile information for a specific user by their ID.
#[utoipa::path(
  get,
  path = "/api/users/{user_id}/profile",
  params(
    ("user_id" = i64, Path, description = "User ID to get profile for")
  ),
  responses(
    (status = 200, description = "User profile retrieved successfully", body = UserProfileResponse),
    (status = 401, description = "Unauthorized"),
    (status = 404, description = "User not found"),
    (status = 500, description = "Internal server error")
  ),
  tag = "users",
  summary = "Get user profile by ID",
  description = "Get the profile information for a specific user by their ID."
)]
#[instrument(skip(state), fields(user_id = %user_id, requester_id = %user.id))]
pub async fn get_user_profile_by_id(
  Extension(state): Extension<AppState>,
  Path(user_id): Path<i64>,
  Extension(user): Extension<AuthUser>,
) -> Result<Json<UserProfileResponse>, AppError> {
  info!(user_id = %user_id, requester_id = %user.id, "Getting user profile by ID");

  // TODO: Add permission check - users should only be able to view profiles
  // in their workspace or public profiles

  // Create profile service
  let user_repo = std::sync::Arc::new(crate::domains::user::repository::UserRepositoryImpl::new(
    state.pool().clone(),
  ));
  let profile_service = UserProfileService::new(user_repo);

  // Get user profile
  let profile = profile_service.get_user_profile(UserId(user_id)).await?;

  info!(user_id = %user_id, requester_id = %user.id, "User profile retrieved successfully");
  Ok(Json(profile))
}

/// Update current user profile
///
/// Updates the profile information for the currently authenticated user.
#[utoipa::path(
  put,
  path = "/api/users/profile",
  request_body = UpdateUserProfileRequest,
  responses(
    (status = 200, description = "User profile updated successfully", body = ProfileUpdateResponse),
    (status = 400, description = "Invalid request data"),
    (status = 401, description = "Unauthorized"),
    (status = 404, description = "User not found"),
    (status = 500, description = "Internal server error")
  ),
  tag = "users",
  summary = "Update user profile",
  description = "Update the profile information for the currently authenticated user."
)]
#[instrument(skip(state, request), fields(user_id = %user.id))]
pub async fn update_user_profile(
  Extension(state): Extension<AppState>,
  Extension(user): Extension<AuthUser>,
  Json(request): Json<UpdateUserProfileRequest>,
) -> Result<Json<ProfileUpdateResponse>, AppError> {
  info!(user_id = %user.id, "Updating user profile");

  // Validate request
  request.validate().map_err(|e| {
    error!(user_id = %user.id, error = %e, "Invalid profile update request");
    AppError::InvalidInput(format!("Validation failed: {}", e))
  })?;

  // Create profile service with cache invalidation support
  let user_repo = std::sync::Arc::new(crate::domains::user::repository::UserRepositoryImpl::new(
    state.pool().clone(),
  ));
  let profile_service = if let Some(cache_service) = state.cache_service() {
    let unified_cache = std::sync::Arc::new(crate::services::infrastructure::cache::UnifiedCacheService::new(cache_service.clone()));
    UserProfileService::with_cache_invalidation(user_repo, unified_cache)
  } else {
    UserProfileService::new(user_repo)
  };

  // Update user profile
  let response = profile_service
    .update_user_profile(user.id, request)
    .await?;

  info!(user_id = %user.id, updated_fields = ?response.updated_fields, "User profile updated successfully");
  Ok(Json(response))
}

/// Update user profile by ID (Admin only)
///
/// Updates the profile information for a specific user by their ID.
/// This endpoint requires admin privileges.
#[utoipa::path(
  put,
  path = "/api/users/{user_id}/profile",
  params(
    ("user_id" = i64, Path, description = "User ID to update profile for")
  ),
  request_body = UpdateUserProfileRequest,
  responses(
    (status = 200, description = "User profile updated successfully", body = ProfileUpdateResponse),
    (status = 400, description = "Invalid request data"),
    (status = 401, description = "Unauthorized"),
    (status = 403, description = "Forbidden - Admin access required"),
    (status = 404, description = "User not found"),
    (status = 500, description = "Internal server error")
  ),
  tag = "users",
  summary = "Update user profile by ID (Admin)",
  description = "Update the profile information for a specific user by their ID. Requires admin privileges."
)]
#[instrument(skip(state, request), fields(user_id = %user_id, admin_id = %user.id))]
pub async fn update_user_profile_by_id(
  Extension(state): Extension<AppState>,
  Path(user_id): Path<i64>,
  Extension(user): Extension<AuthUser>,
  Json(request): Json<UpdateUserProfileRequest>,
) -> Result<Json<ProfileUpdateResponse>, AppError> {
  info!(user_id = %user_id, admin_id = %user.id, "Admin updating user profile");

  // TODO: Add admin permission check
  // For now, we'll allow any authenticated user to update any profile
  // In production, this should check if the user has admin privileges

  // Validate request
  request.validate().map_err(|e| {
    error!(user_id = %user_id, admin_id = %user.id, error = %e, "Invalid profile update request");
    AppError::InvalidInput(format!("Validation failed: {}", e))
  })?;

  // Create profile service with cache invalidation support
  let user_repo = std::sync::Arc::new(crate::domains::user::repository::UserRepositoryImpl::new(
    state.pool().clone(),
  ));
  let profile_service = if let Some(cache_service) = state.cache_service() {
    let unified_cache = std::sync::Arc::new(crate::services::infrastructure::cache::UnifiedCacheService::new(cache_service.clone()));
    UserProfileService::with_cache_invalidation(user_repo, unified_cache)
  } else {
    UserProfileService::new(user_repo)
  };

  // Update user profile
  let response = profile_service
    .update_user_profile(UserId(user_id), request)
    .await?;

  info!(
    user_id = %user_id,
    admin_id = %user.id,
    updated_fields = ?response.updated_fields,
    "User profile updated successfully by admin"
  );
  Ok(Json(response))
}

/// Extract request ID from headers
fn extract_request_id(headers: &HeaderMap) -> String {
  headers
    .get("x-request-id")
    .and_then(|h| h.to_str().ok())
    .map(String::from)
    .unwrap_or_else(|| uuid::Uuid::now_v7().to_string())
}

/// Change user password
pub async fn change_password_handler(
  Extension(state): Extension<AppState>,
  headers: HeaderMap,
  Extension(current_user): Extension<AuthUser>,
  Json(request): Json<ChangePasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
  let request_id = extract_request_id(&headers);

  // Validate request
  if let Err(e) = request.validate() {
    return Ok(
      (
        StatusCode::BAD_REQUEST,
        Json(ApiResponse::<()>::error(
          ApiError {
            code: "VALIDATION_ERROR".to_string(),
            message: "Invalid input".to_string(),
            details: Some(e.to_string()),
            field: None,
            stack: vec![],
            suggestion: Some("Please check your input and try again".to_string()),
            help_url: Some("/docs/auth/change-password".to_string()),
          },
          request_id,
        )),
      )
        .into_response(),
    );
  }

  // Get auth service with domain layer support
  let auth_service = crate::services::application::workers::auth::create_auth_user_service(&state);

  // Change password through service layer
  auth_service
    .change_password(
      current_user.id,
      &request.current_password,
      &request.new_password,
    )
    .await?;

  // Build response
  let response = ChangePasswordResponse {
    message: "Password changed successfully".to_string(),
    success: true,
    changed_at: chrono::Utc::now(),
    logout_other_sessions: false, // TODO: Implement logout other sessions if needed
  };

  Ok(
    (
      StatusCode::OK,
      Json(ApiResponse::success(response, request_id)),
    )
      .into_response(),
  )
}

/// Legacy change password handler (returns StatusCode) - kept for backward compatibility
pub async fn change_password(
  Extension(state): Extension<AppState>,
  Extension(current_user): Extension<AuthUser>,
  Json(payload): Json<ChangePasswordRequest>,
) -> Result<StatusCode, AppError> {
  // Validate request
  payload
    .validate()
    .map_err(|e| AppError::InvalidInput(format!("Validation error: {}", e)))?;

  // Get auth service
  let auth_service = crate::services::application::workers::auth::create_auth_user_service(&state);

  // Change password
  auth_service
    .change_password(
      current_user.id,
      &payload.current_password,
      &payload.new_password,
    )
    .await?;

  Ok(StatusCode::OK)
}

/// List all users in the current workspace
///
/// Returns a list of all users who belong to the same workspace as the authenticated user.
#[utoipa::path(
  get,
  path = "/api/users",
  responses(
    (status = 200, description = "Users retrieved successfully", body = Vec<UserProfileResponse>),
    (status = 401, description = "Unauthorized"),
    (status = 500, description = "Internal server error")
  ),
  tag = "users",
  summary = "List workspace users",
  description = "Get all users in the current workspace."
)]
#[instrument(skip(state), fields(workspace_id = %user.workspace_id))]
pub async fn list_workspace_users_handler(
  Extension(state): Extension<AppState>,
  Extension(user): Extension<AuthUser>,
) -> Result<Json<Vec<UserProfileResponse>>, AppError> {
  info!(workspace_id = %user.workspace_id, "Listing workspace users");
  
  // Use domain layer instead of direct database access
  let user_repo = std::sync::Arc::new(crate::domains::user::repository::UserRepositoryImpl::new(
    state.pool().clone(),
  ));
  
  let users_data = user_repo
    .get_workspace_users(user.workspace_id.into())
    .await
    .map_err(|e| AppError::Internal(format!("Failed to fetch workspace users: {}", e)))?;
  
  let users: Vec<UserProfileResponse> = users_data
    .into_iter()
    .map(|(id, email, fullname, bio, avatar_url, status, created_at, _updated_at)| {
      // Convert status string to UserStatus
      let user_status = match status.as_deref().unwrap_or("Active") {
        "Suspended" => fechatter_core::models::UserStatus::Suspended,
        _ => fechatter_core::models::UserStatus::Active,
      };
      
      UserProfileResponse {
        id,
        email,
        fullname,
        status: user_status,
        created_at,
        workspace_id: i64::from(user.workspace_id),
        phone: None, // Not available in this query
        title: None, // Not available in this query
        department: None, // Not available in this query
        avatar_url,
        bio,
        timezone: None, // Not available in this query
        language: None, // Not available in this query
        last_active_at: None, // Not available in this query
        settings: None, // Not retrieved in this query
      }
    })
    .collect();
  
  info!(
    workspace_id = %user.workspace_id, 
    user_count = users.len(), 
    "Workspace users retrieved successfully"
  );
  
  Ok(Json(users))
}
