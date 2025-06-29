//! # Search Handlers
//!
//! ## Responsibilities
//! - HTTP request/response handling for search operations
//! - Parameter validation and extraction
//! - Response formatting and error handling
//! - Integration with search application service

use axum::{
  extract::{Path, Query, Json},
  http::StatusCode,
  response::Json as ResponseJson,
  Extension,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;
use sqlx::Row;

use crate::{
  error::AppError,
  services::application::workers::search::{MessageSearchResults, SearchApplicationServiceTrait, SearchPage, SearchableMessage},
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
) -> Result<ResponseJson<SearchResponse>, AppError> {
  // Handler responsibility: Parameter validation
  params
    .validate()
    .map_err(|e| AppError::InvalidInput(format!("Invalid search parameters: {}", e)))?;

  info!(
    chat_id = %chat_id,
    user_id = %user.id,
    query = %params.q,
    "Starting simple chat search"
  );

  // CRITICAL SECURITY FIX: Verify user has access to chat BEFORE any search
  if !verify_chat_access(&state, chat_id, user.id.0).await? {
    error!(
      chat_id = %chat_id,
      user_id = %user.id,
      "User attempted to search chat without permission"
    );
    return Err(AppError::Unauthorized(
      "You do not have permission to search this chat".to_string(),
    ));
  }

  // Try search service first, fallback to secure database search if not available
  match state.search_application_service() {
    Some(search_service) => {
      // Use search service
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
            "Search service completed successfully"
          );

          Ok(ResponseJson(SearchResponse {
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
            "Search service failed, falling back to secure database search"
          );

          // Fallback to secure database search with permission validation
          secure_fallback_database_search(&state, chat_id, &params, user.id.0).await
        }
      }
    }
    None => {
      info!(
        chat_id = %chat_id,
        user_id = %user.id,
        query = %params.q,
        "Search service not available, using secure database search"
      );

      // Fallback to secure database search with permission validation
      secure_fallback_database_search(&state, chat_id, &params, user.id.0).await
    }
  }
}

/// SECURITY-ENHANCED: Verify user has access to chat before allowing search
async fn verify_chat_access(state: &AppState, chat_id: i64, user_id: i64) -> Result<bool, AppError> {
  use sqlx::Row;
  
  let sql = r#"
    SELECT EXISTS(
      SELECT 1 FROM chat_members cm
      WHERE cm.chat_id = $1 AND cm.user_id = $2
      UNION
      SELECT 1 FROM chats c
      JOIN users u ON u.id = $2 AND u.workspace_id = c.workspace_id
      WHERE c.id = $1
    ) as has_access
  "#;

  let row = sqlx::query(sql)
    .bind(chat_id)
    .bind(user_id)
    .fetch_one(state.pool().as_ref())
    .await
    .map_err(|e| AppError::Internal(format!("Chat access verification failed: {}", e)))?;

  let has_access: bool = row.get("has_access");
  Ok(has_access)
}

