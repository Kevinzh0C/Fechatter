//! # Message Application Service - Optimized Architecture
//!
//! **Responsibility**: Message use case orchestration and cross-domain service coordination
//! **Architecture**: Application Service Layer with Clean Dependency Injection
//! **Design Principles**:
//! - Single Responsibility + Dependency Inversion
//! - Clean Builder Pattern for Service Creation
//! - Optimized Resource Management without Double Arc Wrapping

use async_trait::async_trait;
use std::sync::Arc;

use crate::domains::messaging::{
  messaging_domain::{MessageConfig, MessageDomainService, MessageDomainServiceImpl},
  repository::MessageRepository,
};
use crate::services::application::{
  application_event_publisher::{
    ApplicationEvent, ApplicationEventPublisher, EventPublisher, MessageEvent,
  },
  message_stream::{MessageStreamService, StreamMessage},
};
use crate::{AppError, AppState};
use fechatter_core::models::{CreateMessage, ListMessages};
use fechatter_core::{ChatId, Message, MessageId, UserId};

/// Message Service Builder - Clean dependency injection without Arc proliferation
pub struct MessageServiceBuilder {
  repository: Option<Arc<MessageRepository>>,
  stream_service: Option<Arc<MessageStreamService>>,
  event_publisher: Option<Arc<ApplicationEventPublisher>>,
  config: Option<MessageConfig>,
}

impl MessageServiceBuilder {
  pub fn new() -> Self {
    Self {
      repository: None,
      stream_service: None,
      event_publisher: None,
      config: None,
    }
  }

  /// Inject repository with optimized pool sharing
  pub fn with_repository_from_state(mut self, state: &AppState) -> Self {
    // Optimized: Get Arc<PgPool> directly from AppState without double wrapping
    let pool = Arc::new(state.pool().clone());
    self.repository = Some(Arc::new(MessageRepository::new(pool)));
    self
  }

  /// Inject stream service
  pub fn with_stream_service(mut self, service: Arc<MessageStreamService>) -> Self {
    self.stream_service = Some(service);
    self
  }

  /// Inject event publisher  
  pub fn with_event_publisher(mut self, publisher: Arc<ApplicationEventPublisher>) -> Self {
    self.event_publisher = Some(publisher);
    self
  }

  /// Configure domain service
  pub fn with_config(mut self, config: MessageConfig) -> Self {
    self.config = Some(config);
    self
  }

  /// Build the complete service with validation
  pub fn build(self) -> Result<MessageApplicationService, AppError> {
    let repository = self
      .repository
      .ok_or_else(|| AppError::InvalidInput("MessageRepository is required".to_string()))?;

    let config = self.config.unwrap_or_default();
    let domain_service = MessageDomainServiceImpl::new(repository, config);

    let stream_service = self
      .stream_service
      .unwrap_or_else(|| Arc::new(MessageStreamService::new()));

    let event_publisher = self
      .event_publisher
      .unwrap_or_else(|| Arc::new(ApplicationEventPublisher::new()));

    Ok(MessageApplicationService::new(
      domain_service,
      stream_service,
      event_publisher,
    ))
  }
}

/// Service Factory with Resource Pool Management
pub struct MessageServiceFactory;

impl MessageServiceFactory {
  /// Create service with optimized resource sharing
  pub fn create_from_state(state: &AppState) -> Result<MessageApplicationService, AppError> {
    MessageServiceBuilder::new()
      .with_repository_from_state(state)
      .with_stream_service(Self::create_stream_service())
      .with_event_publisher(Self::create_event_publisher())
      .with_config(Self::create_config())
      .build()
  }

  /// Create singleton stream service
  fn create_stream_service() -> Arc<MessageStreamService> {
    // TODO: Implement singleton pattern or get from DI container
    Arc::new(MessageStreamService::new())
  }

  /// Create singleton event publisher
  fn create_event_publisher() -> Arc<ApplicationEventPublisher> {
    // TODO: Implement singleton pattern or get from DI container
    Arc::new(ApplicationEventPublisher::new())
  }

  /// Create optimized domain service configuration
  fn create_config() -> MessageConfig {
    MessageConfig::default()
  }
}

/// Message Application Service Trait - Application service interface for messages
#[async_trait]
pub trait MessageApplicationServiceTrait: Send + Sync {
  /// Send message use case
  async fn send_message(
    &self,
    sender_id: UserId,
    chat_id: ChatId,
    create_message: CreateMessage,
  ) -> Result<MessageView, AppError>;

  /// List messages use case
  async fn list_messages(
    &self,
    user_id: UserId,
    chat_id: ChatId,
    query: ListMessages,
  ) -> Result<Vec<MessageView>, AppError>;

  /// Edit message use case
  async fn edit_message(
    &self,
    message_id: MessageId,
    user_id: UserId,
    new_content: String,
  ) -> Result<MessageView, AppError>;

  /// Delete message use case
  async fn delete_message(&self, message_id: MessageId, user_id: UserId) -> Result<(), AppError>;

