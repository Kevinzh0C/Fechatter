//! # Realtime Stream
//!
//! **Responsibility**: Ultra-low latency real-time message delivery and state synchronization
//! **Purpose**: notify-server WebSocket push, focused on user experience
//! **Features**:
//! - Real-time message broadcasting
//! - Real-time read/unread status updates
//! - Real-time presence synchronization
//! - Message delivery acknowledgment
//! - Typing indicators
//! - User activity status
//! **Delivery Semantics**: best-effort (prioritize latency over guaranteed delivery)
//! **Storage**: transient NATS Core (non-persistent)

use crate::domains::messaging::messaging_domain::{MessageDomainService, MessageDomainServiceImpl};
use crate::services::infrastructure::event::{
    EventTransport, LegacyEventPublisher as EventPublisher, Signable,
};
use crate::AppError;
use fechatter_core::models::message::{CreateMessage, ListMessages, MessageView, StreamMessage};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::warn;

// ── Realtime Stream Event Types ──────────────────────────────────────────────────

/// Realtime Stream Events - Focused on user experience and instant feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RealtimeStreamEvent {
    /// New message arrived
    MessageReceived {
        message: StreamMessage,
        chat_id: i64,
        recipients: Vec<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sig: Option<String>,
    },

    /// Message read status update
    MessageRead {
        message_id: i64,
        chat_id: i64,
        reader_id: i64,
        read_at: String, // ISO timestamp
        #[serde(skip_serializing_if = "Option::is_none")]
        sig: Option<String>,
    },

    /// Message unread status update
    MessageUnread {
        message_id: i64,
        chat_id: i64,
        user_id: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        sig: Option<String>,
    },

    /// User started typing
    TypingStarted {
        chat_id: i64,
        user_id: i64,
        user_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        sig: Option<String>,
    },

    /// User stopped typing
    TypingStopped {
        chat_id: i64,
        user_id: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        sig: Option<String>,
    },

    /// User presence status change
    UserPresenceChanged {
        user_id: i64,
        status: String, // "online", "offline", "away"
        last_seen: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sig: Option<String>,
    },

    /// Message delivery acknowledgment (ACK)
    MessageDelivered {
        message_id: i64,
        chat_id: i64,
        delivered_to: i64,
        delivered_at: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        sig: Option<String>,
    },
}

// Implement Signable trait to support event signatures
impl Signable for RealtimeStreamEvent {
    fn set_signature(&mut self, sig: Option<String>) {
        match self {
            RealtimeStreamEvent::MessageReceived { sig: event_sig, .. }
            | RealtimeStreamEvent::MessageRead { sig: event_sig, .. }
            | RealtimeStreamEvent::MessageUnread { sig: event_sig, .. }
            | RealtimeStreamEvent::TypingStarted { sig: event_sig, .. }
            | RealtimeStreamEvent::TypingStopped { sig: event_sig, .. }
            | RealtimeStreamEvent::UserPresenceChanged { sig: event_sig, .. }
            | RealtimeStreamEvent::MessageDelivered { sig: event_sig, .. } => {
                *event_sig = sig;
            }
        }
    }

    fn get_signature(&self) -> &Option<String> {
        match self {
            RealtimeStreamEvent::MessageReceived { sig, .. }
            | RealtimeStreamEvent::MessageRead { sig, .. }
            | RealtimeStreamEvent::MessageUnread { sig, .. }
            | RealtimeStreamEvent::TypingStarted { sig, .. }
            | RealtimeStreamEvent::TypingStopped { sig, .. }
            | RealtimeStreamEvent::UserPresenceChanged { sig, .. }
            | RealtimeStreamEvent::MessageDelivered { sig, .. } => sig,
        }
    }
}

// ── Realtime Stream Service Based on Unified Event Infrastructure ──────────────────

/// Realtime Stream Service - Focused on user experience and immediacy
///
/// **After Refactoring**: Built on EventPublisher<T: EventTransport>
/// **Benefits**:
/// - Reuse transport layer abstraction (NATS, Kafka, etc)
/// - Unified retry, signature, and error handling mechanisms
/// - Support for pluggable transport layers
///
/// **Responsibilities**:
/// - Real-time message pushing and status synchronization
/// - Real-time read/unread status management
/// - User presence synchronization
/// - Typing indicator management
/// - Message delivery confirmation and feedback
pub struct RealtimeStreamService<T: EventTransport + 'static> {
    domain_service: Arc<MessageDomainServiceImpl>,
    event_publisher: Arc<EventPublisher<T>>,
    stream_subjects: RealtimeStreamSubjects,
}

