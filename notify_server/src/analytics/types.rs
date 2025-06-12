// Analytics types for notify_server
//
// This module provides wrapper types and helper functions for creating
// notification-specific analytics events using the standard protobuf definitions.

use chrono::{DateTime, Utc};
use fechatter_core::UserId;
use fechatter_protos::fechatter::v1::{
    AnalyticsEvent, EventContext, SystemInfo,
    analytics_event::EventType,
    AppStartEvent, AppExitEvent, NotificationReceivedEvent,
};

// Re-export standard protobuf types with clear aliases
pub use fechatter_protos::fechatter::v1::AnalyticsEvent as NotifyAnalyticsEvent;

/// Helper functions for creating notification-specific analytics events
pub struct NotifyEventHelper;

impl NotifyEventHelper {
    /// Create user connected event (maps to AppStartEvent)
    pub fn user_connected(
        user_id: UserId,
        _connection_id: String,
        user_agent: Option<String>,
    ) -> AnalyticsEvent {
        AnalyticsEvent {
            context: Some(Self::create_context(user_id, user_agent)),
            event_type: Some(EventType::AppStart(AppStartEvent {})),
        }
    }

    /// Create user disconnected event (maps to AppExitEvent) 
    pub fn user_disconnected(
        user_id: UserId,
        _connection_id: String,
        _duration_ms: u64,
    ) -> AnalyticsEvent {
        AnalyticsEvent {
            context: Some(Self::create_context(user_id, None)),
            event_type: Some(EventType::AppExit(AppExitEvent {
                exit_code: fechatter_protos::fechatter::v1::app_exit_event::ExitCode::Success as i32,
            })),
        }
    }

    /// Create notification received event
    pub fn notification_received(
        user_id: UserId,
        notification_type: String,
        source: String,
        was_delivered: bool,
        _delivery_duration_ms: Option<u64>,
    ) -> AnalyticsEvent {
        AnalyticsEvent {
            context: Some(Self::create_context(user_id, None)),
            event_type: Some(EventType::NotificationReceived(NotificationReceivedEvent {
                notification_type,
                source,
                was_clicked: was_delivered,
            })),
        }
    }

    /// Create event context for notify server
    fn create_context(user_id: UserId, user_agent: Option<String>) -> EventContext {
        let now = chrono::Utc::now().timestamp_millis();
        
        EventContext {
            client_id: format!("notify_server_{}", uuid::Uuid::new_v4()),
            session_id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.0.to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            client_ts: now,
            server_ts: now,
            user_agent: user_agent.unwrap_or_else(|| "notify_server".to_string()),
            ip: String::new(), // Will be filled by analytics server
            system: Some(SystemInfo {
                os: "server".to_string(),
                arch: std::env::consts::ARCH.to_string(),
                locale: "en-US".to_string(),
                timezone: "UTC".to_string(),
                browser: "notify_server".to_string(),
                browser_version: env!("CARGO_PKG_VERSION").to_string(),
            }),
            geo: None, // Will be filled by analytics server
        }
    }
}

/// Event metadata for tracking (legacy compatibility)
#[derive(Debug, Clone)]
pub struct EventMetadata {
    pub event_id: uuid::Uuid,
    pub user_id: UserId,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<String>,
}

impl EventMetadata {
    /// Create new event metadata
    pub fn new(user_id: UserId) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4(),
            user_id,
            timestamp: Utc::now(),
            session_id: None,
        }
    }

    /// Create with session ID
    pub fn with_session(user_id: UserId, session_id: String) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4(),
            user_id,
            timestamp: Utc::now(),
            session_id: Some(session_id),
        }
    }
} 