// Event Publisher Service - Unified Event Publishing with Transport Abstraction
//
// This module provides a flexible event publishing system that supports multiple
// transport mechanisms (NATS, Kafka, etc.) through a common interface.

use crate::error::{AppError, EventTransportError};
use crate::services::infrastructure::event::EventTransport;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use fechatter_core::{
    contracts::events::{
        subjects, ChatMemberJoinedEvent, ChatMemberLeftEvent, DuplicateMessageEvent, EventVersion,
        HmacSha256Verifier, MessageEvent, MessageLifecycle, SignatureVerifier,
    },
    ChatId, Message, MessageId, UserId,
};
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

// Re-export event types for backward compatibility
pub use fechatter_core::contracts::events::{
    subjects::{
        CHAT_MEMBER_JOINED as DEFAULT_USER_JOINED_CHAT, CHAT_MEMBER_LEFT as DEFAULT_USER_LEFT_CHAT,
        DUPLICATE_MESSAGE as DEFAULT_DUPLICATE_MESSAGE, MESSAGE_CREATED as DEFAULT_MESSAGE_CREATED,
        MESSAGE_DELETED as DEFAULT_MESSAGE_DELETED, MESSAGE_UPDATED as DEFAULT_MESSAGE_UPDATED,
        SEARCH_INDEX as SEARCH_SUBJECT,
    },
    MessageLifecycle as MsgLifecycle,
};

// Event Transport Configuration
const SIGNATURE_HEADER: &str = "X-Event-Signature";

type HmacSha256 = Hmac<Sha256>;

/// Trait for events that support signatures
pub trait Signable {
    fn set_signature(&mut self, sig: Option<String>);
    fn get_signature(&self) -> &Option<String>;
}

// Implement Signable for core event types
impl Signable for MessageEvent {
    fn set_signature(&mut self, sig: Option<String>) {
        self.sig = sig;
    }

    fn get_signature(&self) -> &Option<String> {
        &self.sig
    }
}

impl Signable for ChatMemberJoinedEvent {
    fn set_signature(&mut self, sig: Option<String>) {
        self.sig = sig;
    }

    fn get_signature(&self) -> &Option<String> {
        &self.sig
    }
}

impl Signable for ChatMemberLeftEvent {
    fn set_signature(&mut self, sig: Option<String>) {
        self.sig = sig;
    }

    fn get_signature(&self) -> &Option<String> {
        &self.sig
    }
}

impl Signable for DuplicateMessageEvent {
    fn set_signature(&mut self, sig: Option<String>) {
        self.sig = sig;
    }

    fn get_signature(&self) -> &Option<String> {
        &self.sig
    }
}

/// Search-specific types (not in core)
#[derive(Debug, Clone, Serialize)]
pub struct ChatInfo {
    pub chat_id: ChatId,
    pub chat_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum SearchOp {
    Delete {
        id: MessageId,
    },
    Index {
        message: Message,
        chat_info: ChatInfo,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchEvent {
    #[serde(flatten)]
    pub op: SearchOp,
    pub occurred_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
}

impl Signable for SearchEvent {
    fn set_signature(&mut self, sig: Option<String>) {
        self.sig = sig;
    }

    fn get_signature(&self) -> &Option<String> {
        &self.sig
    }
}

// Re-export core event types with aliases for backward compatibility
pub type ChatMemberJoined = ChatMemberJoinedEvent;
pub type ChatMemberLeft = ChatMemberLeftEvent;

/// Result type for publish attempts
enum PublishAttemptResult {
    Retry,
    Fatal(AppError),
}

/// Retry configuration for resilient publishing
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 5000,
        }
    }
}

pub(crate) struct RetryState {
    attempt: u32,
    backoff_ms: u64,
}

impl RetryState {
    fn new(config: &RetryConfig) -> Self {
        Self {
            attempt: 0,
            backoff_ms: config.initial_backoff_ms,
        }
    }