/// SECURITY-ENHANCED: Secure database search with comprehensive permission validation
async fn secure_fallback_database_search(
  state: &AppState,
  chat_id: i64,
  params: &SearchMessagesQuery,
  user_id: i64,
) -> Result<ResponseJson<SearchResponse>, AppError> {
  let start_time = std::time::Instant::now();

  // SECURITY: Double-check chat access with comprehensive permission query
  let access_check_sql = r#"
    SELECT 
      c.id as chat_id,
      c.name as chat_name,
      c.chat_type,
      c.workspace_id,
      CASE 
        WHEN cm.user_id IS NOT NULL THEN 'member'
        WHEN u.workspace_id = c.workspace_id THEN 'workspace_user'
        ELSE NULL
      END as access_level
    FROM chats c
    LEFT JOIN chat_members cm ON c.id = cm.chat_id AND cm.user_id = $2
    LEFT JOIN users u ON u.id = $2 AND u.workspace_id = c.workspace_id
    WHERE c.id = $1 AND (cm.user_id IS NOT NULL OR u.workspace_id = c.workspace_id)
  "#;

  let chat_access_row = sqlx::query(access_check_sql)
    .bind(chat_id)
    .bind(user_id)
    .fetch_optional(state.pool().as_ref())
    .await
    .map_err(|e| AppError::Internal(format!("Secure chat access verification failed: {}", e)))?;

  let (chat_name, chat_type, workspace_id) = match chat_access_row {
    Some(row) => {
      let access_level: Option<String> = row.get("access_level");
      if access_level.is_none() {
        error!(
          chat_id = %chat_id,
          user_id = %user_id,
          "User has no access level to chat - security violation"
        );
        return Err(AppError::Unauthorized(
          "Access denied: You do not have permission to search this chat".to_string(),
        ));
      }

      info!(
        chat_id = %chat_id,
        user_id = %user_id,
        access_level = ?access_level,
        "Chat access verified for secure database search"
      );

      (
        row.get::<String, _>("chat_name"),
        row.get::<String, _>("chat_type"),
        row.get::<i64, _>("workspace_id"),
      )
    }
    None => {
      error!(
        chat_id = %chat_id,
        user_id = %user_id,
        "Chat not found or access denied - security violation"
      );
      return Err(AppError::Unauthorized(
        "Access denied: Chat not found or you do not have permission".to_string(),
      ));
    }
  };

  // SECURITY-ENHANCED: Search query with explicit permission filtering
  let query = format!("%{}%", params.q);
  
  let search_sql = r#"
    SELECT 
      m.id,
      m.chat_id,
      m.sender_id,
      m.content,
      m.created_at,
      m.files,
      u.fullname as sender_name
    FROM messages m
    LEFT JOIN users u ON m.sender_id = u.id
    WHERE m.chat_id = $1 
      AND m.content ILIKE $2
      AND EXISTS(
        SELECT 1 FROM chat_members cm 
        WHERE cm.chat_id = m.chat_id AND cm.user_id = $3
      )
    ORDER BY m.created_at DESC
    LIMIT $4 OFFSET $5
  "#;

  let rows = sqlx::query(search_sql)
    .bind(chat_id)
    .bind(&query)
    .bind(user_id) // SECURITY: Explicit user permission check
    .bind(params.limit as i64)
    .bind(params.offset as i64)
    .fetch_all(state.pool().as_ref())
    .await
    .map_err(|e| AppError::Internal(format!("Secure database search failed: {}", e)))?;

  // SECURITY-ENHANCED: Count query with permission filtering
  let count_sql = r#"
    SELECT COUNT(*) as total
    FROM messages m
    WHERE m.chat_id = $1 
      AND m.content ILIKE $2
      AND EXISTS(
        SELECT 1 FROM chat_members cm 
        WHERE cm.chat_id = m.chat_id AND cm.user_id = $3
      )
  "#;

  let total_row = sqlx::query(count_sql)
    .bind(chat_id)
    .bind(&query)
    .bind(user_id) // SECURITY: Explicit user permission check
    .fetch_one(state.pool().as_ref())
    .await
    .map_err(|e| AppError::Internal(format!("Secure count query failed: {}", e)))?;

  let total: i64 = total_row.get("total");

  // Convert rows to SearchableMessage format with consistent structure
  let hits: Vec<SearchableMessage> = rows
    .into_iter()
    .map(|row| SearchableMessage {
      id: row.get::<i64, _>("id"),
      chat_id: row.get::<i64, _>("chat_id"),
      sender_id: row.get::<i64, _>("sender_id"),
      sender_name: row.get::<Option<String>, _>("sender_name").unwrap_or_else(|| "Unknown".to_string()),
      content: row.get::<String, _>("content"),
      files: row.get::<Option<Vec<String>>, _>("files"),
      created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
      workspace_id,
      chat_name: chat_name.clone(),
      chat_type: chat_type.clone(),
    })
    .collect();

  let elapsed_ms = start_time.elapsed().as_millis() as u64;

  // CONSISTENCY FIX: Ensure response format matches search service response
  let results = MessageSearchResults {
    hits,
    total: total as u64,
    took_ms: elapsed_ms,
    query: params.q.clone(),
    page: SearchPage {
      offset: params.offset,
      limit: params.limit,
      has_more: (params.offset as u64 + params.limit as u64) < total as u64,
    },
  };

  info!(
    chat_id = %chat_id,
    user_id = %user_id,
    query = %params.q,
    results_count = %results.hits.len(),
    total = %results.total,
    took_ms = %results.took_ms,
    "Secure database fallback search completed with permission validation"
  );

  Ok(ResponseJson(SearchResponse {
    success: true,
    data: results,
    message: "Search completed using secure database fallback".to_string(),
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
) -> Result<ResponseJson<SearchResponse>, AppError> {
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

  // CRITICAL SECURITY FIX: Verify user has access to chat BEFORE any search
  if !verify_chat_access(&state, chat_id, user.id.0).await? {
    error!(
      chat_id = %chat_id,
      user_id = %user.id,
      "User attempted to search chat without permission via POST method"
    );
    return Err(AppError::Unauthorized(
      "You do not have permission to search this chat".to_string(),
    ));
  }

  let search_service = match state.search_application_service() {
    Some(service) => service,
    None => {
      // Use secure database fallback when search service unavailable
      return secure_fallback_database_search(&state, chat_id, &params, user.id.0).await;
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

      Ok(ResponseJson(SearchResponse {
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
        "Chat search failed, falling back to secure database search"
      );

      // Fallback to secure database search
      secure_fallback_database_search(&state, chat_id, &params, user.id.0).await
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
  params(SearchMessagesQuery),
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
) -> Result<ResponseJson<SearchResponse>, AppError> {
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

      Ok(ResponseJson(SearchResponse {
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
) -> Result<ResponseJson<SearchSuggestionsResponse>, AppError> {
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

      Ok(ResponseJson(SearchSuggestionsResponse {
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
) -> Result<ResponseJson<serde_json::Value>, AppError> {
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

      Ok(ResponseJson(serde_json::json!({
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

impl From<AppError> for (StatusCode, ResponseJson<SearchErrorResponse>) {
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
      ResponseJson(SearchErrorResponse {
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
