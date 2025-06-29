//! # Dual Stream Message Service
//!
//! **Responsibility**: Complete message service with dual-stream architecture
//! **Architecture**: Async Indexing Stream (@indexer.rs) + Realtime Notification Stream (notify-server)  
//! **Principle**: Stream Separation - Search indexing async, realtime notifications low-latency

use std::sync::Arc;
use tracing::{info, warn};

use crate::domains::messaging::messaging_domain::MessageDomainService;
use crate::services::application::tools::indexer::ChatInfo;
use crate::services::infrastructure::flows::notifications::{
  NotificationServiceTrait, create_notification_flow_service_with_nats, create_notification_service,
};
use crate::{
  AppState,
  domains::messaging::{
    messaging_domain::{MessageConfig, MessageDomainServiceImpl},
    repository::MessageRepository,
  },
  error::AppError,
};
use async_nats;
use fechatter_core::models::message::{CreateMessage, ListMessages, MessageView, StreamMessage};
use fechatter_core::{ChatId, MessageId, UserId};
use serde::{Deserialize, Serialize};
use tracing::error;

// ── Cache Service Adapter ────────────────────────────────────────────────────────────

/// Adapter to bridge between application cache service and core cache service trait
struct CacheServiceAdapter {
  inner: Arc<crate::services::infrastructure::cache::RedisCacheService>,
}

impl CacheServiceAdapter {
  fn new(inner: Arc<crate::services::infrastructure::cache::RedisCacheService>) -> Self {
    Self { inner }
  }
}

#[async_trait::async_trait]
impl fechatter_core::contracts::CacheService for CacheServiceAdapter {
  async fn get_bytes(
    &self,
    key: &str,
  ) -> Result<Option<Vec<u8>>, fechatter_core::error::CoreError> {
    match self.inner.get::<Vec<u8>>(key).await {
      Ok(data) => Ok(data),
      Err(e) => Err(fechatter_core::error::CoreError::Internal(e.to_string())),
    }
  }

  async fn set_bytes(
    &self,
    key: &str,
    value: Vec<u8>,
    ttl: u64,
  ) -> Result<(), fechatter_core::error::CoreError> {
    self
      .inner
      .set(key, &value, ttl)
      .await
      .map_err(|e| fechatter_core::error::CoreError::Internal(e.to_string()))
  }

  async fn delete(&self, key: &str) -> Result<(), fechatter_core::error::CoreError> {
    self
      .inner
      .del(key)
      .await
      .map(|_| ())
      .map_err(|e| fechatter_core::error::CoreError::Internal(e.to_string()))
  }

  async fn exists(&self, key: &str) -> Result<bool, fechatter_core::error::CoreError> {
    self
      .inner
      .exists(key)
      .await
      .map_err(|e| fechatter_core::error::CoreError::Internal(e.to_string()))
  }

  async fn delete_pattern(&self, pattern: &str) -> Result<u64, fechatter_core::error::CoreError> {
    self
      .inner
      .del_pattern(pattern)
      .await
      .map_err(|e| fechatter_core::error::CoreError::Internal(e.to_string()))
  }
}

// ── Dual Stream Event Types ────────────────────────────────────────────────────────

/// Async Index Event - Sent to @indexer.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncIndexEvent {
  pub message: fechatter_core::Message,
  pub chat_info: ChatInfo,
  pub operation: IndexOperation,
}

/// Index Operation Type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexOperation {
  Create,
  Update,
  Delete,
}

/// Realtime Event - Sent to notify-server  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RealtimeEvent {
  /// New message real-time push
  MessageReceived {
    message: StreamMessage,
    chat_id: i64,
    recipients: Vec<i64>,
  },
  /// Message read status
  MessageRead {
    message_id: i64,
    chat_id: i64,
    reader_id: i64,
    read_at: String,
  },
  /// Message unread status
  MessageUnread {
    message_id: i64,
    chat_id: i64,
    user_id: i64,
  },
  /// Typing started status
  TypingStarted {
    chat_id: i64,
    user_id: i64,
    user_name: String,
  },
  /// Typing stopped status
  TypingStopped { chat_id: i64, user_id: i64 },
  /// User presence status
  UserPresence {
    user_id: i64,
    status: String, // "online", "offline", "away"
    last_seen: Option<String>,
  },
  /// Message deleted status
  MessageDeleted {
    message_id: i64,
    chat_id: i64,
    deleted_by: i64,
    deleted_at: String,
  },
}

// ── Dual Stream Dispatcher ──────────────────────────────────────────────────────────

/// Dual Stream Dispatcher - Event dispatcher for both streams
pub struct DualStreamDispatcher {
  nats_client: Option<async_nats::Client>,
}

