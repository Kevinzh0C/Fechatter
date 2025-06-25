use crate::{AppError, AppState, pb::*};
use axum::http::request::Parts;
use clickhouse::Row;
use fechatter_core::AuthUser;
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

const SESSION_TIMEOUT: i64 = 10 * 60 * 1000; // 10 minutes

#[derive(Debug, Default, Clone, Row, Serialize, Deserialize)]
pub struct AnalyticsEventRow {
  // EventContext fields
  pub client_id: String,
  pub session_id: String,
  pub duration: u32,
  pub app_version: String,
  pub system_os: String,
  pub system_arch: String,
  pub system_locale: String,
  pub system_timezone: String,
  pub system_browser: Option<String>,
  pub system_browser_version: Option<String>,
  pub user_id: Option<String>,
  pub ip: Option<String>,
  pub user_agent: Option<String>,
  pub geo_country: Option<String>,
  pub geo_region: Option<String>,
  pub geo_city: Option<String>,
  pub client_ts: i64,
  pub server_ts: i64,
  // Common fields
  pub event_type: String,
  // AppExitEvent fields
  pub exit_code: Option<String>,
  // UserLoginEvent
  pub login_email: Option<String>,
  pub login_method: Option<String>,
  // UserLogoutEvent
  pub logout_email: Option<String>,
  // UserRegisterEvent
  pub register_email: Option<String>,
  pub register_workspace_id: Option<String>,
  pub register_method: Option<String>,
  // ChatCreatedEvent
  pub chat_created_workspace_id: Option<String>,
  pub chat_created_type: Option<String>,
  pub chat_created_members_count: Option<i32>,
  // MessageSentEvent
  pub message_chat_id: Option<String>,
  pub message_type: Option<String>,
  pub message_size: Option<i32>,
  pub message_total_files: Option<i32>,
  pub message_has_mentions: Option<bool>,
  pub message_has_links: Option<bool>,
  // ChatJoinedEvent
  pub chat_joined_id: Option<String>,
  pub chat_joined_method: Option<String>,
  // ChatLeftEvent
  pub chat_left_id: Option<String>,
  pub chat_left_reason: Option<String>,
  // NavigationEvent
  pub navigation_from: Option<String>,
  pub navigation_to: Option<String>,
  pub navigation_duration_ms: Option<i32>,
  // FileUploadedEvent
  pub file_uploaded_type: Option<String>,
  pub file_uploaded_size: Option<i64>,
  pub file_uploaded_method: Option<String>,
  pub file_uploaded_duration_ms: Option<i32>,
  // FileDownloadedEvent
  pub file_downloaded_type: Option<String>,
  pub file_downloaded_size: Option<i64>,
  pub file_downloaded_duration_ms: Option<i32>,
  // SearchPerformedEvent
  pub search_type: Option<String>,
  pub search_query_length: Option<String>,
  pub search_results_count: Option<i32>,
  pub search_duration_ms: Option<i32>,
  pub search_has_filters: Option<bool>,
  // NotificationReceivedEvent
  pub notification_type: Option<String>,
  pub notification_source: Option<String>,
  pub notification_was_clicked: Option<bool>,
  // ErrorOccurredEvent
  pub error_type: Option<String>,
  pub error_code: Option<String>,
  pub error_message: Option<String>,
  pub error_stack_trace: Option<String>,
  pub error_context: Option<String>,
  
  // BotResponseEvent
  pub bot_response_bot_id: Option<String>,
  pub bot_response_chat_id: Option<String>,
  pub bot_response_type: Option<String>,
  pub bot_response_time_ms: Option<u64>,
  pub bot_response_tokens_used: Option<u32>,
  pub bot_response_success: Option<bool>,
  pub bot_response_error_message: Option<String>,
}

trait EventConsume {
  fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError>;
}

