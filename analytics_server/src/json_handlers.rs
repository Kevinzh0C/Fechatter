use crate::{
  AnalyticsEventRow, AppError, AppState, extractors::Geo, pb::AnalyticsEvent,
  handlers::{insert_analytics_event, insert_batch_analytics_events},
};
use axum::{
  Json,
  extract::State,
  http::{StatusCode, request::Parts},
  response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{info, instrument, warn};
use utoipa::ToSchema;

/// JSON format for analytics event context
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct JsonEventContext {
  pub client_id: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub session_id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub user_id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub app_version: Option<String>,
  pub client_ts: i64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub server_ts: Option<i64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub user_agent: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ip: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub system: Option<JsonSystemInfo>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub geo: Option<JsonGeoLocation>,
}

/// JSON format for system info
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct JsonSystemInfo {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub os: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub arch: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub locale: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub timezone: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub browser: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub browser_version: Option<String>,
}

/// JSON format for geo location
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct JsonGeoLocation {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub country: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub region: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub city: Option<String>,
}

/// JSON format for analytics event
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct JsonAnalyticsEvent {
  pub context: JsonEventContext,
  pub event_type: HashMap<String, Value>,
}

/// JSON format for batch analytics events
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct JsonBatchAnalyticsEvents {
  pub events: Vec<JsonAnalyticsEvent>,
}

/// Response format for event creation
#[derive(Debug, Serialize, ToSchema)]
pub struct EventResponse {
  pub status: String,
  pub message: String,
  pub session_id: String,
  pub server_ts: i64,
}

/// Create analytics event from JSON
///
/// This endpoint receives analytics events in JSON format and stores them
/// in ClickHouse for further analysis. It provides compatibility with
/// web frontends that prefer JSON over protobuf.
#[utoipa::path(
    post,
    path = "/api/event/json",
    request_body(
        content = JsonAnalyticsEvent,
        description = "Analytics event in JSON format",
        content_type = "application/json"
    ),
    responses(
        (status = 201, description = "Event created successfully", body = EventResponse),
        (status = 400, description = "Invalid event data"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "analytics"
)]
#[instrument(skip(parts, state, payload), fields(event_type, client_id, user_id))]
pub async fn create_json_event_handler(
  parts: Parts,
  State(state): State<AppState>,
  Json(payload): Json<JsonAnalyticsEvent>,
) -> Result<impl IntoResponse, AppError> {
  // Increment metrics
  state.metrics.increment_events_received();

  // Extract event type
  let event_type = payload.event_type.keys().next()
    .cloned()
    .unwrap_or_else(|| "unknown".to_string());

  // Extract metadata for logging
  let client_id = &payload.context.client_id;
  let user_id = payload.context.user_id.as_deref().unwrap_or("anonymous");

  // Set tracing fields
  tracing::Span::current()
    .record("event_type", &event_type)
    .record("client_id", client_id)
    .record("user_id", user_id);

  info!("EVENT: [ANALYTICS] Processing JSON event: {} from client: {} (user: {})", 
        event_type, client_id, user_id);

  // Convert JSON to protobuf event
  let mut proto_event = json_to_protobuf(payload)?;
  
  // Set server timestamp
  if let Some(ref mut context) = proto_event.context {
    context.server_ts = chrono::Utc::now().timestamp_millis();
  }

  // Convert protobuf event to database row
  let mut row = AnalyticsEventRow::try_from(proto_event).map_err(|e| {
    warn!("ERROR: [ANALYTICS] Failed to convert JSON event to row: {}", e);
    state.metrics.increment_events_failed();
    e
  })?;

  info!("[ANALYTICS] Successfully converted JSON event to database row: {}", event_type);

  // Extract geo information from headers
  let geo = Geo::from_parts(&parts);

  // Enhance with server-side information
  row.update_with_server_info(&parts, geo.0);
  row.set_session_id(&state);

  info!("[ANALYTICS] Enhanced JSON event with server info and session ID");

  // Insert into ClickHouse
  let insert_result = tokio::time::timeout(
    std::time::Duration::from_secs(10),
    insert_analytics_event(&state, &row),
  )
  .await;

  match insert_result {
    Ok(Ok(())) => {
      info!("💾 [ANALYTICS] JSON event stored successfully in ClickHouse: {}", event_type);
      state.metrics.increment_events_processed();
      
      let response = EventResponse {
        status: "success".to_string(),
        message: "Event stored successfully".to_string(),
        session_id: row.session_id.clone(),
        server_ts: row.server_ts,
      };
      
      Ok((StatusCode::CREATED, Json(response)))
    }
    Ok(Err(e)) => {
      warn!("ERROR: [ANALYTICS] Failed to insert JSON event into ClickHouse: {}", e);
      state.metrics.increment_events_failed();
      state.metrics.increment_database_errors();
      Err(e)
    }
    Err(_) => {
      warn!("⏰ [ANALYTICS] JSON event insertion timed out");
      state.metrics.increment_events_failed();
      state.metrics.increment_database_errors();
      Err(AppError::AnyError(anyhow::anyhow!(
        "Database operation timed out"
      )))
    }
  }
}

/// Create batch analytics events from JSON
#[utoipa::path(
    post,
    path = "/api/batch/json",
    request_body(
        content = JsonBatchAnalyticsEvents,
        description = "Batch analytics events in JSON format",
        content_type = "application/json"
    ),
    responses(
        (status = 201, description = "Events created successfully"),
        (status = 400, description = "Invalid event data"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "analytics"
)]
#[instrument(skip(parts, state, payload), fields(event_count))]
pub async fn create_json_batch_events_handler(
  parts: Parts,
  State(state): State<AppState>,
  Json(payload): Json<JsonBatchAnalyticsEvents>,
) -> Result<impl IntoResponse, AppError> {
  let event_count = payload.events.len();
  tracing::Span::current().record("event_count", event_count);
  
  if event_count == 0 {
    info!("📭 [ANALYTICS] Empty JSON batch received, nothing to process");
    return Ok((
      StatusCode::OK,
      Json(json!({
        "status": "success",
        "message": "No events to process",
        "processed": 0
      })),
    ));
  }
  
  info!("EVENT: [ANALYTICS] Processing JSON batch with {} events", event_count);
  
  // Extract geo information from headers
  let geo = Geo::from_parts(&parts);
  
  let mut processed = 0;
  let mut failed = 0;
  let mut rows = Vec::new();
  
  // Convert all events to rows
  for (index, json_event) in payload.events.into_iter().enumerate() {
    state.metrics.increment_events_received();
    
    match json_to_protobuf(json_event).and_then(|proto| AnalyticsEventRow::try_from(proto)) {
      Ok(mut row) => {
        // Enhance with server-side information
        row.update_with_server_info(&parts, geo.0.clone());
        row.set_session_id(&state);
        rows.push(row);
        if (index + 1) % 50 == 0 {
          info!("🔄 [ANALYTICS] Converted {}/{} JSON batch events to rows", index + 1, event_count);
        }
      }
      Err(e) => {
        warn!("ERROR: [ANALYTICS] Failed to convert JSON batch event {} to row: {}", index, e);
        state.metrics.increment_events_failed();
        failed += 1;
      }
    }
  }
  
  info!("[ANALYTICS] JSON batch conversion completed: {} successful, {} failed", rows.len(), failed);
  
  // Batch insert into ClickHouse
  if !rows.is_empty() {
    info!("💾 [ANALYTICS] Inserting {} JSON events into ClickHouse...", rows.len());
    
    let insert_result = tokio::time::timeout(
      std::time::Duration::from_secs(30),
      insert_batch_analytics_events(&state, &rows),
    )
    .await;
    
    match insert_result {
      Ok(Ok(())) => {
        processed = rows.len();
        for _ in 0..processed {
          state.metrics.increment_events_processed();
        }
        info!("[ANALYTICS] JSON batch successfully stored {} events in ClickHouse", processed);
      }
      Ok(Err(e)) => {
        warn!("ERROR: [ANALYTICS] Failed to insert JSON batch into ClickHouse: {}", e);
        failed += rows.len();
        for _ in 0..rows.len() {
          state.metrics.increment_events_failed();
          state.metrics.increment_database_errors();
        }
        return Err(e);
      }
      Err(_) => {
        warn!("⏰ [ANALYTICS] JSON batch insertion timed out after 30s");
        failed += rows.len();
        for _ in 0..rows.len() {
          state.metrics.increment_events_failed();
          state.metrics.increment_database_errors();
        }
        return Err(AppError::AnyError(anyhow::anyhow!(
          "Database operation timed out"
        )));
      }
    }
  } else {
    info!("📭 [ANALYTICS] No valid JSON events to insert after batch processing");
  }
  
  info!("[ANALYTICS] JSON batch processing completed: {} processed, {} failed, {} total", 
        processed, failed, event_count);
  
  Ok((
    StatusCode::CREATED,
    Json(json!({
      "status": "success",
      "message": "Batch events processed",
      "processed": processed,
      "failed": failed,
      "total": event_count
    })),
  ))
}

/// Convert JSON analytics event to protobuf format
fn json_to_protobuf(json_event: JsonAnalyticsEvent) -> Result<AnalyticsEvent, AppError> {
  use crate::pb::{
    EventContext, SystemInfo, GeoLocation,
    analytics_event::EventType,
    AppStartEvent, AppExitEvent, UserLoginEvent, UserLogoutEvent,
    UserRegisterEvent, MessageSentEvent, ErrorOccurredEvent,
  };

  // Convert context
  let context = EventContext {
    client_id: json_event.context.client_id,
    session_id: json_event.context.session_id.unwrap_or_default(),
    user_id: json_event.context.user_id.unwrap_or_default(),
    app_version: json_event.context.app_version.unwrap_or_default(),
    client_ts: json_event.context.client_ts,
    server_ts: json_event.context.server_ts.unwrap_or(0),
    user_agent: json_event.context.user_agent.unwrap_or_default(),
    ip: json_event.context.ip.unwrap_or_default(),
    system: json_event.context.system.map(|s| SystemInfo {
      os: s.os.unwrap_or_default(),
      arch: s.arch.unwrap_or_default(),
      locale: s.locale.unwrap_or_default(),
      timezone: s.timezone.unwrap_or_default(),
      browser: s.browser.unwrap_or_default(),
      browser_version: s.browser_version.unwrap_or_default(),
    }),
    geo: json_event.context.geo.map(|g| GeoLocation {
      country: g.country.unwrap_or_default(),
      region: g.region.unwrap_or_default(),
      city: g.city.unwrap_or_default(),
    }),
  };

  // Convert event type
  let event_type_key = json_event.event_type.keys().next()
    .ok_or_else(|| AppError::AnyError(anyhow::anyhow!("No event type specified")))?;
  
  let event_type = match event_type_key.as_str() {
    "app_start" => EventType::AppStart(AppStartEvent {}),
    "app_exit" => EventType::AppExit(AppExitEvent { 
      exit_code: 0 // Default to success
    }),
    "user_login" => {
      let data = json_event.event_type.get(event_type_key).unwrap();
      EventType::UserLogin(UserLoginEvent {
        email: data.get("email").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        login_method: data.get("login_method").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
      })
    },
    "user_logout" => {
      let data = json_event.event_type.get(event_type_key).unwrap();
      EventType::UserLogout(UserLogoutEvent {
        email: data.get("email").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
      })
    },
    "user_register" => {
      let data = json_event.event_type.get(event_type_key).unwrap();
      EventType::UserRegister(UserRegisterEvent {
        email: data.get("email").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        workspace_id: data.get("workspace_id").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        registration_method: data.get("registration_method").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
      })
    },
    "message_sent" => {
      let data = json_event.event_type.get(event_type_key).unwrap();
      EventType::MessageSent(MessageSentEvent {
        chat_id: data.get("chat_id").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        r#type: data.get("type").and_then(|v| v.as_str()).unwrap_or("text").to_string(),
        size: data.get("size").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        total_files: data.get("total_files").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        has_mentions: data.get("has_mentions").and_then(|v| v.as_bool()).unwrap_or(false),
        has_links: data.get("has_links").and_then(|v| v.as_bool()).unwrap_or(false),
      })
    },
    "error_occurred" => {
      let data = json_event.event_type.get(event_type_key).unwrap();
      EventType::ErrorOccurred(ErrorOccurredEvent {
        error_type: data.get("error_type").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        error_code: data.get("error_code").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        error_message: data.get("error_message").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        stack_trace: data.get("stack_trace").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        context: data.get("context").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
      })
    },
    // Add more event type conversions as needed
    _ => {
      warn!("Unknown event type: {}", event_type_key);
      // Default to app_start for unknown events
      EventType::AppStart(AppStartEvent {})
    }
  };

  Ok(AnalyticsEvent {
    context: Some(context),
    event_type: Some(event_type),
  })
}

 