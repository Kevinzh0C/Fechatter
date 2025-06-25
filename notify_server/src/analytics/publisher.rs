use anyhow::Result;
use async_nats;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use serde::{Serialize, Deserialize};
use prost::Message;

use super::types::{NotifyAnalyticsEvent, NotifyEventHelper};

/// Analytics event publisher configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub enabled: bool,
    pub nats_url: String,
    pub subject_prefix: String,
    pub batch_size: usize,
    pub flush_interval_ms: u64,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            nats_url: "nats://localhost:4222".to_string(),
            subject_prefix: "fechatter.analytics".to_string(), // Use standard prefix
            batch_size: 100,
            flush_interval_ms: 5000, // 5 seconds
        }
    }
}

/// Analytics event publisher
#[derive(Clone)]
pub struct AnalyticsPublisher {
    sender: mpsc::UnboundedSender<NotifyAnalyticsEvent>,
    config: Arc<AnalyticsConfig>,
}

impl Default for AnalyticsPublisher {
    fn default() -> Self {
        let (sender, _) = mpsc::unbounded_channel();
        Self {
            sender,
            config: Arc::new(AnalyticsConfig::default()),
        }
    }
}

impl AnalyticsPublisher {
    /// Create a new analytics publisher
    pub async fn new(config: AnalyticsConfig) -> Result<Self> {
        if !config.enabled {
            info!("Analytics publishing is disabled");
            let (sender, _) = mpsc::unbounded_channel();
            return Ok(Self {
                sender,
                config: Arc::new(config),
            });
        }

        let (sender, receiver) = mpsc::unbounded_channel();
        let config_arc = Arc::new(config);

        // Start the background publisher task
        let publisher_task = AnalyticsPublisherTask::new(config_arc.clone(), receiver).await?;
        tokio::spawn(async move {
            publisher_task.run().await;
        });

        info!("✅ Analytics publisher initialized");
        Ok(Self {
            sender,
            config: config_arc,
        })
    }

    /// Publish an analytics event (non-blocking)
    pub fn publish(&self, event: NotifyAnalyticsEvent) {
        if !self.config.enabled {
            return;
        }

        if let Err(e) = self.sender.send(event) {
            warn!("Failed to queue analytics event: {}", e);
        }
    }

    /// Publish user connected event
    pub fn user_connected(&self, user_id: fechatter_core::UserId, connection_id: String, user_agent: Option<String>) {
        let event = NotifyEventHelper::user_connected(user_id, connection_id, user_agent);
        self.publish(event);
    }

    /// Publish user disconnected event
    pub fn user_disconnected(&self, user_id: fechatter_core::UserId, connection_id: String, duration_ms: u64) {
        let event = NotifyEventHelper::user_disconnected(user_id, connection_id, duration_ms);
        self.publish(event);
    }

    /// Publish notification received event
    pub fn notification_received(
        &self,
        user_id: fechatter_core::UserId,
        notification_type: String,
        source: String,
        was_delivered: bool,
        delivery_duration_ms: Option<u64>,
    ) {
        let event = NotifyEventHelper::notification_received(
            user_id,
            notification_type,
            source,
            was_delivered,
            delivery_duration_ms,
        );
        self.publish(event);
    }

    /// Check if analytics is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get analytics configuration
    pub fn config(&self) -> &AnalyticsConfig {
        &self.config
    }
}

/// Background task for publishing analytics events
struct AnalyticsPublisherTask {
    nats_client: async_nats::Client,
    config: Arc<AnalyticsConfig>,
    receiver: mpsc::UnboundedReceiver<NotifyAnalyticsEvent>,
    event_buffer: Vec<NotifyAnalyticsEvent>,
}

impl AnalyticsPublisherTask {
    /// Create a new publisher task
    async fn new(
        config: Arc<AnalyticsConfig>,
        receiver: mpsc::UnboundedReceiver<NotifyAnalyticsEvent>,
    ) -> Result<Self> {
        // Connect to NATS with retry logic
        let nats_client = Self::connect_with_retry(&config.nats_url).await?;

        Ok(Self {
            nats_client,
            config,
            receiver,
            event_buffer: Vec::new(),
        })
    }

