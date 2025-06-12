//! # Search Handlers
//!
//! ## Responsibilities
//! - HTTP request/response handling for search operations
//! - Parameter validation and extraction
//! - Response formatting and error handling
//! - Integration with search application service

use axum::{
  extract::{Path, Query},
  http::StatusCode,
  response::Json,
  Extension,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::{
  error::AppError,
  services::application::workers::search::{MessageSearchResults, SearchApplicationServiceTrait},
  AppState,
};
use fechatter_core::models::{AuthUser, ChatId, UserId};

// ================================================================================================
// Request/Response DTOs
// ================================================================================================

/// Search messages request parameters
#[derive(Debug, Deserialize, Validate, ToSchema, utoipa::IntoParams)]
pub struct SearchMessagesQuery {
  /// Search query string
  #[validate(length(
    min = 1,
    max = 500,
    message = "Query must be between 1 and 500 characters"
  ))]
  pub q: String,

  /// Results limit (default: 20, max: 100)
  #[validate(range(min = 1, max = 100, message = "Limit must be between 1 and 100"))]
  #[serde(default = "default_limit")]
  pub limit: u32,

  /// Results offset for pagination (default: 0)
  #[validate(range(min = 0, max = 10000, message = "Offset must be between 0 and 10000"))]
  #[serde(default)]
  pub offset: u32,

  /// Sort order (default: "relevance")
  #[serde(default = "default_sort")]
  pub sort: String,
}

/// Search suggestions request parameters
#[derive(Debug, Deserialize, Validate, ToSchema, IntoParams)]
pub struct SearchSuggestionsQuery {
  /// Partial search query
  #[validate(length(
    min = 1,
    max = 100,
    message = "Query must be between 1 and 100 characters"
  ))]
  pub q: String,

  /// Number of suggestions (default: 5, max: 20)
  #[validate(range(min = 1, max = 20, message = "Limit must be between 1 and 20"))]
  #[serde(default = "default_suggestions_limit")]
  pub limit: u32,
}

/// Search response wrapper
#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResponse {
  pub success: bool,
  pub data: MessageSearchResults,
  pub message: String,
}

/// Search suggestions response
#[derive(Debug, Serialize, ToSchema)]
pub struct SearchSuggestionsResponse {
  pub success: bool,
  pub suggestions: Vec<String>,
  pub message: String,
}

/// Error response
#[derive(Debug, Serialize, ToSchema)]
pub struct SearchErrorResponse {
  pub success: bool,
  pub error: String,
  pub code: String,
}

// ================================================================================================
// Default value functions
// ================================================================================================

fn default_limit() -> u32 {
  20
}
fn default_suggestions_limit() -> u32 {
  5
}
fn default_sort() -> String {
  "relevance".to_string()
}

// ================================================================================================
// Search Handlers
// ================================================================================================

/// Simple search messages in a specific chat (GET method)
///
/// Searches for messages within a specific chat using query parameters.
/// This is the simple version for frontend compatibility.
#[utoipa::path(
  get,
  path = "/api/chat/{chat_id}/messages/search",
  params(
    ("chat_id" = i64, Path, description = "Chat ID to search in"),
    SearchMessagesQuery
  ),
  responses(
    (status = 200, description = "Search results", body = SearchResponse),
    (status = 400, description = "Invalid search parameters", body = SearchErrorResponse),
    (status = 403, description = "Access denied to chat", body = SearchErrorResponse),
    (status = 500, description = "Search service error", body = SearchErrorResponse)
  ),
  tag = "search",
  summary = "Simple search messages in chat",
  description = "Simple search for messages within a specific chat using query parameters."
)]
#[instrument(skip(state), fields(chat_id = %chat_id, user_id = %user.id))]
pub async fn simple_search_messages_in_chat(
  Extension(state): Extension<AppState>,
  Path(chat_id): Path<i64>,
  Query(params): Query<SearchMessagesQuery>,
  Extension(user): Extension<AuthUser>,
) -> Result<Json<SearchResponse>, AppError> {
  // Handler responsibility: Parameter validation
  params
    .validate()
    .map_err(|e| AppError::InvalidInput(format!("Invalid search parameters: {}", e)))?;

  // Handler responsibility: Delegate to service layer
  // Get search service if available
  let search_service = match state.search_application_service() {
    Some(service) => service,
    None => {
      error!("Search service not available");
      return Err(AppError::ServiceUnavailable(
        "Search service is not configured".to_string(),
      ));
    }
  };

  let results = search_service
    .search_messages_in_chat(
      ChatId(chat_id),
      &params.q,
      UserId::from(user.id),
      params.limit,
      params.offset,
    )
    .await?;

  // Handler responsibility: Format response
  Ok(Json(SearchResponse {
    success: true,
    data: results,
    message: "Search completed successfully".to_string(),
  }))
}