/// Realtime Stream Subject Configuration
#[derive(Debug, Clone)]
pub struct RealtimeStreamSubjects {
    pub chat_events: String,
    pub user_events: String,
    pub typing_events: String,
    pub presence_events: String,
    pub delivery_events: String,
}

impl Default for RealtimeStreamSubjects {
    fn default() -> Self {
        Self {
            chat_events: "fechatter.realtime.chat".to_string(),
            user_events: "fechatter.realtime.user".to_string(),
            typing_events: "fechatter.realtime.typing".to_string(),
            presence_events: "fechatter.realtime.presence".to_string(),
            delivery_events: "fechatter.realtime.delivery".to_string(),
        }
    }
}

impl<T: EventTransport + 'static> RealtimeStreamService<T> {
    pub fn new(
        domain_service: Arc<MessageDomainServiceImpl>,
        event_publisher: Arc<EventPublisher<T>>,
    ) -> Self {
        Self {
            domain_service,
            event_publisher,
            stream_subjects: RealtimeStreamSubjects::default(),
        }
    }

    pub fn with_subjects(mut self, subjects: RealtimeStreamSubjects) -> Self {
        self.stream_subjects = subjects;
        self
    }

    /// Publish realtime event - Reuse unified event infrastructure
    pub async fn publish_realtime_event(&self, event: RealtimeStreamEvent) -> Result<(), AppError> {
        let subject = self.get_subject_for_event(&event);

        self.event_publisher
            .publish_event(&subject, event, "realtime_event")
            .await
    }

    /// Determine subject based on event type
    fn get_subject_for_event(&self, event: &RealtimeStreamEvent) -> String {
        match event {
            RealtimeStreamEvent::MessageReceived { chat_id, .. }
            | RealtimeStreamEvent::MessageRead { chat_id, .. }
            | RealtimeStreamEvent::MessageUnread { chat_id, .. } => {
                format!("{}.{}", self.stream_subjects.chat_events, chat_id)
            }
            RealtimeStreamEvent::TypingStarted { chat_id, .. }
            | RealtimeStreamEvent::TypingStopped { chat_id, .. } => {
                format!("{}.{}", self.stream_subjects.typing_events, chat_id)
            }
            RealtimeStreamEvent::UserPresenceChanged { user_id, .. } => {
                format!("{}.{}", self.stream_subjects.presence_events, user_id)
            }
            RealtimeStreamEvent::MessageDelivered { chat_id, .. } => {
                format!("{}.{}", self.stream_subjects.delivery_events, chat_id)
            }
        }
    }

    /// Publish message to specific chat
    pub async fn publish_to_chat(
        &self,
        chat_id: i64,
        event: RealtimeStreamEvent,
    ) -> Result<(), AppError> {
        self.publish_realtime_event(event).await
    }

    /// Publish message to specific user
    pub async fn publish_to_user(
        &self,
        user_id: i64,
        event: RealtimeStreamEvent,
    ) -> Result<(), AppError> {
        self.publish_realtime_event(event).await
    }

    /// Send message - Handle only realtime push and status sync
    pub async fn send_message(
        &self,
        sender_id: fechatter_core::UserId,
        chat_id: fechatter_core::ChatId,
        message: CreateMessage,
    ) -> Result<MessageView, AppError> {
        // 1. Business logic processing
        let saved_message = self
            .domain_service
            .send_message(message, i64::from(chat_id), i64::from(sender_id))
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let message_view = MessageView::from(saved_message.clone());

        // 2. Get chat members (for realtime push)
        let chat_members = self
            .domain_service
            .get_chat_members(i64::from(chat_id))
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // 3. Prepare realtime push
        let event_publisher = Arc::clone(&self.event_publisher);
        let message_for_push = saved_message.clone();
        let subjects = self.stream_subjects.clone();

        // 4. Non-blocking realtime push
        tokio::spawn(async move {
            let stream_message = StreamMessage {
                id: message_for_push.id.to_string(),
                chat_id: i64::from(message_for_push.chat_id),
                sender_id: i64::from(message_for_push.sender_id),
                content: message_for_push.content.clone(),
                files: message_for_push.files.clone().unwrap_or_default(),
                timestamp: message_for_push.created_at.timestamp(),
            };

            let realtime_event = RealtimeStreamEvent::MessageReceived {
                message: stream_message,
                chat_id: i64::from(message_for_push.chat_id),
                recipients: chat_members,
                sig: None,
            };

            let subject = format!("{}.{}", subjects.chat_events, message_for_push.chat_id);

            if let Err(e) = event_publisher
                .publish_event(&subject, realtime_event, "realtime_message_push")
                .await
            {
                warn!("Realtime message push failed: {}", e);
            }
        });

        Ok(message_view)
    }

    /// Mark message as read - Realtime status sync
    pub async fn mark_message_read(
        &self,
        message_id: fechatter_core::MessageId,
        chat_id: fechatter_core::ChatId,
        reader_id: fechatter_core::UserId,
    ) -> Result<(), AppError> {
        // TODO: Persist read status to database

        // Publish read event
        let read_event = RealtimeStreamEvent::MessageRead {
            message_id: i64::from(message_id),
            chat_id: i64::from(chat_id),
            reader_id: i64::from(reader_id),
            read_at: chrono::Utc::now().to_rfc3339(),
            sig: None,
        };

        self.publish_realtime_event(read_event).await?;
        Ok(())
    }

    /// Mark message as unread - Realtime status sync
    pub async fn mark_message_unread(
        &self,
        message_id: fechatter_core::MessageId,
        chat_id: fechatter_core::ChatId,
        user_id: fechatter_core::UserId,
    ) -> Result<(), AppError> {
        // TODO: Persist unread status to database

        // Publish unread event
        let unread_event = RealtimeStreamEvent::MessageUnread {
            message_id: i64::from(message_id),
            chat_id: i64::from(chat_id),
            user_id: i64::from(user_id),
            sig: None,
        };

        self.publish_realtime_event(unread_event).await?;
        Ok(())
    }

    /// Publish typing started event - typing indicators
    pub async fn start_typing(
        &self,
        chat_id: fechatter_core::ChatId,
        user_id: fechatter_core::UserId,
        user_name: String,
    ) -> Result<(), AppError> {
        let typing_event = RealtimeStreamEvent::TypingStarted {
            chat_id: i64::from(chat_id),
            user_id: i64::from(user_id),
            user_name,
            sig: None,
        };

        self.publish_realtime_event(typing_event).await?;
        Ok(())
    }

    /// Publish typing stopped event - typing indicators
    pub async fn stop_typing(
        &self,
        chat_id: fechatter_core::ChatId,
        user_id: fechatter_core::UserId,
    ) -> Result<(), AppError> {
        let typing_event = RealtimeStreamEvent::TypingStopped {
            chat_id: i64::from(chat_id),
            user_id: i64::from(user_id),
            sig: None,
        };

        self.publish_realtime_event(typing_event).await?;
        Ok(())
    }

    /// Update user presence status - user presence
    pub async fn update_user_presence(
        &self,
        user_id: fechatter_core::UserId,
        status: &str,
        last_seen: Option<String>,
    ) -> Result<(), AppError> {
        let presence_event = RealtimeStreamEvent::UserPresenceChanged {
            user_id: i64::from(user_id),
            status: status.to_string(),
            last_seen,
            sig: None,
        };

        self.publish_realtime_event(presence_event).await?;
        Ok(())
    }

    /// Confirm message delivery - delivery acknowledgment
    pub async fn confirm_message_delivery(
        &self,
        message_id: fechatter_core::MessageId,
        chat_id: fechatter_core::ChatId,
        delivered_to: fechatter_core::UserId,
    ) -> Result<(), AppError> {
        let delivery_event = RealtimeStreamEvent::MessageDelivered {
            message_id: i64::from(message_id),
            chat_id: i64::from(chat_id),
            delivered_to: i64::from(delivered_to),
            delivered_at: chrono::Utc::now().to_rfc3339(),
            sig: None,
        };

        self.publish_realtime_event(delivery_event).await?;
        Ok(())
    }

    /// Proxy other query methods
    pub async fn get_message(
        &self,
        message_id: fechatter_core::MessageId,
    ) -> Result<Option<MessageView>, AppError> {
        let message = self
            .domain_service
            .get_message(i64::from(message_id))
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(message.map(MessageView::from))
    }

    pub async fn list_messages(
        &self,
        user_id: fechatter_core::UserId,
        chat_id: fechatter_core::ChatId,
        input: ListMessages,
    ) -> Result<Vec<MessageView>, AppError> {
        let messages = self
            .domain_service
            .list_messages(input, i64::from(chat_id), i64::from(user_id))
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(messages.into_iter().map(MessageView::from).collect())
    }
}

