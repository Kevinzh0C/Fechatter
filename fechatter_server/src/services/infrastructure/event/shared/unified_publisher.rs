//! # Enhanced Event Publisher - Production-grade NATS Event Publishing
//!
//! **INTEGRATED with notify_server compatibility**
//!
//! **Key Features:**
//! - Compatible with notify_server SSE broadcasting
//! - Complete message content delivery for real-time notifications
//! - Unified subject naming with backward compatibility
//! - Enhanced metadata for rich event context

use crate::error::AppError;
use async_nats::Client as NatsClient;
use chrono::{DateTime, Utc};
use fechatter_core::{ChatId, MessageId, UserId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// =============================================================================
// NOTIFY-SERVER COMPATIBLE EVENT DEFINITIONS
// =============================================================================

/// Complete message data for notify_server SSE broadcasting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteMessageData {
    pub id: i64,
    pub chat_id: i64,
    pub sender_id: i64,
    pub sender_name: String,
    pub content: String,
    pub files: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub idempotency_key: Option<String>,
}

/// Enhanced message event for notify_server compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyCompatibleMessageEvent {
    pub event_type: String, // "new_message", "message_edited", "message_deleted"
    pub chat_id: i64,
    pub sender_id: i64,
    pub sender_name: String,
    pub message: CompleteMessageData,
    pub workspace_id: i64,
    pub timestamp: DateTime<Utc>,
    pub event_id: String,
}

/// Chat member event for notify_server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyCompatibleChatMemberEvent {
    pub event_type: String, // "member_added", "member_removed"
    pub chat_id: i64,
    pub user_id: i64,
    pub user_name: String,
    pub chat_name: String,
    pub workspace_id: i64,
    pub timestamp: DateTime<Utc>,
}

/// Read receipt event for notify_server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyCompatibleReadReceiptEvent {
    pub event_type: String, // "messages_read"
    pub chat_id: i64,
    pub user_id: i64,
    pub message_ids: Vec<i64>,
    pub read_at: DateTime<Utc>,
}

// =============================================================================
// ENHANCED EVENT PUBLISHER - NOTIFY_SERVER INTEGRATION
// =============================================================================

/// Enhanced event publisher with notify_server compatibility
pub struct EnhancedEventPublisher {
    nats_client: Option<NatsClient>,
    config: PublisherConfig,
}

/// Publisher configuration
#[derive(Debug, Clone)]
pub struct PublisherConfig {
    pub subjects: SubjectConfig,
    pub enable_signatures: bool,
    pub hmac_secret: Option<String>,
    pub service_instance_id: String,
    pub default_workspace_id: i64,
}

/// Subject configuration - compatible with notify_server
#[derive(Debug, Clone)]
pub struct SubjectConfig {
    // notify_server compatible subjects
    pub message_events: String,
    pub chat_events: String,
    pub user_events: String,
    pub realtime_events: String,
}

impl Default for SubjectConfig {
    fn default() -> Self {
        Self {
            // Use notify_server compatible subjects
            message_events: "fechatter.message.events".to_string(),
            chat_events: "fechatter.chat.events".to_string(),
            user_events: "fechatter.user.events".to_string(),
            realtime_events: "fechatter.realtime.events".to_string(),
        }
    }
}

impl EnhancedEventPublisher {
    /// Create new enhanced publisher
    pub fn new(nats_client: Option<NatsClient>, config: PublisherConfig) -> Self {
        Self {
            nats_client,
            config,
        }
    }

    /// Create publisher without NATS (for testing or when NATS is disabled)
    pub fn disabled() -> Self {
        Self {
            nats_client: None,
            config: PublisherConfig {
                subjects: SubjectConfig::default(),
                enable_signatures: false,
                hmac_secret: None,
                service_instance_id: "fechatter_server_disabled".to_string(),
                default_workspace_id: 0,
            },
        }
    }

    /// Publish message created event - COMPLETE content for notify_server SSE
    pub async fn publish_message_created_for_notify(
        &self,
        message: CompleteMessageData,
        workspace_id: i64,
    ) -> Result<(), AppError> {
        let event = NotifyCompatibleMessageEvent {
            event_type: "new_message".to_string(),
            chat_id: message.chat_id,
            sender_id: message.sender_id,
            sender_name: message.sender_name.clone(),
            message: message.clone(),
            workspace_id,
            timestamp: Utc::now(),
            event_id: Uuid::new_v4().to_string(),
        };

        self.publish_event(&self.config.subjects.message_events, event)
            .await
    }

    /// Publish message edited event for notify_server
    pub async fn publish_message_edited_for_notify(
        &self,
        message: CompleteMessageData,
        original_content: String,
        workspace_id: i64,
    ) -> Result<(), AppError> {
        let mut enhanced_message = message.clone();
        // Add edit metadata
        enhanced_message.content = format!("EDITED: {}", message.content);

        let event = NotifyCompatibleMessageEvent {
            event_type: "message_edited".to_string(),
            chat_id: message.chat_id,
            sender_id: message.sender_id,
            sender_name: message.sender_name.clone(),
            message: enhanced_message,
            workspace_id,
            timestamp: Utc::now(),
            event_id: Uuid::new_v4().to_string(),
        };

        self.publish_event(&self.config.subjects.message_events, event)
            .await
    }