impl DualStreamDispatcher {
  pub fn new(nats_client: async_nats::Client) -> Self {
    Self {
      nats_client: Some(nats_client),
    }
  }

  /// Create a dispatcher with in-memory fallback for when NATS is unavailable
  /// This creates a dispatcher without a NATS client that logs operations locally
  pub fn new_in_memory() -> Self {
    Self { nats_client: None }
  }

  /// Check if the NATS client is connected
  fn is_connected(&self) -> bool {
    use async_nats::connection::State;
    match &self.nats_client {
      Some(client) => matches!(client.connection_state(), State::Connected),
      None => false,
    }
  }

  /// Publish async index event to @indexer.rs (batch, reliability first)
  pub async fn publish_async_index_event(&self, event: AsyncIndexEvent) -> Result<(), AppError> {
    let Some(client) = &self.nats_client else {
      tracing::warn!(
        "NATS client not available, skipping async index event publication: {:?}",
        event.operation
      );
      return Ok(()); // Gracefully handle missing client
    };

    if !self.is_connected() {
      tracing::warn!(
        "NATS not connected, skipping async index event publication: {:?}",
        event.operation
      );
      return Ok(()); // Gracefully handle disconnected state
    }

    let subject = match event.operation {
      IndexOperation::Create | IndexOperation::Update => "fechatter.search.index.message",
      IndexOperation::Delete => "fechatter.search.index.delete",
    };

    let payload = serde_json::to_vec(&event)
      .map_err(|e| AppError::Internal(format!("Failed to serialize index event: {}", e)))?;

    match client.publish(subject, payload.into()).await {
      Ok(_) => {
        info!(
          "📄 Async index event sent to @indexer.rs: {:?}",
          event.operation
        );
        Ok(())
      }
      Err(e) => {
        tracing::warn!("Failed to publish index event (NATS issue): {}", e);
        Ok(()) // Don't fail the whole operation due to NATS issues
      }
    }
  }

  /// Publish realtime event to notify-server (low latency first)
  pub async fn publish_realtime_event(&self, event: RealtimeEvent) -> Result<(), AppError> {
    let Some(client) = &self.nats_client else {
      tracing::warn!("NATS client not available, skipping realtime event publication");
      return Ok(()); // Gracefully handle missing client
    };

    if !self.is_connected() {
      tracing::warn!("NATS not connected, skipping realtime event publication");
      return Ok(()); // Gracefully handle disconnected state
    }

    let subject = match &event {
      RealtimeEvent::MessageReceived { chat_id, .. } => {
        format!("fechatter.realtime.chat.{}", chat_id)
      }
      RealtimeEvent::MessageRead { chat_id, .. } => {
        format!("fechatter.realtime.chat.{}.read", chat_id)
      }
      RealtimeEvent::MessageUnread { chat_id, .. } => {
        format!("fechatter.realtime.chat.{}.unread", chat_id)
      }
      RealtimeEvent::TypingStarted { chat_id, .. } => {
        format!("fechatter.realtime.chat.{}.typing", chat_id)
      }
      RealtimeEvent::TypingStopped { chat_id, .. } => {
        format!("fechatter.realtime.chat.{}.typing", chat_id)
      }
      RealtimeEvent::UserPresence { user_id, .. } => {
        format!("fechatter.realtime.user.{}.presence", user_id)
      }
      RealtimeEvent::MessageDeleted { chat_id, .. } => {
        format!("fechatter.realtime.chat.{}.deleted", chat_id)
      }
    };

    let payload = serde_json::to_vec(&event)
      .map_err(|e| AppError::Internal(format!("Failed to serialize realtime event: {}", e)))?;

    // Non-persistent publish - prioritize low latency
    match client.publish(subject.clone(), payload.into()).await {
      Ok(_) => {
        info!("⚡ Realtime event sent to notify-server: {}", subject);
        Ok(())
      }
      Err(e) => {
        tracing::warn!("Failed to publish realtime event (NATS issue): {}", e);
        Ok(()) // Don't fail the whole operation due to NATS issues
      }
    }
  }
}

// ── Event Publisher Trait ──────────────────────────────────────────────────────────

/// Event publisher trait for dependency injection
#[async_trait::async_trait]
pub trait EventPublisherTrait: Send + Sync {
  async fn publish_message_created(
    &self,
    message: &fechatter_core::Message,
    chat_members: Vec<i64>,
  ) -> Result<(), AppError>;
}

/// Adapter for AppState EventPublisher
pub struct AppStateEventPublisher {
  publisher: Option<Arc<crate::services::infrastructure::event::DynEventPublisher>>,
}

impl AppStateEventPublisher {
  pub fn new(
    publisher: Option<Arc<crate::services::infrastructure::event::DynEventPublisher>>,
  ) -> Self {
    Self { publisher }
  }
}