// ── Factory Functions ────────────────────────────────────────────────────────────

/// Create NATS-based realtime stream service
pub fn create_realtime_stream_service_with_nats(
    domain_service: Arc<MessageDomainServiceImpl>,
    nats_client: async_nats::Client,
) -> Arc<RealtimeStreamService<crate::services::infrastructure::event::NatsTransport>> {
    use crate::services::infrastructure::event::{
        LegacyEventPublisher as EventPublisher, NatsTransport,
    };

    let transport = NatsTransport::new(nats_client);
    let event_publisher = Arc::new(EventPublisher::with_transport(transport));

    Arc::new(RealtimeStreamService::new(domain_service, event_publisher))
}

/// Create realtime stream service with any transport layer
pub fn create_realtime_stream_service<T: EventTransport>(
    domain_service: Arc<MessageDomainServiceImpl>,
    event_publisher: Arc<EventPublisher<T>>,
) -> Arc<RealtimeStreamService<T>> {
    Arc::new(RealtimeStreamService::new(domain_service, event_publisher))
}

// ── Backward Compatibility ────────────────────────────────────────────────────────────

/// Backward compatible type aliases
pub type MessagingService<T> = RealtimeStreamService<T>;
pub type InstantMessagingService<T> = RealtimeStreamService<T>;

