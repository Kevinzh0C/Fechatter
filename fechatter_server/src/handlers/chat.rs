use axum::{
  Json,
  extract::{Extension, Path, State},
  http::StatusCode,
  response::IntoResponse,
};

use tracing::info;

use crate::{AppError, AppState, error::ErrorOutput, models::AuthUser};
use fechatter_core::{Chat, CreateChat, UpdateChat};

/// 获取当前用户的聊天列表
#[utoipa::path(
    get,
    path = "/api/chats",
    security(
        ("access_token" = [])
    ),
    responses(
        (status = 200, description = "Chats retrieved successfully", body = Vec<Chat>),
        (status = 401, description = "Unauthorized", body = ErrorOutput)
    ),
    tag = "chats"
)]
pub(crate) async fn list_chats_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
  info!("User {} listing chats", user.id);
  let chats_arc = state.list_chats_of_user(user.id).await?;
  Ok((StatusCode::OK, Json(chats_arc)))
}

/// 创建新聊天
#[utoipa::path(
    post,
    path = "/api/chats",
    request_body = CreateChat,
    security(
        ("access_token" = [])
    ),
    responses(
        (status = 201, description = "Chat created successfully", body = Chat),
        (status = 400, description = "Invalid input", body = ErrorOutput),
        (status = 401, description = "Unauthorized", body = ErrorOutput)
    ),
    tag = "chats"
)]
pub(crate) async fn create_chat_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Json(payload): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
  let chat = state
    .create_new_chat(
      user.id,
      &payload.name,
      payload.chat_type,
      payload.members,
      Some(payload.description.as_deref().unwrap_or("")),
      user.workspace_id,
    )
    .await?;

  Ok((StatusCode::CREATED, Json(chat)))
}

/// 更新聊天信息
#[utoipa::path(
    put,
    path = "/api/chats/{chat_id}",
    params(
        ("chat_id" = i64, Path, description = "Chat ID")
    ),
    request_body = UpdateChat,
    security(
        ("access_token" = [])
    ),
    responses(
        (status = 200, description = "Chat updated successfully", body = Chat),
        (status = 400, description = "Invalid input", body = ErrorOutput),
        (status = 401, description = "Unauthorized", body = ErrorOutput),
        (status = 403, description = "Permission denied", body = ErrorOutput),
        (status = 404, description = "Chat not found", body = ErrorOutput)
    ),
    tag = "chats"
)]
pub(crate) async fn update_chat_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
  Json(payload): Json<UpdateChat>,
) -> Result<impl IntoResponse, AppError> {
  info!("User {} updating chat: {}", user.id, chat_id);

  let updated_chat = state.update_chat(chat_id, user.id, payload).await?;

  Ok((StatusCode::OK, Json(updated_chat)))
}