    fn can_retry(&self, config: &RetryConfig) -> bool {
        self.attempt < config.max_retries
    }

    fn update_backoff(&mut self, config: &RetryConfig) {
        // Exponential backoff with max limit
        self.backoff_ms = (self.backoff_ms * 2).min(config.max_backoff_ms);
    }
}

/// Generic Event Publisher with transport abstraction
///
/// Supports any transport that implements the EventTransport trait,
/// allowing for easy swapping between NATS, Kafka, or custom transports.
pub struct EventPublisher<T: EventTransport + ?Sized> {
    transport: Arc<T>,
    shutdown_token: CancellationToken,
    hmac_secret: Option<Vec<u8>>,
    sign_headers: bool,
    retry_config: RetryConfig,
}

impl<T: EventTransport> EventPublisher<T> {
    /// Create a new EventPublisher with owned transport
    pub fn with_transport(transport: T) -> Self {
        Self::build(Arc::new(transport), CancellationToken::new(), None, false)
    }

    /// Create with transport and shutdown token
    pub fn with_transport_and_shutdown(transport: T, shutdown_token: CancellationToken) -> Self {
        Self::build(Arc::new(transport), shutdown_token, None, false)
    }

    /// Create with transport and signature configuration
    pub fn with_transport_and_signature(transport: T, secret: Vec<u8>, use_headers: bool) -> Self {
        Self::build(
            Arc::new(transport),
            CancellationToken::new(),
            Some(secret),
            use_headers,
        )
    }
}

impl<T: EventTransport + ?Sized> EventPublisher<T> {
    /// Create a new EventPublisher with shared transport
    /// Useful when you need to share the same transport across multiple publishers
    pub fn with_shared_transport(transport: Arc<T>) -> Self {
        Self::build(transport, CancellationToken::new(), None, false)
    }

    fn build(
        transport: Arc<T>,
        shutdown_token: CancellationToken,
        hmac_secret: Option<Vec<u8>>,
        sign_headers: bool,
    ) -> Self {
        Self {
            transport,
            shutdown_token,
            hmac_secret,
            sign_headers,
            retry_config: RetryConfig::default(),
        }
    }

    /// Access the underlying transport
    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// Access the underlying transport as Arc
    pub fn shared_transport(&self) -> &Arc<T> {
        &self.transport
    }

    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Sign payload using HMAC-SHA256
    fn sign_payload(&self, payload: &[u8]) -> Option<String> {
        self.hmac_secret.as_ref().map(|secret| {
            let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC can handle any key size");
            mac.update(payload);
            hex::encode(mac.finalize().into_bytes())
        })
    }

    fn is_retryable_error(error: &EventTransportError) -> bool {
        matches!(
            error,
            EventTransportError::Timeout | EventTransportError::Disconnected
        )
    }

    pub async fn publish_event<E>(
        &self,
        subject: &str,
        mut event: E,
        context: &str,
    ) -> Result<(), AppError>
    where
        E: Serialize + Signable,
    {
        // Serialize once for both signature and sending
        let payload = serde_json::to_vec(&event).map_err(|e| {
            AppError::SerializationError(format!("Failed to serialize {}: {}", context, e))
        })?;

        // Handle signature based on configuration
        let headers = if self.sign_headers {
            // Sign and put in headers
            self.sign_payload(&payload).map(|sig| {
                let mut headers = HashMap::new();
                headers.insert(SIGNATURE_HEADER.to_string(), sig);
                headers
            })
        } else if let Some(sig) = self.sign_payload(&payload) {
            // Sign and include in payload
            event.set_signature(Some(sig));
            // Re-serialize with signature
            let signed_payload = serde_json::to_vec(&event).map_err(|e| {
                AppError::SerializationError(format!(
                    "Failed to serialize signed {}: {}",
                    context, e
                ))
            })?;

            return self
                .publish_with_retry(subject, &signed_payload, None, context)
                .await;
        } else {
            None
        };

        self.publish_with_retry(subject, &payload, headers, context)
            .await
    }

