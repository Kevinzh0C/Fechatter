//! # Workspace Handlers - Protocol Compliant Version
//!
//! **Design Philosophy**: Strictly follows HANDLER_RESPONSIBILITY_ANALYSIS.md protocol
//! **Responsibility**: Only Service calls + Response construction, no business logic
//! **Layer**: Handler Layer (Layer 1) - Minimal coordination layer
//!
//! ## Protocol Compliance
//! - Each Handler ≤ 20 lines of code
//! - Only handles request coordination and response building
//! - All business logic delegated to Application Service
//! - No direct database access or SQL execution
//! - Unified error handling and conversion

use axum::{
  extract::{Extension, Path, State},
  http::StatusCode,
  response::Json,
};
use serde::{Deserialize, Serialize};

use crate::{AppError, AppState};
use fechatter_core::AuthUser;

// Use types and traits from application service layer
use crate::services::application::workers::workspace::{
  InviteUserCommand, UpdateWorkspaceCommand, create_workspace_application_service,
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

/// List all workspace users - With workspace permission validation
pub async fn list_all_workspace_users_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(workspace_id): Path<i64>,
) -> Result<Json<Vec<UserSummary>>, AppError> {
  // 1. Validate user has access to the specified workspace
  if i64::from(user.workspace_id) != workspace_id {
    return Err(AppError::Forbidden("Access denied to workspace".to_string()));
  }

  // 2. Get Application Service with proper Result handling
  let workspace_service = create_workspace_application_service(&state)?;

  // 3. Delegate business logic
  let users = workspace_service
    .list_workspace_users(workspace_id.into())
    .await?;

  // 4. Transform response
  let response: Vec<UserSummary> = users
    .into_iter()
    .map(|u| UserSummary {
      id: u.id,
      fullname: u.fullname,
      email: u.email,
      status: u.status,
    })
    .collect();

  Ok(Json(response))
}

/// Update workspace info - With workspace permission validation
pub async fn update_workspace_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(workspace_id): Path<i64>,
  Json(payload): Json<UpdateWorkspaceRequest>,
) -> Result<Json<WorkspaceInfo>, AppError> {
  // 1. Validate user has access to the specified workspace
  if i64::from(user.workspace_id) != workspace_id {
    return Err(AppError::Forbidden("Access denied to workspace".to_string()));
  }

  // 2. Get Application Service with proper Result handling
  let workspace_service = create_workspace_application_service(&state)?;

  // 3. Delegate business logic
  let workspace = workspace_service
    .update_workspace(
      workspace_id.into(),
      user.id,
      UpdateWorkspaceCommand { name: payload.name },
    )
    .await?;

  // 4. Transform response
  let response = WorkspaceInfo {
    id: workspace.id,
    name: workspace.name,
    owner_id: workspace.owner_id,
    member_count: workspace.member_count,
    created_at: workspace.created_at,
  };

  // 5. Return response
  Ok(Json(response))
}

/// Get current workspace info - For /api/workspaces endpoint
pub async fn get_current_workspace_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
) -> Result<Json<WorkspaceInfo>, AppError> {
  // 1. Get Application Service with proper Result handling
  let workspace_service = create_workspace_application_service(&state)?;

  // 2. Use current user's workspace ID
  let workspace_id = i64::from(user.workspace_id);

  // 3. Delegate business logic to get workspace details
  let workspace = workspace_service
    .get_workspace_details(workspace_id.into())
    .await?;

  // 4. Transform response
  let response = WorkspaceInfo {
    id: workspace_id,
    name: workspace.name,
    owner_id: workspace.owner_id,
    member_count: workspace.member_count,
    created_at: workspace.created_at,
  };

  // 5. Return response
  Ok(Json(response))
}

/// List workspace chats - For /api/workspace/chats endpoint
pub async fn list_workspace_chats_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
) -> Result<Json<Vec<crate::dtos::models::responses::chat::ChatSidebarDto>>, AppError> {
  // 1. Get chat service
  let chat_service = state.application_services().chat_application_service();

  // 2. Use current user's workspace ID to filter chats
  let workspace_id = i64::from(user.workspace_id);
  let user_id = i64::from(user.id);

  // 3. Get chats for the current workspace
  let chats = chat_service.list_user_chats(user_id).await?;

  // 4. Transform to ChatInfo format
  let response: Vec<crate::dtos::models::responses::chat::ChatSidebarDto> = chats
    .into_iter()
    .map(
      |chat| crate::dtos::models::responses::chat::ChatSidebarDto {
        id: chat.id.into(),
        name: chat.name,
        chat_type: chat.chat_type,
        description: None,  // TODO: add description to chat sidebar view
        last_message: None, // TODO: add last message to chat sidebar view
        last_message_at: chat.last_message.as_ref().and_then(|_| None), // String doesn't have created_at
        unread_count: 0,   // TODO: implement unread count tracking
        is_creator: false, // Field not available in current ChatSidebar, set default
        is_muted: false,   // TODO: add muted status to chat sidebar view
        is_pinned: false,  // TODO: add pinned status to chat sidebar view
      },
    )
    .collect();

  // 5. Return response
  Ok(Json(response))
}

/// Invite user to workspace - With workspace permission validation
pub async fn invite_user_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(workspace_id): Path<i64>,
  Json(payload): Json<InviteUserRequest>,
) -> Result<StatusCode, AppError> {
  // 1. Validate user has access to the specified workspace
  if i64::from(user.workspace_id) != workspace_id {
    return Err(AppError::Forbidden("Access denied to workspace".to_string()));
  }

  // 2. Get Application Service with proper Result handling
  let workspace_service = create_workspace_application_service(&state)?;

  // 3. Delegate business logic
  workspace_service
    .invite_user(
      workspace_id.into(),
      user.id,
      InviteUserCommand {
        email: payload.email,
      },
    )
    .await?;

  // 4. Return status
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
    .get_workspace_details(workspace_id.into())
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

/// Add members to workspace - With workspace permission validation
pub async fn add_members_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(workspace_id): Path<i64>,
  Json(member_ids): Json<Vec<i64>>,
) -> Result<Json<Vec<UserSummary>>, AppError> {
  // 1. Validate user has access to the specified workspace
  if i64::from(user.workspace_id) != workspace_id {
    return Err(AppError::Forbidden("Access denied to workspace".to_string()));
  }

  // 2. Get Application Service
  let workspace_service = create_workspace_application_service(&state)?;

  // 3. Create command
  let command = crate::services::application::workers::workspace::AddMembersCommand { member_ids };

  // 4. Delegate to service
  let users = workspace_service
    .add_members(workspace_id.into(), user.id, command)
    .await?;

  // 5. Transform response
  let response: Vec<UserSummary> = users
    .into_iter()
    .map(|u| UserSummary {
      id: u.id,
      fullname: u.fullname,
      email: u.email,
      status: u.status,
    })
    .collect();

  Ok(Json(response))
}
