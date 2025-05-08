use axum::{
  Extension, Json,
  extract::{Path, State},
  response::IntoResponse,
};

use crate::{AppError, AppState, models::AuthUser};

pub async fn list_all_workspace_users_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
  let users = state.get_users_in_workspace(user.workspace_id).await?;
  Ok(Json(users))
}

#[allow(unused)]
pub async fn get_workspace_by_id(
  State(state): State<AppState>,
  Path(id): Path<i64>,
  Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
  if user.workspace_id != id {
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