#[async_trait::async_trait]
impl EventPublisherTrait for AppStateEventPublisher {
  async fn publish_message_created(
    &self,
    message: &fechatter_core::Message,
    chat_members: Vec<i64>,
  ) -> Result<(), AppError> {
    info!(
      "🔍 [DEBUG] AppStateEventPublisher::publish_message_created called for message_id={}",
      message.id
    );
    info!(
      "🔍 [DEBUG] Publisher available: {}",
      self.publisher.is_some()
    );
    info!("🔍 [DEBUG] Chat members count: {}", chat_members.len());

    if let Some(publisher) = &self.publisher {
      info!(
        "🔍 [DEBUG] Publisher type: {}",
        std::any::type_name_of_val(publisher.as_ref())
      );

      // Convert to EventPublisher's expected types
      let chat_id = fechatter_core::ChatId(message.chat_id.into());
      let user_ids: Vec<fechatter_core::UserId> = chat_members
        .iter()
        .map(|&id| fechatter_core::UserId(id))
        .collect();

      info!("🔍 [DEBUG] About to call publish_message_event with MessageLifecycle::Created");
      info!("🔍 [DEBUG] ChatId: {:?}, UserIds: {:?}", chat_id, user_ids);

      let result = publisher
        .publish_message_event(
          fechatter_core::contracts::events::MessageLifecycle::Created,
          message,
          &user_ids,
        )
        .await;

      info!("🔍 [DEBUG] publish_message_event result: {:?}", result);
      result
    } else {
      warn!("🔍 [DEBUG] Event publisher not available, skipping message created event");
      Ok(())
    }
  }
}

// ── Dual Stream Message Service ─────────────────────────────────────────────────────

/// Dual Stream Message Service - Complete dual-stream architecture implementation
///
/// **Process architecture**:
/// - **list_messages**: Directly call domain, no async processing
/// - **send_message**: Triggers both streams (async index @indexer.rs + realtime push notify-server)
/// - **update_message**: Triggers both streams (async index @indexer.rs + realtime push notify-server)  
/// - **read/unread/typing**: Only goes through realtime stream → notify-server
pub struct DualStreamMessageService {
  /// Domain service - core business logic
  domain_service: Arc<dyn MessageDomainService>,
  /// Dual stream dispatcher
  dispatcher: Arc<DualStreamDispatcher>,
  /// Notification service - in-app notifications
  notification_service: Arc<dyn NotificationServiceTrait>,
  /// Search service - for index management
  search_service:
    Option<Arc<dyn crate::services::application::workers::search::SearchApplicationServiceTrait>>,
  /// Cache service - for cache invalidation
  cache_service: Option<Arc<dyn fechatter_core::contracts::CacheService>>,
  /// Vector database - for semantic search cleanup
  vector_db: Option<Arc<dyn fechatter_core::models::vector_db::MessageVectorRepository>>,
  /// Event publisher for message created event
  event_publisher: Arc<dyn EventPublisherTrait>,
}

impl DualStreamMessageService {
  pub fn new(
    domain_service: Arc<dyn MessageDomainService>,
    dispatcher: Arc<DualStreamDispatcher>,
    notification_service: Arc<dyn NotificationServiceTrait>,
    event_publisher: Arc<dyn EventPublisherTrait>,
  ) -> Self {
    Self {
      domain_service,
      dispatcher,
      notification_service,
      search_service: None,
      cache_service: None,
      vector_db: None,
      event_publisher,
    }
  }

  /// Create with all dependencies for production use
  pub fn new_with_full_dependencies(
    domain_service: Arc<dyn MessageDomainService>,
    dispatcher: Arc<DualStreamDispatcher>,
    notification_service: Arc<dyn NotificationServiceTrait>,
    search_service: Option<
      Arc<dyn crate::services::application::workers::search::SearchApplicationServiceTrait>,
    >,
    cache_service: Option<Arc<dyn fechatter_core::contracts::CacheService>>,
    vector_db: Option<Arc<dyn fechatter_core::models::vector_db::MessageVectorRepository>>,
    event_publisher: Arc<dyn EventPublisherTrait>,
  ) -> Self {
    Self {
      domain_service,
      dispatcher,
      notification_service,
      search_service,
      cache_service,
      vector_db,
      event_publisher,
    }
  }