    async fn publish_with_retry(
        &self,
        subject: &str,
        payload: &[u8],
        headers: Option<HashMap<String, String>>,
        context: &str,
    ) -> Result<(), AppError> {
        let mut retry_state = RetryState::new(&self.retry_config);
        let payload_bytes = Bytes::copy_from_slice(payload);

        loop {
            match self
                .try_publish_once(subject, &payload_bytes, &headers, &retry_state)
                .await
            {
                Ok(()) => {
                    if retry_state.attempt > 0 {
                        info!(
                            "Successfully published {} after {} retries",
                            context, retry_state.attempt
                        );
                    }
                    return Ok(());
                }
                Err(PublishAttemptResult::Fatal(e)) => return Err(e),
                Err(PublishAttemptResult::Retry) => {
                    retry_state.attempt += 1;
                    if !retry_state.can_retry(&self.retry_config) {
                        return Err(AppError::EventPublishError(format!(
                            "Failed to publish {} after {} retries",
                            context, retry_state.attempt
                        )));
                    }
                    retry_state.update_backoff(&self.retry_config);
                }
            }
        }
    }

    async fn try_publish_once(
        &self,
        subject: &str,
        payload: &Bytes,
        headers: &Option<HashMap<String, String>>,
        retry_state: &RetryState,
    ) -> Result<(), PublishAttemptResult> {
        if retry_state.attempt > 0 {
            info!(
                "Retrying publish to {} (attempt {}/{}), waiting {}ms",
                subject,
                retry_state.attempt + 1,
                self.retry_config.max_retries,
                retry_state.backoff_ms
            );

            if let Err(()) = self.wait_with_cancellation(retry_state.backoff_ms).await {
                return Err(PublishAttemptResult::Fatal(AppError::EventPublishError(
                    "Operation cancelled during retry backoff".to_string(),
                )));
            }
        }

        match self.try_publish(subject, payload, headers).await {
            Ok(()) => Ok(()),
            Err(e) if Self::is_retryable_error(&e) => {
                warn!("Retryable error publishing to {}: {}", subject, e);
                Err(PublishAttemptResult::Retry)
            }
            Err(e) => {
                error!("Fatal error publishing to {}: {}", subject, e);
                Err(PublishAttemptResult::Fatal(e.into()))
            }
        }
    }

    async fn try_publish(
        &self,
        subject: &str,
        payload: &Bytes,
        headers: &Option<HashMap<String, String>>,
    ) -> Result<(), EventTransportError> {
        // Use tokio::select! to handle cancellation
        tokio::select! {
          result = self.publish_with_optional_headers(subject, payload, headers) => result,
          _ = self.shutdown_token.cancelled() => {
            Err(EventTransportError::Disconnected)
          }
        }
    }

    async fn wait_with_cancellation(&self, ms: u64) -> Result<(), ()> {
        tokio::select! {
          _ = sleep(std::time::Duration::from_millis(ms)) => Ok(()),
          _ = self.shutdown_token.cancelled() => Err(()),
        }
    }

