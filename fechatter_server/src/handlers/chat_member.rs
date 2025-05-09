use axum::{
  Extension, Json,
  extract::{Path, State},
  http::StatusCode,
  response::IntoResponse,
};

use tracing::info;

use crate::{
  AppError, AppState,
  models::{AuthUser, ChatMember},
};

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

  let is_deleted = state
    .remove_group_chat_members(chat_id, user.id, payload)
    .await?;

  if is_deleted {
    Ok(StatusCode::NO_CONTENT)
  } else {
    Err(AppError::ChatValidationError(format!(
      "User {} was not found in chat {} or already removed.",
      user.id, chat_id
    )))
  }
}

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

  #[tokio::test]
  async fn test_list_chat_members_handler_success() -> Result<()> {
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
      "List Member Test Chat"
    )
    .await;

    assert_chat_member_count!(state, auth_user, chat.id, 3);

    Ok(())
  }

  #[tokio::test]
  async fn test_list_chat_members_handler_not_member() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(4).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let user4 = &users[3];
    let non_member_auth = auth_user!(user4);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      "List Member Permission Test"
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
  async fn test_add_chat_members_batch_handler_success() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(4).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let user4 = &users[3];
    let creator_auth = auth_user!(user1);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      "Add Member Batch Test"
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
  async fn test_add_chat_members_batch_handler_permission_denied() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(4).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let user4 = &users[3];
    let non_creator_auth = auth_user!(user2);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      "Add Member Perm Test"
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
  async fn test_remove_chat_member_handler_success() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(4).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let user4 = &users[3];
    let creator_auth = auth_user!(user1);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3, user4],
      "Remove Member Test"
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
  async fn test_remove_chat_member_handler_permission_denied() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(4).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let user4 = &users[3];
    let non_creator_auth = auth_user!(user2);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3, user4],
      "Remove Member Perm Test"
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
  async fn test_transfer_chat_ownership_handler_success() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let creator_auth = auth_user!(user1);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      "Transfer Owner Test"
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

    let updated_chat_info = sqlx::query!("SELECT created_by FROM chats WHERE id = $1", chat.id)
      .fetch_one(state.pool())
      .await?;
    assert_eq!(updated_chat_info.created_by, user2.id);

    Ok(())
  }

  #[tokio::test]
  async fn test_transfer_chat_ownership_handler_not_creator() -> Result<()> {
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
      "Transfer Owner Perm Test"
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
  async fn test_transfer_chat_ownership_handler_target_not_member() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(4).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];
    let user4 = &users[3];
    let creator_auth = auth_user!(user1);

    let chat = create_new_test_chat!(
      state,
      user1,
      ChatType::Group,
      [user2, user3],
      "Transfer Target Member Test"
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
