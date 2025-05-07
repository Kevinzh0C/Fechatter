use axum::{
  Json,
  extract::{Extension, Path, State},
  http::StatusCode,
  response::IntoResponse,
};

use tracing::info;

use crate::{
  AppError, AppState,
  models::{
    AuthUser,
    chat::{CreateChat, UpdateChat, create_new_chat, delete_chat, list_chats_of_user, update_chat},
  },
};

pub(crate) async fn list_chats_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
  info!("User {} listing chats", user.id);
  let chats_arc = state.list_chats_of_user(user.id).await?;
  Ok((StatusCode::OK, Json(chats_arc)))
}

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
      Some(payload.chat_members),
      Some(&payload.description),
      user.workspace_id,
    )
    .await?;

  Ok((StatusCode::CREATED, Json(chat)))
}

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

  use crate::models::chat::{CreateChat, UpdateChat};
  use crate::{
    assert_chat_list_count, assert_handler_error, assert_handler_success, create_new_test_chat,
    setup_test_users,
  };
  use anyhow::Result;
  use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};

  use fechatter_core::auth_user;

  #[tokio::test]
  async fn create_chat_handler_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let auth_user = auth_user!(user1);

    let payload = CreateChat {
      name: "Test Group Chat".to_string(),
      chat_type: ChatType::Group,
      chat_members: vec![user2.id, user3.id], // IDs needed here
      description: "A test group".to_string(),
    };

    let created_chat = assert_handler_success!(
      create_chat_handler(State(state), Extension(auth_user), Json(payload)),
      StatusCode::CREATED,
      Chat // Expected response type
    );

    // Additional assertions on the deserialized chat object
    assert_eq!(created_chat.name, "Test Group Chat");
    assert_eq!(created_chat.chat_type, ChatType::Group);
    assert_eq!(created_chat.chat_members.len(), 3);
    assert!(created_chat.chat_members.contains(&user1.id));
    assert!(created_chat.chat_members.contains(&user2.id));
    assert!(created_chat.chat_members.contains(&user3.id));
    assert_eq!(created_chat.created_by, user1.id);

    Ok(())
  }

  #[tokio::test]
  async fn test_create_chat_handler_single() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(2).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let auth_user = auth_user!(user1);

    let payload = CreateChat {
      name: "Test Single Chat".to_string(),
      chat_type: ChatType::Single,
      chat_members: vec![user2.id],
      description: "".to_string(),
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
  async fn test_list_chats_handler() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let auth_user = auth_user!(user1);

    create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      "Group Chat 1"
    )
    .await;
    create_new_test_chat!(state, user1, ChatType::Single, [user2], "Single Chat 1").await;
    create_new_test_chat!(
      state,
      user2,
      ChatType::Group,
      [user1, user3],
      "Group Chat 2"
    )
    .await;

    assert_chat_list_count!(state, auth_user, 3);

    Ok(())
  }

  #[tokio::test]
  async fn test_update_chat_handler_success() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let auth_user = auth_user!(user1);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      "Chat to Update",
      "Original Desc"
    )
    .await;

    let payload = UpdateChat {
      name: Some("Updated Chat Name".to_string()),
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

    assert_eq!(updated_chat.name, "Updated Chat Name");
    assert_eq!(updated_chat.description, "Updated Desc");
    assert_eq!(updated_chat.id, chat.id);

    Ok(())
  }

  #[tokio::test]
  async fn test_update_chat_handler_permission_denied() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let non_creator_auth = auth_user!(user2);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      "Permission Test Chat"
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
  async fn test_delete_chat_handler_success() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let auth_user = auth_user!(user1);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      "Chat to Delete"
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
  async fn test_delete_chat_handler_permission_denied() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let non_creator_auth = auth_user!(user2);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      "Delete Permission Test"
    )
    .await;

    assert_handler_error!(
      delete_chat_handler(State(state), Extension(non_creator_auth), Path(chat.id)),
      AppError::ChatPermissionError(_)
    );

    Ok(())
  }

  #[tokio::test]
  async fn test_delete_chat_handler_not_found() -> Result<()> {
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
