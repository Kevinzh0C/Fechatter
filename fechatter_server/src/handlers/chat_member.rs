use axum::{
  Extension, Json,
  extract::{Path, State},
  http::StatusCode,
  response::IntoResponse,
};

use tracing::info;

use crate::{
  AppError, AppState,
  error::ErrorOutput,
  models::{AuthUser, ChatMember},
};

#[utoipa::path(
    get,
    path = "/api/chats/{chat_id}/members",
    params(
        ("chat_id" = i64, Path, description = "Chat ID")
    ),
    security(
        ("access_token" = [])
    ),
    responses(
        (status = 200, description = "Chat members retrieved successfully", body = Vec<ChatMember>),
        (status = 401, description = "Unauthorized", body = ErrorOutput),
        (status = 403, description = "Permission denied", body = ErrorOutput),
        (status = 404, description = "Chat not found", body = ErrorOutput)
    ),
    tag = "chat members"
)]
pub(crate) async fn list_chat_members_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
) -> Result<(StatusCode, Json<Vec<ChatMember>>), AppError> {
  info!("User {} listing members for chat {}", user.id, chat_id);

  let members = state.list_chat_members(chat_id).await?;

  Ok((StatusCode::OK, Json(members)))
}

// pub(crate) async fn add_chat_member_handler(
//   State(state): State<AppState>,
//   Extension(user): Extension<AuthUser>,
//   Path(chat_id): Path<i64>,
//   Json(member_id): Json<i64>,
// ) -> Result<(StatusCode, Json<ChatMember>), AppError> {
//   let added_member = add_single_member(&state, chat_id, user.id, member_id).await?;

//   Ok((StatusCode::CREATED, Json(added_member)))
// }

/// 批量添加聊天成员
#[utoipa::path(
    post,
    path = "/api/chats/{chat_id}/members",
    params(
        ("chat_id" = i64, Path, description = "Chat ID")
    ),
    request_body = Vec<i64>,
    security(
        ("access_token" = [])
    ),
    responses(
        (status = 201, description = "Members added successfully", body = Vec<ChatMember>),
        (status = 400, description = "Invalid input", body = ErrorOutput),
        (status = 401, description = "Unauthorized", body = ErrorOutput),
        (status = 403, description = "Permission denied", body = ErrorOutput),
        (status = 404, description = "Chat not found", body = ErrorOutput)
    ),
    tag = "chat members"
)]
pub(crate) async fn add_chat_members_batch_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
  Json(member_ids): Json<Vec<i64>>,
) -> Result<(StatusCode, Json<Vec<ChatMember>>), AppError> {
  info!(
    "User {} adding batch members {:?} to chat {}",
    user.id, member_ids, chat_id
  );

  let members = state.add_chat_members(chat_id, user.id, member_ids).await?;

  Ok((StatusCode::CREATED, Json(members)))
}

/// 移除聊天成员
#[utoipa::path(
    delete,
    path = "/api/chats/{chat_id}/members",
    params(
        ("chat_id" = i64, Path, description = "Chat ID")
    ),
    request_body = Vec<i64>,
    security(
        ("access_token" = [])
    ),
    responses(
        (status = 204, description = "Members removed successfully"),
        (status = 400, description = "Invalid input", body = ErrorOutput),
        (status = 401, description = "Unauthorized", body = ErrorOutput),
        (status = 403, description = "Permission denied", body = ErrorOutput),
        (status = 404, description = "Chat not found", body = ErrorOutput)
    ),
    tag = "chat members"
)]
pub(crate) async fn remove_chat_member_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
  Json(payload): Json<Vec<i64>>,
) -> Result<StatusCode, AppError> {
  info!(
    "User {} attempting to remove members {:?} from chat {}",
    user.id, payload, chat_id
  );

  match state
    .remove_group_chat_members(chat_id, user.id, payload)
    .await
  {
    Ok(is_deleted) => {
      if is_deleted {
        Ok(StatusCode::NO_CONTENT)
      } else {
        Err(AppError::ChatValidationError(format!(
          "User {} was not found in chat {} or already removed.",
          user.id, chat_id
        )))
      }
    }
    Err(e) => Err(e),
  }
}