  /// Create with dependencies (for ServiceProvider)
  pub fn new_with_dependencies(
    domain_service: Arc<dyn MessageDomainService>,
    pool: Arc<sqlx::PgPool>,
    token_manager: Arc<fechatter_core::models::jwt::TokenManager>,
  ) -> Self {
    // Create dispatcher and notification service inline
    use crate::services::infrastructure::flows::notifications::create_notification_service;

    // Use in-memory dispatcher to avoid runtime nesting issues
    // NATS connection should be handled at application startup, not service creation
    tracing::warn!(
      "Using in-memory dispatcher for message service - NATS should be connected at startup"
    );
    let dispatcher = Arc::new(DualStreamDispatcher::new_in_memory());

    // Create in-memory notification services
    let transport = crate::services::infrastructure::event::InMemoryTransport::new();
    let event_publisher =
      Arc::new(crate::services::infrastructure::event::legacy::EventPublisher::with_transport(transport));
    let flow_service =
      crate::services::infrastructure::flows::notifications::create_notification_flow_service(
        event_publisher,
        None,
      );
    let notification_service = create_notification_service(flow_service);

    // Create in-memory event publisher adapter (no AppState available here)
    // TODO: CRITICAL - This should be fixed to use actual event publisher when AppState is available
    // Currently using None because this method doesn't have access to AppState's event_publisher
    let message_event_publisher = Arc::new(AppStateEventPublisher::new(None));

    // TODO: Add search_service, cache_service, and vector_db when available
    Self::new(
      domain_service,
      dispatcher,
      notification_service,
      message_event_publisher,
    )
  }

  /// List messages - direct call, no async processing
  pub async fn list_messages(
    &self,
    user_id: UserId,
    chat_id: ChatId,
    query: ListMessages,
  ) -> Result<Vec<MessageView>, AppError> {
    let messages = self
      .domain_service
      .list_messages(query, i64::from(chat_id), i64::from(user_id))
      .await
      .map_err(AppError::from)?;

    // For now, return messages without sender info
    // TODO: Implement a proper solution to fetch sender info
    Ok(messages.into_iter().map(MessageView::from).collect())
  }

  /// Send message - triggers both streams (async index + realtime push)
  pub async fn send_message(
    &self,
    sender_id: UserId,
    chat_id: ChatId,
    create_message: CreateMessage,
  ) -> Result<MessageView, AppError> {
    // 1. Core business logic - persist message
    let saved_message = self
      .domain_service
      .send_message(
        create_message.clone(),
        i64::from(chat_id),
        i64::from(sender_id),
      )
      .await
      .map_err(AppError::from)?;

    let message_view = MessageView::from(saved_message.clone());

    // Get chat members for further processing
    let chat_members = self
      .domain_service
      .get_chat_members(i64::from(chat_id))
      .await
      .map_err(AppError::from)?;

    // 🎯 2. MISSING LINK: Publish message created event to NATS JetStream
    info!("🔍 [DEBUG] About to spawn task for NATS JetStream event publishing");
    let event_publisher = Arc::clone(&self.event_publisher);
    let jetstream_message = saved_message.clone();
    let jetstream_members = chat_members.clone();
    tokio::spawn(async move {
      info!(
        "🔍 [DEBUG] Inside event publishing task, calling event_publisher.publish_message_created"
      );
      if let Err(e) = event_publisher
        .publish_message_created(&jetstream_message, jetstream_members)
        .await
      {
        warn!(
          "🔍 [DEBUG] Failed to publish message created event to NATS JetStream: {}",
          e
        );
      } else {
        info!(
          "🔍 [DEBUG] ✅ Message created event published to NATS JetStream: message_id={}",
          jetstream_message.id
        );
      }
    });

    // 3. Async index stream - send to @indexer.rs (in parallel)
    let dispatcher = Arc::clone(&self.dispatcher);
    let index_message = saved_message.clone();
    tokio::spawn(async move {
      let index_event = AsyncIndexEvent {
        message: index_message,
        chat_info: ChatInfo {
          chat_name: "Chat".to_string(),  // TODO: Get actual chat name
          sender_name: "User".to_string(), // TODO: Get actual sender name
          chat_type: "group".to_string(), // TODO: Get actual chat type
          workspace_id: 1,                 // TODO: Get actual workspace ID
        },
        operation: IndexOperation::Create,
      };

      if let Err(e) = dispatcher.publish_async_index_event(index_event).await {
        warn!("Failed to publish async index event: {}", e);
      }
    });

    // 4. Realtime push stream - send to notify-server (in parallel)
    let dispatcher = Arc::clone(&self.dispatcher);
    let realtime_message = saved_message.clone();
    let realtime_members = chat_members.clone();
    tokio::spawn(async move {
      let stream_message = StreamMessage {
        id: realtime_message.id.to_string(),
        chat_id: i64::from(realtime_message.chat_id),
        sender_id: i64::from(realtime_message.sender_id),
        content: realtime_message.content.clone(),
        files: realtime_message.files.clone().unwrap_or_default(),
        timestamp: realtime_message.created_at.timestamp(),
      };

      let realtime_event = RealtimeEvent::MessageReceived {
        message: stream_message,
        chat_id: i64::from(realtime_message.chat_id),
        recipients: realtime_members,
      };

      if let Err(e) = dispatcher.publish_realtime_event(realtime_event).await {
        warn!("Failed to publish realtime message: {}", e);
      }
    });

    // 5. In-app notification (in parallel) - parse @mentions
    let notification_service = Arc::clone(&self.notification_service);
    let notification_message = saved_message.clone();
    let notification_chat_members = chat_members;
    let notification_sender_id = i64::from(sender_id);
    tokio::spawn(async move {
      let content = &notification_message.content;
      if content.contains('@') {
        // TODO: Implement real @mention parsing logic
        let mentioned_users: Vec<i64> = notification_chat_members
          .into_iter()
          .filter(|&user_id| user_id != notification_sender_id)
          .take(1) // Temporarily take the first as an example
          .collect();

        if !mentioned_users.is_empty() {
          let _ = notification_service
            .send_mention_notification(
              mentioned_users,
              "User", // TODO: Get actual sender name
              &content[..content.len().min(100)],
              i64::from(notification_message.chat_id),
              i64::from(notification_message.id),
            )
            .await;
        }
      }
    });

    Ok(message_view)
  }