/// Search messages in a specific chat (POST method)
///
/// Searches for messages within a specific chat that the user has access to.
/// Requires chat membership validation.
#[utoipa::path(
  post,
  path = "/api/chat/{chat_id}/messages/search",
  params(
    ("chat_id" = i64, Path, description = "Chat ID to search in")
  ),
  request_body = SearchMessagesQuery,
  responses(
    (status = 200, description = "Search results", body = SearchResponse),
    (status = 400, description = "Invalid search parameters", body = SearchErrorResponse),
    (status = 403, description = "Access denied to chat", body = SearchErrorResponse),
    (status = 500, description = "Search service error", body = SearchErrorResponse)
  ),
  tag = "search",
  summary = "Search messages in chat",
  description = "Search for messages within a specific chat. User must be a member of the chat."
)]
#[instrument(skip(state), fields(chat_id = %chat_id, user_id = %user.id))]
pub async fn search_messages_in_chat(
  Extension(state): Extension<AppState>,
  Path(chat_id): Path<i64>,
  Query(params): Query<SearchMessagesQuery>,
  Extension(user): Extension<AuthUser>,
) -> Result<Json<SearchResponse>, AppError> {
  // Validate input parameters
  params
    .validate()
    .map_err(|e| AppError::InvalidInput(format!("Invalid search parameters: {}", e)))?;

  info!(
    chat_id = %chat_id,
    user_id = %user.id,
    query = %params.q,
    "Starting chat message search"
  );

  // TODO: Verify user has access to the chat
  // This should check chat membership through chat service
  // For now, we'll proceed with the assumption that middleware handles this

  let search_service = match state.search_application_service() {
    Some(service) => service,
    None => {
      return Err(AppError::ServiceUnavailable(
        "Search service not available".to_string(),
      ));
    }
  };

  match search_service
    .search_messages_in_chat(
      ChatId(chat_id),
      &params.q,
      user.id,
      params.limit,
      params.offset,
    )
    .await
  {
    Ok(results) => {
      info!(
        chat_id = %chat_id,
        user_id = %user.id,
        query = %params.q,
        results_count = %results.hits.len(),
        total = %results.total,
        took_ms = %results.took_ms,
        "Chat search completed successfully"
      );

      Ok(Json(SearchResponse {
        success: true,
        data: results,
        message: "Search completed successfully".to_string(),
      }))
    }
    Err(e) => {
      error!(
        chat_id = %chat_id,
        user_id = %user.id,
        query = %params.q,
        error = %e,
        "Chat search failed"
      );

      Err(e)
    }
  }
}

