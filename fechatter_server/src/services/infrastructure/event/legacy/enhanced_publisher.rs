//! # Enhanced Event Publisher - notify_server Integration
//!
//! **Production-grade NATS Event Publishing with notify_server compatibility**
//! 
//! **Key Features:**
//! - ✅ Compatible with notify_server SSE broadcasting
//! - ✅ Complete message content delivery for real-time notifications  
//! - ✅ notify_server expected subject naming
//! - ✅ Enhanced metadata for rich event context
//! - ✅ Seamless integration with existing EventPublisher

use crate::error::AppError;
use async_nats::Client as NatsClient;
use chrono::{DateTime, Utc};
use fechatter_core::{ChatId, MessageId, UserId};
use serde::{Deserialize, Serialize};
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

/// notify_server compatible message event (matches processor.rs expectations)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyMessageEvent {
    pub event_type: String, // "new_message", "message_edited", "message_deleted"
    pub chat_id: i64,
    pub sender_id: i64,
    pub sender_name: String,
    pub content: String, // Direct content for notify_server
    pub files: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub message_id: i64,
    pub workspace_id: i64,
}

/// notify_server compatible chat member event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyChatMemberEvent {
    pub event_type: String, // "member_added", "member_removed"
    pub chat_id: i64,
    pub user_id: i64,
    pub user_name: String,
    pub timestamp: DateTime<Utc>,
}

/// notify_server compatible read receipt event  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyReadReceiptEvent {
    pub event_type: String, // "messages_read"
    pub chat_id: i64,
    pub user_id: i64,
    pub message_ids: Vec<i64>,
    pub read_at: DateTime<Utc>,
}

// =============================================================================
// ENHANCED EVENT PUBLISHER FOR NOTIFY_SERVER
// =============================================================================

/// Enhanced event publisher specifically for notify_server integration
pub struct EnhancedEventPublisher {
    nats_client: Option<NatsClient>,
    service_instance_id: String,
}

impl EnhancedEventPublisher {
    /// Create new enhanced publisher for notify_server integration
    pub fn new(nats_client: Option<NatsClient>) -> Self {
        Self {
            nats_client,
            service_instance_id: format!("fechatter_server_{}", Uuid::new_v4()),
        }
    }

    /// Create disabled publisher for testing/fallback
    pub fn disabled() -> Self {
        Self {
            nats_client: None,
            service_instance_id: "fechatter_server_disabled".to_string(),
        }
    }

    // =============================================================================
    // NOTIFY-SERVER COMPATIBLE PUBLISHING METHODS
    // =============================================================================

    /// Publish message created event - COMPLETE content for notify_server SSE
    /// notify_server processes this on subject "fechatter.message.*"
    pub async fn publish_complete_message_for_sse(
        &self,
        message_data: CompleteMessageData,
        workspace_id: i64,
    ) -> Result<(), AppError> {
        let event = NotifyMessageEvent {
            event_type: "new_message".to_string(),
            chat_id: message_data.chat_id,
            sender_id: message_data.sender_id,
            sender_name: message_data.sender_name,
            content: message_data.content,
            files: message_data.files,
            timestamp: message_data.created_at,
            message_id: message_data.id,
            workspace_id,
        };

        // Use notify_server expected subject pattern
        self.publish_to_notify_server("fechatter.message.new", event).await
    }

    /// Publish message edited event for notify_server
    pub async fn publish_message_edited_for_sse(
        &self,
        message_data: CompleteMessageData,
        workspace_id: i64,
    ) -> Result<(), AppError> {
        let event = NotifyMessageEvent {
            event_type: "message_edited".to_string(),
            chat_id: message_data.chat_id,
            sender_id: message_data.sender_id,
            sender_name: message_data.sender_name,
            content: format!("✏️ [EDITED] {}", message_data.content),
            files: message_data.files,
            timestamp: Utc::now(),
            message_id: message_data.id,
            workspace_id,
        };

        self.publish_to_notify_server("fechatter.message.edited", event).await
    }

    /// Publish message deleted event for notify_server
    pub async fn publish_message_deleted_for_sse(
        &self,
        message_id: i64,
        chat_id: i64,
        sender_id: i64,
        sender_name: String,
        workspace_id: i64,
    ) -> Result<(), AppError> {
        let event = NotifyMessageEvent {
            event_type: "message_deleted".to_string(),
            chat_id,
            sender_id,
            sender_name,
            content: "🗑️ [Message Deleted]".to_string(),
            files: vec![],
            timestamp: Utc::now(),
            message_id,
            workspace_id,
        };

        self.publish_to_notify_server("fechatter.message.deleted", event).await
    }