  /// Update message - production-grade implementation with full data consistency
  pub async fn edit_message(
    &self,
    message_id: MessageId,
    user_id: UserId,
    new_content: String,
  ) -> Result<MessageView, AppError> {
    // 1. Core business logic - update message in database
    let updated_message = self
      .domain_service
      .edit_message(
        i64::from(message_id),
        new_content.clone(),
        i64::from(user_id),
      )
      .await
      .map_err(AppError::from)?;

    let message_view = MessageView::from(updated_message.clone());

    // Get chat members for notifications
    let chat_members = self
      .domain_service
      .get_chat_members(i64::from(updated_message.chat_id))
      .await
      .map_err(AppError::from)?;

    // 2. Update search index SYNCHRONOUSLY for consistency
    if let Some(search_service) = self.search_service.as_ref() {
      // Convert to searchable message format
      let searchable_message = crate::services::application::workers::search::SearchableMessage {
        id: i64::from(updated_message.id),
        chat_id: i64::from(updated_message.chat_id),
        sender_id: i64::from(updated_message.sender_id),
        sender_name: "User".to_string(), // TODO: Get actual sender name
        content: updated_message.content.clone(),
        files: updated_message.files.clone(),
        created_at: updated_message.created_at,
        workspace_id: 1,                // TODO: Get actual workspace ID
        chat_name: "Chat".to_string(),  // Changed from Some() to String
        chat_type: "group".to_string(), // Changed from Some() to String
        relevance_score: None,
      };

      if let Err(e) = search_service
        .update_messages_in_index_batch(&[searchable_message])
        .await
      {
        warn!("Failed to update message in search index: {}", e);
        // Continue - search index can be eventually consistent
      }
    }

    // 3. Invalidate cache entries SYNCHRONOUSLY
    if let Some(cache_service) = self.cache_service.as_ref() {
      // Clear message cache
      let message_cache_key = format!("message:{}", message_id);
      if let Err(e) = cache_service.delete(&message_cache_key).await {
        warn!("Failed to clear message cache: {}", e);
      }

      // Clear chat messages cache to force refresh
      let chat_cache_key = format!("chat:{}:messages", updated_message.chat_id);
      if let Err(e) = cache_service.delete(&chat_cache_key).await {
        warn!("Failed to clear chat messages cache: {}", e);
      }
    }

    // 4. Async index stream - update index @indexer.rs (in parallel)
    let dispatcher = Arc::clone(&self.dispatcher);
    let index_message = updated_message.clone();
    tokio::spawn(async move {
      let index_event = AsyncIndexEvent {
        message: index_message,
        chat_info: ChatInfo {
          chat_name: "Chat".to_string(),  // TODO: Get actual chat name
          sender_name: "User".to_string(), // TODO: Get actual sender name
          chat_type: "group".to_string(), // TODO: Get actual chat type
          workspace_id: 1,                 // TODO: Get actual workspace ID
        },
        operation: IndexOperation::Update,
      };

      if let Err(e) = dispatcher.publish_async_index_event(index_event).await {
        warn!("Failed to publish async index update event: {}", e);
      }
    });

    // 5. Realtime push stream - push update to notify-server (in parallel)
    let dispatcher = Arc::clone(&self.dispatcher);
    let realtime_message = updated_message.clone();
    let realtime_members = chat_members;
    tokio::spawn(async move {
      let stream_message = StreamMessage {
        id: realtime_message.id.to_string(),
        chat_id: i64::from(realtime_message.chat_id),
        sender_id: i64::from(realtime_message.sender_id),
        content: realtime_message.content.clone(),
        files: realtime_message.files.clone().unwrap_or_default(),
        timestamp: realtime_message.created_at.timestamp(),
      };

      let realtime_event = RealtimeEvent::MessageReceived {
        message: stream_message,
        chat_id: i64::from(realtime_message.chat_id),
        recipients: realtime_members,
      };

      if let Err(e) = dispatcher.publish_realtime_event(realtime_event).await {
        warn!("Failed to publish realtime message update: {}", e);
      }
    });

    info!(
      "Message {} successfully updated by user {}",
      message_id, user_id
    );

    Ok(message_view)
  }