/// Global message search across all accessible chats
///
/// Searches for messages across all chats that the user has access to within their workspace.
/// Results are filtered based on user permissions.
#[utoipa::path(
  post,
  path = "/api/search/messages",
  request_body = SearchMessagesQuery,
  responses(
    (status = 200, description = "Search results", body = SearchResponse),
    (status = 400, description = "Invalid search parameters", body = SearchErrorResponse),
    (status = 500, description = "Search service error", body = SearchErrorResponse)
  ),
  tag = "search",
  summary = "Global message search",
  description = "Search for messages across all accessible chats in the user's workspace."
)]
#[instrument(skip(state), fields(user_id = %user.id, workspace_id = %user.workspace_id))]
pub async fn global_search_messages(
  Extension(state): Extension<AppState>,
  Query(params): Query<SearchMessagesQuery>,
  Extension(user): Extension<AuthUser>,
) -> Result<Json<SearchResponse>, AppError> {
  // Validate input parameters
  params
    .validate()
    .map_err(|e| AppError::InvalidInput(format!("Invalid search parameters: {}", e)))?;

  info!(
    user_id = %user.id,
    workspace_id = %user.workspace_id,
    query = %params.q,
    "Starting global message search"
  );

  let search_service = match state.search_application_service() {
    Some(service) => service,
    None => {
      return Err(AppError::ServiceUnavailable(
        "Search service not available".to_string(),
      ));
    }
  };

  match search_service
    .global_search_messages(
      &params.q,
      user.id,
      user.workspace_id,
      params.limit,
      params.offset,
    )
    .await
  {
    Ok(results) => {
      info!(
        user_id = %user.id,
        workspace_id = %user.workspace_id,
        query = %params.q,
        results_count = %results.hits.len(),
        total = %results.total,
        took_ms = %results.took_ms,
        "Global search completed successfully"
      );

      Ok(Json(SearchResponse {
        success: true,
        data: results,
        message: "Search completed successfully".to_string(),
      }))
    }
    Err(e) => {
      error!(
        user_id = %user.id,
        workspace_id = %user.workspace_id,
        query = %params.q,
        error = %e,
        "Global search failed"
      );

      Err(e)
    }
  }
}

/// Get search suggestions
///
/// Provides search query suggestions based on partial input.
/// Can include popular searches, user history, and autocomplete suggestions.
#[utoipa::path(
  get,
  path = "/api/search/suggestions",
  params(SearchSuggestionsQuery),
  responses(
    (status = 200, description = "Search suggestions", body = SearchSuggestionsResponse),
    (status = 400, description = "Invalid parameters", body = SearchErrorResponse),
    (status = 500, description = "Service error", body = SearchErrorResponse)
  ),
  tag = "search",
  summary = "Get search suggestions",
  description = "Get search query suggestions based on partial input."
)]
#[instrument(skip(state), fields(user_id = %user.id))]
pub async fn get_search_suggestions(
  Extension(state): Extension<AppState>,
  Query(params): Query<SearchSuggestionsQuery>,
  Extension(user): Extension<AuthUser>,
) -> Result<Json<SearchSuggestionsResponse>, AppError> {
  // Validate input parameters
  params
    .validate()
    .map_err(|e| AppError::InvalidInput(format!("Invalid parameters: {}", e)))?;

  info!(
    user_id = %user.id,
    partial_query = %params.q,
    "Getting search suggestions"
  );

  let search_service = match state.search_application_service() {
    Some(service) => service,
    None => {
      return Err(AppError::ServiceUnavailable(
        "Search service not available".to_string(),
      ));
    }
  };

  match search_service
    .get_search_suggestions(&params.q, params.limit)
    .await
  {
    Ok(suggestions) => {
      info!(
        user_id = %user.id,
        partial_query = %params.q,
        suggestions_count = %suggestions.len(),
        "Search suggestions retrieved successfully"
      );

      Ok(Json(SearchSuggestionsResponse {
        success: true,
        suggestions,
        message: "Suggestions retrieved successfully".to_string(),
      }))
    }
    Err(e) => {
      error!(
        user_id = %user.id,
        partial_query = %params.q,
        error = %e,
        "Failed to get search suggestions"
      );

      Err(e)
    }
  }
}