impl AnalyticsEventRow {
  /// Update analytics row with server-side information
  /// This ensures data integrity by overriding client-provided values
  /// that could be spoofed or inaccurate
  #[instrument(skip(self, parts, geo), fields(client_id = %self.client_id))]
  pub fn update_with_server_info(&mut self, parts: &Parts, geo: Option<GeoLocation>) {
    // Extract authenticated user info if available
    if let Some(user) = parts.extensions.get::<AuthUser>() {
      self.user_id = Some(user.id.to_string());
      info!("User authenticated for analytics event");
    } else {
      self.user_id = None;
    }

    // Use server-provided geo information for accuracy
    if let Some(geo) = geo {
      self.geo_country = Some(geo.country);
      self.geo_region = Some(geo.region);
      self.geo_city = Some(geo.city);
      info!("Geo information extracted from headers");
    } else {
      self.geo_country = None;
      self.geo_region = None;
      self.geo_city = None;
    }

    // Always use server timestamp for consistency
    self.server_ts = chrono::Utc::now().timestamp_millis();

    // Extract real IP address from headers (considering proxies)
    if let Some(forwarded_for) = parts.headers.get("x-forwarded-for") {
      if let Ok(ip_str) = forwarded_for.to_str() {
        // Take the first IP in the chain (original client)
        if let Some(first_ip) = ip_str.split(',').next() {
          self.ip = Some(first_ip.trim().to_string());
        }
      }
    } else if let Some(real_ip) = parts.headers.get("x-real-ip") {
      if let Ok(ip_str) = real_ip.to_str() {
        self.ip = Some(ip_str.to_string());
      }
    }
  }

  /// Manage session tracking with automatic session creation and expiration
  #[instrument(skip(self, state), fields(client_id = %self.client_id))]
  pub fn set_session_id(&mut self, state: &AppState) {
    if let Some(mut entry) = state.sessions.get_mut(&self.client_id) {
      let (session_id, last_server_ts) = entry.value_mut();
      let mut duration = self.server_ts - *last_server_ts;

      if duration < 0 {
        warn!(
          session_id = %session_id,
          client_id = %self.client_id,
          "Negative session duration detected, resetting to 0"
        );
        duration = 0;
      }

      if duration < SESSION_TIMEOUT {
        // Continue existing session
        self.session_id = session_id.clone();
        self.duration = duration as u32;
        *last_server_ts = self.server_ts;

        info!(
          session_id = %session_id,
          duration_ms = %duration,
          "Continuing existing session"
        );
      } else {
        // Session expired, create new one
        let new_session_id = Uuid::now_v7().to_string();
        self.session_id = new_session_id.clone();
        self.duration = 0;

        info!(
          old_session_id = %session_id,
          new_session_id = %new_session_id,
          expired_duration_ms = %duration,
          "Session expired, creating new session"
        );

        *last_server_ts = self.server_ts;
        *session_id = new_session_id;
      }
    } else {
      // First time seeing this client, create new session
      let session_id = Uuid::now_v7().to_string();
      self.session_id = session_id.clone();
      self.duration = 0;

      info!(
        session_id = %session_id,
        client_id = %self.client_id,
        "Creating new session for client"
      );

      state
        .sessions
        .insert(self.client_id.clone(), (session_id, self.server_ts));
    }
  }

  /// Validate the analytics event data for consistency and security
  pub fn validate(&self) -> Result<(), AppError> {
    // Basic validation
    if self.client_id.is_empty() {
      return Err(AppError::AnyError(anyhow::anyhow!(
        "Client ID cannot be empty"
      )));
    }

    if self.app_version.is_empty() {
      return Err(AppError::AnyError(anyhow::anyhow!(
        "App version cannot be empty"
      )));
    }

    // Validate timestamps
    if self.client_ts <= 0 {
      return Err(AppError::AnyError(anyhow::anyhow!(
        "Invalid client timestamp"
      )));
    }

    if self.server_ts <= 0 {
      return Err(AppError::AnyError(anyhow::anyhow!(
        "Invalid server timestamp"
      )));
    }

    // Check for reasonable timestamp skew (within 24 hours)
    let skew = (self.server_ts - self.client_ts).abs();
    if skew > 24 * 60 * 60 * 1000 {
      warn!(
        client_ts = %self.client_ts,
        server_ts = %self.server_ts,
        skew_ms = %skew,
        "Large timestamp skew detected"
      );
    }

    Ok(())
  }
}

impl TryFrom<AnalyticsEvent> for AnalyticsEventRow {
  type Error = crate::AppError;

  #[instrument(skip(event))]
  fn try_from(event: AnalyticsEvent) -> Result<Self, Self::Error> {
    let mut ret = Self::default();

    // Process event context
    match event.context {
      Some(context) => {
        context.consume(&mut ret).map_err(|e| {
          error!("Failed to process event context: {}", e);
          e
        })?;
      }
      None => {
        error!("Analytics event missing context");
        return Err(AppError::MissingEventContext);
      }
    }

    // Process event data
    match event.event_type {
      Some(event_type) => {
        event_type.consume(&mut ret).map_err(|e| {
          error!("Failed to process event type: {}", e);
          e
        })?;
      }
      None => {
        error!("Analytics event missing event type");
        return Err(AppError::MissingEventData);
      }
    }

    // Validate the resulting row
    ret.validate()?;

    Ok(ret)
  }
}

