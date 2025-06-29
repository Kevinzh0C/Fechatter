use async_nats::Client;
use fechatter_protos::fechatter::v1::{
    analytics_event::EventType, AnalyticsEvent, BotResponseEvent, ErrorOccurredEvent,
    EventContext, SystemInfo,
};
use std::sync::Arc;
use tracing::{debug, error};

/// Unified NATS + Protobuf analytics publisher for bot_server
/// Compatible with analytics_server architecture
pub struct UnifiedBotAnalyticsPublisher {
    client: Arc<Client>,
    client_id: String,
    session_id: String,
}

impl UnifiedBotAnalyticsPublisher {
    pub fn new(nats_client: Client) -> Self {
        Self {
            client: Arc::new(nats_client),
            client_id: format!("bot_server_{}", uuid::Uuid::new_v4()),
            session_id: format!("session_{}", uuid::Uuid::new_v4()),
        }
    }

    /// Create event context for bot events
    fn create_event_context(&self, bot_id: Option<String>) -> EventContext {
        EventContext {
            client_id: self.client_id.clone(),
            session_id: self.session_id.clone(),
            user_id: bot_id.unwrap_or_default(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            client_ts: chrono::Utc::now().timestamp_millis(),
            server_ts: 0,
            user_agent: format!("bot_server/{}", env!("CARGO_PKG_VERSION")),
            ip: String::new(),
            system: Some(SystemInfo {
                os: std::env::consts::OS.to_string(),
                arch: std::env::consts::ARCH.to_string(),
                locale: "en-US".to_string(),
                timezone: "UTC".to_string(),
                browser: String::new(),
                browser_version: String::new(),
            }),
            geo: None,
        }
    }

    /// Track bot response event using unified protobuf format
    pub async fn track_bot_response(
        &self,
        bot_id: String,
        chat_id: String,
        response_type: String,
        response_time_ms: u64,
        tokens_used: u32,
        success: bool,
        error_message: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event = AnalyticsEvent {
            context: Some(self.create_event_context(Some(bot_id.clone()))),
            event_type: Some(EventType::BotResponse(BotResponseEvent {
                bot_id,
                chat_id,
                response_type,
                response_time_ms,
                tokens_used,
                success,
                error_message: error_message.unwrap_or_default(),
            })),
        };

        self.publish_event("fechatter.analytics.bot.response", event)
            .await
    }

    /// Track bot error event
    pub async fn track_bot_error(
        &self,
        bot_id: String,
        chat_id: String,
        error_type: String,
        error_message: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event = AnalyticsEvent {
            context: Some(self.create_event_context(Some(bot_id))),
            event_type: Some(EventType::ErrorOccurred(ErrorOccurredEvent {
                error_type,
                error_code: "BOT_ERROR".to_string(),
                error_message,
                stack_trace: String::new(),
                context: format!("Bot error in chat: {}", chat_id),
            })),
        };

        self.publish_event("fechatter.analytics.bot.error", event)
            .await
    }

    /// Publish protobuf event to NATS
    async fn publish_event(
        &self,
        subject: &str,
        event: AnalyticsEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use prost::Message;

        let mut buf = Vec::new();
        event.encode(&mut buf)?;

        debug!("📊 Publishing bot analytics event to NATS: {}", subject);

        self.client
            .publish(subject.to_string(), buf.into())
            .await
            .map_err(|e| {
                error!("Failed to publish bot analytics event: {}", e);
                Box::new(e) as Box<dyn std::error::Error + Send + Sync>
            })?;

        Ok(())
    }
} 