/// 转移聊天所有权
#[utoipa::path(
    post,
    path = "/api/chats/{chat_id}/transfer/{target_user_id}",
    params(
        ("chat_id" = i64, Path, description = "Chat ID"),
        ("target_user_id" = i64, Path, description = "Target user ID to transfer ownership to")
    ),
    security(
        ("access_token" = [])
    ),
    responses(
        (status = 200, description = "Ownership transferred successfully"),
        (status = 400, description = "Invalid input", body = ErrorOutput),
        (status = 401, description = "Unauthorized", body = ErrorOutput),
        (status = 403, description = "Permission denied", body = ErrorOutput),
        (status = 404, description = "Chat or user not found", body = ErrorOutput)
    ),
    tag = "chat members"
)]
pub(crate) async fn transfer_chat_ownership_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path((chat_id, target_user_id)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, AppError> {
  let result = state
    .transfer_chat_ownership(chat_id, user.id, target_user_id)
    .await?;

  if result {
    Ok((
      StatusCode::OK,
      Json("Chat ownership transferred successfully".to_string()),
    ))
  } else {
    Err(AppError::ChatValidationError(format!(
      "User {} tried to transfer ownership of chat {} but failed",
      user.id, chat_id
    )))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::models::ChatMember;
  use crate::{
    assert_chat_member_count, assert_handler_error, assert_handler_success, auth_user,
    create_new_test_chat, setup_test_users,
  };
  use anyhow::Result;
  use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
  use sqlx::Row;

  #[tokio::test]
  async fn list_chat_members_handler_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let auth_user = auth_user!(user1);

    // Generate unique chat name to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let unique_chat_name = format!("List Member Test Chat {}", timestamp);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      &unique_chat_name
    )
    .await;

    assert_chat_member_count!(state, auth_user, chat.id, 3);

    Ok(())
  }

  #[tokio::test]
  async fn list_chat_members_handler_should_deny_non_member() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(4).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let user4 = &users[3];
    let _non_member_auth = auth_user!(user4);

    // Generate unique chat name to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let unique_chat_name = format!("List Member Permission Test {}", timestamp);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      &unique_chat_name
    )
    .await;

    let result = state.ensure_user_is_chat_member(chat.id, user4.id).await;

    assert!(result.is_err());
    match result {
      Err(AppError::ChatPermissionError(_)) => (),
      Err(e) => panic!("Unexpected error type: {:?}", e),
      Ok(_) => panic!("Expected error for non-member, but got success"),
    }

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

    // Generate unique chat name to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let unique_chat_name = format!("Add Member Batch Test {}", timestamp);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      &unique_chat_name
    )
    .await;

    let members_to_add = vec![user4.id];

    let added_members = assert_handler_success!(
      add_chat_members_batch_handler(
        State(state.clone()),
        Extension(creator_auth),
        Path(chat.id),
        Json(members_to_add.clone()),
      ),
      StatusCode::CREATED,
      Vec<ChatMember>
    );

    assert_eq!(added_members.len(), 1);
    let added_member_ids: Vec<i64> = added_members.iter().map(|m| m.user_id).collect();
    assert!(added_member_ids.contains(&user4.id));

    assert_chat_member_count!(state, auth_user!(user1), chat.id, 4);

    Ok(())
  }

  #[tokio::test]
  async fn add_chat_members_batch_handler_should_deny_non_creator() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(4).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let user4 = &users[3];
    let non_creator_auth = auth_user!(user2);

    // Generate unique chat name to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let unique_chat_name = format!("Add Member Perm Test {}", timestamp);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      &unique_chat_name
    )
    .await;

    let members_to_add = vec![user4.id];

    assert_handler_error!(
      add_chat_members_batch_handler(
        State(state),
        Extension(non_creator_auth),
        Path(chat.id),
        Json(members_to_add),
      ),
      AppError::ChatPermissionError(_)
    );

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

    // Generate unique chat name to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let unique_chat_name = format!("Remove Member Test {}", timestamp);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3, user4],
      &unique_chat_name
    )
    .await;

    let members_to_remove = vec![user3.id, user4.id];

    assert_handler_success!(
      remove_chat_member_handler(
        State(state.clone()),
        Extension(creator_auth),
        Path(chat.id),
        Json(members_to_remove.clone()),
      ),
      StatusCode::NO_CONTENT
    );

    assert_chat_member_count!(state, auth_user!(user1), chat.id, 2);

    Ok(())
  }

  #[tokio::test]
  async fn remove_chat_member_handler_should_deny_non_creator() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(4).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let user4 = &users[3];
    let non_creator_auth = auth_user!(user2);

    // Generate unique chat name to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let unique_chat_name = format!("Remove Member Perm Test {}", timestamp);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3, user4],
      &unique_chat_name
    )
    .await;

    let members_to_remove = vec![user3.id];

    assert_handler_error!(
      remove_chat_member_handler(
        State(state),
        Extension(non_creator_auth),
        Path(chat.id),
        Json(members_to_remove),
      ),
      AppError::ChatPermissionError(_)
    );
    Ok(())
  }

  #[tokio::test]
  async fn transfer_chat_ownership_handler_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let creator_auth = auth_user!(user1);

    // Generate unique chat name to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let unique_chat_name = format!("Transfer Owner Test {}", timestamp);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      &unique_chat_name
    )
    .await;

    let response_msg: String = assert_handler_success!(
      transfer_chat_ownership_handler(
        State(state.clone()),
        Extension(creator_auth),
        Path((chat.id, user2.id)),
      ),
      StatusCode::OK,
      String
    );
    assert_eq!(response_msg, "Chat ownership transferred successfully");

    let query = "SELECT created_by FROM chats WHERE id = $1";
    let updated_chat_info = sqlx::query(query)
      .bind(chat.id)
      .fetch_one(state.pool())
      .await?;

    let created_by: i64 = updated_chat_info
      .try_get("created_by")
      .map_err(|e| AppError::SqlxError(e))?;
    assert_eq!(created_by, user2.id);

    Ok(())
  }

  #[tokio::test]
  async fn transfer_chat_ownership_handler_should_deny_non_creator() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let non_creator_auth = auth_user!(user2);

    // Generate unique chat name to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let unique_chat_name = format!("Transfer Owner Perm Test {}", timestamp);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      &unique_chat_name
    )
    .await;

    assert_handler_error!(
      transfer_chat_ownership_handler(
        State(state),
        Extension(non_creator_auth),
        Path((chat.id, user3.id)),
      ),
      AppError::ChatPermissionError(_)
    );

    Ok(())
  }

  #[tokio::test]
  async fn transfer_chat_ownership_handler_should_reject_non_member_target() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(4).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let user4 = &users[3];
    let creator_auth = auth_user!(user1);

    // Generate unique chat name to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let unique_chat_name = format!("Transfer Target Member Test {}", timestamp);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      &unique_chat_name
    )
    .await;

    assert_handler_error!(
      transfer_chat_ownership_handler(
        State(state),
        Extension(creator_auth),
        Path((chat.id, user4.id)),
      ),
      AppError::ChatValidationError(_)
    );

    Ok(())
  }
}
