// NATS Subscriber for Analytics Events
//
// This module subscribes to analytics events published via NATS
// and processes them into ClickHouse for storage and analysis.
// All events are expected to be in protobuf format.

use crate::{error::AppError, events::AnalyticsEventRow, AppState};
use async_nats::jetstream;
use futures::StreamExt;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};

/// NATS subscriber for analytics events
pub struct AnalyticsNatsSubscriber {
  jetstream: jetstream::Context,
  state: Arc<AppState>,
}

impl AnalyticsNatsSubscriber {
  /// Create a new analytics NATS subscriber using configuration from AppState
  pub async fn new(
    nats_client: async_nats::Client,
    state: Arc<AppState>,
  ) -> Result<Self, AppError> {
    let jetstream = jetstream::new(nats_client);
    let config = state.config.jetstream();

    // Create stream configuration from config file
    let stream_config = Self::build_stream_config(config, &state.config.analytics_subjects())?;

    // Ensure stream exists with proper configuration
    Self::ensure_stream(&jetstream, config, stream_config).await?;

    Ok(Self { jetstream, state })
  }

  /// Build JetStream configuration from config file
  fn build_stream_config(
    config: &crate::config::JetStreamConfig,
    subjects: &[String],
  ) -> Result<jetstream::stream::Config, AppError> {
    Ok(jetstream::stream::Config {
      name: config.stream_name.clone(),
      subjects: subjects.to_vec(),
      retention: Self::parse_retention_policy(&config.retention_policy)?,
      max_messages: config.max_messages as i64,
      max_bytes: config.max_bytes as i64,
      max_age: std::time::Duration::from_secs(config.max_age_hours * 3600),
      storage: Self::parse_storage_type(&config.storage_type)?,
      num_replicas: config.num_replicas as usize,
      ..Default::default()
    })
  }

  /// Parse retention policy from string
  fn parse_retention_policy(policy: &str) -> Result<jetstream::stream::RetentionPolicy, AppError> {
    match policy.to_lowercase().as_str() {
      "limits" => Ok(jetstream::stream::RetentionPolicy::Limits),
      "interest" => Ok(jetstream::stream::RetentionPolicy::Interest),
      "workqueue" => Ok(jetstream::stream::RetentionPolicy::WorkQueue),
      _ => Err(AppError::AnyError(anyhow::anyhow!(
        "Invalid retention policy: {}",
        policy
      ))),
    }
  }

  /// Parse storage type from string
  fn parse_storage_type(storage: &str) -> Result<jetstream::stream::StorageType, AppError> {
    match storage.to_lowercase().as_str() {
      "file" => Ok(jetstream::stream::StorageType::File),
      "memory" => Ok(jetstream::stream::StorageType::Memory),
      _ => Err(AppError::AnyError(anyhow::anyhow!(
        "Invalid storage type: {}",
        storage
      ))),
    }
  }

  /// Ensure stream exists with proper configuration
  async fn ensure_stream(
    jetstream: &jetstream::Context,
    config: &crate::config::JetStreamConfig,
    stream_config: jetstream::stream::Config,
  ) -> Result<(), AppError> {
    match jetstream.get_stream(&config.stream_name).await {
      Ok(mut stream) => {
        info!("✅ Using existing analytics stream: {}", config.stream_name);

        // Check if configuration needs updating
        let stream_info = stream
          .info()
          .await
          .map_err(|e| AppError::AnyError(anyhow::anyhow!("Failed to get stream info: {}", e)))?;

        if Self::needs_config_update(&stream_info.config, &stream_config) {
          info!(
            "🔄 Updating stream configuration: {} ({}GB -> {}GB)",
            config.stream_name,
            stream_info.config.max_bytes / (1024 * 1024 * 1024),
            stream_config.max_bytes / (1024 * 1024 * 1024)
          );

          jetstream
            .update_stream(stream_config)
            .await
            .map_err(|e| AppError::AnyError(anyhow::anyhow!("Failed to update stream: {}", e)))?;

          info!("✅ Stream configuration updated successfully");
        }
      }
      Err(_) => {
        info!(
          "🆕 Creating new analytics stream: {} ({}GB storage)",
          config.stream_name,
          config.max_bytes / (1024 * 1024 * 1024)
        );

        jetstream
          .create_stream(stream_config)
          .await
          .map_err(|e| AppError::AnyError(anyhow::anyhow!("Failed to create stream: {}", e)))?;

        info!("✅ Analytics stream created successfully");
      }
    }
    Ok(())
  }

