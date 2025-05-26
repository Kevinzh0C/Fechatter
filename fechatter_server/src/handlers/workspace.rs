use axum::{
  Extension, Json,
  extract::{Path, State},
  response::IntoResponse,
};
use fechatter_core::{User, Workspace};

use crate::{AppError, AppState, error::ErrorOutput, models::AuthUser};

/// 获取工作区所有用户
#[utoipa::path(
    get,
    path = "/api/workspace/users",
    security(
        ("access_token" = [])
    ),
    responses(
        (status = 200, description = "Users retrieved successfully", body = Vec<User>),
        (status = 401, description = "Unauthorized", body = ErrorOutput)
    ),
    tag = "workspace"
)]
pub async fn list_all_workspace_users_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
  let users = state
    .get_users_in_workspace(user.workspace_id.into())
    .await?;
  Ok(Json(users))
}

/// 获取工作区信息
#[utoipa::path(
    get,
    path = "/api/workspaces/{id}",
    params(
        ("id" = i64, Path, description = "Workspace ID")
    ),
    security(
        ("access_token" = [])
    ),
    responses(
        (status = 200, description = "Workspace retrieved successfully", body = Workspace),
        (status = 401, description = "Unauthorized", body = ErrorOutput),
        (status = 403, description = "Permission denied", body = ErrorOutput),
        (status = 404, description = "Workspace not found", body = ErrorOutput)
    ),
    tag = "workspace"
)]
#[allow(unused)]
pub async fn get_workspace_by_id(
  State(state): State<AppState>,
  Path(id): Path<i64>,
  Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
  if user.workspace_id != fechatter_core::WorkspaceId(id) {
    return Err(AppError::ChatPermissionError(
      "No access to this workspace".into(),
    ));
  }

  let workspace = state
    .find_by_id_with_pool(id)
    .await?
    .ok_or_else(|| AppError::NotFound(vec![id.to_string()]))?;

  Ok(Json(workspace))
}