  /// Get single message
  async fn get_message(
    &self,
    message_id: MessageId,
    user_id: UserId,
  ) -> Result<Option<MessageView>, AppError>;
}

/// Message View - Application layer message view
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageView {
  pub id: i64,
  pub chat_id: i64,
  pub sender_id: i64,
  pub content: String,
  pub files: Option<Vec<String>>,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub idempotency_key: Option<uuid::Uuid>,
}

impl From<Message> for MessageView {
  fn from(message: Message) -> Self {
    Self {
      id: i64::from(message.id),
      chat_id: i64::from(message.chat_id),
      sender_id: i64::from(message.sender_id),
      content: message.content,
      files: message.files,
      created_at: message.created_at,
      idempotency_key: message.idempotency_key,
    }
  }
}

/// Message Application Service - Clean, optimized implementation
pub struct MessageApplicationService {
  /// Domain Service - Business logic layer
  domain_service: MessageDomainServiceImpl,
  /// Stream Service - Real-time message stream
  stream_service: Arc<MessageStreamService>,
  /// Event Publisher - Application event publisher
  event_publisher: Arc<ApplicationEventPublisher>,
}

impl MessageApplicationService {
  /// Create a new message application service
  pub fn new(
    domain_service: MessageDomainServiceImpl,
    stream_service: Arc<MessageStreamService>,
    event_publisher: Arc<ApplicationEventPublisher>,
  ) -> Self {
    Self {
      domain_service,
      stream_service,
      event_publisher,
    }
  }

  /// DEPRECATED: Use MessageServiceFactory::create_from_state instead
  #[deprecated(
    note = "Use MessageServiceFactory::create_from_state for better resource management"
  )]
  pub fn from_app_state(state: &AppState) -> Self {
    MessageServiceFactory::create_from_state(state)
      .expect("Failed to create MessageApplicationService")
  }

  /// Publish message event to real-time stream
  async fn publish_to_stream(&self, message: &Message) -> Result<(), AppError> {
    let stream_message = StreamMessage {
      id: message.id.to_string(),
      chat_id: i64::from(message.chat_id),
      sender_id: i64::from(message.sender_id),
      content: message.content.clone(),
      files: message.files.clone().unwrap_or_default(),
      timestamp: message.created_at.timestamp(),
    };

    self.stream_service.publish_message(stream_message).await
  }

  /// Publish message application event
  async fn publish_message_event(&self, event: MessageEvent) {
    let app_event = ApplicationEvent::Message(event);
    if let Err(e) = self.event_publisher.publish(app_event).await {
      tracing::error!("Failed to publish message event: {}", e);
    }
  }

  /// Invalidate message-related caches (TODO: Re-enable)
  async fn invalidate_message_caches(&self, _chat_id: ChatId, _affected_users: &[UserId]) {
    // TODO: Re-enable cache invalidation
    tracing::debug!("Cache invalidation temporarily disabled");
  }
}

#[async_trait]
impl MessageApplicationServiceTrait for MessageApplicationService {
  /// Send message use case - Complete application layer orchestration
  async fn send_message(
    &self,
    sender_id: UserId,
    chat_id: ChatId,
    create_message: CreateMessage,
  ) -> Result<MessageView, AppError> {
    tracing::info!(
      "MessageApplicationService::send_message - sender:{}, chat:{}",
      i64::from(sender_id),
      i64::from(chat_id)
    );

    // 1. Call Domain Service to handle business logic
    let message = self
      .domain_service
      .send_message(create_message, i64::from(chat_id), i64::from(sender_id))
      .await
      .map_err(AppError::from)?;

    // 2. Publish to real-time stream
    if let Err(e) = self.publish_to_stream(&message).await {
      tracing::error!("Failed to publish message to stream: {}", e);
    }

    // 3. Publish application event
    self
      .publish_message_event(MessageEvent::MessageSent {
        message_id: i64::from(message.id),
        chat_id: i64::from(message.chat_id),
        sender_id: i64::from(message.sender_id),
        content_preview: message.content.chars().take(100).collect(),
        mentioned_users: vec![], // TODO: Parse mentions
      })
      .await;

    // 4. Invalidate related caches
    self.invalidate_message_caches(chat_id, &[sender_id]).await;

    // 5. Return view
    Ok(MessageView::from(message))
  }

  /// List messages use case
  async fn list_messages(
    &self,
    user_id: UserId,
    chat_id: ChatId,
    query: ListMessages,
  ) -> Result<Vec<MessageView>, AppError> {
    tracing::debug!(
      "MessageApplicationService::list_messages - user:{}, chat:{}",
      i64::from(user_id),
      i64::from(chat_id)
    );

    let messages = self
      .domain_service
      .list_messages(query, i64::from(chat_id), i64::from(user_id))
      .await
      .map_err(AppError::from)?;

    let message_views: Vec<MessageView> = messages.into_iter().map(MessageView::from).collect();
    Ok(message_views)
  }