    /// Determines subject for message lifecycle events
    fn message_subject_for(&self, kind: &MessageLifecycle) -> &'static str {
        match kind {
            MessageLifecycle::Created => subjects::MESSAGE_CREATED,
            MessageLifecycle::Updated => subjects::MESSAGE_UPDATED,
            MessageLifecycle::Deleted => subjects::MESSAGE_DELETED,
        }
    }

    // Public API methods - now much simpler thanks to generic implementation

    #[instrument(skip(self, message, chat_members))]
    pub async fn publish_message_event(
        &self,
        kind: MessageLifecycle,
        message: &Message,
        chat_members: &[UserId],
    ) -> Result<(), AppError> {
        let subject = self.message_subject_for(&kind);

        let event = MessageEvent {
            version: EventVersion::default(),
            kind,
            msg: message.clone(),
            members: chat_members.to_vec(),
            occurred_at: Utc::now(),
            sig: None,
        };

        self.publish_event(subject, event, "message_event").await
    }

    #[instrument(skip(self, chat_id, user_id))]
    pub async fn publish_chat_member_joined(
        &self,
        chat_id: &ChatId,
        user_id: &UserId,
    ) -> Result<(), AppError> {
        let event = ChatMemberJoinedEvent {
            version: EventVersion::default(),
            chat_id: *chat_id,
            user_id: *user_id,
            occurred_at: Utc::now(),
            sig: None,
        };

        self.publish_event(subjects::CHAT_MEMBER_JOINED, event, "chat_member_joined")
            .await
    }

    #[instrument(skip(self, chat_id, user_id))]
    pub async fn publish_chat_member_left(
        &self,
        chat_id: &ChatId,
        user_id: &UserId,
    ) -> Result<(), AppError> {
        let event = ChatMemberLeftEvent {
            version: EventVersion::default(),
            chat_id: *chat_id,
            user_id: *user_id,
            occurred_at: Utc::now(),
            sig: None,
        };

        self.publish_event(subjects::CHAT_MEMBER_LEFT, event, "chat_member_left")
            .await
    }

    #[instrument(skip(self, idempotency_key, chat_id, sender_id))]
    pub async fn publish_duplicate_message_attempted(
        &self,
        idempotency_key: &Uuid,
        chat_id: &ChatId,
        sender_id: &UserId,
    ) -> Result<(), AppError> {
        let event = DuplicateMessageEvent {
            version: EventVersion::default(),
            idempotency_key: *idempotency_key,
            chat_id: *chat_id,
            sender_id: *sender_id,
            occurred_at: Utc::now(),
            sig: None,
        };

        self.publish_event(subjects::DUPLICATE_MESSAGE, event, "duplicate_message")
            .await
    }

    #[instrument(skip(self, id), fields(message_id = %id))]
    pub async fn publish_search_delete(&self, id: &MessageId) -> Result<(), AppError> {
        let event = SearchEvent {
            op: SearchOp::Delete { id: *id },
            occurred_at: Utc::now(),
            sig: None,
        };

        self.publish_event(subjects::SEARCH_INDEX, event, "search_delete")
            .await
    }

    #[instrument(skip(self, message, chat_info), fields(message_id = %message.id, chat_id = %message.chat_id))]
    pub async fn publish_search_index(
        &self,
        message: &Message,
        chat_info: &ChatInfo,
    ) -> Result<(), AppError> {
        let event = SearchEvent {
            op: SearchOp::Index {
                message: message.clone(),
                chat_info: chat_info.clone(),
            },
            occurred_at: Utc::now(),
            sig: None,
        };

        self.publish_event(subjects::SEARCH_INDEX, event, "search_index")
            .await
    }

    /// Verifies HMAC signature against payload (delegates to core)
    pub fn verify_signature(payload: &[u8], signature: &str, secret: &[u8]) -> bool {
        let verifier = HmacSha256Verifier;
        verifier.verify_signature(payload, signature, secret)
    }

    async fn publish_with_optional_headers(
        &self,
        subject: &str,
        payload: &[u8],
        headers: &Option<HashMap<String, String>>,
    ) -> Result<(), EventTransportError> {
        if let Some(header_map) = headers {
            self.transport
                .publish_with_headers(subject, header_map.clone(), Bytes::copy_from_slice(payload))
                .await
        } else {
            self.transport
                .publish(subject, Bytes::copy_from_slice(payload))
                .await
        }
    }

    // =============================================================================
    // UNIFIED PUBLISHING METHODS - Enhanced methods for unified architecture
    // =============================================================================

    /// Publish enhanced message created event with unified naming and metadata
    #[instrument(skip(self, message, chat_members))]
    pub async fn publish_unified_message_created(
        &self,
        message: &Message,
        chat_members: &[UserId],
        sender_name: String,
        chat_name: String,
        workspace_id: i64,
    ) -> Result<(), AppError> {
        let event = EnhancedMessageEvent {
            base: MessageEvent {
                version: EventVersion::default(),
                kind: MessageLifecycle::Created,
                msg: message.clone(),
                members: chat_members.to_vec(),
                occurred_at: Utc::now(),
                sig: None,
            },
            sender_name,
            chat_name,
            workspace_id,
            event_id: Uuid::new_v4().to_string(),
            trace_context: HashMap::new(),
        };

        self.publish_event(
            unified_subjects::MESSAGE_CREATED_V1,
            event,
            "unified_message_created",
        )
        .await
    }

    /// Publish enhanced message edited event with unified naming
    #[instrument(skip(self, message))]
    pub async fn publish_unified_message_edited(
        &self,
        message: &Message,
        editor_name: String,
        chat_name: String,
        workspace_id: i64,
    ) -> Result<(), AppError> {
        let event = EnhancedMessageEvent {
            base: MessageEvent {
                version: EventVersion::default(),
                kind: MessageLifecycle::Updated,
                msg: message.clone(),
                members: vec![], // For edits, we don't need all members
                occurred_at: Utc::now(),
                sig: None,
            },
            sender_name: editor_name,
            chat_name,
            workspace_id,
            event_id: Uuid::new_v4().to_string(),
            trace_context: HashMap::new(),
        };

        self.publish_event(
            unified_subjects::MESSAGE_EDITED_V1,
            event,
            "unified_message_edited",
        )
        .await
    }

    /// Publish enhanced message deleted event with unified naming
    #[instrument(skip(self, message))]
    pub async fn publish_unified_message_deleted(
        &self,
        message: &Message,
        deleted_by_name: String,
        chat_name: String,
        workspace_id: i64,
    ) -> Result<(), AppError> {
        let event = EnhancedMessageEvent {
            base: MessageEvent {
                version: EventVersion::default(),
                kind: MessageLifecycle::Deleted,
                msg: message.clone(),
                members: vec![], // For deletes, we don't need all members
                occurred_at: Utc::now(),
                sig: None,
            },
            sender_name: deleted_by_name,
            chat_name,
            workspace_id,
            event_id: Uuid::new_v4().to_string(),
            trace_context: HashMap::new(),
        };

        self.publish_event(
            unified_subjects::MESSAGE_DELETED_V1,
            event,
            "unified_message_deleted",
        )
        .await
    }

    /// Publish enhanced chat member joined event with unified naming
    #[instrument(skip(self, chat_id, user_id))]
    pub async fn publish_unified_chat_member_joined(
        &self,
        chat_id: &ChatId,
        user_id: &UserId,
        chat_name: String,
        user_name: String,
        workspace_id: i64,
    ) -> Result<(), AppError> {
        let event = EnhancedChatMemberEvent {
            base: ChatMemberJoinedEvent {
                version: EventVersion::default(),
                chat_id: *chat_id,
                user_id: *user_id,
                occurred_at: Utc::now(),
                sig: None,
            },
            event_id: Uuid::new_v4().to_string(),
            workspace_id,
            chat_name,
            user_name,
            trace_context: HashMap::new(),
        };

        self.publish_event(
            unified_subjects::CHAT_MEMBER_JOINED_V1,
            event,
            "unified_chat_member_joined",
        )
        .await
    }

    /// Publish enhanced chat member left event with unified naming
    #[instrument(skip(self, chat_id, user_id))]
    pub async fn publish_unified_chat_member_left(
        &self,
        chat_id: &ChatId,
        user_id: &UserId,
        chat_name: String,
        user_name: String,
        workspace_id: i64,
    ) -> Result<(), AppError> {
        let left_event = ChatMemberLeftEvent {
            version: EventVersion::default(),
            chat_id: *chat_id,
            user_id: *user_id,
            occurred_at: Utc::now(),
            sig: None,
        };

        // Reuse EnhancedChatMemberEvent structure for consistency
        let event = EnhancedChatMemberEvent {
            base: ChatMemberJoinedEvent {
                version: left_event.version,
                chat_id: left_event.chat_id,
                user_id: left_event.user_id,
                occurred_at: left_event.occurred_at,
                sig: left_event.sig,
            },
            event_id: Uuid::new_v4().to_string(),
            workspace_id,
            chat_name,
            user_name,
            trace_context: HashMap::new(),
        };

        self.publish_event(
            unified_subjects::CHAT_MEMBER_LEFT_V1,
            event,
            "unified_chat_member_left",
        )
        .await
    }

    /// Publish batch events for high-throughput scenarios
    #[instrument(skip(self, events))]
    pub async fn publish_unified_batch_events<E>(
        &self,
        events: Vec<(String, E)>,
    ) -> Result<Vec<Result<(), AppError>>, AppError>
    where
        E: Serialize + Signable + Clone + Send + 'static,
    {
        let mut results = Vec::with_capacity(events.len());

        // For now, publish sequentially. In future, we can optimize with true batch support
        for (subject, event) in events {
            let result = self.publish_event(&subject, event, "batch_event").await;
            results.push(result);
        }

        Ok(results)
    }

    /// Publish unified message read receipt event
    #[instrument(skip(self, message_ids))]
    pub async fn publish_unified_message_read_receipt(
        &self,
        chat_id: &ChatId,
        user_id: &UserId,
        message_ids: Vec<MessageId>,
        workspace_id: i64,
    ) -> Result<(), AppError> {
        // Create a message event for read receipts using the first message ID
        let first_message_id = message_ids.first().copied().unwrap_or(MessageId(0));
        let message = Message {
            id: first_message_id,
            chat_id: *chat_id,
            sender_id: *user_id, // User who read the messages
            content: format!("Read {} messages", message_ids.len()),
            files: None,
            created_at: Utc::now(),
            idempotency_key: None,
        };

        let event = EnhancedMessageEvent {
            base: MessageEvent {
                version: EventVersion::default(),
                kind: MessageLifecycle::Created, // Use Created as a placeholder for read receipts
                msg: message,
                members: vec![*user_id], // Only the user who read
                occurred_at: Utc::now(),
                sig: None,
            },
            sender_name: "System".to_string(), // Read receipts are system events
            chat_name: format!("Chat {}", chat_id.0),
            workspace_id,
            event_id: Uuid::new_v4().to_string(),
            trace_context: {
                let mut context = HashMap::new();
                context.insert("event_type".to_string(), "read_receipt".to_string());
                context.insert("message_count".to_string(), message_ids.len().to_string());
                context.insert("read_by".to_string(), user_id.0.to_string());
                context
            },
        };

        // Use a special subject for read receipts
        self.publish_event(
            "fechatter.realtime.message.read_receipt.v1",
            event,
            "unified_read_receipt",
        )
        .await
    }
}

