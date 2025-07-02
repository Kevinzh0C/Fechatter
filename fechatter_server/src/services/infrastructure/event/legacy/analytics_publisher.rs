//! Unified Analytics Event Publisher
//!
//! Provides analytics event publishing through NATS with protobuf format
//! Replaces HTTP/gRPC analytics clients with NATS-based approach

use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use prost::Message;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::error::AppError;
use crate::services::infrastructure::event::EventTransport;
use crate::services::infrastructure::event::NatsTransport;
use analytics_server::pb::*;

/// Analytics publisher configuration
#[derive(Debug, Clone)]
pub struct AnalyticsConfig {
    pub enabled: bool,
    pub subject_prefix: String,
    pub batch_size: usize,
    pub flush_interval_ms: u64,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            subject_prefix: "fechatter.analytics".to_string(),
            batch_size: 100,
            flush_interval_ms: 5000,
        }
    }
}

/// Unified analytics event publisher using NATS + Protobuf
pub struct AnalyticsEventPublisher<T: EventTransport + 'static> {
    transport: Arc<T>,
    config: AnalyticsConfig,
    sender: mpsc::UnboundedSender<AnalyticsEvent>,
}

impl<T: EventTransport + 'static> AnalyticsEventPublisher<T> {
    /// Create new analytics publisher
    pub fn new(transport: Arc<T>, config: AnalyticsConfig) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        if config.enabled {
            // Start background batch processing task
            let transport_clone = transport.clone();
            let config_clone = config.clone();
            tokio::spawn(async move {
                Self::batch_processing_task(transport_clone, config_clone, receiver).await;
            });
        }

        Self {
            transport,
            config,
            sender,
        }
    }

    /// Background task for batch processing analytics events
    async fn batch_processing_task(
        transport: Arc<T>,
        config: AnalyticsConfig,
        mut receiver: mpsc::UnboundedReceiver<AnalyticsEvent>,
    ) {
        let mut event_buffer = Vec::new();
        let mut flush_interval =
            tokio::time::interval(std::time::Duration::from_millis(config.flush_interval_ms));

        loop {
            tokio::select! {
                // Receive new events
                event = receiver.recv() => {
                    match event {
                        Some(event) => {
                            event_buffer.push(event);

                            // Flush if buffer is full
                            if event_buffer.len() >= config.batch_size {
                                Self::flush_events(&transport, &config, &mut event_buffer).await;
                            }
                        }
                        None => {
                            // Channel closed, flush remaining events and exit
                            if !event_buffer.is_empty() {
                                Self::flush_events(&transport, &config, &mut event_buffer).await;
                            }
                            break;
                        }
                    }
                }
                // Periodic flush
                _ = flush_interval.tick() => {
                    if !event_buffer.is_empty() {
                        Self::flush_events(&transport, &config, &mut event_buffer).await;
                    }
                }
            }
        }
    }

    /// Flush buffered events to NATS
    async fn flush_events(
        transport: &Arc<T>,
        config: &AnalyticsConfig,
        event_buffer: &mut Vec<AnalyticsEvent>,
    ) {
        if event_buffer.is_empty() {
            return;
        }

        let event_count = event_buffer.len();
        debug!("Flushing {} analytics events", event_count);

        // Process events individually (could be optimized for batch in future)
        let events: Vec<_> = event_buffer.drain(..).collect();
        let mut success_count = 0;

        for event in events {
            let event_type_name = match &event.event_type {
                Some(analytics_event::EventType::UserLogin(_)) => "user.login",
                Some(analytics_event::EventType::MessageSent(_)) => "message.sent",
                Some(analytics_event::EventType::ChatCreated(_)) => "chat.created",
                Some(analytics_event::EventType::UserLogout(_)) => "user.logout",
                Some(analytics_event::EventType::UserRegister(_)) => "user.register",
                _ => "unknown",
            };

            match Self::publish_single_event(transport, config, event).await {
                Ok(()) => {
                    success_count += 1;
                    debug!("📤 Published analytics event: {}", event_type_name);
                }
                Err(e) => {
                    error!(
                        "ERROR: Failed to publish analytics event {}: {}",
                        event_type_name, e
                    );
                }
            }
        }

        if success_count > 0 {
            info!(
                "📤 Successfully flushed {}/{} analytics events",
                success_count, event_count
            );
        }
    }

    /// Publish single event to NATS
    async fn publish_single_event(
        transport: &Arc<T>,
        config: &AnalyticsConfig,
        event: AnalyticsEvent,
    ) -> Result<()> {
        // Determine subject based on event type
        let subject = Self::determine_subject(config, &event);

        // Encode as protobuf
        let payload = event.encode_to_vec();

        // Publish to NATS
        transport
            .publish(&subject, Bytes::from(payload))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to publish to NATS: {}", e))?;

        debug!(
            "SUBSCRIPTION: Published analytics event to subject: {}",
            subject
        );
        Ok(())
    }

    /// Determine NATS subject based on event type
    fn determine_subject(config: &AnalyticsConfig, event: &AnalyticsEvent) -> String {
        let event_type = match &event.event_type {
            Some(analytics_event::EventType::AppStart(_)) => "app.start",
            Some(analytics_event::EventType::AppExit(_)) => "app.exit",
            Some(analytics_event::EventType::UserLogin(_)) => "user.login",
            Some(analytics_event::EventType::UserLogout(_)) => "user.logout",
            Some(analytics_event::EventType::UserRegister(_)) => "user.register",
            Some(analytics_event::EventType::ChatCreated(_)) => "chat.created",
            Some(analytics_event::EventType::MessageSent(_)) => "message.sent",
            Some(analytics_event::EventType::ChatJoined(_)) => "chat.joined",
            Some(analytics_event::EventType::ChatLeft(_)) => "chat.left",
            Some(analytics_event::EventType::Navigation(_)) => "navigation",
            Some(analytics_event::EventType::FileUploaded(_)) => "file.uploaded",
            Some(analytics_event::EventType::FileDownloaded(_)) => "file.downloaded",
            Some(analytics_event::EventType::SearchPerformed(_)) => "search.performed",
            Some(analytics_event::EventType::NotificationReceived(_)) => "notification.received",
            Some(analytics_event::EventType::ErrorOccurred(_)) => "error.occurred",
            Some(analytics_event::EventType::BotResponse(_)) => "bot.response",
            None => "unknown",
        };

        format!("{}.{}", config.subject_prefix, event_type)
    }

    /// Publish analytics event (non-blocking)
    pub fn publish(&self, event: AnalyticsEvent) -> Result<(), AppError> {
        if !self.config.enabled {
            debug!("Analytics disabled, skipping event");
            return Ok(());
        }

        // Log the event type for debugging
        let event_type_name = match &event.event_type {
            Some(analytics_event::EventType::UserLogin(_)) => "user.login",
            Some(analytics_event::EventType::MessageSent(_)) => "message.sent",
            Some(analytics_event::EventType::ChatCreated(_)) => "chat.created",
            Some(analytics_event::EventType::UserLogout(_)) => "user.logout",
            Some(analytics_event::EventType::UserRegister(_)) => "user.register",
            _ => "unknown",
        };

        debug!(
            "Queuing analytics event: {} for user: {}",
            event_type_name,
            event
                .context
                .as_ref()
                .map(|c| &c.user_id)
                .unwrap_or(&"unknown".to_string())
        );

        self.sender.send(event).map_err(|e| {
            error!(
                "ERROR: Failed to queue analytics event {}: {}",
                event_type_name, e
            );
            AppError::Internal(format!("Failed to queue analytics event: {}", e))
        })?;

        debug!("Analytics event {} queued successfully", event_type_name);
        Ok(())
    }

    /// Check if analytics is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Trait for analytics tracking with convenient methods