  /// Edit message use case - Complete application layer orchestration
  async fn edit_message(
    &self,
    message_id: MessageId,
    user_id: UserId,
    new_content: String,
  ) -> Result<MessageView, AppError> {
    tracing::info!(
      "MessageApplicationService::edit_message - message:{}, user:{}",
      i64::from(message_id),
      i64::from(user_id)
    );

    let updated_message = self
      .domain_service
      .edit_message(i64::from(message_id), new_content, i64::from(user_id))
      .await
      .map_err(AppError::from)?;

    self
      .publish_message_event(MessageEvent::MessageEdited {
        message_id: i64::from(message_id),
        chat_id: i64::from(updated_message.chat_id),
        editor_id: i64::from(user_id),
      })
      .await;

    self
      .invalidate_message_caches(updated_message.chat_id, &[user_id])
      .await;

    Ok(MessageView::from(updated_message))
  }

  /// Delete message use case - Complete application layer orchestration
  async fn delete_message(&self, message_id: MessageId, user_id: UserId) -> Result<(), AppError> {
    tracing::info!(
      "MessageApplicationService::delete_message - message:{}, user:{}",
      i64::from(message_id),
      i64::from(user_id)
    );

    let message_info = self
      .domain_service
      .get_message(i64::from(message_id))
      .await
      .map_err(AppError::from)?;

    self
      .domain_service
      .delete_message(i64::from(message_id), i64::from(user_id))
      .await
      .map_err(AppError::from)?;

    if let Some(message) = message_info {
      self
        .publish_message_event(MessageEvent::MessageDeleted {
          message_id: i64::from(message_id),
          chat_id: i64::from(message.chat_id),
          deleted_by: i64::from(user_id),
        })
        .await;

      self
        .invalidate_message_caches(message.chat_id, &[user_id])
        .await;
    }

    Ok(())
  }

  /// Get single message
  async fn get_message(
    &self,
    message_id: MessageId,
    user_id: UserId,
  ) -> Result<Option<MessageView>, AppError> {
    tracing::debug!(
      "MessageApplicationService::get_message - message:{}, user:{}",
      i64::from(message_id),
      i64::from(user_id)
    );

    let message = self
      .domain_service
      .get_message(i64::from(message_id))
      .await
      .map_err(AppError::from)?;

    Ok(message.map(MessageView::from))
  }
}

/// AppState Message Service Adapter - Clean resource management
pub struct AppStateMessageServiceAdapter {
  service: MessageApplicationService,
}

impl AppStateMessageServiceAdapter {
  /// Create adapter with optimized resource management
  pub fn new(state: &AppState) -> Result<Self, AppError> {
    let service = MessageServiceFactory::create_from_state(state)?;
    Ok(Self { service })
  }

  /// DEPRECATED: Use new() instead
  #[deprecated(note = "Use new() for better error handling")]
  pub fn from_state_unchecked(state: AppState) -> Self {
    let service = MessageServiceFactory::create_from_state(&state)
      .expect("Failed to create MessageApplicationService");
    Self { service }
  }
}

#[async_trait]
impl MessageApplicationServiceTrait for AppStateMessageServiceAdapter {
  async fn send_message(
    &self,
    sender_id: UserId,
    chat_id: ChatId,
    create_message: CreateMessage,
  ) -> Result<MessageView, AppError> {
    self
      .service
      .send_message(sender_id, chat_id, create_message)
      .await
  }

  async fn list_messages(
    &self,
    user_id: UserId,
    chat_id: ChatId,
    query: ListMessages,
  ) -> Result<Vec<MessageView>, AppError> {
    self.service.list_messages(user_id, chat_id, query).await
  }

  async fn edit_message(
    &self,
    message_id: MessageId,
    user_id: UserId,
    new_content: String,
  ) -> Result<MessageView, AppError> {
    self
      .service
      .edit_message(message_id, user_id, new_content)
      .await
  }

  async fn delete_message(&self, message_id: MessageId, user_id: UserId) -> Result<(), AppError> {
    self.service.delete_message(message_id, user_id).await
  }

  async fn get_message(
    &self,
    message_id: MessageId,
    user_id: UserId,
  ) -> Result<Option<MessageView>, AppError> {
    self.service.get_message(message_id, user_id).await
  }
}

/// Create message application service with optimized resource management
pub fn create_message_application_service(
  state: &AppState,
) -> Result<MessageApplicationService, AppError> {
  MessageServiceFactory::create_from_state(state)
}

/// Create message service adapter with proper error handling
pub fn create_message_service_adapter(
  state: &AppState,
) -> Result<AppStateMessageServiceAdapter, AppError> {
  AppStateMessageServiceAdapter::new(state)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_message_view_conversion() {
    // TODO: Add MessageView conversion tests
  }

  #[test]
  fn test_builder_pattern() {
    // TODO: Add Builder pattern tests
  }

  #[test]
  fn test_factory_pattern() {
    // TODO: Add Factory pattern tests
  }
}