/// Type alias for NATS-based EventPublisher (for backward compatibility)
pub type NatsEventPublisher = EventPublisher<NatsTransport>;

use crate::services::infrastructure::event::NatsTransport;
use async_nats::Client as NatsClient;

impl NatsEventPublisher {
    /// Create a simple NatsEventPublisher (for backward compatibility with tests)
    pub fn new(client: NatsClient) -> Self {
        let transport = NatsTransport::new(client);
        Self::with_transport(transport)
    }

    /// Create NatsEventPublisher with shutdown token
    pub fn with_shutdown_token(client: NatsClient, shutdown_token: CancellationToken) -> Self {
        let transport = NatsTransport::new(client);
        Self::with_transport_and_shutdown(transport, shutdown_token)
    }

    /// Create NATS EventPublisher with signature (backward compatible)
    pub fn with_signature(client: NatsClient, secret: Vec<u8>, use_headers: bool) -> Self {
        let transport = NatsTransport::new(client);
        Self::with_transport_and_signature(transport, secret, use_headers)
    }

    /// Access the underlying NATS client (backward compatible)
    pub fn nats_client(&self) -> &NatsClient {
        self.transport.client()
    }
}

/// Support for dynamic dispatch with trait objects
pub type DynEventPublisher = EventPublisher<dyn EventTransport>;