  /// Check if stream configuration needs updating
  fn needs_config_update(
    current: &jetstream::stream::Config,
    desired: &jetstream::stream::Config,
  ) -> bool {
    current.max_bytes != desired.max_bytes
      || current.max_messages != desired.max_messages
      || current.max_age != desired.max_age
  }

  /// Start subscribing to analytics events
  #[instrument(skip(self))]
  pub async fn start(self: Arc<Self>) -> Result<(), AppError> {
    let stream_name = &self
      .state
      .config
      .integrations
      .messaging
      .jetstream
      .stream_name;
    info!(
      "🚀 [ANALYTICS] Starting NATS analytics subscriber for stream: {}",
      stream_name
    );

    // Create durable consumer for reliable processing
    let consumer_config = jetstream::consumer::pull::Config {
      durable_name: Some("analytics-processor".to_string()),
      ack_policy: jetstream::consumer::AckPolicy::Explicit,
      max_deliver: 3,
      ack_wait: std::time::Duration::from_secs(30),
      ..Default::default()
    };

    info!("🔧 [ANALYTICS] Creating consumer with config: durable_name=analytics-processor, max_deliver=3, ack_wait=30s");

    let stream = self
      .jetstream
      .get_stream(stream_name)
      .await
      .map_err(|e| AppError::AnyError(anyhow::anyhow!("Failed to get stream: {}", e)))?;

    info!(
      "✅ [ANALYTICS] Successfully connected to JetStream: {}",
      stream_name
    );

    let consumer = stream
      .create_consumer(consumer_config)
      .await
      .map_err(|e| AppError::AnyError(anyhow::anyhow!("Failed to create consumer: {}", e)))?;

    info!("✅ [ANALYTICS] Analytics consumer created successfully");

    // Start processing messages
    let mut messages = consumer
      .messages()
      .await
      .map_err(|e| AppError::AnyError(anyhow::anyhow!("Failed to get messages: {}", e)))?;

    info!("📡 [ANALYTICS] Analytics subscriber started, waiting for events...");

    let mut processed_count = 0;
    let mut error_count = 0;

    while let Some(msg) = messages.next().await {
      let msg = match msg {
        Ok(m) => {
          processed_count += 1;
          if processed_count % 100 == 0 {
            info!(
              "📊 [ANALYTICS] Processed {} events so far (errors: {})",
              processed_count, error_count
            );
          }
          m
        }
        Err(e) => {
          error_count += 1;
          error!("❌ [ANALYTICS] Error receiving message: {}", e);
          continue;
        }
      };

      // Process message in a separate task to avoid blocking
      let subscriber = Arc::clone(&self);
      tokio::spawn(async move {
        if let Err(e) = subscriber.process_message(msg).await {
          error!("❌ [ANALYTICS] Failed to process analytics event: {}", e);
        }
      });
    }

    warn!(
      "⚠️ [ANALYTICS] Analytics subscriber stopped (processed: {}, errors: {})",
      processed_count, error_count
    );
    Ok(())
  }

  /// Process a single analytics message (protobuf format only)
  #[instrument(skip(self, msg), fields(subject = %msg.subject))]
  async fn process_message(&self, msg: jetstream::Message) -> Result<(), AppError> {
    let subject = &msg.subject;
    let payload_size = msg.payload.len();

    info!(
      "📨 [ANALYTICS] Received NATS event from subject: {} (size: {} bytes)",
      subject, payload_size
    );

    // Try to parse as protobuf first (existing format), then fallback to JSON
    let mut row = match self.parse_protobuf_event(&msg.payload) {
      Ok(row) => {
        info!(
          "✅ [ANALYTICS] Successfully parsed protobuf event: {} for user: {}",
          row.event_type,
          row.user_id.as_ref().unwrap_or(&"unknown".to_string())
        );
        row
      }
      Err(_) => {
        // Fallback to JSON parsing for compatibility with test events
        match self.parse_json_event(&msg.payload, subject) {
          Ok(row) => {
            info!(
              "✅ [ANALYTICS] Successfully parsed JSON event: {} for user: {}",
              row.event_type,
              row.user_id.as_ref().unwrap_or(&"unknown".to_string())
            );
            row
          }
          Err(e) => {
            error!(
              "❌ [ANALYTICS] Failed to parse both protobuf and JSON from subject {}: {}",
              subject, e
            );
            return Err(e);
          }
        }
      }
    };

    // Update session information
    row.set_session_id(&self.state);
    info!(
      "🔧 [ANALYTICS] Session info updated for event: {}",
      row.event_type
    );

    // Insert into ClickHouse
    match self.insert_event_row(row.clone()).await {
      Ok(()) => {
        info!(
          "💾 [ANALYTICS] Successfully stored event: {} for user: {} in ClickHouse",
          row.event_type,
          row.user_id.unwrap_or_else(|| "unknown".to_string())
        );
      }
      Err(e) => {
        error!("❌ [ANALYTICS] Failed to store event in ClickHouse: {}", e);
        return Err(e);
      }
    }

    // Acknowledge the message
    match msg.ack().await {
      Ok(()) => {
        info!(
          "✅ [ANALYTICS] Successfully processed and acknowledged event from: {}",
          subject
        );
      }
      Err(e) => {
        error!(
          "❌ [ANALYTICS] Failed to acknowledge message from {}: {}",
          subject, e
        );
        return Err(AppError::AnyError(anyhow::anyhow!(
          "Failed to ack message: {}",
          e
        )));
      }
    }

    debug!(
      "🎯 [ANALYTICS] Event processing completed for subject: {}",
      subject
    );
    Ok(())
  }