#[async_trait]
pub trait AnalyticsTracking {
    async fn track_user_login(
        &self,
        user_id: String,
        email: String,
        method: String,
    ) -> Result<(), AppError>;
    async fn track_message_sent(
        &self,
        user_id: String,
        chat_id: String,
        message_type: String,
        size: i32,
    ) -> Result<(), AppError>;
    async fn track_chat_created(
        &self,
        user_id: String,
        chat_id: String,
        chat_type: String,
        members_count: i32,
    ) -> Result<(), AppError>;
    async fn track_error(
        &self,
        user_id: Option<String>,
        error_type: String,
        error_message: String,
    ) -> Result<(), AppError>;
    async fn track_bot_response(
        &self,
        bot_id: String,
        chat_id: String,
        response_type: String,
        success: bool,
    ) -> Result<(), AppError>;
}

#[async_trait]
impl<T: EventTransport + 'static> AnalyticsTracking for AnalyticsEventPublisher<T> {
    async fn track_user_login(
        &self,
        user_id: String,
        email: String,
        method: String,
    ) -> Result<(), AppError> {
        let event = AnalyticsEvent {
            context: Some(EventContext {
                client_id: format!("fechatter_server_{}", uuid::Uuid::new_v4()),
                session_id: uuid::Uuid::new_v4().to_string(),
                user_id: user_id.clone(),
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                client_ts: chrono::Utc::now().timestamp_millis(),
                server_ts: chrono::Utc::now().timestamp_millis(),
                user_agent: "fechatter-server".to_string(),
                ip: "127.0.0.1".to_string(),
                system: Some(SystemInfo {
                    os: std::env::consts::OS.to_string(),
                    arch: std::env::consts::ARCH.to_string(),
                    locale: "en-US".to_string(),
                    timezone: "UTC".to_string(),
                    browser: "server".to_string(),
                    browser_version: "1.0".to_string(),
                }),
                geo: None,
            }),
            event_type: Some(analytics_event::EventType::UserLogin(UserLoginEvent {
                email,
                login_method: method,
            })),
        };

        self.publish(event)
    }

    async fn track_message_sent(
        &self,
        user_id: String,
        chat_id: String,
        message_type: String,
        size: i32,
    ) -> Result<(), AppError> {
        let event = AnalyticsEvent {
            context: Some(EventContext {
                client_id: format!("fechatter_server_{}", uuid::Uuid::new_v4()),
                session_id: uuid::Uuid::new_v4().to_string(),
                user_id: user_id.clone(),
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                client_ts: chrono::Utc::now().timestamp_millis(),
                server_ts: chrono::Utc::now().timestamp_millis(),
                user_agent: "fechatter-server".to_string(),
                ip: "127.0.0.1".to_string(),
                system: Some(SystemInfo {
                    os: std::env::consts::OS.to_string(),
                    arch: std::env::consts::ARCH.to_string(),
                    locale: "en-US".to_string(),
                    timezone: "UTC".to_string(),
                    browser: "server".to_string(),
                    browser_version: "1.0".to_string(),
                }),
                geo: None,
            }),
            event_type: Some(analytics_event::EventType::MessageSent(MessageSentEvent {
                chat_id,
                r#type: message_type,
                size,
                total_files: 0,
                has_mentions: false,
                has_links: false,
            })),
        };

        self.publish(event)
    }

    async fn track_chat_created(
        &self,
        user_id: String,
        chat_id: String,
        chat_type: String,
        members_count: i32,
    ) -> Result<(), AppError> {
        let event = AnalyticsEvent {
            context: Some(EventContext {
                client_id: format!("fechatter_server_{}", uuid::Uuid::new_v4()),
                session_id: uuid::Uuid::new_v4().to_string(),
                user_id: user_id.clone(),
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                client_ts: chrono::Utc::now().timestamp_millis(),
                server_ts: chrono::Utc::now().timestamp_millis(),
                user_agent: "fechatter-server".to_string(),
                ip: "127.0.0.1".to_string(),
                system: Some(SystemInfo {
                    os: std::env::consts::OS.to_string(),
                    arch: std::env::consts::ARCH.to_string(),
                    locale: "en-US".to_string(),
                    timezone: "UTC".to_string(),
                    browser: "server".to_string(),
                    browser_version: "1.0".to_string(),
                }),
                geo: None,
            }),
            event_type: Some(analytics_event::EventType::ChatCreated(ChatCreatedEvent {
                workspace_id: "1".to_string(), // Default workspace
                chat_type,
                initial_members_count: members_count,
            })),
        };

        self.publish(event)
    }

    async fn track_error(
        &self,
        user_id: Option<String>,
        error_type: String,
        error_message: String,
    ) -> Result<(), AppError> {
        let event = AnalyticsEvent {
            context: Some(EventContext {
                client_id: format!("fechatter_server_{}", uuid::Uuid::new_v4()),
                session_id: uuid::Uuid::new_v4().to_string(),
                user_id: user_id.unwrap_or_default(),
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                client_ts: chrono::Utc::now().timestamp_millis(),
                server_ts: chrono::Utc::now().timestamp_millis(),
                user_agent: "fechatter-server".to_string(),
                ip: "127.0.0.1".to_string(),
                system: Some(SystemInfo {
                    os: std::env::consts::OS.to_string(),
                    arch: std::env::consts::ARCH.to_string(),
                    locale: "en-US".to_string(),
                    timezone: "UTC".to_string(),
                    browser: "server".to_string(),
                    browser_version: "1.0".to_string(),
                }),
                geo: None,
            }),
            event_type: Some(analytics_event::EventType::ErrorOccurred(
                ErrorOccurredEvent {
                    error_type,
                    error_code: "500".to_string(),
                    error_message,
                    stack_trace: String::new(),
                    context: String::new(),
                },
            )),
        };

        self.publish(event)
    }

    async fn track_bot_response(
        &self,
        bot_id: String,
        chat_id: String,
        response_type: String,
        success: bool,
    ) -> Result<(), AppError> {
        let event = AnalyticsEvent {
            context: Some(EventContext {
                client_id: format!("fechatter_server_{}", uuid::Uuid::new_v4()),
                session_id: uuid::Uuid::new_v4().to_string(),
                user_id: bot_id.clone(),
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                client_ts: chrono::Utc::now().timestamp_millis(),
                server_ts: chrono::Utc::now().timestamp_millis(),
                user_agent: "fechatter-bot".to_string(),
                ip: "127.0.0.1".to_string(),
                system: Some(SystemInfo {
                    os: std::env::consts::OS.to_string(),
                    arch: std::env::consts::ARCH.to_string(),
                    locale: "en-US".to_string(),
                    timezone: "UTC".to_string(),
                    browser: "bot".to_string(),
                    browser_version: "1.0".to_string(),
                }),
                geo: None,
            }),
            event_type: Some(analytics_event::EventType::BotResponse(BotResponseEvent {
                bot_id,
                chat_id,
                response_type,
                response_time_ms: 0,
                tokens_used: 0,
                success,
                error_message: if success {
                    String::new()
                } else {
                    "Unknown error".to_string()
                },
            })),
        };

        self.publish(event)
    }
}

/// Type alias for NATS analytics publisher
pub type NatsAnalyticsPublisher = AnalyticsEventPublisher<NatsTransport>;

impl NatsAnalyticsPublisher {
    /// Create NATS analytics publisher with client
    pub fn with_nats_client(client: async_nats::Client, config: AnalyticsConfig) -> Self {
        let transport = Arc::new(NatsTransport::new(client));
        Self::new(transport, config)
    }
}