impl DynEventPublisher {
    /// Create EventPublisher from a trait object
    pub fn with_dyn_transport(transport: Arc<dyn EventTransport>) -> Self {
        Self::with_shared_transport(transport)
    }
}

// Builder pattern for retry configuration
impl RetryConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn with_backoff_range(mut self, initial_ms: u64, max_ms: u64) -> Self {
        self.initial_backoff_ms = initial_ms;
        self.max_backoff_ms = max_ms;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use fechatter_core::{ChatId, Message, MessageId, UserId};

    // Remove async_nats::Server import and related tests for now
    // TODO: Add integration tests with actual NATS server setup

    fn create_test_message() -> Message {
        Message {
            id: MessageId(1),
            chat_id: ChatId(100),
            sender_id: UserId(10),
            content: "Hello".to_string(),
            files: None,
            created_at: Utc::now(),
            idempotency_key: Some(Uuid::new_v4()),
        }
    }

    #[test]
    fn test_retry_config_builder() {
        let config = RetryConfig::new()
            .with_max_retries(3)
            .with_backoff_range(50, 2000);

        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_backoff_ms, 50);
        assert_eq!(config.max_backoff_ms, 2000);
    }
}

// =============================================================================
// ENHANCED EVENT STRUCTURES - Extended events for unified architecture
// =============================================================================

