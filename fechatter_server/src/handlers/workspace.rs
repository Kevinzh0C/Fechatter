use axum::{Extension, Json, extract::State, response::IntoResponse};

use crate::{
  AppError, AppState,
  models::{AuthUser, Workspace},
};

pub async fn list_all_workspace_users_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
  let users = Workspace::fetch_all_users_with_pool(user.workspace_id, &state.pool).await?;
  Ok(Json(users))
}