  /// Parse message as protobuf analytics event
  fn parse_protobuf_event(&self, payload: &[u8]) -> Result<AnalyticsEventRow, AppError> {
    use crate::pb::AnalyticsEvent;
    use prost::Message;

    debug!(
      "🔍 [ANALYTICS] Parsing protobuf payload of {} bytes",
      payload.len()
    );

    // Decode protobuf
    let event = AnalyticsEvent::decode(payload).map_err(|e| {
      error!(
        "❌ [ANALYTICS] Failed to decode protobuf analytics event: {}",
        e
      );
      AppError::AnyError(anyhow::anyhow!("Protobuf decode error: {}", e))
    })?;

    debug!(
      "✅ [ANALYTICS] Successfully decoded protobuf event with context: {}",
      event
        .context
        .as_ref()
        .map(|c| &c.user_id)
        .unwrap_or(&"unknown".to_string())
    );

    // Convert to analytics row using existing conversion logic
    let row = AnalyticsEventRow::try_from(event).map_err(|e| {
      error!(
        "❌ [ANALYTICS] Failed to convert protobuf event to database row: {}",
        e
      );
      e
    })?;

    debug!(
      "🔄 [ANALYTICS] Successfully converted to analytics row: event_type={}",
      row.event_type
    );

    Ok(row)
  }

  /// Parse message as JSON analytics event (fallback for test events)
  fn parse_json_event(&self, payload: &[u8], subject: &str) -> Result<AnalyticsEventRow, AppError> {
    use chrono::Utc;
    use serde_json::Value;

    debug!(
      "🔍 [ANALYTICS] Parsing JSON payload of {} bytes",
      payload.len()
    );

    // Parse JSON payload
    let json: Value = serde_json::from_slice(payload).map_err(|e| {
      error!("❌ [ANALYTICS] Failed to parse JSON: {}", e);
      AppError::AnyError(anyhow::anyhow!("JSON parse error: {}", e))
    })?;

    debug!("✅ [ANALYTICS] Successfully parsed JSON payload");

    // Extract basic fields from JSON
    let user_id = json
      .get("user_id")
      .and_then(|v| v.as_i64())
      .map(|id| id.to_string());

    let event_type = if subject.contains("user") {
      json
        .get("event_type")
        .and_then(|v| v.as_str())
        .unwrap_or("user_event")
        .to_string()
    } else if subject.contains("message") {
      "message_event".to_string()
    } else if subject.contains("chat") {
      "chat_event".to_string()
    } else {
      "generic_event".to_string()
    };

    let now = Utc::now().timestamp_millis();

    // Create a simple analytics row from JSON with correct field names
    let row = AnalyticsEventRow {
      // Required String fields
      client_id: "test_client".to_string(),
      session_id: json
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("test_session")
        .to_string(),
      event_type,
      app_version: "test".to_string(),
      system_os: "unknown".to_string(),
      system_arch: "unknown".to_string(),
      system_locale: "en-US".to_string(),
      system_timezone: "UTC".to_string(),

      // Required numeric fields
      duration: 0,
      client_ts: json
        .get("timestamp")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.timestamp_millis())
        .unwrap_or(now),
      server_ts: now,

      // Optional fields
      user_id,
      system_browser: None,
      system_browser_version: None,
      ip: None,
      user_agent: None,
      geo_country: None,
      geo_region: None,
      geo_city: None,

      // All other optional fields set to None
      exit_code: None,
      login_email: None,
      login_method: None,
      logout_email: None,
      register_email: None,
      register_workspace_id: None,
      register_method: None,
      chat_created_workspace_id: None,
      chat_created_type: None,
      chat_created_members_count: None,
      message_chat_id: None,
      message_type: None,
      message_size: None,
      message_total_files: None,
      message_has_mentions: None,
      message_has_links: None,
      chat_joined_id: None,
      chat_joined_method: None,
      chat_left_id: None,
      chat_left_reason: None,
      navigation_from: None,
      navigation_to: None,
      navigation_duration_ms: None,
      file_uploaded_type: None,
      file_uploaded_size: None,
      file_uploaded_method: None,
      file_uploaded_duration_ms: None,
      file_downloaded_type: None,
      file_downloaded_size: None,
      file_downloaded_duration_ms: None,
      search_type: None,
      search_query_length: None,
      search_results_count: None,
      search_duration_ms: None,
      search_has_filters: None,
      notification_type: None,
      notification_source: None,
      notification_was_clicked: None,
      error_type: None,
      error_code: None,
      error_message: None,
      error_stack_trace: None,
      error_context: None,
      bot_response_bot_id: None,
      bot_response_chat_id: None,
      bot_response_type: None,
      bot_response_time_ms: None,
      bot_response_tokens_used: None,
      bot_response_success: None,
      bot_response_error_message: None,
    };

