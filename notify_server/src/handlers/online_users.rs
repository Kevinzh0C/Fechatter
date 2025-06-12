use fechatter_protos::fechatter::v1::*;
use axum::{
  Extension,
  extract::{Query, State},
  response::Json,
  http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{error, info};

use crate::{error::NotifyError, state::AppState};
use fechatter_core::{AuthUser, ChatId, UserId, WorkspaceId};

/// Query parameters for online users
#[derive(Debug, Deserialize)]
pub struct OnlineUsersQuery {
  pub chat_id: Option<i64>,
  pub workspace_id: Option<i64>,
}

/// Response for an online user
#[derive(Debug, Serialize)]
pub struct OnlineUserResponse {
  pub user_id: i64,
  pub fullname: String,
  pub email: String,
  pub status: String,
  pub last_seen: DateTime<Utc>,
  pub connected_at: DateTime<Utc>,
}

/// Online users query API - migrated from fechatter_server
pub async fn get_online_users_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Query(params): Query<OnlineUsersQuery>,
) -> Result<Json<Vec<OnlineUserResponse>>, NotifyError> {
  info!("User {} querying online users: {:?}", user.id, params);

  let online_users = if let Some(chat_id) = params.chat_id {
    // Query online users in a specific chat
    let chat_id = ChatId(chat_id);

    // Permission check: ensure the user is a member of the chat
    if !is_user_chat_member(&state, UserId(user.id.into()), chat_id).await {
      return Err(NotifyError::Unauthorized(
        "Not a member of this chat".to_string(),
      ));
    }

    get_online_users_for_chat(&state, chat_id).await
  } else if let Some(workspace_id) = params.workspace_id {
    // Query online users in a workspace

    // Permission check: ensure the user belongs to the workspace
    if user.workspace_id != WorkspaceId(workspace_id) {
      return Err(NotifyError::Unauthorized(
        "Not a member of this workspace".to_string(),
      ));
    }

    get_online_users_for_workspace(&state, workspace_id).await
  } else {
    // Query online users in all chats the user is in
    get_online_users_for_user(&state, UserId(user.id.into())).await
  };

  info!(
    "Returning {} online users to user {}",
    online_users.len(),
    user.id
  );
  Ok(Json(online_users))
}

/// Check if a user is a member of a chat
async fn is_user_chat_member(state: &AppState, user_id: UserId, chat_id: ChatId) -> bool {
  // First check the cache
  if let Some(members) = state.chat_members.get(&chat_id) {
    return members.contains(&user_id);
  }

  // Not in cache, query the database
  if let Ok(members) = state.get_chat_members(chat_id).await {
    state.chat_members.insert(chat_id, members.clone());
    return members.contains(&user_id);
  }

  false
}

/// Get online users for a specific chat
async fn get_online_users_for_chat(state: &AppState, chat_id: ChatId) -> Vec<OnlineUserResponse> {
  let online_members = state.get_online_chat_members(chat_id).await;

  // TODO: Fetch user details from the database
  // Currently returns a simplified version
  online_members
    .into_iter()
    .map(|user_id| OnlineUserResponse {
      user_id: user_id.0,
      fullname: format!("User {}", user_id.0),
      email: format!("user{}@example.com", user_id.0),
      status: "online".to_string(),
      last_seen: chrono::Utc::now(),
      connected_at: chrono::Utc::now(),
    })
    .collect()
}

/// Get online users for a workspace
async fn get_online_users_for_workspace(
  state: &AppState,
  workspace_id: i64,
) -> Vec<OnlineUserResponse> {
  // TODO: Implement workspace online user query
  // Should query workspace members from the database, then check online status

  // Currently returns a simplified version of all online users
  state
    .user_connections
    .iter()
    .map(|entry| {
      let user_id = entry.key();
      OnlineUserResponse {
        user_id: user_id.0,
        fullname: format!("User {}", user_id.0),
        email: format!("user{}@example.com", user_id.0),
        status: "online".to_string(),
        last_seen: chrono::Utc::now(),
        connected_at: chrono::Utc::now(),
      }
    })
    .collect()
}

/// Get online users in all chats the user is in
async fn get_online_users_for_user(state: &AppState, user_id: UserId) -> Vec<OnlineUserResponse> {
  let mut all_online_users = HashSet::new();

  // Get all chats the user is in
  if let Some(user_chats) = state.user_chats.get(&user_id) {
    for chat_id in user_chats.iter() {
      let online_members = state.get_online_chat_members(*chat_id).await;
      all_online_users.extend(online_members);
    }
  }

  // Remove the user themselves
  all_online_users.remove(&user_id);

  // Convert to response format
  all_online_users
    .into_iter()
    .map(|user_id| OnlineUserResponse {
      user_id: user_id.0,
      fullname: format!("User {}", user_id.0),
      email: format!("user{}@example.com", user_id.0),
      status: "online".to_string(),
      last_seen: chrono::Utc::now(),
      connected_at: chrono::Utc::now(),
    })
    .collect()
}

// Note: Analytics tracking has been moved to a dedicated analytics service
// and is handled by the analytics publisher in the state module
