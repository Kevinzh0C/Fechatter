//! # Messages Handlers - Clean Architecture Implementation ✅
//!
//! **Architecture Principles**: Clean Architecture + DDD
//! **Handler Responsibilities**: HTTP Coordination + Service Delegation + Response Construction (≤20 lines/function)
//! **Dependency Direction**: Handler → Application Service → Domain Service → Infrastructure
//!
//! ## Correct Modern Architecture Implementation
//! - Handlers only handle HTTP coordination, no business logic
//! - Using existing Domain Layer (domains/messaging)
//! - Using existing Application Services (services/application)
//! - Using existing Adapter pattern
//! - Completely removed SQL code, delegated to Service
//! - All business logic moved to appropriate layers
//!
//! ## Architectural Layer Responsibilities
//! ```
//! Handler:           HTTP coordination + DTO conversion + Service delegation
//! Application:       Use case orchestration + Event publishing + Cache invalidation
//! Domain:            Business logic + Rule validation + Domain events
//! Repository:        Data access + SQL queries + Persistence
//! ```

use axum::{
  extract::{Extension, Path, Query, State},
  http::StatusCode,
  response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::services::application::message_app_service::{
  AppStateMessageServiceAdapter, MessageApplicationServiceTrait, MessageView,
};
use crate::{AppError, AppState};
use fechatter_core::{AuthUser, ChatId, CreateMessage, ListMessages, UserId};

// =============================================================================
// REQUEST/RESPONSE DTOs - Data Transfer Objects
// =============================================================================

/// Send Message Request DTO
#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
  pub content: String,
  #[serde(default)]
  pub files: Vec<String>,
}

/// Message List Query DTO
#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
  #[serde(default = "default_limit")]
  pub limit: i64,
  pub before: Option<i64>,
}

fn default_limit() -> i64 {
  50
}

/// Edit Message Request DTO
#[derive(Debug, Deserialize)]
pub struct EditMessageRequest {
  pub content: String,
}

/// Unified API Response Wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
  pub data: T,
  pub status: String,
}

impl<T> ApiResponse<T> {
  pub fn success(data: T) -> Self {
    Self {
      data,
      status: "success".to_string(),
    }
  }
}

/// Message Response DTO
#[derive(Debug, Serialize)]
pub struct MessageResponse {
  pub id: i64,
  pub chat_id: i64,
  pub sender_id: i64,
  pub content: String,
  pub files: Option<Vec<String>>,
  pub created_at: chrono::DateTime<chrono::Utc>,
}

// =============================================================================
// DTO CONVERSIONS - Data Conversion Layer
// =============================================================================

// DTO conversions are manually handled in handlers, removed From trait implementation to avoid type errors
// impl From<Message> for MessageResponse {
//   fn from(msg: Message) -> Self {
//     Self {
//       id: i64::from(msg.id),
//       chat_id: i64::from(msg.chat_id),
//       sender_id: i64::from(msg.sender_id),
//       content: msg.content,
//       files: msg.files,
//       created_at: msg.created_at,
//     }
//   }
// }

impl From<SendMessageRequest> for CreateMessage {
  fn from(request: SendMessageRequest) -> Self {
    Self {
      content: request.content,
      files: request.files,
      idempotency_key: uuid::Uuid::now_v7(),
    }
  }
}

impl From<ListMessagesQuery> for ListMessages {
  fn from(query: ListMessagesQuery) -> Self {
    Self {
      limit: query.limit,
      last_id: query.before,
    }
  }
}

// =============================================================================
// HANDLERS - HTTP Coordination Layer (Using Modern Architecture)
// =============================================================================

/// Send Message Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
#[instrument(skip(state), fields(chat_id = %chat_id, user_id = %user.id))]
pub async fn send_message_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
  Json(request): Json<SendMessageRequest>,
) -> Result<Json<ApiResponse<MessageResponse>>, AppError> {
  // 1. Convert request DTO
  let create_message = CreateMessage::from(request);

  // 2. Create Service with proper Result handling
  let message_service = AppStateMessageServiceAdapter::new(&state)?;

  // 3. Call service method
  let message_view = message_service
    .send_message(UserId::from(user.id), ChatId::from(chat_id), create_message)
    .await?;

  // 4. Convert to HTTP response
  let response = MessageResponse {
    id: message_view.id,
    chat_id: message_view.chat_id,
    sender_id: message_view.sender_id,
    content: message_view.content,
    files: message_view.files,
    created_at: message_view.created_at,
  };

  Ok(Json(ApiResponse::success(response)))
}

/// List Messages Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
#[instrument(skip(state), fields(chat_id = %chat_id, user_id = %user.id))]
pub async fn list_messages_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
  Query(query): Query<ListMessagesQuery>,
) -> Result<Json<ApiResponse<Vec<MessageResponse>>>, AppError> {
  // 1. Convert query DTO
  let list_query = ListMessages::from(query);

  // 2. Create Service with proper Result handling
  let message_service = AppStateMessageServiceAdapter::new(&state)?;

  // 3. Call service method
  let message_views = message_service
    .list_messages(UserId::from(user.id), ChatId::from(chat_id), list_query)
    .await?;

  // 4. Convert to HTTP response
  let responses: Vec<MessageResponse> = message_views
    .into_iter()
    .map(|view| MessageResponse {
      id: view.id,
      chat_id: view.chat_id,
      sender_id: view.sender_id,
      content: view.content,
      files: view.files,
      created_at: view.created_at,
    })
    .collect();

  Ok(Json(ApiResponse::success(responses)))
}

/// Edit Message Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
#[instrument(skip(state), fields(message_id = %message_id, user_id = %user.id))]
pub async fn edit_message_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(message_id): Path<i64>,
  Json(request): Json<EditMessageRequest>,
) -> Result<Json<ApiResponse<MessageResponse>>, AppError> {
  // 1. Create Service with proper Result handling
  let message_service = AppStateMessageServiceAdapter::new(&state)?;

  // 2. Call service method
  let message_view = message_service
    .edit_message(
      fechatter_core::MessageId(message_id),
      UserId::from(user.id),
      request.content,
    )
    .await?;

  // 3. Convert to HTTP response
  let response = MessageResponse {
    id: message_view.id,
    chat_id: message_view.chat_id,
    sender_id: message_view.sender_id,
    content: message_view.content,
    files: message_view.files,
    created_at: message_view.created_at,
  };

  Ok(Json(ApiResponse::success(response)))
}

/// Delete Message Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
#[instrument(skip(state), fields(message_id = %message_id, user_id = %user.id))]
pub async fn delete_message_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(message_id): Path<i64>,
) -> Result<StatusCode, AppError> {
  // 1. Create Service with proper Result handling
  let message_service = AppStateMessageServiceAdapter::new(&state)?;

  // 2. Call service method
  message_service
    .delete_message(fechatter_core::MessageId(message_id), UserId::from(user.id))
    .await?;

  // 3. Return HTTP status code
  Ok(StatusCode::NO_CONTENT)
}