    /// Publish read receipt event for notify_server realtime updates
    pub async fn publish_read_receipts_for_sse(
        &self,
        chat_id: i64,
        user_id: i64,
        message_ids: Vec<i64>,
    ) -> Result<(), AppError> {
        let event = NotifyReadReceiptEvent {
            event_type: "messages_read".to_string(),
            chat_id,
            user_id,
            message_ids,
            read_at: Utc::now(),
        };

        self.publish_to_notify_server("fechatter.realtime.read_receipts", event).await
    }

    /// Publish chat member joined event for notify_server
    pub async fn publish_member_joined_for_sse(
        &self,
        chat_id: i64,
        user_id: i64,
        user_name: String,
    ) -> Result<(), AppError> {
        let event = NotifyChatMemberEvent {
            event_type: "member_added".to_string(),
            chat_id,
            user_id,
            user_name,
            timestamp: Utc::now(),
        };

        self.publish_to_notify_server("fechatter.chat.member_joined", event).await
    }

    /// Publish chat member left event for notify_server
    pub async fn publish_member_left_for_sse(
        &self,
        chat_id: i64,
        user_id: i64,
        user_name: String,
    ) -> Result<(), AppError> {
        let event = NotifyChatMemberEvent {
            event_type: "member_removed".to_string(),
            chat_id,
            user_id,
            user_name,
            timestamp: Utc::now(),
        };

        self.publish_to_notify_server("fechatter.chat.member_left", event).await
    }

    // =============================================================================
    // INTERNAL NATS PUBLISHING
    // =============================================================================

    /// Publish event to NATS for notify_server consumption
    async fn publish_to_notify_server<T: Serialize>(
        &self, 
        subject: &str, 
        event: T
    ) -> Result<(), AppError> {
        // Check if NATS is available
        let Some(nats_client) = &self.nats_client else {
            debug!("🔇 NATS client not available, skipping notify_server event publication");
            return Ok(());
        };

        // Serialize event to JSON (notify_server expects JSON)
        let payload_bytes = match serde_json::to_vec(&event) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("❌ Failed to serialize event for notify_server: {}", e);
                return Err(AppError::Internal(format!("Serialization error: {}", e)));
            }
        };

        // Publish to NATS
        match nats_client.publish(subject.to_string(), payload_bytes.into()).await {
            Ok(_) => {
                debug!("✅ Successfully published event to notify_server: {}", subject);
                info!("📡 notify_server event: {} -> SSE broadcasting", subject);
                Ok(())
            }
            Err(e) => {
                error!("❌ Failed to publish event to notify_server: {}", e);
                Err(AppError::Internal(format!("notify_server NATS publish error: {}", e)))
            }
        }
    }

    /// Check if publisher is connected
    pub fn is_connected(&self) -> bool {
        self.nats_client.is_some()
    }

    /// Get service instance ID
    pub fn service_instance_id(&self) -> &str {
        &self.service_instance_id
    }
}

// =============================================================================
// HELPER FUNCTIONS FOR CONVERSION
// =============================================================================

/// Convert fechatter_core::Message to CompleteMessageData
pub fn message_to_complete_data(
    message: &fechatter_core::Message,
    sender_name: String,
) -> CompleteMessageData {
    CompleteMessageData {
        id: message.id.0,
        chat_id: message.chat_id.0,
        sender_id: message.sender_id.0,
        sender_name,
        content: message.content.clone(),
        files: message.files.clone().unwrap_or_default(),
        created_at: message.created_at,
        idempotency_key: message.idempotency_key.map(|k| k.to_string()),
    }
}

/// Create enhanced publisher with NATS connection
pub async fn create_enhanced_publisher_for_notify_server(
    nats_url: &str,
) -> Result<EnhancedEventPublisher, AppError> {
    match async_nats::connect(nats_url).await {
        Ok(client) => {
            info!("✅ Enhanced publisher connected to NATS for notify_server: {}", nats_url);
            Ok(EnhancedEventPublisher::new(Some(client)))
        }
        Err(e) => {
            warn!("⚠️ Failed to connect enhanced publisher to NATS: {}", e);
            info!("📄 Creating disabled enhanced publisher");
            Ok(EnhancedEventPublisher::disabled())
        }
    }
} 