impl EventConsume for EventContext {
  fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
    row.client_id = self.client_id;
    row.app_version = self.app_version;

    if let Some(system) = self.system {
      row.system_os = system.os;
      row.system_arch = system.arch;
      row.system_locale = system.locale;
      row.system_timezone = system.timezone;
      if !system.browser.is_empty() {
        row.system_browser = Some(system.browser);
      }
      if !system.browser_version.is_empty() {
        row.system_browser_version = Some(system.browser_version);
      }
    } else {
      return Err(AppError::MissingSystemInfo);
    }

    if !self.user_id.is_empty() {
      row.user_id = Some(self.user_id);
    }

    if !self.ip.is_empty() {
      row.ip = Some(self.ip);
    }
    if !self.user_agent.is_empty() {
      row.user_agent = Some(self.user_agent);
    }

    if let Some(geo) = self.geo {
      row.geo_country = Some(geo.country);
      row.geo_region = Some(geo.region);
      row.geo_city = Some(geo.city);
    }

    row.client_ts = self.client_ts;
    row.server_ts = self.server_ts;
    Ok(())
  }
}

impl EventConsume for analytics_event::EventType {
  fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
    match self {
      analytics_event::EventType::AppExit(event) => event.consume(row),
      analytics_event::EventType::AppStart(event) => event.consume(row),
      analytics_event::EventType::UserLogin(event) => event.consume(row),
      analytics_event::EventType::UserLogout(event) => event.consume(row),
      analytics_event::EventType::UserRegister(event) => event.consume(row),
      analytics_event::EventType::ChatCreated(event) => event.consume(row),
      analytics_event::EventType::MessageSent(event) => event.consume(row),
      analytics_event::EventType::ChatJoined(event) => event.consume(row),
      analytics_event::EventType::ChatLeft(event) => event.consume(row),
      analytics_event::EventType::Navigation(event) => event.consume(row),
      analytics_event::EventType::FileUploaded(event) => event.consume(row),
      analytics_event::EventType::FileDownloaded(event) => event.consume(row),
      analytics_event::EventType::SearchPerformed(event) => event.consume(row),
      analytics_event::EventType::NotificationReceived(event) => event.consume(row),
      analytics_event::EventType::ErrorOccurred(event) => event.consume(row),
      analytics_event::EventType::BotResponse(event) => event.consume(row),
    }
  }
}

impl EventConsume for AppStartEvent {
  fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
    row.event_type = "app_start".to_string();
    Ok(())
  }
}

impl EventConsume for AppExitEvent {
  fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
    row.event_type = "app_exit".to_string();
    row.exit_code = Some(
      app_exit_event::ExitCode::try_from(self.exit_code)
        .unwrap_or(app_exit_event::ExitCode::Unspecified)
        .as_str_name()
        .to_string(),
    );
    Ok(())
  }
}

impl EventConsume for UserLoginEvent {
  fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
    row.event_type = "user_login".to_string();
    row.login_email = Some(self.email);
    if !self.login_method.is_empty() {
      row.login_method = Some(self.login_method);
    }
    Ok(())
  }
}

impl EventConsume for UserLogoutEvent {
  fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
    row.event_type = "user_logout".to_string();
    row.logout_email = Some(self.email);
    Ok(())
  }
}

impl EventConsume for UserRegisterEvent {
  fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
    row.event_type = "user_register".to_string();
    row.register_email = Some(self.email);
    row.register_workspace_id = Some(self.workspace_id);
    if !self.registration_method.is_empty() {
      row.register_method = Some(self.registration_method);
    }
    Ok(())
  }
}

impl EventConsume for ChatCreatedEvent {
  fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
    row.event_type = "chat_created".to_string();
    row.chat_created_workspace_id = Some(self.workspace_id);
    row.chat_created_type = Some(self.chat_type);
    row.chat_created_members_count = Some(self.initial_members_count);
    Ok(())
  }
}

impl EventConsume for MessageSentEvent {
  fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
    row.event_type = "message_sent".to_string();
    row.message_chat_id = Some(self.chat_id);
    row.message_type = Some(self.r#type);
    row.message_size = Some(self.size);
    row.message_total_files = Some(self.total_files);
    row.message_has_mentions = Some(self.has_mentions);
    row.message_has_links = Some(self.has_links);
    Ok(())
  }
}

