use axum::{Json, extract::State, response::IntoResponse};

use crate::{AppError, AppState};

pub async fn list_all_workspace_users_handler(
  State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
  let users = state.get_users_in_workspace(&state.pool).await?;
  Ok(Json(users))
}
