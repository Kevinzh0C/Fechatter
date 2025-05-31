// User resolvers
use crate::AppState;
use crate::interfaces::graphql::types::User;
use async_graphql::{Error, Result};

pub async fn get_user(state: &AppState, user_id: i64) -> Result<User> {
  // TODO: Implement using repository
  let user = state
    .find_user_by_id(user_id)
    .await
    .map_err(|e| Error::new(e.to_string()))?
    .ok_or_else(|| Error::new("User not found"))?;

  Ok(User {
    id: user.id,
    email: user.email,
    fullname: user.fullname,
    created_at: user.created_at,
    workspace_id: user.workspace_id,
  })
}

pub async fn search_users(state: &AppState, query: String, limit: i32) -> Result<Vec<User>> {
  // TODO: Implement user search
  // For now, return mock data
  Ok(vec![
    User {
      id: 1,
      email: "user1@example.com".to_string(),
      fullname: "User One".to_string(),
      created_at: chrono::Utc::now(),
      workspace_id: 1,
    },
    User {
      id: 2,
      email: "user2@example.com".to_string(),
      fullname: "User Two".to_string(),
      created_at: chrono::Utc::now(),
      workspace_id: 1,
    },
  ])
}