/// Reindex chat messages (Admin endpoint)
///
/// Reindexes all messages in a specific chat. This is typically used for maintenance
/// or when search index becomes corrupted.
#[utoipa::path(
  post,
  path = "/api/admin/chat/{chat_id}/reindex",
  params(
    ("chat_id" = i64, Path, description = "Chat ID to reindex")
  ),
  responses(
    (status = 200, description = "Reindexing completed", body = serde_json::Value),
    (status = 403, description = "Admin access required", body = SearchErrorResponse),
    (status = 500, description = "Reindexing failed", body = SearchErrorResponse)
  ),
  tag = "admin",
  summary = "Reindex chat messages",
  description = "Reindex all messages in a chat. Admin access required."
)]
#[instrument(skip(state), fields(chat_id = %chat_id, user_id = %user.id))]
pub async fn reindex_chat_messages(
  Extension(state): Extension<AppState>,
  Path(chat_id): Path<i64>,
  Extension(user): Extension<AuthUser>,
) -> Result<Json<serde_json::Value>, AppError> {
  // TODO: Add admin permission check
  // For now, any authenticated user can trigger reindexing

  info!(
    chat_id = %chat_id,
    user_id = %user.id,
    "Starting chat reindexing"
  );

  let search_service = match state.search_application_service() {
    Some(service) => service,
    None => {
      return Err(AppError::ServiceUnavailable(
        "Search service not available".to_string(),
      ));
    }
  };

  match search_service.reindex_chat_messages(ChatId(chat_id)).await {
    Ok(indexed_count) => {
      info!(
        chat_id = %chat_id,
        user_id = %user.id,
        indexed_count = %indexed_count,
        "Chat reindexing completed"
      );

      Ok(Json(serde_json::json!({
        "success": true,
        "message": "Reindexing completed successfully",
        "indexed_count": indexed_count
      })))
    }
    Err(e) => {
      error!(
        chat_id = %chat_id,
        user_id = %user.id,
        error = %e,
        "Chat reindexing failed"
      );

      Err(e)
    }
  }
}

// ================================================================================================
// Error Handling
// ================================================================================================

impl From<AppError> for (StatusCode, Json<SearchErrorResponse>) {
  fn from(err: AppError) -> Self {
    let (status, code, message) = match &err {
      AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, "INVALID_INPUT", msg.as_str()),
      AppError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND", "Resource not found"),
      AppError::Unauthorized(_) => (StatusCode::FORBIDDEN, "FORBIDDEN", "Access denied"),
      AppError::ServiceUnavailable(msg) => (
        StatusCode::SERVICE_UNAVAILABLE,
        "SERVICE_UNAVAILABLE",
        msg.as_str(),
      ),
      _ => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "INTERNAL_ERROR",
        "Internal server error",
      ),
    };

    (
      status,
      Json(SearchErrorResponse {
        success: false,
        error: message.to_string(),
        code: code.to_string(),
      }),
    )
  }
}

// ================================================================================================
// Unit Tests
// ================================================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_search_messages_query_validation() {
    // Valid query
    let valid_query = SearchMessagesQuery {
      q: "test query".to_string(),
      limit: 20,
      offset: 0,
      sort: "relevance".to_string(),
    };
    assert!(valid_query.validate().is_ok());

    // Empty query
    let empty_query = SearchMessagesQuery {
      q: "".to_string(),
      limit: 20,
      offset: 0,
      sort: "relevance".to_string(),
    };
    assert!(empty_query.validate().is_err());

    // Query too long
    let long_query = SearchMessagesQuery {
      q: "a".repeat(501),
      limit: 20,
      offset: 0,
      sort: "relevance".to_string(),
    };
    assert!(long_query.validate().is_err());

    // Invalid limit
    let invalid_limit = SearchMessagesQuery {
      q: "test".to_string(),
      limit: 101,
      offset: 0,
      sort: "relevance".to_string(),
    };
    assert!(invalid_limit.validate().is_err());
  }

  #[test]
  fn test_search_suggestions_query_validation() {
    // Valid query
    let valid_query = SearchSuggestionsQuery {
      q: "test".to_string(),
      limit: 5,
    };
    assert!(valid_query.validate().is_ok());

    // Empty query
    let empty_query = SearchSuggestionsQuery {
      q: "".to_string(),
      limit: 5,
    };
    assert!(empty_query.validate().is_err());

    // Invalid limit
    let invalid_limit = SearchSuggestionsQuery {
      q: "test".to_string(),
      limit: 21,
    };
    assert!(invalid_limit.validate().is_err());
  }
}
