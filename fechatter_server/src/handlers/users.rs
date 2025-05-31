// User handlers - TODO: Implement user management functionality

use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  response::Json,
};
use serde_json::Value;

use crate::{AppState, error::AppError};
use fechatter_core::AuthUser;

/// Get user profile
/// TODO: Implement user profile retrieval
pub async fn get_user_profile(
  State(_state): State<AppState>,
  Path(_user_id): Path<i64>,
  _current_user: AuthUser,
) -> Result<Json<Value>, AppError> {
  // TODO: Implement user profile retrieval
  Err(AppError::AnyError(anyhow::anyhow!(
    "User profile retrieval not yet implemented"
  )))
}

/// Update user profile
/// TODO: Implement user profile update
pub async fn update_user_profile(
  State(_state): State<AppState>,
  _current_user: AuthUser,
  Json(_payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
  // TODO: Implement user profile update
  Err(AppError::AnyError(anyhow::anyhow!(
    "User profile update not yet implemented"
  )))
}

/// Change user password
/// TODO: Implement password change
pub async fn change_password(
  State(_state): State<AppState>,
  _current_user: AuthUser,
  Json(_payload): Json<Value>,
) -> Result<StatusCode, AppError> {
  // TODO: Implement password change
  Err(AppError::AnyError(anyhow::anyhow!(
    "Password change not yet implemented"
  )))
}
