//! # Chat Handlers - Clean Architecture Implementation
//!
//! **Architecture Principles**: Clean Architecture + DDD  
//! **Handler Responsibilities**: HTTP coordination + Service delegation + Response construction (≤20 lines/function)
//! **Dependency Direction**: Handler → Application Service → Domain Service → Infrastructure
//!
//! ##  Correct Modern Architecture Implementation
//! -  Handlers only handle HTTP coordination, no business logic
//! -  Using concrete services for better performance
//! -  All business logic delegated to Service layer
//! -  Follow proper dependency chain

use crate::services::application::workers::chat::CreateChatInput;
use crate::{AppError, AppState};
use axum::{
  Extension,
  extract::{Path, Query},
  http::StatusCode,
  response::Json,
};
use fechatter_core::{AuthUser, CreateChat, UpdateChat};
use std::collections::HashMap;

// =============================================================================
// HANDLERS - HTTP Coordination Layer (Using Concrete Services)
// =============================================================================

/// Create Chat Handler
///
/// **Modern Architecture**: Handler → Concrete Application Service → Domain Service
pub async fn create_chat_handler(
  Extension(state): Extension<AppState>,
  Extension(user): Extension<AuthUser>,
  Json(create_chat): Json<CreateChat>,
) -> Result<Json<serde_json::Value>, AppError> {
  // 1. Use Concrete Application Service (better performance)
  let chat_service = state.application_services().chat_application_service();

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

  // 4. Return the created chat details
  Ok(Json(serde_json::json!({
      "success": true,
      "data": chat_detail,
      "message": "Chat created successfully"
  })))
}

/// Update Chat Handler
///
/// **Modern Architecture**: Handler → Concrete Application Service → Domain Service
pub async fn update_chat_handler(
  Extension(state): Extension<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
  Json(update_chat): Json<UpdateChat>,
) -> Result<Json<serde_json::Value>, AppError> {
  // 1. Use Concrete Application Service
  let _chat_service = state.application_services().chat_application_service();

  // 2. TODO: Implement update_chat method in ChatApplicationService
  // For now, return a placeholder response
  Ok(Json(serde_json::json!({
      "message": "update_chat method needs implementation",
      "chat_id": chat_id,
      "user_id": i64::from(user.id)
  })))
}

/// List Chats Handler
///
/// **Modern Architecture**: Handler → Concrete Application Service → Domain Service
pub async fn list_chats_handler(
  Extension(state): Extension<AppState>,
  Extension(user): Extension<AuthUser>,
  Query(_params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
  // 1. Use Concrete Application Service
  let chat_service = state.application_services().chat_application_service();

  // 2. Delegate to Application Service - 完整实现调用链
  let chats = chat_service.list_user_chats(i64::from(user.id)).await?;

  // 3. 构建响应数据 - Handler只负责响应格式化
  let response = serde_json::json!({
    "success": true,
    "data": chats,
    "total": chats.len(),
    "user_id": i64::from(user.id)
  });

  // 4. 记录操作并返回结果
  tracing::info!("Listed {} chats for user {}", chats.len(), user.id);
  Ok(Json(response))
}

/// Delete Chat Handler
///
/// **Modern Architecture**: Handler → Concrete Application Service → Domain Service
pub async fn delete_chat_handler(
  Extension(state): Extension<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
) -> Result<StatusCode, AppError> {
  // 1. Use Concrete Application Service
  let chat_service = state.application_services().chat_application_service();

  // 2. Delegate to Application Service - 完整实现调用链
  let deleted = chat_service
    .delete_chat(chat_id, i64::from(user.id))
    .await?;

  // 3. 检查删除结果
  if !deleted {
    return Err(AppError::Internal(
      "Failed to delete chat or chat not found".to_string(),
    ));
  }

  // 4. 记录操作并返回成功状态
  tracing::info!("Chat {} successfully deleted by user {}", chat_id, user.id);
  Ok(StatusCode::NO_CONTENT)
}

/// Get Chat Details Handler
///
/// **Modern Architecture**: Handler → Concrete Application Service → Domain Service
pub async fn get_chat_handler(
  Extension(state): Extension<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
  // 1. Use Concrete Application Service
  let chat_service = state.application_services().chat_application_service();

  // 2. Delegate to Application Service - 完整实现调用链
  let chat_detail = chat_service.get_chat(chat_id, i64::from(user.id)).await?;

  // 3. 检查聊天是否存在
  let chat = match chat_detail {
    Some(detail) => detail,
    None => {
      return Err(AppError::Internal(format!("Chat {} not found", chat_id)));
    }
  };

  // 4. 构建响应数据 - Handler只负责响应格式化
  let response = serde_json::json!({
    "success": true,
    "data": chat,
    "chat_id": chat_id,
    "user_id": i64::from(user.id)
  });

  // 5. 记录操作并返回结果
  tracing::info!("Chat {} details retrieved by user {}", chat_id, user.id);
  Ok(Json(response))
}

/// Add Chat Members Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
pub async fn add_chat_members_handler(
  Extension(state): Extension<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
  Json(member_ids): Json<Vec<i64>>,
) -> Result<Json<serde_json::Value>, AppError> {
  // 1. Use Application Service (correct architecture)
  let chat_service = state.application_services().chat_application_service();

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
  Extension(state): Extension<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
  Json(member_id): Json<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
  // 1. Use Application Service
  let chat_service = state.application_services().chat_application_service();

  // 2. Delegate to Application Service (remove_members expects a Vec)
  chat_service
    .remove_members(chat_id, i64::from(user.id), vec![member_id])
    .await?;

  // 3. Convert to HTTP response
  Ok(Json(serde_json::json!({
      "chat_id": chat_id,
      "removed_member": member_id,
      "removed_by": i64::from(user.id)
  })))
}
