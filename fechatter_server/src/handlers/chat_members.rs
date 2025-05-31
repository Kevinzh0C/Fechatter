//! # Chat Member Handlers - Clean Architecture Implementation
//!
//! **Architecture Principles**: Clean Architecture + DDD  
//! **Handler Responsibilities**: HTTP coordination + Service delegation + Response construction (≤20 lines/function)
//! **Dependency Direction**: Handler → Application Service → Domain Service → Infrastructure
//!
//! ## ✅ Correct Modern Architecture Implementation
//! - ✅ Handlers only handle HTTP coordination, no business logic
//! - ✅ Using existing AppStateChatServiceAdapter (services/application/adapters)
//! - ✅ All business logic delegated to Service layer
//! - ✅ Simple response construction, no complex DTO mapping
//! - ✅ Follow proper dependency chain

use crate::services::application::adapters::AppStateChatServiceAdapter;
use crate::services::application::chat_app_service::ChatServiceTrait;
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

#[derive(Debug, Serialize, ToSchema)]
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

#[derive(Debug, Serialize, ToSchema)]
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
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
) -> Result<Json<Vec<ChatMemberDto>>, AppError> {
  info!("User {} listing members for chat {}", user.id, chat_id);

  // 1. Use Application Service (correct architecture)
  let chat_service = AppStateChatServiceAdapter::new(state.clone());

  // 2. Delegate to Application Service
  let members = chat_service.list_chat_members(chat_id).await?;

  // 3. Simple response construction
  let member_dtos: Vec<ChatMemberDto> = members
    .iter()
    .map(|member| ChatMemberDto {
      user_id: i64::from(member.user_id),
      chat_id: i64::from(member.chat_id),
      username: format!("User {}", i64::from(member.user_id)),
      role: "member".to_string(),
      joined_at: member.joined_at,
      is_online: false,
      is_creator: false,
    })
    .collect();

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
pub(crate) async fn add_chat_members_batch_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
  Json(member_ids): Json<Vec<i64>>,
) -> Result<(StatusCode, Json<ChatMemberOperationResponse>), AppError> {
  info!(
    "User {} adding {} members to chat {}",
    user.id,
    member_ids.len(),
    chat_id
  );

  // 1. Use Application Service (correct architecture)
  let chat_service = AppStateChatServiceAdapter::new(state.clone());

  // 2. Delegate to Application Service
  chat_service
    .add_members(chat_id, i64::from(user.id), member_ids.clone())
    .await?;

  // 3. Build simple response
  let response = ChatMemberOperationResponse::success(
    format!("Successfully added {} members", member_ids.len()),
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
pub(crate) async fn remove_chat_member_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
  Json(member_ids): Json<Vec<i64>>,
) -> Result<StatusCode, AppError> {
  info!(
    "User {} removing {} members from chat {}",
    user.id,
    member_ids.len(),
    chat_id
  );

  // 1. Use Application Service (correct architecture)
  let chat_service = AppStateChatServiceAdapter::new(state.clone());

  // 2. Delegate to Application Service
  chat_service
    .remove_members(chat_id, i64::from(user.id), member_ids)
    .await?;

  // 3. Return success status
  Ok(StatusCode::NO_CONTENT)
}

/// Transfer Chat Ownership Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
#[utoipa::path(
    post,
    path = "/api/chats/{chat_id}/transfer/{target_user_id}",
    params(
        ("chat_id" = i64, Path, description = "Chat ID"),
        ("target_user_id" = i64, Path, description = "Target user ID")
    ),
    security(("access_token" = [])),
    responses(
        (status = 200, description = "Ownership transferred successfully", body = TransferOwnershipResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Permission denied")
    ),
    tag = "chat members"
)]
pub(crate) async fn transfer_chat_ownership_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path((chat_id, target_user_id)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, AppError> {
  // 1. Use Application Service (correct architecture)
  let chat_service = AppStateChatServiceAdapter::new(state.clone());

  // 2. Delegate to Application Service
  let result = chat_service
    .transfer_chat_ownership(chat_id, i64::from(user.id), target_user_id)
    .await?;

  // 3. Build simple response
  if result {
    let response = TransferOwnershipResponse {
      success: true,
      message: "Chat ownership transferred successfully".to_string(),
      old_owner_id: i64::from(user.id),
      new_owner_id: target_user_id,
      timestamp: chrono::Utc::now(),
    };
    Ok((StatusCode::OK, Json(response)))
  } else {
    Err(AppError::ChatValidationError(format!(
      "Failed to transfer ownership of chat {}",
      chat_id
    )))
  }
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
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let auth_user = auth_user!(user1);

    let unique_chat_name = format!("List Member Test Chat {}", Uuid::now_v7());
    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      &unique_chat_name
    )
    .await;

    let chat_id_i64: i64 = chat.id.into();
    assert_chat_member_count!(state, auth_user, chat_id_i64, 3);

    Ok(())
  }

  #[tokio::test]
  async fn add_chat_members_batch_handler_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(4).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let user4 = &users[3];
    let creator_auth = auth_user!(user1);

    let unique_chat_name = format!("Add Member Batch Test {}", Uuid::now_v7());
    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      &unique_chat_name
    )
    .await;

    let user4_id_i64: i64 = user4.id.into();
    let members_to_add: Vec<i64> = vec![user4_id_i64];
    let chat_id_i64: i64 = chat.id.into();

    let _added_members = assert_handler_success!(
      add_chat_members_batch_handler(
        State(state.clone()),
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
    let (_tdb, state, users) = setup_test_users!(4).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let user4 = &users[3];
    let creator_auth = auth_user!(user1);

    let unique_chat_name = format!("Remove Member Test {}", Uuid::now_v7());
    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3, user4],
      &unique_chat_name
    )
    .await;

    let user3_id_i64: i64 = user3.id.into();
    let user4_id_i64: i64 = user4.id.into();
    let chat_id_i64: i64 = chat.id.into();
    let members_to_remove: Vec<i64> = vec![user3_id_i64, user4_id_i64];

    assert_handler_success!(
      remove_chat_member_handler(
        State(state.clone()),
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
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let creator_auth = auth_user!(user1);

    let unique_chat_name = format!("Transfer Owner Test {}", Uuid::now_v7());
    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      &unique_chat_name
    )
    .await;

    let chat_id_i64: i64 = chat.id.into();
    let user2_id_i64: i64 = user2.id.into();
    let user1_id_i64: i64 = user1.id.into();

    let response_msg: TransferOwnershipResponse = assert_handler_success!(
      transfer_chat_ownership_handler(
        State(state.clone()),
        Extension(creator_auth),
        Path((chat_id_i64, user2_id_i64))
      ),
      StatusCode::OK,
      TransferOwnershipResponse
    );

    assert_eq!(
      response_msg.message,
      "Chat ownership transferred successfully"
    );
    assert_eq!(response_msg.old_owner_id, user1_id_i64);
    assert_eq!(response_msg.new_owner_id, user2_id_i64);

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
