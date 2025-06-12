//! # Chat Member Handlers - Clean Architecture Implementation
//!
//! **Architecture Principles**: Clean Architecture + DDD  
//! **Handler Responsibilities**: HTTP coordination + Service delegation + Response construction (≤20 lines/function)
//! **Dependency Direction**: Handler → Application Service → Domain Service → Infrastructure
//!
//! ## Correct Modern Architecture Implementation
//! - Handlers only handle HTTP coordination, no business logic
//! - Using concrete services for better performance and no trait overhead
//! - All business logic delegated to Service layer
//! - Simple response construction, no complex DTO mapping
//! - Follow proper dependency chain

use crate::{AppError, AppState};
use axum::{
  Extension, Json,
  extract::{Path, State},
  http::StatusCode,
  response::IntoResponse,
};
use fechatter_core::models::AuthUser;
use serde::Serialize;
use tracing::info;
use utoipa::ToSchema;

// =============================================================================
// RESPONSE DTOs - Simple Response Types
// =============================================================================

#[derive(Debug, Serialize, ToSchema, serde::Deserialize)]
pub struct ChatMemberDto {
  pub user_id: i64,
  pub chat_id: i64,
  pub username: String,
  pub role: String,
  #[schema(value_type = String, format = DateTime)]
  pub joined_at: chrono::DateTime<chrono::Utc>,
  pub is_online: bool,
  pub is_creator: bool,
}

#[derive(Debug, Serialize, ToSchema, serde::Deserialize)]
pub struct ChatMemberOperationResponse {
  pub success: bool,
  pub message: String,
  pub affected_count: i32,
}

impl ChatMemberOperationResponse {
  pub fn success(message: String, count: i32) -> Self {
    Self {
      success: true,
      message,
      affected_count: count,
    }
  }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TransferOwnershipResponse {
  pub success: bool,
  pub message: String,
  pub old_owner_id: i64,
  pub new_owner_id: i64,
  #[schema(value_type = String, format = DateTime)]
  pub timestamp: chrono::DateTime<chrono::Utc>,
}

// =============================================================================
// HANDLERS - HTTP Coordination Layer (Using Modern Architecture)
// =============================================================================

/// List Chat Members Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
#[utoipa::path(
    get,
    path = "/api/chats/{chat_id}/members",
    params(("chat_id" = i64, Path, description = "Chat ID")),
    security(("access_token" = [])),
    responses(
        (status = 200, description = "Chat members retrieved successfully", body = Vec<ChatMemberDto>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Permission denied"),
        (status = 404, description = "Chat not found")
    ),
    tag = "chat members"
)]
pub(crate) async fn list_chat_members_handler(
  Extension(state): Extension<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
) -> Result<Json<Vec<ChatMemberDto>>, AppError> {
  info!("User {} listing members for chat {}", user.id, chat_id);

  // 1. Use Concrete Application Service (better performance)
  let _chat_service = state.application_services().chat_application_service();

  // 2. TODO: Implement list_chat_members method in ChatApplicationService
  // For now, return empty list as placeholder
  let member_dtos: Vec<ChatMemberDto> = vec![];

  Ok(Json(member_dtos))
}

/// Add Chat Members Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
#[utoipa::path(
    post,
    path = "/api/chats/{chat_id}/members",
    params(("chat_id" = i64, Path, description = "Chat ID")),
    request_body = Vec<i64>,
    security(("access_token" = [])),
    responses(
        (status = 201, description = "Members added successfully", body = ChatMemberOperationResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Permission denied")
    ),
    tag = "chat members"
)]
pub(crate) async fn add_chat_members_handler(
  Extension(state): Extension<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
  Json(member_ids): Json<Vec<i64>>,
) -> Result<impl IntoResponse, AppError> {
  info!(
    "User {} adding members {:?} to chat {}",
    user.id, member_ids, chat_id
  );

  // 1. Use Concrete Application Service (better performance)
  let chat_service = state.application_services().chat_application_service();

  // 2. Delegate to Concrete Service (already implemented)
  chat_service
    .add_members(chat_id, i64::from(user.id), member_ids.clone())
    .await?;

  // 3. Simple response construction
  let response = ChatMemberOperationResponse::success(
    format!("Added {} members to chat {}", member_ids.len(), chat_id),
    member_ids.len() as i32,
  );

  Ok((StatusCode::CREATED, Json(response)))
}