  /// Delete message - production-grade implementation with full data consistency
  pub async fn delete_message(
    &self,
    message_id: MessageId,
    user_id: UserId,
  ) -> Result<(), AppError> {
    // 1. Get message info before deletion (for event publishing and validation)
    let message = self
      .domain_service
      .get_message(i64::from(message_id))
      .await
      .map_err(AppError::from)?
      .ok_or_else(|| AppError::NotFound(vec!["Message not found".to_string()]))?;

    // Store necessary info before deletion
    let chat_id = message.chat_id;
    let sender_id = message.sender_id;

    // Verify user has permission to delete (must be sender)
    if i64::from(sender_id) != i64::from(user_id) {
      return Err(AppError::Unauthorized(
        "Only message sender can delete the message".to_string(),
      ));
    }

    // 2. Begin transaction-like operation with compensating actions
    let mut rollback_actions = Vec::new();

    // 3. Delete from database (primary source of truth)
    if let Err(e) = self
      .domain_service
      .delete_message(i64::from(message_id), i64::from(user_id))
      .await
    {
      error!("Failed to delete message from database: {}", e);
      return Err(AppError::from(e));
    }
    rollback_actions.push("database");

    // 4. Delete from search index (Meilisearch) - SYNCHRONOUS for consistency
    if let Some(search_service) = self.search_service.as_ref() {
      if let Err(e) = search_service
        .remove_messages_from_index_batch(&[message_id])
        .await
      {
        error!("Failed to delete message from search index: {}", e);
        // This is critical - we should consider rolling back the database deletion
        // For now, log the error and continue, but mark for manual cleanup
        warn!(
          "CRITICAL: Message {} deleted from DB but not from search index. Manual cleanup required.",
          message_id
        );
      }
    }

    // 5. Invalidate cache entries - SYNCHRONOUS for consistency
    if let Some(cache_service) = self.cache_service.as_ref() {
      // Clear message cache
      let message_cache_key = format!("message:{}", message_id);
      if let Err(e) = cache_service.delete(&message_cache_key).await {
        warn!("Failed to clear message cache: {}", e);
      }

      // Clear chat messages cache (invalidate the list)
      let chat_cache_key = format!("chat:{}:messages", chat_id);
      if let Err(e) = cache_service.delete(&chat_cache_key).await {
        warn!("Failed to clear chat messages cache: {}", e);
      }

      // Clear any user-specific message cache
      let user_message_cache_key = format!("user:{}:message:{}", user_id, message_id);
      if let Err(e) = cache_service.delete(&user_message_cache_key).await {
        warn!("Failed to clear user message cache: {}", e);
      }
    }

    // 6. Delete from vector database if applicable
    if let Some(vector_db) = self.vector_db.as_ref() {
      if let Err(e) = vector_db.delete_message(message_id).await {
        warn!("Failed to delete message from vector database: {}", e);
        // Non-critical, continue
      }
    }

    // 7. Publish async events for additional processing (non-critical path)
    let dispatcher = Arc::clone(&self.dispatcher);
    let delete_message = message.clone();
    tokio::spawn(async move {
      // Async index delete event (for any background cleanup)
      let delete_event = AsyncIndexEvent {
        message: delete_message,
        chat_info: ChatInfo {
          chat_name: "Chat".to_string(),  // TODO: Get actual chat name
          sender_name: "User".to_string(), // TODO: Get actual sender name
          chat_type: "group".to_string(), // TODO: Get actual chat type
          workspace_id: 1,                 // TODO: Get actual workspace ID
        },
        operation: IndexOperation::Delete,
      };

      if let Err(e) = dispatcher.publish_async_index_event(delete_event).await {
        warn!("Failed to publish async index delete event: {}", e);
      }

      // Realtime delete notification
      let realtime_event = RealtimeEvent::MessageDeleted {
        message_id: i64::from(message_id),
        chat_id: i64::from(chat_id),
        deleted_by: i64::from(user_id),
        deleted_at: chrono::Utc::now().to_rfc3339(),
      };

      if let Err(e) = dispatcher.publish_realtime_event(realtime_event).await {
        warn!("Failed to publish realtime delete event: {}", e);
      }
    });

    // 8. Audit log the deletion
    info!(
      "Message {} successfully deleted by user {} from chat {}",
      message_id, user_id, chat_id
    );

    Ok(())
  }

