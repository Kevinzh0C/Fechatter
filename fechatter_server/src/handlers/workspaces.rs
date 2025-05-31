//! # Workspace Handlers - Protocol Compliant Version
//!
//! **Design Philosophy**: Strictly follows HANDLER_RESPONSIBILITY_ANALYSIS.md protocol
//! **Responsibility**: Only Service calls + Response construction, no business logic
//! **Layer**: Handler Layer (Layer 1) - Minimal coordination layer
//!
//! ## ✅ Protocol Compliance
//! - ✅ Each Handler ≤ 20 lines of code
//! - ✅ Only handles request coordination and response building
//! - ✅ All business logic delegated to Application Service
//! - ✅ No direct database access or SQL execution
//! - ✅ Unified error handling and conversion

use axum::{
  extract::{Extension, Path, State},
  http::StatusCode,
  response::Json,
};
use serde::{Deserialize, Serialize};

use crate::{AppError, AppState};
use fechatter_core::AuthUser;

// Use types and traits from application service layer
use crate::services::application::{
  InviteUserCommand, UpdateWorkspaceCommand, UserSummaryView, WorkspaceApplicationServiceTrait,
  WorkspaceView, create_workspace_application_service,
};

// =============================================================================
// Legacy Request/Response Types - Maintaining API Backwards Compatibility
// =============================================================================

#[derive(Serialize)]
pub struct UserSummary {
  pub id: i64,
  pub fullname: String,
  pub email: String,
  pub status: String,
}

#[derive(Serialize)]
pub struct WorkspaceInfo {
  pub id: i64,
  pub name: String,
  pub owner_id: i64,
  pub member_count: i32,
  pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct UpdateWorkspaceRequest {
  pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct InviteUserRequest {
  pub email: String,
}

// =============================================================================
// Handler Functions - Minimal Implementation (Protocol Compliant)
// =============================================================================

/// List workspace users - Minimal Service call
pub async fn list_users_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(workspace_id): Path<i64>,
) -> Result<Json<Vec<UserSummary>>, AppError> {
  // 1. Get Application Service with proper Result handling
  let workspace_service = create_workspace_application_service(&state)?;

  // 2. Delegate business logic
  let users = workspace_service
    .list_workspace_users(workspace_id, i64::from(user.id))
    .await?;

  // 3. Transform response
  let response = users
    .into_iter()
    .map(|u| UserSummary {
      id: u.id,
      fullname: u.fullname,
      email: u.email,
      status: u.status,
    })
    .collect();

  // 4. Return response
  Ok(Json(response))
}

/// List all workspace users - Alias function (Backwards compatibility)
pub async fn list_all_workspace_users_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
) -> Result<Json<Vec<UserSummary>>, AppError> {
  // Use current user's workspace ID
  let workspace_id = i64::from(user.workspace_id);
  list_users_handler(State(state), Extension(user), Path(workspace_id)).await
}

/// Update workspace info - Minimal Service call
pub async fn update_workspace_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(workspace_id): Path<i64>,
  Json(payload): Json<UpdateWorkspaceRequest>,
) -> Result<Json<WorkspaceInfo>, AppError> {
  // 1. Get Application Service with proper Result handling
  let workspace_service = create_workspace_application_service(&state)?;

  // 2. Delegate business logic
  let workspace = workspace_service
    .update_workspace(
      workspace_id,
      i64::from(user.id),
      UpdateWorkspaceCommand { name: payload.name },
    )
    .await?;

  // 3. Transform response
  let response = WorkspaceInfo {
    id: workspace.id,
    name: workspace.name,
    owner_id: workspace.owner_id,
    member_count: workspace.member_count,
    created_at: workspace.created_at,
  };

  // 4. Return response
  Ok(Json(response))
}

/// Invite user to workspace - Minimal Service call
pub async fn invite_user_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(workspace_id): Path<i64>,
  Json(payload): Json<InviteUserRequest>,
) -> Result<StatusCode, AppError> {
  // 1. Get Application Service with proper Result handling
  let workspace_service = create_workspace_application_service(&state)?;

  // 2. Delegate business logic
  workspace_service
    .invite_user(
      workspace_id,
      i64::from(user.id),
      InviteUserCommand {
        email: payload.email,
      },
    )
    .await?;

  // 3. Return status
  Ok(StatusCode::OK)
}

/// Get workspace info - Minimal Service call
pub async fn get_workspace_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(workspace_id): Path<i64>,
) -> Result<Json<WorkspaceInfo>, AppError> {
  // 1. Get Application Service with proper Result handling
  let workspace_service = create_workspace_application_service(&state)?;

  // 2. Delegate business logic
  let workspace = workspace_service
    .get_workspace_details(workspace_id, i64::from(user.id))
    .await?;

  // 3. Transform response
  let response = WorkspaceInfo {
    id: workspace.id,
    name: workspace.name,
    owner_id: workspace.owner_id,
    member_count: workspace.member_count,
    created_at: workspace.created_at,
  };

  // 4. Return response
  Ok(Json(response))
}