    /// Publish message deleted event for notify_server
    pub async fn publish_message_deleted_for_notify(
        &self,
        message_id: i64,
        chat_id: i64,
        sender_id: i64,
        sender_name: String,
        workspace_id: i64,
    ) -> Result<(), AppError> {
        let deleted_message = CompleteMessageData {
            id: message_id,
            chat_id,
            sender_id,
            sender_name: sender_name.clone(),
            content: "[Message Deleted]".to_string(),
            files: vec![],
            created_at: Utc::now(),
            idempotency_key: None,
        };

        let event = NotifyCompatibleMessageEvent {
            event_type: "message_deleted".to_string(),
            chat_id,
            sender_id,
            sender_name,
            message: deleted_message,
            workspace_id,
            timestamp: Utc::now(),
            event_id: Uuid::new_v4().to_string(),
        };

        self.publish_event(&self.config.subjects.message_events, event)
            .await
    }

    /// Publish read receipt event for notify_server
    pub async fn publish_read_receipt_for_notify(
        &self,
        chat_id: i64,
        user_id: i64,
        message_ids: Vec<i64>,
        workspace_id: i64,
    ) -> Result<(), AppError> {
        let event = NotifyCompatibleReadReceiptEvent {
            event_type: "messages_read".to_string(),
            chat_id,
            user_id,
            message_ids,
            read_at: Utc::now(),
        };

        self.publish_event(&self.config.subjects.realtime_events, event)
            .await
    }

    /// Publish chat member joined event for notify_server
    pub async fn publish_chat_member_joined_for_notify(
        &self,
        chat_id: i64,
        user_id: i64,
        user_name: String,
        chat_name: String,
        workspace_id: i64,
    ) -> Result<(), AppError> {
        let event = NotifyCompatibleChatMemberEvent {
            event_type: "member_added".to_string(),
            chat_id,
            user_id,
            user_name,
            chat_name,
            workspace_id,
            timestamp: Utc::now(),
        };

        self.publish_event(&self.config.subjects.chat_events, event)
            .await
    }

    /// Publish event to NATS with error handling
    async fn publish_event<T: Serialize>(&self, subject: &str, event: T) -> Result<(), AppError> {
        // Check if NATS is available
        let Some(nats_client) = &self.nats_client else {
            debug!("NATS client not available, skipping event publication");
            return Ok(());
        };

        // Serialize event to JSON for notify_server compatibility
        let payload_bytes = match serde_json::to_vec(&event) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("Failed to serialize event: {}", e);
                return Err(AppError::Internal(format!("Serialization error: {}", e)));
            }
        };

        // Publish to NATS
        match nats_client
            .publish(subject.to_string(), payload_bytes.into())
            .await
        {
            Ok(_) => {
                debug!("Successfully published event to subject: {}", subject);
                info!(
                    "SUBSCRIPTION: Event published to notify_server: {}",
                    subject
                );
                Ok(())
            }
            Err(e) => {
                error!("Failed to publish event to NATS: {}", e);
                Err(AppError::Internal(format!("NATS publish error: {}", e)))
            }
        }
    }

    /// Get publisher statistics
    pub fn get_stats(&self) -> PublisherStats {
        PublisherStats {
            nats_connected: self.nats_client.is_some(),
            service_instance: self.config.service_instance_id.clone(),
            signatures_enabled: self.config.enable_signatures,
        }
    }
}

/// Publisher statistics
#[derive(Debug, Clone, Serialize)]
pub struct PublisherStats {
    pub nats_connected: bool,
    pub service_instance: String,
    pub signatures_enabled: bool,
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Create default publisher configuration
pub fn create_default_config(workspace_id: i64) -> PublisherConfig {
    PublisherConfig {
        subjects: SubjectConfig::default(),
        enable_signatures: false,
        hmac_secret: None,
        service_instance_id: format!("fechatter_server_{}", Uuid::new_v4()),
        default_workspace_id: workspace_id,
    }
}

/// Create publisher with NATS client for notify_server integration
pub async fn create_enhanced_publisher_with_nats(
    nats_url: &str,
    config: PublisherConfig,
) -> Result<EnhancedEventPublisher, AppError> {
    match async_nats::connect(nats_url).await {
        Ok(client) => {
            info!(
                "Connected to NATS for notify_server integration: {}",
                nats_url
            );
            Ok(EnhancedEventPublisher::new(Some(client), config))
        }
        Err(e) => {
            warn!(
                "WARNING: Failed to connect to NATS, creating disabled publisher: {}",
                e
            );
            Ok(EnhancedEventPublisher::disabled())
        }
    }
}
