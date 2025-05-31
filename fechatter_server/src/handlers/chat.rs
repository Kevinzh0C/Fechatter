//! # Chat Handlers - Clean Architecture Implementation
//!
//! **Architecture Principles**: Clean Architecture + DDD  
//! **Handler Responsibilities**: HTTP coordination + Service delegation + Response construction (≤20 lines/function)
//! **Dependency Direction**: Handler → Application Service → Domain Service → Infrastructure
//!
//! ## ✅ Correct Modern Architecture Implementation
//! - ✅ Handlers only handle HTTP coordination, no business logic
//! - ✅ Using existing Application Services (services/application)
//! - ✅ Using existing Adapter pattern
//! - ✅ All business logic delegated to Service layer
//! - ✅ Follow proper dependency chain

use crate::services::application::adapters::AppStateChatServiceAdapter;
use crate::services::application::chat_app_service::{ChatServiceTrait, CreateChatInput};
use crate::{AppError, AppState};
use axum::{
  Extension,
  extract::{Path, Query, State},
  http::StatusCode,
  response::Json,
};
use fechatter_core::{AuthUser, CreateChat, UpdateChat, UserId};
use std::collections::HashMap;

// =============================================================================
// HANDLERS - HTTP Coordination Layer (Using Modern Architecture)
// =============================================================================

/// Create Chat Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
pub async fn create_chat_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Json(create_chat): Json<CreateChat>,
) -> Result<Json<serde_json::Value>, AppError> {
  // 1. Use Application Service (correct architecture)
  let chat_service = AppStateChatServiceAdapter::new(state.clone());

  // 2. Convert to proper input format
  let initial_members = create_chat
    .members
    .map(|members| members.into_iter().map(|id| i64::from(id)).collect())
    .unwrap_or_default();

  let create_input = CreateChatInput {
    name: create_chat.name,
    chat_type: create_chat.chat_type,
    description: create_chat.description,
    created_by: i64::from(user.id),
    workspace_id: Some(i64::from(user.workspace_id)),
    initial_members,
    members: None, // Not used in the current implementation
  };

  // 3. Delegate to Application Service
  let chat_detail = chat_service.create_chat(create_input).await?;

  // 4. Convert to HTTP response
  Ok(Json(serde_json::json!({
      "id": chat_detail.id,
      "name": chat_detail.name,
      "chat_type": chat_detail.chat_type,
      "description": chat_detail.description,
      "created_by": chat_detail.created_by,
      "created_at": chat_detail.created_at,
      "workspace_id": chat_detail.workspace_id
  })))
}

/// Update Chat Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
pub async fn update_chat_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
  Json(update_chat): Json<UpdateChat>,
) -> Result<Json<serde_json::Value>, AppError> {
  // 1. Use Application Service (correct architecture)
  let chat_service = AppStateChatServiceAdapter::new(state.clone());

  // 2. Delegate to Application Service
  let updated_chat = chat_service
    .update_chat(
      fechatter_core::ChatId::new(chat_id),
      UserId::new(i64::from(user.id)),
      update_chat,
    )
    .await?;

  // 3. Convert to HTTP response
  Ok(Json(serde_json::json!({
      "id": updated_chat.id,
      "name": updated_chat.name,
      "chat_type": updated_chat.chat_type,
      "description": updated_chat.description,
      "updated_at": updated_chat.updated_at // Use actual updated_at field
  })))
}

/// List Chats Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
pub async fn list_chats_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Query(_params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
  // 1. Use Application Service (correct architecture)
  let chat_service = AppStateChatServiceAdapter::new(state.clone());

  // 2. Delegate to Application Service
  let chats = chat_service.list_user_chats(i64::from(user.id)).await?;

  // 3. Convert to HTTP response
  let chat_list: Vec<serde_json::Value> = chats
    .iter()
    .map(|chat| {
      serde_json::json!({
          "id": i64::from(chat.id),
          "name": chat.name,
          "chat_type": chat.chat_type,
          "is_creator": chat.is_creator,
          "last_message": chat.last_message
      })
    })
    .collect();

  Ok(Json(serde_json::json!({
      "chats": chat_list,
      "total": chat_list.len()
  })))
}

/// Delete Chat Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
pub async fn delete_chat_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
) -> Result<StatusCode, AppError> {
  // 1. Use Application Service (correct architecture)
  let chat_service = AppStateChatServiceAdapter::new(state.clone());

  // 2. Delegate to Application Service
  let deleted = chat_service
    .delete_chat(
      fechatter_core::ChatId::new(chat_id),
      UserId::new(i64::from(user.id)),
    )
    .await?;

  // 3. Return HTTP status code
  if deleted {
    Ok(StatusCode::NO_CONTENT)
  } else {
    Err(AppError::NotFound(vec![format!(
      "Chat with id {}",
      chat_id
    )]))
  }
}

/// Get Chat Details Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
pub async fn get_chat_handler(
  State(state): State<AppState>,
  Extension(_user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
  // 1. Use Application Service (correct architecture)
  let chat_service = AppStateChatServiceAdapter::new(state.clone());

  // 2. Delegate to Application Service
  let chat_detail = chat_service.get_chat(chat_id).await?;

  // 3. Convert to HTTP response
  match chat_detail {
    Some(chat) => Ok(Json(serde_json::json!({
        "id": chat.id,
        "name": chat.name,
        "chat_type": chat.chat_type,
        "description": chat.description,
        "created_by": chat.created_by,
        "created_at": chat.created_at,
        "workspace_id": chat.workspace_id
    }))),
    None => Err(AppError::NotFound(vec![format!(
      "Chat with id {}",
      chat_id
    )])),
  }
}

/// Add Chat Members Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
pub async fn add_chat_members_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
  Json(member_ids): Json<Vec<i64>>,
) -> Result<Json<serde_json::Value>, AppError> {
  // 1. Use Application Service (correct architecture)
  let chat_service = AppStateChatServiceAdapter::new(state.clone());

  // 2. Delegate to Application Service
  chat_service
    .add_members(chat_id, i64::from(user.id), member_ids.clone())
    .await?;

  // 3. Convert to HTTP response
  Ok(Json(serde_json::json!({
      "chat_id": chat_id,
      "added_members": member_ids,
      "added_by": i64::from(user.id)
  })))
}

/// Remove Chat Member Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
pub async fn remove_chat_member_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path((chat_id, member_id)): Path<(i64, i64)>,
) -> Result<StatusCode, AppError> {
  // 1. Use Application Service (correct architecture)
  let chat_service = AppStateChatServiceAdapter::new(state.clone());

  // 2. Delegate to Application Service
  chat_service
    .remove_members(chat_id, i64::from(user.id), vec![member_id])
    .await?;

  // 3. Return HTTP status code
  Ok(StatusCode::NO_CONTENT)
}