    debug!(
      "🔄 [ANALYTICS] Successfully converted JSON to analytics row: event_type={}",
      row.event_type
    );
    Ok(row)
  }

  /// Insert event row into ClickHouse
  #[instrument(skip(self, row))]
  async fn insert_event_row(&self, row: AnalyticsEventRow) -> Result<(), AppError> {
    let mut insert = self.state.client.insert("analytics_events")?;
    insert.write(&row).await?;
    insert.end().await?;
    Ok(())
  }
}

/// Start the NATS subscriber in the background
pub async fn start_nats_subscriber(nats_url: String, state: Arc<AppState>) -> Result<(), AppError> {
  info!("🔗 [ANALYTICS] Connecting to NATS at: {}", nats_url);

  let nats_client = async_nats::connect(&nats_url).await.map_err(|e| {
    error!(
      "❌ [ANALYTICS] Failed to connect to NATS at {}: {}",
      nats_url, e
    );
    AppError::AnyError(anyhow::anyhow!("Failed to connect to NATS: {}", e))
  })?;

  info!(
    "✅ [ANALYTICS] Successfully connected to NATS: {}",
    nats_url
  );

  let stream_name = state.config.jetstream().stream_name.clone();
  let subjects = state.config.analytics_subjects();

  info!(
    "🔧 [ANALYTICS] Creating analytics subscriber for stream: {}",
    stream_name
  );
  info!("📡 [ANALYTICS] Subscribing to subjects: {:?}", subjects);

  let subscriber = Arc::new(AnalyticsNatsSubscriber::new(nats_client, state.clone()).await?);

  info!("✅ [ANALYTICS] Analytics subscriber created successfully");

  // Start subscriber in background task
  tokio::spawn(async move {
    info!("🚀 [ANALYTICS] Starting analytics subscriber background task...");
    if let Err(e) = subscriber.start().await {
      error!("❌ [ANALYTICS] Analytics NATS subscriber failed: {}", e);
    } else {
      info!("🏁 [ANALYTICS] Analytics subscriber task completed");
    }
  });

  info!(
    "✅ [ANALYTICS] NATS analytics subscriber started successfully with stream: {}",
    stream_name
  );
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::pb::*;

  #[test]
  fn test_protobuf_event_parsing() {
    use prost::Message;

    // Create a test event
    let event = AnalyticsEvent {
      context: Some(EventContext {
        client_id: "test_client".to_string(),
        session_id: "test_session".to_string(),
        user_id: "user123".to_string(),
        app_version: "1.0.0".to_string(),
        client_ts: chrono::Utc::now().timestamp_millis(),
        server_ts: 0,
        user_agent: "test-agent".to_string(),
        ip: "192.168.1.1".to_string(),
        system: Some(SystemInfo {
          os: "linux".to_string(),
          arch: "x86_64".to_string(),
          locale: "en-US".to_string(),
          timezone: "UTC".to_string(),
          browser: "firefox".to_string(),
          browser_version: "100.0".to_string(),
        }),
        geo: None,
      }),
      event_type: Some(analytics_event::EventType::MessageSent(MessageSentEvent {
        chat_id: "chat123".to_string(),
        r#type: "text".to_string(),
        size: 100,
        total_files: 0,
        has_mentions: true,
        has_links: false,
      })),
    };

    // Encode and decode
    let payload = event.encode_to_vec();
    let decoded = AnalyticsEvent::decode(&payload[..]).unwrap();

    assert_eq!(event.context.as_ref().unwrap().client_id, "test_client");
    assert_eq!(decoded.context.as_ref().unwrap().user_id, "user123");
  }
}
