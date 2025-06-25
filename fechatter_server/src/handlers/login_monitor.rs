use axum::extract::ws::{Message, WebSocket};
use axum::{
  extract::{Path, Query, State, WebSocketUpgrade},
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use futures::SinkExt;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitorSummaryResponse {
  pub active_sessions: usize,
  pub total_events: usize,
  pub error_rate: f64,
  pub average_duration_ms: u64,
  pub sessions: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct MonitorQueryParams {
  pub limit: Option<usize>,
  pub category: Option<String>,
  pub status: Option<String>,
  pub session_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginMonitorStatus {
  pub status: String,
  pub message: String,
}

// ============================================================================
// Login Monitor Stubs - Login monitoring moved to Gateway
// ============================================================================

/// Get monitor summary (stub - moved to Gateway)
pub async fn get_monitor_summary(
  State(_state): State<AppState>,
  Query(_params): Query<MonitorQueryParams>,
) -> impl IntoResponse {
  let response = LoginMonitorStatus {
    status: "moved".to_string(),
    message: "Login monitoring has been moved to Gateway".to_string(),
  };

  (StatusCode::OK, Json(response))
}

/// Get session details (stub - moved to Gateway)
pub async fn get_session_details(
  State(_state): State<AppState>,
  Path(_session_id): Path<String>,
) -> impl IntoResponse {
  let response = LoginMonitorStatus {
    status: "moved".to_string(),
    message: "Login monitoring has been moved to Gateway".to_string(),
  };

  (StatusCode::OK, Json(response))
}

/// Monitor events websocket (stub - moved to Gateway)
pub async fn monitor_events_websocket(
  ws: WebSocketUpgrade,
  State(_state): State<AppState>,
) -> Response {
  ws.on_upgrade(|socket| handle_monitor_websocket_stub(socket))
}

async fn handle_monitor_websocket_stub(mut socket: WebSocket) {
  // Send notification that monitoring has moved to Gateway
  let msg = serde_json::json!({
      "type": "error",
      "message": "Login monitoring has been moved to Gateway"
  });

  if let Ok(text) = serde_json::to_string(&msg) {
    let _ = socket.send(Message::Text(text.into())).await;
  }

  // Close the connection
  let _ = socket.close().await;
}

/// Cleanup sessions (stub - moved to Gateway)
pub async fn cleanup_sessions(State(_state): State<AppState>) -> impl IntoResponse {
  let response = LoginMonitorStatus {
    status: "moved".to_string(),
    message: "Login monitoring has been moved to Gateway".to_string(),
  };

  (StatusCode::OK, Json(response))
}

/// Export monitor data (stub - moved to Gateway)
pub async fn export_monitor_data(
  State(_state): State<AppState>,
  Query(_params): Query<MonitorQueryParams>,
) -> impl IntoResponse {
  let response = LoginMonitorStatus {
    status: "moved".to_string(),
    message: "Login monitoring has been moved to Gateway".to_string(),
  };

  (StatusCode::OK, Json(response))
}

/// Monitor health check
pub async fn monitor_health() -> impl IntoResponse {
  Json(serde_json::json!({
      "status": "healthy",
      "timestamp": chrono::Utc::now(),
      "service": "login_flow_monitor",
      "note": "Login monitoring has been moved to Gateway"
  }))
}

/// Get monitor stats (stub - moved to Gateway)
pub async fn get_monitor_stats(State(_state): State<AppState>) -> impl IntoResponse {
  let response = LoginMonitorStatus {
    status: "moved".to_string(),
    message: "Login monitoring has been moved to Gateway".to_string(),
  };

  (StatusCode::OK, Json(response))
}

/// Reset monitor data (stub - moved to Gateway)
pub async fn reset_monitor_data(State(_state): State<AppState>) -> impl IntoResponse {
  let response = LoginMonitorStatus {
    status: "moved".to_string(),
    message: "Login monitoring has been moved to Gateway".to_string(),
  };

  (StatusCode::OK, Json(response))
}

/// Get login monitor status (stub - moved to Gateway)
pub async fn get_login_monitor_status(
  State(_state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
  let response = LoginMonitorStatus {
    status: "moved".to_string(),
    message: "Login monitoring has been moved to Gateway".to_string(),
  };

  Ok((StatusCode::OK, Json(response)))
}

/// Get login attempts (stub - moved to Gateway)
pub async fn get_login_attempts(
  State(_state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
  let response = LoginMonitorStatus {
    status: "moved".to_string(),
    message: "Login monitoring has been moved to Gateway".to_string(),
  };

  Ok((StatusCode::OK, Json(response)))
}

/// Get failed logins (stub - moved to Gateway)
pub async fn get_failed_logins(
  State(_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
  let response = LoginMonitorStatus {
    status: "moved".to_string(),
    message: "Login monitoring has been moved to Gateway".to_string(),
  };

  Ok((StatusCode::OK, Json(response)))
}

/// Get suspicious activity (stub - moved to Gateway)
pub async fn get_suspicious_activity(
  State(_state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
  let response = LoginMonitorStatus {
    status: "moved".to_string(),
    message: "Login monitoring has been moved to Gateway".to_string(),
  };

  Ok((StatusCode::OK, Json(response)))
}

/// Clear login history (stub - moved to Gateway)
pub async fn clear_login_history(
  State(_state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
  let response = LoginMonitorStatus {
    status: "moved".to_string(),
    message: "Login monitoring has been moved to Gateway".to_string(),
  };

  Ok((StatusCode::OK, Json(response)))
}

/// Block suspicious IPs (stub - moved to Gateway)
pub async fn block_suspicious_ips(
  State(_state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
  let response = LoginMonitorStatus {
    status: "moved".to_string(),
    message: "Login monitoring has been moved to Gateway".to_string(),
  };

  Ok((StatusCode::OK, Json(response)))
}