/// NATS Realtime Stream Publisher (backward compatible interface)
pub struct NatsRealtimeStreamPublisher {
    service: Arc<RealtimeStreamService<crate::services::infrastructure::event::NatsTransport>>,
}

impl NatsRealtimeStreamPublisher {
    pub fn new(
        client: Option<async_nats::Client>,
        domain_service: Arc<MessageDomainServiceImpl>,
    ) -> Self {
        let service = if let Some(client) = client {
            create_realtime_stream_service_with_nats(domain_service, client)
        } else {
            // Create a dummy service if no client is provided
            // Mainly used for testing or configuration error cases
            panic!("NATS client is required for RealtimeStreamPublisher")
        };

        Self { service }
    }
}

/// Realtime Stream Publisher trait (backward compatible)
#[async_trait::async_trait]
pub trait RealtimeStreamPublisher: Send + Sync {
    /// Publish realtime event to WebSocket
    async fn publish_realtime_event(&self, event: RealtimeStreamEvent) -> Result<(), AppError>;

    /// Publish message to specific chat
    async fn publish_to_chat(
        &self,
        chat_id: i64,
        event: RealtimeStreamEvent,
    ) -> Result<(), AppError>;

    /// Publish message to specific user
    async fn publish_to_user(
        &self,
        user_id: i64,
        event: RealtimeStreamEvent,
    ) -> Result<(), AppError>;
}

#[async_trait::async_trait]
impl RealtimeStreamPublisher for NatsRealtimeStreamPublisher {
    async fn publish_realtime_event(&self, event: RealtimeStreamEvent) -> Result<(), AppError> {
        self.service.publish_realtime_event(event).await
    }

    async fn publish_to_chat(
        &self,
        chat_id: i64,
        event: RealtimeStreamEvent,
    ) -> Result<(), AppError> {
        self.service.publish_to_chat(chat_id, event).await
    }

    async fn publish_to_user(
        &self,
        user_id: i64,
        event: RealtimeStreamEvent,
    ) -> Result<(), AppError> {
        self.service.publish_to_user(user_id, event).await
    }
}

/// Backward compatible factory functions
pub fn create_messaging_service(
    domain_service: Arc<MessageDomainServiceImpl>,
    nats_client: Option<async_nats::Client>,
) -> Arc<RealtimeStreamService<crate::services::infrastructure::event::NatsTransport>> {
    if let Some(client) = nats_client {
        create_realtime_stream_service_with_nats(domain_service, client)
    } else {
        panic!("NATS client is required")
    }
}

pub fn create_instant_messaging_service(
    domain_service: Arc<MessageDomainServiceImpl>,
    nats_client: Option<async_nats::Client>,
) -> Arc<RealtimeStreamService<crate::services::infrastructure::event::NatsTransport>> {
    create_messaging_service(domain_service, nats_client)
}

// TechniqueUsed: RustTechnique::RealtimeStreaming | DesignPrinciple: UserExperienceFirst
// Architecture: UnifiedEventInfrastructure | Pattern: StrategyPattern
