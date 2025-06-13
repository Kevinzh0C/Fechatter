use crate::{AppError, AppState, error::ErrorOutput};
use axum::{
  extract::{Extension, Path, State},
  response::Json,
};
use fechatter_core::{SearchMessages, SearchResult, models::jwt::UserClaims};

/// Search messages in a specific chat
#[utoipa::path(
    post,
    path = "/api/chat/{id}/messages/search",
    params(
        ("id" = i64, Path, description = "Chat ID")
    ),
    request_body = SearchMessages,
    responses(
        (status = 200, description = "Search results returned successfully", body = SearchResult),
        (status = 400, description = "Invalid search parameters", body = ErrorOutput),
        (status = 401, description = "Unauthorized", body = ErrorOutput),
        (status = 403, description = "Permission denied", body = ErrorOutput),
        (status = 404, description = "Chat not found", body = ErrorOutput),
        (status = 500, description = "Search service error", body = ErrorOutput)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "search"
)]
pub async fn search_messages(
  Extension(current_user): Extension<UserClaims>,
  State(app_state): State<AppState>,
  Path(chat_id): Path<i64>,
  Json(mut search_request): Json<SearchMessages>,
) -> Result<Json<SearchResult>, AppError> {
  use validator::Validate;

  // Set chat_id from path parameter
  search_request.chat_id = Some(fechatter_core::ChatId(chat_id));

  // Set workspace_id to current user's workspace for consistency
  search_request.workspace_id = current_user.workspace_id;

  // Validate the search request
  if let Err(validation_errors) = search_request.validate() {
    return Err(AppError::InvalidInput(format!(
      "Validation failed: {:?}",
      validation_errors
    )));
  }

  // Get search service (permission check is handled by middleware)
  let search_service = app_state
    .search_service()
    .ok_or_else(|| AppError::SearchError("Search service is not available".to_string()))?;

  // Perform the search
  let search_result = search_service.search_messages(&search_request).await?;

  Ok(Json(search_result))
}
