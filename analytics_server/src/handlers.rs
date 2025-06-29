use crate::{
  AnalyticsEventRow, AppError, AppState, ErrorOutput, extractors::Geo, pb::AnalyticsEvent,
};
use axum::{
  Json,
  body::Bytes,
  extract::State,
  http::{StatusCode, request::Parts},
  response::IntoResponse,
};
use prost::Message;
use serde_json::json;
use tracing::{info, instrument, warn};

/// Create analytics event
///
/// This endpoint receives analytics events in protobuf format and stores them
/// in ClickHouse for further analysis. The endpoint supports:
/// - Automatic session tracking
/// - Geographic information extraction from headers
/// - Server-side timestamp override for accuracy
/// - Comprehensive error handling
#[utoipa::path(
    post,
    path = "/api/event",
    request_body(
        content = Vec<u8>,
        description = "Analytics event in protobuf format",
        content_type = "application/protobuf"
    ),
    responses(
        (status = 201, description = "Event created successfully"),
        (status = 400, description = "Invalid event data", body = ErrorOutput),
        (status = 500, description = "Internal server error", body = ErrorOutput),
    ),
    tag = "analytics"
)]
#[instrument(skip(parts, state, body), fields(event_type, client_id, user_id))]
pub(crate) async fn create_event_handler(
  parts: Parts,
  State(state): State<AppState>,
  body: Bytes,
) -> Result<impl IntoResponse, AppError> {
  // Increment metrics
  state.metrics.increment_events_received();

  // Extract geo information from headers
  let geo = Geo::from_parts(&parts);

  // Parse protobuf event
  let event = AnalyticsEvent::decode(body.as_ref())
    .map_err(|e| AppError::AnyError(anyhow::anyhow!("Failed to decode protobuf: {}", e)))?;

  // Extract event metadata for logging
  let event_type = event
    .event_type
    .as_ref()
    .map(|et| match et {
      crate::pb::analytics_event::EventType::AppStart(_) => "app_start",
      crate::pb::analytics_event::EventType::AppExit(_) => "app_exit",
      crate::pb::analytics_event::EventType::UserLogin(_) => "user_login",
      crate::pb::analytics_event::EventType::UserLogout(_) => "user_logout",
      crate::pb::analytics_event::EventType::UserRegister(_) => "user_register",
      crate::pb::analytics_event::EventType::ChatCreated(_) => "chat_created",
      crate::pb::analytics_event::EventType::MessageSent(_) => "message_sent",
      crate::pb::analytics_event::EventType::ChatJoined(_) => "chat_joined",
      crate::pb::analytics_event::EventType::ChatLeft(_) => "chat_left",
      crate::pb::analytics_event::EventType::Navigation(_) => "navigation",
      crate::pb::analytics_event::EventType::FileUploaded(_) => "file_uploaded",
      crate::pb::analytics_event::EventType::FileDownloaded(_) => "file_downloaded",
      crate::pb::analytics_event::EventType::SearchPerformed(_) => "search_performed",
      crate::pb::analytics_event::EventType::NotificationReceived(_) => "notification_received",
      crate::pb::analytics_event::EventType::ErrorOccurred(_) => "error_occurred",
      crate::pb::analytics_event::EventType::BotResponse(_) => "bot_response",
    })
    .unwrap_or("unknown");

  let client_id = event
    .context
    .as_ref()
    .map(|c| c.client_id.as_str())
    .unwrap_or("unknown");
  let user_id = event
    .context
    .as_ref()
    .and_then(|c| {
      if c.user_id.is_empty() {
        None
      } else {
        Some(c.user_id.as_str())
      }
    })
    .unwrap_or("anonymous");

  // Set tracing fields
  tracing::Span::current()
    .record("event_type", event_type)
    .record("client_id", client_id)
    .record("user_id", user_id);

  info!("EVENT: [ANALYTICS] Processing HTTP event: {} from client: {} (user: {})", 
        event_type, client_id, user_id);

  // Convert protobuf event to database row
  let mut row = AnalyticsEventRow::try_from(event).map_err(|e| {
    warn!("ERROR: [ANALYTICS] Failed to convert HTTP event to row: {}", e);
    state.metrics.increment_events_failed();
    e
  })?;

  info!("[ANALYTICS] Successfully converted HTTP event to database row: {}", event_type);

  // Enhance with server-side information
  row.update_with_server_info(&parts, geo.0);
  row.set_session_id(&state);

  info!("[ANALYTICS] Enhanced event with server info and session ID");

  // Insert into ClickHouse with retry logic
  let insert_result = tokio::time::timeout(
    std::time::Duration::from_secs(10), // 10 second timeout
    insert_analytics_event(&state, &row),
  )
  .await;

  match insert_result {
    Ok(Ok(())) => {
      info!("💾 [ANALYTICS] HTTP event stored successfully in ClickHouse: {}", event_type);
      state.metrics.increment_events_processed();
      Ok((
        StatusCode::CREATED,
        Json(json!({
            "status": "success",
            "message": "Event stored successfully",
            "session_id": row.session_id,
            "server_ts": row.server_ts
        })),
      ))
    }
    Ok(Err(e)) => {
      warn!("ERROR: [ANALYTICS] Failed to insert HTTP event into ClickHouse: {}", e);
      state.metrics.increment_events_failed();
      state.metrics.increment_database_errors();
      Err(e)
    }
    Err(_) => {
      warn!("⏰ [ANALYTICS] HTTP event insertion timed out");
      state.metrics.increment_events_failed();
      state.metrics.increment_database_errors();
      Err(AppError::AnyError(anyhow::anyhow!(
        "Database operation timed out"
      )))
    }
  }
}