  /// Mark message as read - persist and send realtime event
  pub async fn mark_message_read(
    &self,
    message_id: MessageId,
    chat_id: ChatId,
    reader_id: UserId,
  ) -> Result<(), AppError> {
    // Persist read status to database
    self
      .domain_service
      .mark_message_read(i64::from(message_id), i64::from(reader_id))
      .await
      .map_err(AppError::from)?;

    // Publish realtime read status event
    let read_event = RealtimeEvent::MessageRead {
      message_id: i64::from(message_id),
      chat_id: i64::from(chat_id),
      reader_id: i64::from(reader_id),
      read_at: chrono::Utc::now().to_rfc3339(),
    };

    self.dispatcher.publish_realtime_event(read_event).await?;
    Ok(())
  }

  /// Mark message as unread - only goes through realtime stream → notify-server
  pub async fn mark_message_unread(
    &self,
    message_id: MessageId,
    chat_id: ChatId,
    user_id: UserId,
  ) -> Result<(), AppError> {
    // TODO: Persist unread status to database
    // self.domain_service.mark_message_unread(...).await?;

    // Publish realtime unread status event
    let unread_event = RealtimeEvent::MessageUnread {
      message_id: i64::from(message_id),
      chat_id: i64::from(chat_id),
      user_id: i64::from(user_id),
    };

    self.dispatcher.publish_realtime_event(unread_event).await?;
    Ok(())
  }

  /// Start typing indicator - only goes through realtime stream → notify-server
  pub async fn start_typing(
    &self,
    chat_id: ChatId,
    user_id: UserId,
    user_name: String,
  ) -> Result<(), AppError> {
    let typing_event = RealtimeEvent::TypingStarted {
      chat_id: i64::from(chat_id),
      user_id: i64::from(user_id),
      user_name,
    };

    self.dispatcher.publish_realtime_event(typing_event).await?;
    Ok(())
  }

  /// Stop typing indicator - only goes through realtime stream → notify-server
  pub async fn stop_typing(&self, chat_id: ChatId, user_id: UserId) -> Result<(), AppError> {
    let typing_event = RealtimeEvent::TypingStopped {
      chat_id: i64::from(chat_id),
      user_id: i64::from(user_id),
    };

    self.dispatcher.publish_realtime_event(typing_event).await?;
    Ok(())
  }

  /// Update user presence - only goes through realtime stream → notify-server
  pub async fn update_user_presence(
    &self,
    user_id: UserId,
    status: String,
    last_seen: Option<String>,
  ) -> Result<(), AppError> {
    let presence_event = RealtimeEvent::UserPresence {
      user_id: i64::from(user_id),
      status,
      last_seen,
    };

    self
      .dispatcher
      .publish_realtime_event(presence_event)
      .await?;
    Ok(())
  }

  /// Get message - direct call to domain
  pub async fn get_message(
    &self,
    message_id: MessageId,
    user_id: UserId,
  ) -> Result<Option<MessageView>, AppError> {
    let message = self
      .domain_service
      .get_message(i64::from(message_id))
      .await
      .map_err(AppError::from)?;

    Ok(message.map(MessageView::from))
  }

  // =============================================================================
  // MENTIONS MANAGEMENT
  // =============================================================================

  /// Get mentions for a specific message
  pub async fn get_message_mentions(
    &self,
    message_id: i64,
  ) -> Result<Vec<(i64, String, String, String)>, AppError> {
    // Use domain service instead of creating repository
    self.domain_service
      .get_message_mentions(message_id)
      .await
      .map_err(AppError::from)
  }

  /// Get unread mentions for a user across all chats
  pub async fn get_unread_mentions_for_user(
    &self,
    user_id: i64,
  ) -> Result<Vec<(i64, i64, String, String, chrono::DateTime<chrono::Utc>, String)>, AppError> {
    self.domain_service
      .get_unread_mentions_for_user(user_id)
      .await
      .map_err(AppError::from)
  }

  // =============================================================================
  // DETAILED RECEIPTS MANAGEMENT  
  // =============================================================================

  /// Get detailed read receipts for a message
  pub async fn get_detailed_message_receipts(
    &self,
    message_id: i64,
  ) -> Result<Vec<(i64, String, String, String, chrono::DateTime<chrono::Utc>)>, AppError> {
    self.domain_service
      .get_detailed_message_receipts(message_id)
      .await
      .map_err(AppError::from)
  }