/// 删除聊天
#[utoipa::path(
    delete,
    path = "/api/chats/{chat_id}",
    params(
        ("chat_id" = i64, Path, description = "Chat ID")
    ),
    security(
        ("access_token" = [])
    ),
    responses(
        (status = 204, description = "Chat deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorOutput),
        (status = 403, description = "Permission denied", body = ErrorOutput),
        (status = 404, description = "Chat not found", body = ErrorOutput)
    ),
    tag = "chats"
)]
pub(crate) async fn delete_chat_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
  info!("User {} deleting chat: {}", user.id, chat_id);

  let deleted = state.delete_chat(chat_id, user.id).await?;

  if deleted {
    Ok(StatusCode::NO_CONTENT.into_response())
  } else {
    Err(AppError::NotFound(vec![chat_id.to_string()]))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::models::{Chat, ChatType};

  use crate::{
    assert_chat_list_count, assert_handler_error, assert_handler_success, auth_user,
    create_new_test_chat, setup_test_users,
  };
  use anyhow::Result;
  use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
  use fechatter_core::CreateChat;

  #[tokio::test]
  async fn create_chat_handler_should_work() -> Result<()> {
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
    let unique_chat_name = format!("Test Group Chat {}", timestamp);

    let payload = CreateChat {
      name: unique_chat_name.clone(),
      chat_type: ChatType::Group,
      members: Some(vec![user2.id, user3.id]),
      description: Some("A test group".to_string()),
    };

    let created_chat = assert_handler_success!(
      create_chat_handler(State(state), Extension(auth_user), Json(payload)),
      StatusCode::CREATED,
      Chat // Expected response type
    );

    // Additional assertions on the deserialized chat object
    assert_eq!(created_chat.name, unique_chat_name);
    assert_eq!(created_chat.chat_type, ChatType::Group);
    assert_eq!(created_chat.chat_members.len(), 3);
    assert!(created_chat.chat_members.contains(&user1.id));
    assert!(created_chat.chat_members.contains(&user2.id));
    assert!(created_chat.chat_members.contains(&user3.id));
    assert_eq!(created_chat.created_by, user1.id);

    Ok(())
  }

  #[tokio::test]
  async fn create_chat_handler_should_work_for_single() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(2).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let auth_user = auth_user!(user1);

    // Generate unique chat name to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let unique_chat_name = format!("Test Single Chat {}", timestamp);

    let payload = CreateChat {
      name: unique_chat_name,
      chat_type: ChatType::Single,
      members: Some(vec![user2.id]),
      description: Some("".to_string()),
    };

    let created_chat = assert_handler_success!(
      create_chat_handler(State(state), Extension(auth_user), Json(payload)),
      StatusCode::CREATED,
      Chat
    );

    assert_eq!(created_chat.chat_type, ChatType::Single);
    assert_eq!(created_chat.chat_members.len(), 2);
    assert!(created_chat.chat_members.contains(&user1.id));
    assert!(created_chat.chat_members.contains(&user2.id));
    assert_eq!(created_chat.created_by, user1.id);

    Ok(())
  }

  #[tokio::test]
  async fn list_chats_handler_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let auth_user = auth_user!(user1);

    // Generate unique chat names to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();

    create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      &format!("Group Chat 1 {}", timestamp)
    )
    .await;
    create_new_test_chat!(
      state,
      user1,
      ChatType::Single,
      [user2],
      &format!("Single Chat 1 {}", timestamp)
    )
    .await;
    create_new_test_chat!(
      state,
      user2,
      ChatType::Group,
      [user1, user3],
      &format!("Group Chat 2 {}", timestamp)
    )
    .await;

    assert_chat_list_count!(state, auth_user, 3);

    Ok(())
  }

  #[tokio::test]
  async fn update_chat_handler_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let auth_user = auth_user!(user1);

    // Generate unique chat names to avoid conflicts
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let original_name = format!("Chat to Update {}", timestamp);
    let updated_name = format!("Updated Chat Name {}", timestamp);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      &original_name,
      "Original Desc"
    )
    .await;

    let payload = UpdateChat {
      name: Some(updated_name.clone()),
      description: Some("Updated Desc".to_string()),
    };

    let updated_chat = assert_handler_success!(
      update_chat_handler(
        State(state),
        Extension(auth_user),
        Path(chat.id),
        Json(payload)
      ),
      StatusCode::OK,
      Chat
    );

    assert_eq!(updated_chat.name, updated_name);
    assert_eq!(updated_chat.description, "Updated Desc");
    assert_eq!(updated_chat.id, chat.id);

    Ok(())
  }

  #[tokio::test]
  async fn update_chat_handler_should_deny_non_creator() -> Result<()> {
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
    let unique_chat_name = format!("Permission Test Chat {}", timestamp);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      &unique_chat_name
    )
    .await;

    let payload = UpdateChat {
      name: Some("Try Update".to_string()),
      description: None,
    };

    assert_handler_error!(
      update_chat_handler(
        State(state),
        Extension(non_creator_auth),
        Path(chat.id),
        Json(payload)
      ),
      AppError::ChatPermissionError(_)
    );

    Ok(())
  }

  #[tokio::test]
  async fn delete_chat_handler_should_work() -> Result<()> {
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
    let unique_chat_name = format!("Chat to Delete {}", timestamp);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      &unique_chat_name
    )
    .await;

    assert_handler_success!(
      delete_chat_handler(
        State(state.clone()),
        Extension(auth_user.clone()),
        Path(chat.id)
      ),
      StatusCode::NO_CONTENT
    );

    assert_chat_list_count!(state, auth_user, 0);

    Ok(())
  }

  #[tokio::test]
  async fn delete_chat_handler_should_deny_non_creator() -> Result<()> {
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
    let unique_chat_name = format!("Delete Permission Test {}", timestamp);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      &unique_chat_name
    )
    .await;

    assert_handler_error!(
      delete_chat_handler(State(state), Extension(non_creator_auth), Path(chat.id)),
      AppError::ChatPermissionError(_)
    );

    Ok(())
  }

  #[tokio::test]
  async fn delete_chat_handler_should_return_not_found() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user1 = &users[0];
    let auth_user = auth_user!(user1);
    let non_existent_chat_id = 9999;

    assert_handler_error!(
      delete_chat_handler(State(state),
      Extension(auth_user), Path(non_existent_chat_id)),
      AppError::NotFound(ids) if ids.len() == 1 && ids[0] == non_existent_chat_id.to_string()
    );

    Ok(())
  }
}