/// Enhanced message event with additional metadata for unified architecture
#[derive(Debug, Clone, Serialize)]
pub struct EnhancedMessageEvent {
    #[serde(flatten)]
    pub base: MessageEvent,

    // Enhanced metadata for unified system
    pub sender_name: String,
    pub chat_name: String,
    pub workspace_id: i64,
    pub event_id: String,
    pub trace_context: HashMap<String, String>,
}

impl Signable for EnhancedMessageEvent {
    fn set_signature(&mut self, sig: Option<String>) {
        self.base.sig = sig;
    }

    fn get_signature(&self) -> &Option<String> {
        &self.base.sig
    }
}

/// Enhanced chat member event with additional context
#[derive(Debug, Clone, Serialize)]
pub struct EnhancedChatMemberEvent {
    #[serde(flatten)]
    pub base: ChatMemberJoinedEvent,

    // Enhanced metadata
    pub event_id: String,
    pub workspace_id: i64,
    pub chat_name: String,
    pub user_name: String,
    pub trace_context: HashMap<String, String>,
}

impl Signable for EnhancedChatMemberEvent {
    fn set_signature(&mut self, sig: Option<String>) {
        self.base.sig = sig;
    }

    fn get_signature(&self) -> &Option<String> {
        &self.base.sig
    }
}

// =============================================================================
// UNIFIED SUBJECT NAMING - Standardized topic naming for v1 architecture
// =============================================================================

/// Unified subject naming for consistent event routing
pub mod unified_subjects {
    /// Message domain subjects using new naming convention
    pub const MESSAGE_CREATED_V1: &str = "fechatter.messages.message.created.v1";
    pub const MESSAGE_EDITED_V1: &str = "fechatter.messages.message.edited.v1";
    pub const MESSAGE_DELETED_V1: &str = "fechatter.messages.message.deleted.v1";

    /// Chat domain subjects using new naming convention
    pub const CHAT_MEMBER_JOINED_V1: &str = "fechatter.chats.member.joined.v1";
    pub const CHAT_MEMBER_LEFT_V1: &str = "fechatter.chats.member.left.v1";

    /// System domain subjects
    pub const DUPLICATE_MESSAGE_V1: &str = "fechatter.system.duplicate.detected.v1";

    /// Search domain subjects
    pub const SEARCH_INDEX_V1: &str = "fechatter.search.index.v1";
}
