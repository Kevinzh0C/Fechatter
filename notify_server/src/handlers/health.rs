use axum::{extract::State, response::Json};
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::state::AppState;

// SSE Health Check Response
#[derive(Debug, Serialize)]
pub struct SSEHealthResponse {
  pub status: String,
  pub service: String,
  pub transport: String,
  pub timestamp: DateTime<Utc>,
  pub connected_users: usize,
  pub active_chats: usize,
}

/// SSE Health Check API
pub async fn sse_health_check(State(state): State<AppState>) -> Json<SSEHealthResponse> {
  let connected_users = state.user_connections.len();
  let active_chats = state.chat_members.len();

  Json(SSEHealthResponse {
    status: "healthy".to_string(),
    service: "notify_server".to_string(),
    transport: "sse".to_string(),
    timestamp: chrono::Utc::now(),
    connected_users,
    active_chats,
  })
}