/// Health check endpoint for the analytics service
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy"),
        (status = 503, description = "Service is unhealthy"),
    ),
    tag = "health"
)]
pub(crate) async fn health_check_handler(
  State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
  // Check ClickHouse connectivity
  let health_check = tokio::time::timeout(
    std::time::Duration::from_secs(5),
    state.client.query("SELECT 1").execute(),
  )
  .await;

  match health_check {
    Ok(Ok(_)) => Ok((
      StatusCode::OK,
      Json(json!({
          "status": "healthy",
          "service": "analytics_server",
          "timestamp": chrono::Utc::now(),
          "version": env!("CARGO_PKG_VERSION")
      })),
    )),
    Ok(Err(e)) => {
      warn!("ClickHouse health check failed: {}", e);
      Ok((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({
            "status": "unhealthy",
            "service": "analytics_server",
            "error": "Database connectivity issue",
            "timestamp": chrono::Utc::now()
        })),
      ))
    }
    Err(_) => {
      warn!("ClickHouse health check timed out");
      Ok((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({
            "status": "unhealthy",
            "service": "analytics_server",
            "error": "Database timeout",
            "timestamp": chrono::Utc::now()
        })),
      ))
    }
  }
}

/// Create batch analytics events
///
/// This endpoint receives multiple analytics events in protobuf format and stores them
/// in ClickHouse for further analysis. Useful for reducing network overhead when 
/// multiple events need to be sent.
#[utoipa::path(
    post,
    path = "/api/batch",
    request_body(
        content = Vec<u8>,
        description = "Batch analytics events in protobuf format (BatchRecordEventsRequest)",
        content_type = "application/protobuf"
    ),
    responses(
        (status = 201, description = "Events created successfully"),
        (status = 400, description = "Invalid event data", body = ErrorOutput),
        (status = 500, description = "Internal server error", body = ErrorOutput),
    ),
    tag = "analytics"
)]
#[instrument(skip(parts, state, body), fields(event_count))]
pub(crate) async fn create_batch_events_handler(
  parts: Parts,
  State(state): State<AppState>,
  body: Bytes,
) -> Result<impl IntoResponse, AppError> {
  use crate::pb::BatchRecordEventsRequest;
  
  // Parse protobuf batch request
  let batch_request = BatchRecordEventsRequest::decode(body.as_ref())
    .map_err(|e| AppError::AnyError(anyhow::anyhow!("Failed to decode protobuf: {}", e)))?;
  
  let event_count = batch_request.events.len();
  tracing::Span::current().record("event_count", event_count);
  
  if event_count == 0 {
    info!("📭 [ANALYTICS] Empty batch received, nothing to process");
    return Ok((
      StatusCode::OK,
      Json(json!({
        "status": "success",
        "message": "No events to process",
        "processed": 0
      })),
    ));
  }
  
  info!("EVENT: [ANALYTICS] Processing HTTP batch with {} events", event_count);
  
  // Extract geo information from headers
  let geo = Geo::from_parts(&parts);
  
  let mut processed = 0;
  let mut failed = 0;
  let mut rows = Vec::new();
  
  // Convert all events to rows
  for (index, event) in batch_request.events.into_iter().enumerate() {
    state.metrics.increment_events_received();
    
    match AnalyticsEventRow::try_from(event) {
      Ok(mut row) => {
        // Enhance with server-side information
        row.update_with_server_info(&parts, geo.0.clone());
        row.set_session_id(&state);
        rows.push(row);
        if (index + 1) % 50 == 0 {
          info!("🔄 [ANALYTICS] Converted {}/{} batch events to rows", index + 1, event_count);
        }
      }
      Err(e) => {
        warn!("ERROR: [ANALYTICS] Failed to convert batch event {} to row: {}", index, e);
        state.metrics.increment_events_failed();
        failed += 1;
      }
    }
  }
  
  info!("[ANALYTICS] Batch conversion completed: {} successful, {} failed", rows.len(), failed);
  
  // Batch insert into ClickHouse
  if !rows.is_empty() {
    info!("💾 [ANALYTICS] Inserting {} events into ClickHouse...", rows.len());
    
    let insert_result = tokio::time::timeout(
      std::time::Duration::from_secs(30), // 30 second timeout for batch
      insert_batch_analytics_events(&state, &rows),
    )
    .await;
    
    match insert_result {
      Ok(Ok(())) => {
        processed = rows.len();
        for _ in 0..processed {
          state.metrics.increment_events_processed();
        }
        info!("[ANALYTICS] Batch successfully stored {} events in ClickHouse", processed);
      }
      Ok(Err(e)) => {
        warn!("ERROR: [ANALYTICS] Failed to insert batch into ClickHouse: {}", e);
        failed += rows.len();
        for _ in 0..rows.len() {
          state.metrics.increment_events_failed();
          state.metrics.increment_database_errors();
        }
        return Err(e);
      }
      Err(_) => {
        warn!("⏰ [ANALYTICS] Batch insertion timed out after 30s");
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
    info!("📭 [ANALYTICS] No valid events to insert after batch processing");
  }
  
  info!("[ANALYTICS] Batch processing completed: {} processed, {} failed, {} total", 
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

/// Insert analytics event into ClickHouse with proper error handling
pub(crate) async fn insert_analytics_event(state: &AppState, row: &AnalyticsEventRow) -> Result<(), AppError> {
  let mut insert = state.client.insert("analytics_events")?;
  insert.write(row).await?;
  insert.end().await?;
  Ok(())
}

/// Insert batch analytics events into ClickHouse with proper error handling
pub(crate) async fn insert_batch_analytics_events(state: &AppState, rows: &[AnalyticsEventRow]) -> Result<(), AppError> {
  let mut insert = state.client.insert("analytics_events")?;
  for row in rows {
    insert.write(row).await?;
  }
  insert.end().await?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::pb::*;

  fn create_test_event() -> AnalyticsEvent {
    AnalyticsEvent {
      context: Some(EventContext {
        client_id: "test_client".to_string(),
        session_id: "test_session".to_string(),
        user_id: "test_user".to_string(),
        app_version: "1.0.0".to_string(),
        client_ts: chrono::Utc::now().timestamp_millis(),
        server_ts: 0,
        user_agent: "test".to_string(),
        ip: "127.0.0.1".to_string(),
        system: Some(SystemInfo {
          os: "linux".to_string(),
          arch: "x86_64".to_string(),
          locale: "en-US".to_string(),
          timezone: "UTC".to_string(),
          browser: "test".to_string(),
          browser_version: "1.0".to_string(),
        }),
        geo: None,
      }),
      event_type: Some(analytics_event::EventType::AppStart(AppStartEvent {})),
    }
  }

  #[test]
  fn test_event_conversion() {
    let event = create_test_event();
    let row = AnalyticsEventRow::try_from(event);
    assert!(row.is_ok());

    let row = row.unwrap();
    assert_eq!(row.client_id, "test_client");
    assert_eq!(row.event_type, "app_start");
  }
}