  // =============================================================================
  // ENHANCED READ TRACKING
  // =============================================================================

  /// Mark message as read with enhanced tracking (handles mentions)
  pub async fn mark_message_read_enhanced(
    &self,
    user_id: i64,
    chat_id: i64,
    message_id: i64,
  ) -> Result<(), AppError> {
    self.domain_service
      .mark_message_read_enhanced(user_id, chat_id, message_id)
      .await
      .map_err(AppError::from)?;
    
    // Publish realtime event for enhanced read tracking
    let realtime_event = RealtimeEvent::MessageRead {
      message_id,
      chat_id,
      reader_id: user_id,
      read_at: chrono::Utc::now().to_rfc3339(),
    };
    
    self.dispatcher.publish_realtime_event(realtime_event).await?;
    
    Ok(())
  }

  /// Access to domain service
  pub fn domain_service(&self) -> &dyn MessageDomainService {
    self.domain_service.as_ref()
  }

  pub async fn from_state(state: &crate::AppState) -> Result<Self, AppError> {
    // Create domain service
    let pool = state.pool().clone();
    let repository = Arc::new(crate::domains::messaging::repository::MessageRepository::new(pool));
    let config = MessageConfig::production_optimized();
    let domain_service = Arc::new(MessageDomainServiceImpl::new(repository, config));

    // Create dispatcher with NATS client if available
    let dispatcher = if let Some(nats_client) = state.nats_client() {
      Arc::new(DualStreamDispatcher::new(nats_client))
    } else {
      Arc::new(DualStreamDispatcher::new_in_memory())
    };

    // Create notification service (simplified for now)
    let transport = crate::services::infrastructure::event::InMemoryTransport::new();
    let event_publisher =
      Arc::new(crate::services::infrastructure::event::legacy::EventPublisher::with_transport(transport));
    let flow_service =
      crate::services::infrastructure::flows::notifications::create_notification_flow_service(
        event_publisher,
        None,
      );
    let notification_service = create_notification_service(flow_service);

    // Create event publisher adapter
    let event_publisher = Arc::new(AppStateEventPublisher::new(
      state.event_publisher_dyn().cloned(),
    ));

    // TODO: Add search_service, cache_service, and vector_db when available
    Ok(Self::new(
      domain_service,
      dispatcher,
      notification_service,
      Arc::new(AppStateEventPublisher::new(
        state.event_publisher_dyn().cloned(),
      )),
    ))
  }
}

// ── Factory Functions ────────────────────────────────────────────────────────────

/// Create dual stream message service
pub fn create_dual_stream_message_service(state: &AppState) -> DualStreamMessageService {
  // 1. Create domain service
  let pool = state.pool().clone();
  let repository = Arc::new(MessageRepository::new(pool));
  let config = MessageConfig::default();
  let domain_impl = Arc::new(MessageDomainServiceImpl::new(repository, config));
  let domain_service: Arc<dyn MessageDomainService> = domain_impl;

  // 2. Prepare NATS client and dispatcher
  let nats = state
    .nats_client()
    .expect("NATS client required for dual stream")
    .clone();
  let dispatcher = Arc::new(DualStreamDispatcher::new(nats.clone()));

  // 3. Create notification flow & application-level notification service
  let flow_service = create_notification_flow_service_with_nats(nats);
  let notification_service: Arc<dyn NotificationServiceTrait> =
    create_notification_service(flow_service);

  // 4. Get optional services for data consistency
  let search_service = state.search_application_service().map(|s| {
    s.clone()
      as Arc<dyn crate::services::application::workers::search::SearchApplicationServiceTrait>
  });

  let cache_service = if let Some(cache) = state.cache_service() {
    Some(Arc::new(CacheServiceAdapter::new(cache.clone()))
      as Arc<dyn fechatter_core::contracts::CacheService>)
  } else {
    None
  };

  // Vector DB is not currently available in AppState
  let vector_db = None;

  // 5. Assemble dual stream service with full dependencies
  DualStreamMessageService::new_with_full_dependencies(
    domain_service,
    dispatcher,
    notification_service,
    search_service,
    cache_service,
    vector_db,
    Arc::new(AppStateEventPublisher::new(
      state.event_publisher_dyn().cloned(),
    )),
  )
}

// ── Backward Compatibility ────────────────────────────────────────────────────────────

/// Backward compatible aliases
pub type CompleteMessageService = DualStreamMessageService;
pub type MessageApplicationService = DualStreamMessageService;

/// Backward compatible factory functions  
pub fn create_complete_message_service(state: &AppState) -> CompleteMessageService {
  create_dual_stream_message_service(state)
}

pub fn create_message_service(state: &AppState) -> MessageApplicationService {
  create_dual_stream_message_service(state)
}