impl EventConsume for ChatJoinedEvent {
  fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
    row.event_type = "chat_joined".to_string();
    row.chat_joined_id = Some(self.chat_id);
    if !self.join_method.is_empty() {
      row.chat_joined_method = Some(self.join_method);
    }
    Ok(())
  }
}

impl EventConsume for ChatLeftEvent {
  fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
    row.event_type = "chat_left".to_string();
    row.chat_left_id = Some(self.chat_id);
    if !self.leave_reason.is_empty() {
      row.chat_left_reason = Some(self.leave_reason);
    }
    Ok(())
  }
}

impl EventConsume for NavigationEvent {
  fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
    row.event_type = "navigation".to_string();
    row.navigation_from = Some(self.from);
    row.navigation_to = Some(self.to);
    row.navigation_duration_ms = Some(self.duration_ms);
    Ok(())
  }
}

impl EventConsume for FileUploadedEvent {
  fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
    row.event_type = "file_uploaded".to_string();
    row.file_uploaded_type = Some(self.file_type);
    row.file_uploaded_size = Some(self.file_size);
    row.file_uploaded_method = Some(self.upload_method);
    row.file_uploaded_duration_ms = Some(self.upload_duration_ms);
    Ok(())
  }
}

impl EventConsume for FileDownloadedEvent {
  fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
    row.event_type = "file_downloaded".to_string();
    row.file_downloaded_type = Some(self.file_type);
    row.file_downloaded_size = Some(self.file_size);
    row.file_downloaded_duration_ms = Some(self.download_duration_ms);
    Ok(())
  }
}

impl EventConsume for SearchPerformedEvent {
  fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
    row.event_type = "search_performed".to_string();
    row.search_type = Some(self.search_type);
    row.search_query_length = Some(self.query_length);
    row.search_results_count = Some(self.results_count);
    row.search_duration_ms = Some(self.search_duration_ms);
    row.search_has_filters = Some(self.has_filters);
    Ok(())
  }
}

impl EventConsume for NotificationReceivedEvent {
  fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
    row.event_type = "notification_received".to_string();
    row.notification_type = Some(self.notification_type);
    row.notification_source = Some(self.source);
    row.notification_was_clicked = Some(self.was_clicked);
    Ok(())
  }
}

impl EventConsume for ErrorOccurredEvent {
  fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
    row.event_type = "error_occurred".to_string();
    row.error_type = Some(self.error_type);
    row.error_code = Some(self.error_code);
    row.error_message = Some(self.error_message);
    if !self.stack_trace.is_empty() {
      row.error_stack_trace = Some(self.stack_trace);
    }
    if !self.context.is_empty() {
      row.error_context = Some(self.context);
    }
    Ok(())
  }
}

impl EventConsume for BotResponseEvent {
  fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
    row.event_type = "bot_response".to_string();
    // Use dedicated BotResponseEvent fields
    row.bot_response_bot_id = Some(self.bot_id);
    row.bot_response_chat_id = Some(self.chat_id);
    row.bot_response_type = Some(self.response_type);
    row.bot_response_time_ms = Some(self.response_time_ms);
    row.bot_response_tokens_used = Some(self.tokens_used);
    row.bot_response_success = Some(self.success);
    if !self.error_message.is_empty() {
      row.bot_response_error_message = Some(self.error_message);
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn create_test_context() -> EventContext {
    EventContext {
      client_id: "test_client".to_string(),
      session_id: "test_session".to_string(),
      user_id: "test_user".to_string(),
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
      geo: Some(GeoLocation {
        country: "US".to_string(),
        region: "CA".to_string(),
        city: "San Francisco".to_string(),
      }),
    }
  }

  #[test]
  fn test_event_context_consumption() {
    let context = create_test_context();
    let mut row = AnalyticsEventRow::default();

    context.consume(&mut row).unwrap();

    assert_eq!(row.client_id, "test_client");
    assert_eq!(row.app_version, "1.0.0");
    assert_eq!(row.system_os, "linux");
    assert_eq!(row.geo_country, Some("US".to_string()));
  }

  #[test]
  fn test_row_validation() {
    let mut row = AnalyticsEventRow::default();

    // Should fail with empty client_id
    assert!(row.validate().is_err());

    row.client_id = "test".to_string();
    row.app_version = "1.0.0".to_string();
    row.client_ts = chrono::Utc::now().timestamp_millis();
    row.server_ts = chrono::Utc::now().timestamp_millis();

    // Should pass with valid data
    assert!(row.validate().is_ok());
  }
}