/// Remove Chat Members Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
#[utoipa::path(
    delete,
    path = "/api/chats/{chat_id}/members",
    params(("chat_id" = i64, Path, description = "Chat ID")),
    request_body = Vec<i64>,
    security(("access_token" = [])),
    responses(
        (status = 204, description = "Members removed successfully"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Permission denied")
    ),
    tag = "chat members"
)]
pub(crate) async fn remove_chat_members_handler(
  Extension(state): Extension<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
  Json(member_ids): Json<Vec<i64>>,
) -> Result<Json<ChatMemberOperationResponse>, AppError> {
  info!(
    "User {} removing members {:?} from chat {}",
    user.id, member_ids, chat_id
  );

  // 1. Use Concrete Application Service (better performance)
  let chat_service = state.application_services().chat_application_service();

  // 2. Delegate to Concrete Service (already implemented)
  chat_service
    .remove_members(chat_id, i64::from(user.id), member_ids)
    .await?;

  // 3. Simple response construction
  let response = ChatMemberOperationResponse::success(
    "Members removed successfully".to_string(),
    1, // TODO: Return actual count from service
  );

  Ok(Json(response))
}

/// Transfer Chat Ownership Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
#[utoipa::path(
    post,
    path = "/api/chat/{chat_id}/transfer/{target_user_id}",
    params(
        ("chat_id" = i64, Path, description = "Chat ID"),
        ("target_user_id" = i64, Path, description = "Target user to transfer ownership to")
    ),
    security(("access_token" = [])),
    responses(
        (status = 200, description = "Ownership transferred successfully", body = TransferOwnershipResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Permission denied"),
        (status = 404, description = "Chat or user not found")
    ),
    tag = "chat members"
)]
pub(crate) async fn transfer_chat_ownership_handler(
  Extension(state): Extension<AppState>,
  Extension(user): Extension<AuthUser>,
  Path((chat_id, target_user_id)): Path<(i64, i64)>,
) -> Result<Json<TransferOwnershipResponse>, AppError> {
  info!(
    "User {} transferring ownership of chat {} to user {}",
    user.id, chat_id, target_user_id
  );

  // 1. Use Application Service - 关注点分离
  let chat_service = state.application_services().chat_application_service();

  // 2. Delegate to service - Handler只负责协调
  let transferred = chat_service
    .transfer_chat_ownership(chat_id, i64::from(user.id), target_user_id)
    .await?;

  if !transferred {
    return Err(AppError::Internal(
      "Failed to transfer ownership".to_string(),
    ));
  }

  // 3. Construct response - Handler只负责响应构建
  let response = TransferOwnershipResponse {
    success: transferred,
    message: format!(
      "Chat ownership transferred successfully from user {} to user {}",
      user.id, target_user_id
    ),
    old_owner_id: i64::from(user.id),
    new_owner_id: target_user_id,
    timestamp: chrono::Utc::now(),
  };

  Ok(Json(response))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    assert_chat_member_count, assert_handler_error, assert_handler_success, auth_user,
    create_new_test_chat, setup_test_users,
  };
  use anyhow::Result;
  use axum::{Json, extract::Path, http::StatusCode};
  use fechatter_core::models::ChatType;
  use sqlx::Row;
  use uuid::Uuid;

  #[tokio::test]
  async fn list_chat_members_handler_should_work() -> Result<()> {
    let (state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let auth_user = auth_user!(user1);

    let member_ids: Vec<fechatter_core::UserId> = vec![user2.id, user3.id];
    let chat = state
      .create_new_chat(
        fechatter_core::ChatType::Group,
        Some("List Member Test Chat".to_string()),
        None,
        user1.id,
        member_ids,
      )
      .await
      .expect("Failed to create test chat");

    let chat_id_i64: i64 = chat.id.into();
    assert_chat_member_count!(state, auth_user, chat_id_i64, 3);

    Ok(())
  }

  #[tokio::test]
  async fn add_chat_members_batch_handler_should_work() -> Result<()> {
    let (state, users) = setup_test_users!(4).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let user4 = &users[3];
    let creator_auth = auth_user!(user1);

    let member_ids: Vec<fechatter_core::UserId> = vec![user2.id, user3.id];
    let chat = state
      .create_new_chat(
        fechatter_core::ChatType::Group,
        Some("Add Member Batch Test".to_string()),
        None,
        user1.id,
        member_ids,
      )
      .await
      .expect("Failed to create test chat");

    let user4_id_i64: i64 = user4.id.into();
    let members_to_add: Vec<i64> = vec![user4_id_i64];
    let chat_id_i64: i64 = chat.id.into();

    let _added_members = assert_handler_success!(
      add_chat_members_handler(
        Extension(state.clone()),
        Extension(creator_auth),
        Path(chat_id_i64),
        Json(members_to_add.clone())
      ),
      StatusCode::CREATED,
      ChatMemberOperationResponse
    );

    assert_chat_member_count!(state, auth_user!(user1), chat_id_i64, 4);
    Ok(())
  }

  #[tokio::test]
  async fn remove_chat_member_handler_should_work() -> Result<()> {
    let (state, users) = setup_test_users!(4).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let user4 = &users[3];
    let creator_auth = auth_user!(user1);

    let member_ids: Vec<fechatter_core::UserId> = vec![user2.id, user3.id, user4.id];
    let chat = state
      .create_new_chat(
        fechatter_core::ChatType::Group,
        Some("Remove Member Test".to_string()),
        None,
        user1.id,
        member_ids,
      )
      .await
      .expect("Failed to create test chat");

    let user3_id_i64: i64 = user3.id.into();
    let user4_id_i64: i64 = user4.id.into();
    let chat_id_i64: i64 = chat.id.into();
    let members_to_remove: Vec<i64> = vec![user3_id_i64, user4_id_i64];

    assert_handler_success!(
      remove_chat_members_handler(
        Extension(state.clone()),
        Extension(creator_auth),
        Path(chat_id_i64),
        Json(members_to_remove.clone())
      ),
      StatusCode::NO_CONTENT
    );

    assert_chat_member_count!(state, auth_user!(user1), chat_id_i64, 2);
    Ok(())
  }

  #[tokio::test]
  async fn transfer_chat_ownership_handler_should_work() -> Result<()> {
    let (state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let creator_auth = auth_user!(user1);

    let member_ids: Vec<fechatter_core::UserId> = vec![user2.id, user3.id];
    let chat = state
      .create_new_chat(
        fechatter_core::ChatType::Group,
        Some("Transfer Owner Test".to_string()),
        None,
        user1.id,
        member_ids,
      )
      .await
      .expect("Failed to create test chat");

    let chat_id_i64: i64 = chat.id.into();
    let user2_id_i64: i64 = user2.id.into();
    let user1_id_i64: i64 = user1.id.into();

    let response_msg: ChatMemberOperationResponse = assert_handler_success!(
      transfer_chat_ownership_handler(
        Extension(state.clone()),
        Extension(creator_auth),
        Path((chat_id_i64, user2_id_i64))
      ),
      StatusCode::OK,
      ChatMemberOperationResponse
    );

    assert_eq!(
      response_msg.message,
      "Ownership of chat 1 transferred to user 2"
    );
    assert_eq!(response_msg.affected_count, 1);

    let query = "SELECT created_by FROM chats WHERE id = $1";
    let updated_chat_info = sqlx::query(query)
      .bind(chat_id_i64)
      .fetch_one(state.pool())
      .await?;
    let created_by: i64 = updated_chat_info
      .try_get("created_by")
      .map_err(|e| AppError::SqlxError(e))?;
    assert_eq!(created_by, user2_id_i64);

    Ok(())
  }
}