    /// Main event processing loop
    async fn run(mut self) {
        info!("🚀 Analytics publisher task started");

        // Create flush timer
        let mut flush_interval = tokio::time::interval(
            std::time::Duration::from_millis(self.config.flush_interval_ms)
        );

        loop {
            tokio::select! {
                // Receive new events
                event = self.receiver.recv() => {
                    match event {
                        Some(event) => {
                            debug!("Received analytics event for processing");
                            self.event_buffer.push(event);

                            // Flush if buffer is full
                            if self.event_buffer.len() >= self.config.batch_size {
                                self.flush_events().await;
                            }
                        }
                        None => {
                            info!("Analytics publisher receiver closed, shutting down");
                            break;
                        }
                    }
                }
                // Periodic flush
                _ = flush_interval.tick() => {
                    if !self.event_buffer.is_empty() {
                        self.flush_events().await;
                    }
                }
            }
        }

        // Final flush on shutdown
        if !self.event_buffer.is_empty() {
            self.flush_events().await;
        }

        info!("📴 Analytics publisher task stopped");
    }

    /// Flush buffered events to NATS
    async fn flush_events(&mut self) {
        if self.event_buffer.is_empty() {
            return;
        }

        let event_count = self.event_buffer.len();
        debug!("Flushing {} analytics events", event_count);

        // Process events in batches
        let events: Vec<_> = self.event_buffer.drain(..).collect();
        for event in events {
            if let Err(e) = self.publish_single_event(event).await {
                error!("Failed to publish analytics event: {}", e);
            }
        }

        info!("📤 Flushed {} analytics events", event_count);
    }

    /// Publish a single event to NATS using protobuf format
    async fn publish_single_event(&self, analytics_event: NotifyAnalyticsEvent) -> Result<()> {
        // Encode to protobuf (event is already in the correct format)
        let payload = analytics_event.encode_to_vec();

        // Use the standard analytics subject for consistency
        let subject = format!("{}.events", self.config.subject_prefix);

        // Publish to NATS
        self.nats_client
            .publish(subject.clone(), payload.into())
            .await?;

        debug!("📡 Published protobuf analytics event to subject: {}", subject);
        Ok(())
    }

    /// Connect to NATS with retry logic
    async fn connect_with_retry(nats_url: &str) -> Result<async_nats::Client> {
        let mut retries = 0;
        const MAX_RETRIES: u32 = 5;
        const RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(2);

        loop {
            info!("🔗 Attempting to connect to NATS for analytics: {}", nats_url);

            match async_nats::connect(nats_url).await {
                Ok(client) => {
                    info!("✅ Connected to NATS for analytics: {}", nats_url);
                    return Ok(client);
                }
                Err(e) => {
                    retries += 1;
                    if retries >= MAX_RETRIES {
                        error!("❌ Failed to connect to NATS after {} retries: {}", MAX_RETRIES, e);
                        return Err(e.into());
                    }
                    warn!(
                        "⚠️ Failed to connect to NATS (attempt {}/{}): {}. Retrying in {:?}",
                        retries, MAX_RETRIES, e, RETRY_DELAY
                    );
                    tokio::time::sleep(RETRY_DELAY).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fechatter_core::UserId;

    #[test]
    fn test_analytics_config_default() {
        let config = AnalyticsConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.nats_url, "nats://localhost:4222");
        assert_eq!(config.batch_size, 100);
    }

    #[tokio::test]
    async fn test_disabled_publisher() {
        let config = AnalyticsConfig {
            enabled: false,
            ..Default::default()
        };

        let publisher = AnalyticsPublisher::new(config).await.unwrap();
        assert!(!publisher.is_enabled());

        // Should not fail when publishing while disabled
        publisher.user_connected(UserId(123), "conn_123".to_string(), None);
    }
} 