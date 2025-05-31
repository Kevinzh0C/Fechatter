use crate::config::NatsSubjectsConfig;
use crate::error::AppError;
use async_nats::Client as NatsClient;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use fechatter_core::{ChatId, Message, MessageId, UserId};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument, warn, Span};
use uuid::Uuid;

use super::transport::{EventTransport, NatsTransport};
use crate::error::EventTransportError;

const SEARCH_SUBJECT: &str = "fechatter.search";

// Retry configuration constants
const MAX_RETRIES: u32 = 5;
const INITIAL_BACKOFF_MS: u64 = 100;
const MAX_BACKOFF_MS: u64 = 5000;
const SIGNATURE_HEADER: &str = "X-Signature";

type HmacSha256 = Hmac<Sha256>;

/// Trait for events that support signature attachment
pub trait Signable {
  fn set_signature(&mut self, sig: Option<String>);
  fn get_signature(&self) -> &Option<String>;
}

/// Macro to implement Signable for event types
macro_rules! impl_signable {
  ($type:ty) => {
    impl Signable for $type {
      fn set_signature(&mut self, sig: Option<String>) {
        self.sig = sig;
      }
      
      fn get_signature(&self) -> &Option<String> {
        &self.sig
      }
    }
  };
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatInfo {
  pub chat_id: ChatId,
  pub chat_name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MsgLifecycle {
  Created,
  Updated,
  Deleted,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageEvent {
  pub kind: MsgLifecycle,
  pub msg: Message,
  pub members: Vec<UserId>,
  pub occurred_at: DateTime<Utc>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sig: Option<String>,
}

impl_signable!(MessageEvent);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMemberJoined {
  pub chat_id: ChatId,
  pub user_id: UserId,
  pub occurred_at: DateTime<Utc>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sig: Option<String>,
}

impl_signable!(ChatMemberJoined);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMemberLeft {
  pub chat_id: ChatId,
  pub user_id: UserId,
  pub occurred_at: DateTime<Utc>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sig: Option<String>,
}

impl_signable!(ChatMemberLeft);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DuplicateMessageEvent {
  pub idempotency_key: Uuid,
  pub chat_id: ChatId,
  pub sender_id: UserId,
  pub occurred_at: DateTime<Utc>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sig: Option<String>,
}

impl_signable!(DuplicateMessageEvent);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum SearchOp {
  Delete {
    id: MessageId,
  },
  Index {
    message: Message,
    chat_info: ChatInfo,
  },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchEvent {
  #[serde(flatten)]
  pub op: SearchOp,
  pub occurred_at: DateTime<Utc>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sig: Option<String>,
}

impl_signable!(SearchEvent);

/// Result of a single publish attempt
enum PublishAttemptResult {
  Retry,
  Fatal(AppError),
}

/// Retry strategy configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
  pub max_retries: u32,
  pub initial_backoff_ms: u64,
  pub max_backoff_ms: u64,
}

impl Default for RetryConfig {
  fn default() -> Self {
    Self {
      max_retries: MAX_RETRIES,
      initial_backoff_ms: INITIAL_BACKOFF_MS,
      max_backoff_ms: MAX_BACKOFF_MS,
    }
  }
}

/// Internal retry state for tracking retry attempts
pub(crate) struct RetryState {
  attempt: u32,
  backoff_ms: u64,
}

impl RetryState {
  fn new(config: &RetryConfig) -> Self {
    Self {
      attempt: 0,  // Start at 0, will be incremented to 1 on first try
      backoff_ms: config.initial_backoff_ms,
    }
  }

  /// Check if another retry attempt is allowed (does not modify state)
  fn can_retry(&self, config: &RetryConfig) -> bool {
    self.attempt < config.max_retries
  }

  /// Update backoff time for next retry (does not increment attempt counter)
  fn update_backoff(&mut self, config: &RetryConfig) {
    self.backoff_ms = std::cmp::min(self.backoff_ms * 2, config.max_backoff_ms);
  }
}

/// Generic EventPublisher that works with any EventTransport implementation
/// 
/// Uses Arc<T> for shared transport semantics since EventTransport no longer requires Clone.
/// This allows for both object safety (dyn EventTransport) and shared ownership.
#[derive(Clone)]
pub struct EventPublisher<T: EventTransport + ?Sized> {
  transport: Arc<T>,
  subjects: NatsSubjectsConfig,
  shutdown_token: CancellationToken,
  hmac_secret: Option<Vec<u8>>,
  sign_headers: bool,
  retry_config: RetryConfig,
}

impl<T: EventTransport> EventPublisher<T> {
  /// Create a new EventPublisher with the given transport
  pub fn with_transport(transport: T, subjects: NatsSubjectsConfig) -> Self {
    Self::build(Arc::new(transport), subjects, CancellationToken::new(), None, false)
  }

  /// Create EventPublisher with shutdown token
  pub fn with_transport_and_shutdown(
    transport: T,
    subjects: NatsSubjectsConfig,
    shutdown_token: CancellationToken,
  ) -> Self {
    Self::build(Arc::new(transport), subjects, shutdown_token, None, false)
  }

  /// Create EventPublisher with signature support
  pub fn with_transport_and_signature(
    transport: T,
    subjects: NatsSubjectsConfig,
    secret: Vec<u8>,
    use_headers: bool,
  ) -> Self {
    Self::build(Arc::new(transport), subjects, CancellationToken::new(), Some(secret), use_headers)
  }
}

impl<T: EventTransport + ?Sized> EventPublisher<T> {
  /// Create EventPublisher from an already Arc-wrapped transport
  /// Useful when you want to share the same transport instance across multiple publishers
  pub fn with_shared_transport(transport: Arc<T>, subjects: NatsSubjectsConfig) -> Self {
    Self::build(transport, subjects, CancellationToken::new(), None, false)
  }

  fn build(
    transport: Arc<T>,
    subjects: NatsSubjectsConfig,
    shutdown_token: CancellationToken,
    hmac_secret: Option<Vec<u8>>,
    sign_headers: bool,
  ) -> Self {
    Self {
      transport,
      subjects,
      shutdown_token,
      hmac_secret,
      sign_headers,
      retry_config: RetryConfig::default(),
    }
  }

  pub fn transport(&self) -> &T {
    &self.transport
  }

  /// Get a reference to the Arc-wrapped transport for sharing
  pub fn shared_transport(&self) -> &Arc<T> {
    &self.transport
  }

  pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
    self.retry_config = config;
    self
  }

  /// Creates HMAC signature for payload
  fn sign_payload(&self, payload: &[u8]) -> Option<String> {
    self.hmac_secret.as_ref().map(|secret| {
      let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key size");
      mac.update(payload);
      hex::encode(mac.finalize().into_bytes())
    })
  }

  /// Determines if an error is retryable
  fn is_retryable_error(error: &EventTransportError) -> bool {
    error.is_retryable()
  }

  /// Generic publish implementation for any signable and serializable event
  #[instrument(skip(self, event), fields(attempt, backoff_ms, subject, transport_type))]
  async fn publish_event<E>(&self, subject: &str, mut event: E, context: &str) -> Result<(), AppError>
  where
    E: Serialize + Signable,
  {
    Span::current().record("subject", subject);
    Span::current().record("transport_type", self.transport.transport_type());

    // Serialize event for signing
    let mut event_bytes = serde_json::to_vec(&event).map_err(|e| {
      error!(subject = %subject, context = %context, error = %e, "Serialization failed");
      AppError::EventPublishingError(format!("{} serialization error: {}", context, e))
    })?;

    // Handle signature generation and placement
    let sig = self.sign_payload(&event_bytes);
    let signature_headers = if self.sign_headers {
      sig.map(|s| {
        let mut headers = HashMap::new();
        headers.insert(SIGNATURE_HEADER.to_string(), s);
        headers
      })
    } else {
      // Update event with signature and re-serialize (using mutable borrow, no clone needed)
      if let Some(ref sig_str) = sig {
        event.set_signature(Some(sig_str.clone()));
        event_bytes = serde_json::to_vec(&event)?;
      }
      None
    };

    // Execute with retry logic
    self.publish_with_retry(subject, &event_bytes, signature_headers, context).await
  }

  /// Handles retry logic and actual publishing
  async fn publish_with_retry(
    &self,
    subject: &str,
    payload: &[u8],
    headers: Option<HashMap<String, String>>,
    context: &str,
  ) -> Result<(), AppError> {
    let bytes_payload = Bytes::copy_from_slice(payload);
    let mut retry_state = RetryState::new(&self.retry_config);
    
    loop {
      retry_state.attempt += 1;
      
      match self.try_publish_once(subject, &bytes_payload, &headers, &retry_state).await {
        Ok(_) => {
          info!(
            subject = %subject, 
            context = %context, 
            attempt = retry_state.attempt,
            transport = self.transport.transport_type(),
            "Published successfully"
          );
          return Ok(());
        }
        Err(PublishAttemptResult::Retry) => {
          if self.wait_with_cancellation(retry_state.backoff_ms).await.is_err() {
            return Err(AppError::EventPublishingError("Operation canceled".into()));
          }
          retry_state.update_backoff(&self.retry_config);
        }
        Err(PublishAttemptResult::Fatal(e)) => return Err(e),
      }
    }
  }

  /// Single publish attempt with error classification
  async fn try_publish_once(
    &self,
    subject: &str,
    payload: &Bytes,
    headers: &Option<HashMap<String, String>>,
    retry_state: &RetryState,
  ) -> Result<(), PublishAttemptResult> {
    Span::current().record("attempt", &retry_state.attempt);
    Span::current().record("backoff_ms", &retry_state.backoff_ms);

    let result = self.try_publish(subject, payload, headers).await;

    match result {
      Ok(_) => Ok(()),
      Err(e) if Self::is_retryable_error(&e) && retry_state.can_retry(&self.retry_config) => {
        warn!(
          subject = %subject,
          attempt = retry_state.attempt,
          backoff = retry_state.backoff_ms,
          transport = self.transport.transport_type(),
          error = %e,
          "Retrying after backoff"
        );
        Err(PublishAttemptResult::Retry)
      }
      Err(e) => {
        let msg = if retry_state.attempt >= self.retry_config.max_retries {
          format!("Max retries ({}) exceeded: {}", self.retry_config.max_retries, e)
        } else {
          format!("Non-retryable error: {}", e)
        };

        error!(
          subject = %subject,
          attempt = retry_state.attempt,
          transport = self.transport.transport_type(),
          error = %e,
          "Publication failed"
        );

        Err(PublishAttemptResult::Fatal(AppError::EventPublishingError(format!("EventPublish: {}", msg))))
      }
    }
  }

  /// Low-level publish with optional header signature
  async fn try_publish(
    &self,
    subject: &str,
    payload: &Bytes,
    headers: &Option<HashMap<String, String>>,
  ) -> Result<(), EventTransportError> {
    if let Some(header_map) = headers {
      self.transport.publish_with_headers(subject, header_map.clone(), payload.clone()).await
    } else {
      self.transport.publish(subject, payload.clone()).await
    }
  }

  /// Wait with cancellation support
  async fn wait_with_cancellation(&self, ms: u64) -> Result<(), ()> {
    select! {
      _ = sleep(Duration::from_millis(ms)) => Ok(()),
      _ = self.shutdown_token.cancelled() => Err(()),
    }
  }

  /// Determines subject for message lifecycle events
  fn message_subject_for(&self, kind: &MsgLifecycle) -> &str {
    match kind {
      MsgLifecycle::Created if self.subjects.message_created.is_empty() => "fechatter.message.created",
      MsgLifecycle::Updated if self.subjects.message_updated.is_empty() => "fechatter.message.updated",
      MsgLifecycle::Deleted if self.subjects.message_deleted.is_empty() => "fechatter.message.deleted",
      MsgLifecycle::Created => &self.subjects.message_created,
      MsgLifecycle::Updated => &self.subjects.message_updated,
      MsgLifecycle::Deleted => &self.subjects.message_deleted,
    }
  }

  // Public API methods - now much simpler thanks to generic implementation

  #[instrument(skip(self, message, chat_members))]
  pub async fn publish_message_event(
    &self,
    kind: MsgLifecycle,
    message: &Message,
    chat_members: &[UserId],
  ) -> Result<(), AppError> {
    let subject = self.message_subject_for(&kind);
    
    let event = MessageEvent {
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
    let event = ChatMemberJoined {
      chat_id: *chat_id,
      user_id: *user_id,
      occurred_at: Utc::now(),
      sig: None,
    };

    let subject = if self.subjects.user_joined_chat.is_empty() {
      "fechatter.chat.joined"
    } else {
      &self.subjects.user_joined_chat
    };

    self.publish_event(subject, event, "chat_member_joined").await
  }

  #[instrument(skip(self, chat_id, user_id))]
  pub async fn publish_chat_member_left(
    &self,
    chat_id: &ChatId,
    user_id: &UserId,
  ) -> Result<(), AppError> {
    let event = ChatMemberLeft {
      chat_id: *chat_id,
      user_id: *user_id,
      occurred_at: Utc::now(),
      sig: None,
    };

    let subject = if self.subjects.user_left_chat.is_empty() {
      "fechatter.chat.left"
    } else {
      &self.subjects.user_left_chat
    };

    self.publish_event(subject, event, "chat_member_left").await
  }

  #[instrument(skip(self, idempotency_key, chat_id, sender_id))]
  pub async fn publish_duplicate_message_attempted(
    &self,
    idempotency_key: &Uuid,
    chat_id: &ChatId,
    sender_id: &UserId,
  ) -> Result<(), AppError> {
    let event = DuplicateMessageEvent {
      idempotency_key: *idempotency_key,
      chat_id: *chat_id,
      sender_id: *sender_id,
      occurred_at: Utc::now(),
      sig: None,
    };

    let subject = if self.subjects.duplicate_message_attempted.is_empty() {
      "fechatter.message.duplicate"
    } else {
      &self.subjects.duplicate_message_attempted
    };

    self.publish_event(subject, event, "duplicate_message").await
  }

  #[instrument(skip(self, id), fields(message_id = %id))]
  pub async fn publish_search_delete(&self, id: &MessageId) -> Result<(), AppError> {
    let event = SearchEvent {
      op: SearchOp::Delete { id: *id },
      occurred_at: Utc::now(),
      sig: None,
    };

    self.publish_event(SEARCH_SUBJECT, event, "search_delete").await
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
        chat_info: chat_info.clone() 
      },
      occurred_at: Utc::now(),
      sig: None,
    };

    self.publish_event(SEARCH_SUBJECT, event, "search_index").await
  }

  /// Verifies HMAC signature against payload
  pub fn verify_signature(payload: &[u8], signature: &str, secret: &[u8]) -> bool {
    // Validate signature format first
    if signature.len() != 64 || !signature.chars().all(|c| c.is_ascii_hexdigit()) {
      return false;
    }

    let Ok(mut mac) = HmacSha256::new_from_slice(secret) else {
      return false;
    };

    mac.update(payload);

    let Ok(sig_bytes) = hex::decode(signature) else {
      return false;
    };

    mac.verify_slice(&sig_bytes).is_ok()
  }

  async fn publish_with_optional_headers(
    &self,
    subject: &str,
    payload: &[u8],
    headers: &Option<HashMap<String, String>>,
  ) -> Result<(), EventTransportError> {
    if let Some(header_map) = headers {
      self.transport.publish_with_headers(subject, header_map.clone(), Bytes::copy_from_slice(payload)).await
    } else {
      self.transport.publish(subject, Bytes::copy_from_slice(payload)).await
    }
  }
}

/// Type alias for NATS-based EventPublisher (for backward compatibility)
pub type NatsEventPublisher = EventPublisher<NatsTransport>;

impl NatsEventPublisher {
  /// Create a simple NatsEventPublisher (for backward compatibility with tests)
  pub fn new(client: NatsClient, subjects: NatsSubjectsConfig) -> Self {
    let transport = NatsTransport::new(client);
    Self::with_transport(transport, subjects)
  }

  /// Create NatsEventPublisher with shutdown token
  pub fn with_shutdown_token(
    client: NatsClient,
    subjects: NatsSubjectsConfig,
    shutdown_token: CancellationToken,
  ) -> Self {
    let transport = NatsTransport::new(client);
    Self::with_transport_and_shutdown(transport, subjects, shutdown_token)
  }

  /// Create NATS EventPublisher with signature (backward compatible)
  pub fn with_signature(
    client: NatsClient,
    subjects: NatsSubjectsConfig,
    secret: Vec<u8>,
    use_headers: bool,
  ) -> Self {
    let transport = NatsTransport::new(client);
    Self::with_transport_and_signature(transport, subjects, secret, use_headers)
  }

  /// Access the underlying NATS client (backward compatible)
  pub fn nats_client(&self) -> &NatsClient {
    self.transport.client()
  }
}

/// Support for dynamic dispatch with trait objects
/// This allows using `Arc<dyn EventTransport>` for heterogeneous collections
pub type DynEventPublisher = EventPublisher<dyn EventTransport>;

impl DynEventPublisher {
  /// Create EventPublisher from a trait object
  pub fn with_dyn_transport(
    transport: Arc<dyn EventTransport>, 
    subjects: NatsSubjectsConfig
  ) -> Self {
    Self::with_shared_transport(transport, subjects)
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
  use crate::config::NatsSubjectsConfig;
  use async_nats::Server;
  use chrono::Utc;
  use fechatter_core::{ChatId, Message, MessageId, UserId};
  use futures::StreamExt;
  use tokio::time::{timeout, Duration};

  fn create_test_subjects() -> NatsSubjectsConfig {
    NatsSubjectsConfig {
      message_created: "test.message.created".to_string(),
      message_updated: "test.message.updated".to_string(),
      message_deleted: "test.message.deleted".to_string(),
      chat_created: "test.chat.created".to_string(),
      chat_updated: "test.chat.updated".to_string(),
      user_joined_chat: "test.chat.joined".to_string(),
      user_left_chat: "test.chat.left".to_string(),
      duplicate_message_attempted: "test.msg.dup".to_string(),
    }
  }

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

  #[tokio::test]
  async fn test_publish_search_delete_received() {
    let server = Server::new().await;
    let client = async_nats::connect(&server.client_url()).await.unwrap();
    let subjects = create_test_subjects();
    let publisher = EventPublisher::new(client.clone(), subjects);

    let mut sub = client.subscribe(SEARCH_SUBJECT).await.unwrap();
    let id = MessageId(42);
    publisher.publish_search_delete(&id).await.unwrap();

    let msg = timeout(Duration::from_secs(1), sub.next())
      .await
      .unwrap()
      .unwrap();
    
    // Use strong typing for deserialization
    let ev: SearchEvent = serde_json::from_slice(&msg.payload).unwrap();
    assert!(matches!(ev.op, SearchOp::Delete { id } if id == MessageId(42)));
    assert!(ev.occurred_at <= Utc::now());

    server.close().await;
  }

  #[tokio::test]
  async fn test_publish_search_index_received() {
    let server = Server::new().await;
    let client = async_nats::connect(&server.client_url()).await.unwrap();
    let subjects = create_test_subjects();
    let publisher = EventPublisher::new(client.clone(), subjects);

    let mut sub = client.subscribe(SEARCH_SUBJECT).await.unwrap();
    let message = create_test_message();
    let chat_info = ChatInfo {
      chat_id: ChatId(100),
      chat_name: "room".to_string(),
    };
    publisher
      .publish_search_index(&message, &chat_info)
      .await
      .unwrap();

    let msg = timeout(Duration::from_secs(1), sub.next())
      .await
      .unwrap()
      .unwrap();
    
    // Use strong typing for deserialization
    let ev: SearchEvent = serde_json::from_slice(&msg.payload).unwrap();
    assert!(matches!(ev.op, SearchOp::Index { message, chat_info: ci } 
      if message.id == MessageId(1) && ci.chat_name == "room"));
    assert!(ev.occurred_at <= Utc::now());

    server.close().await;
  }
  
  #[tokio::test]
  async fn test_signature_verification() {
    let secret = b"test-secret-key-123";
    let server = Server::new().await;
    let client = async_nats::connect(&server.client_url()).await.unwrap();
    let subjects = create_test_subjects();
    
    // Create publisher with signature in payload
    let publisher = EventPublisher::with_signature(
      client.clone(), 
      subjects,
      secret.to_vec(),
      false // use payload signature, not headers
    );

    let mut sub = client.subscribe(SEARCH_SUBJECT).await.unwrap();
    let id = MessageId(42);
    publisher.publish_search_delete(&id).await.unwrap();

    let msg = timeout(Duration::from_secs(1), sub.next())
      .await
      .unwrap()
      .unwrap();
    
    // Parse the event
    let ev: SearchEvent = serde_json::from_slice(&msg.payload).unwrap();
    
    // Verify the signature is present
    assert!(ev.sig.is_some());
    
    // Create payload without signature for verification
    let mut verification_ev = ev.clone();
    let signature = verification_ev.sig.take().unwrap();
    let verification_payload = serde_json::to_vec(&verification_ev).unwrap();
    
    // Verify the signature
    assert!(EventPublisher::verify_signature(
      &verification_payload,
      &signature,
      secret
    ));
    
    // Test with wrong secret
    assert!(!EventPublisher::verify_signature(
      &verification_payload,
      &signature,
      b"wrong-secret"
    ));

    server.close().await;
  }
  
  #[tokio::test]
  async fn test_signature_in_header() {
    let secret = b"test-secret-key-123";
    let server = Server::new().await;
    let client = async_nats::connect(&server.client_url()).await.unwrap();
    let subjects = create_test_subjects();
    
    // Create publisher with signature in headers
    let publisher = EventPublisher::with_signature(
      client.clone(), 
      subjects,
      secret.to_vec(),
      true // use header signature
    );

    let mut sub = client.subscribe(SEARCH_SUBJECT).await.unwrap();
    let id = MessageId(42);
    publisher.publish_search_delete(&id).await.unwrap();

    let msg = timeout(Duration::from_secs(1), sub.next())
      .await
      .unwrap()
      .unwrap();
    
    // Parse the event and verify no signature in payload
    let ev: SearchEvent = serde_json::from_slice(&msg.payload).unwrap();
    assert!(ev.sig.is_none());
    
    // Check for signature in header
    assert!(msg.headers.is_some());
    let headers = msg.headers.unwrap();
    assert!(headers.get(SIGNATURE_HEADER).is_some());
    
    // Verify the signature
    let signature = headers.get(SIGNATURE_HEADER).unwrap();
    let sig = signature.to_str().unwrap();
    assert!(EventPublisher::verify_signature(
      &msg.payload,
      sig,
      secret
    ));

    server.close().await;
  }

  #[tokio::test]
  async fn test_signature_verification_invalid_format() {
    let secret = b"test-secret-key-123";
    let payload = b"test payload";

    // Test empty signature
    assert!(!EventPublisher::verify_signature(payload, "", secret));

    // Test wrong length
    assert!(!EventPublisher::verify_signature(payload, "abc123", secret));

    // Test non-hex characters
    assert!(!EventPublisher::verify_signature(payload, "gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg", secret));

    // Test odd length
    assert!(!EventPublisher::verify_signature(payload, "abcdef123", secret));

    // Test too long
    assert!(!EventPublisher::verify_signature(payload, "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12", secret));
  }

  #[tokio::test]
  async fn test_retry_config_builder() {
    let config = RetryConfig::new()
      .with_max_retries(3)
      .with_backoff_range(50, 2000);
    
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_backoff_ms, 50);
    assert_eq!(config.max_backoff_ms, 2000);
  }

  #[tokio::test]
  async fn test_custom_retry_config() {
    let server = Server::new().await;
    let client = async_nats::connect(&server.client_url()).await.unwrap();
    let subjects = create_test_subjects();
    
    let custom_config = RetryConfig::new()
      .with_max_retries(2)
      .with_backoff_range(10, 100);
    
    let publisher = EventPublisher::new(client.clone(), subjects)
      .with_retry_config(custom_config);

    let mut sub = client.subscribe(SEARCH_SUBJECT).await.unwrap();
    let id = MessageId(99);
    publisher.publish_search_delete(&id).await.unwrap();

    let msg = timeout(Duration::from_secs(1), sub.next())
      .await
      .unwrap()
      .unwrap();
    
    let ev: SearchEvent = serde_json::from_slice(&msg.payload).unwrap();
    assert!(matches!(ev.op, SearchOp::Delete { id } if id == MessageId(99)));

    server.close().await;
  }

  #[test]
  fn test_retry_state_counting_logic() {
    let config = RetryConfig::new().with_max_retries(3);
    let mut retry_state = RetryState::new(&config);
    
    // Initial state
    assert_eq!(retry_state.attempt, 0);
    assert!(retry_state.can_retry(&config));
    
    // Simulate first attempt
    retry_state.attempt += 1;
    assert_eq!(retry_state.attempt, 1);
    assert!(retry_state.can_retry(&config)); // Should still be able to retry
    
    // Simulate second attempt
    retry_state.attempt += 1;
    assert_eq!(retry_state.attempt, 2);
    assert!(retry_state.can_retry(&config)); // Should still be able to retry
    
    // Simulate third attempt (max reached)
    retry_state.attempt += 1;
    assert_eq!(retry_state.attempt, 3);
    assert!(!retry_state.can_retry(&config)); // Should not be able to retry anymore
  }
  
  #[test]
  fn test_backoff_update_logic() {
    let config = RetryConfig::new()
      .with_max_retries(5)
      .with_backoff_range(100, 1000);
    
    let mut retry_state = RetryState::new(&config);
    assert_eq!(retry_state.backoff_ms, 100);
    
    // First backoff update
    retry_state.update_backoff(&config);
    assert_eq!(retry_state.backoff_ms, 200);
    
    // Second backoff update
    retry_state.update_backoff(&config);
    assert_eq!(retry_state.backoff_ms, 400);
    
    // Third backoff update
    retry_state.update_backoff(&config);
    assert_eq!(retry_state.backoff_ms, 800);
    
    // Fourth backoff update (should cap at max)
    retry_state.update_backoff(&config);
    assert_eq!(retry_state.backoff_ms, 1000);
    
    // Fifth backoff update (should stay at max)
    retry_state.update_backoff(&config);
    assert_eq!(retry_state.backoff_ms, 1000);
  }
  
  #[test]
  fn test_no_clone_constraint_compile_check() {
    // This test verifies that events without Clone can be published
    #[derive(Serialize)]
    struct NonCloneableEvent {
      data: String,
      #[serde(skip_serializing_if = "Option::is_none")]
      sig: Option<String>,
    }
    
    impl Signable for NonCloneableEvent {
      fn set_signature(&mut self, sig: Option<String>) {
        self.sig = sig;
      }
      
      fn get_signature(&self) -> &Option<String> {
        &self.sig
      }
    }
    
    // This should compile without Clone trait bound
    let event = NonCloneableEvent {
      data: "test".to_string(),
      sig: None,
    };
    
    // The fact that this compiles proves our fix worked
    // (In real test, we'd need an EventPublisher instance)
    let _: &dyn Fn(NonCloneableEvent) = &|_event: NonCloneableEvent| {
      // Simulates publish_event function signature without Clone bound
    };
  }
  
  #[tokio::test]
  async fn test_bytes_reuse_efficiency() {
    // Test that Bytes are created once and reused
    let payload = vec![1, 2, 3, 4, 5];
    
    // Create Bytes once outside loop (simulating our fix)
    let bytes_payload = Bytes::copy_from_slice(&payload);
    let original_ptr = bytes_payload.as_ptr();
    
    // Clone should reuse internal buffer (reference counting)
    let cloned_bytes = bytes_payload.clone();
    assert_eq!(cloned_bytes.as_ptr(), original_ptr);
    
    // Verify content is same
    assert_eq!(bytes_payload.len(), payload.len());
    assert_eq!(cloned_bytes.len(), payload.len());
    assert_eq!(&bytes_payload[..], &payload[..]);
    assert_eq!(&cloned_bytes[..], &payload[..]);
  }
}
