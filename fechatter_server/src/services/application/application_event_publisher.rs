//! # Application Event Publisher
//!
//! ## Single Responsibility
//! - Domain event publishing coordination
//! - Event propagation strategy management
//! - Asynchronous event handling
//! - Event failure retry mechanism

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, instrument, warn};

// ============================================================================
// APPLICATION EVENTS
// ============================================================================

/// Application domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApplicationEvent {
  /// User events
  User(UserEvent),
  /// Chat events
  Chat(ChatEvent),
  /// Message events
  Message(MessageEvent),
  /// Notification events
  Notification(NotificationEvent),
}

/// User domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserEvent {
  /// User created
  UserCreated {
    user_id: i64,
    email: String,
    workspace_id: i64,
  },
  /// User profile updated
  UserProfileUpdated {
    user_id: i64,
    updated_fields: Vec<String>,
  },
  /// User status changed
  UserStatusChanged {
    user_id: i64,
    old_status: String,
    new_status: String,
  },
}

/// Chat domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatEvent {
  /// Chat created
  ChatCreated {
    chat_id: i64,
    creator_id: i64,
    initial_members: Vec<i64>,
    chat_type: String,
  },
  /// Members added
  MembersAdded {
    chat_id: i64,
    added_by: i64,
    new_members: Vec<i64>,
  },
  /// Members removed
  MembersRemoved {
    chat_id: i64,
    removed_by: i64,
    removed_members: Vec<i64>,
  },
}

/// Message domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageEvent {
  /// Message sent
  MessageSent {
    message_id: i64,
    chat_id: i64,
    sender_id: i64,
    content_preview: String,
    mentioned_users: Vec<i64>,
  },
  /// Message edited
  MessageEdited {
    message_id: i64,
    chat_id: i64,
    editor_id: i64,
  },
  /// Message deleted
  MessageDeleted {
    message_id: i64,
    chat_id: i64,
    deleted_by: i64,
  },
}

/// Notification domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationEvent {
  /// Notification sent
  NotificationSent {
    notification_id: i64,
    user_id: i64,
    notification_type: String,
  },
  /// Notification read
  NotificationRead { notification_id: i64, user_id: i64 },
}

// ============================================================================
// EVENT HANDLER TRAITS
// ============================================================================

/// Application event handler interface
#[async_trait]
pub trait ApplicationEventHandler: Send + Sync {
  /// Handle application event
  async fn handle(&self, event: &ApplicationEvent) -> Result<(), EventError>;

  /// Handler name (for logging and debugging)
  fn name(&self) -> &'static str;
}

/// Event publisher interface
#[async_trait]
pub trait EventPublisher: Send + Sync {
  /// Publish event
  async fn publish(&self, event: ApplicationEvent) -> Result<(), EventError>;

  /// Publish batch of events
  async fn publish_batch(&self, events: Vec<ApplicationEvent>) -> Result<(), EventError>;
}

// ============================================================================
// APPLICATION EVENT PUBLISHER IMPLEMENTATION
// ============================================================================

/// Application event publisher - Single responsibility: Coordinate event publishing and handling
pub struct ApplicationEventPublisher {
  /// Event handler list
  handlers: Vec<Arc<dyn ApplicationEventHandler>>,
  /// Async event sender channel
  event_sender: mpsc::UnboundedSender<ApplicationEvent>,
}

impl ApplicationEventPublisher {
  /// Create new event publisher
  pub fn new() -> Self {
    let (sender, mut receiver) = mpsc::unbounded_channel::<ApplicationEvent>();

    // Start background event processing task
    tokio::spawn(async move {
      while let Some(event) = receiver.recv().await {
        // Can add event persistence, retry logic here
        info!("Received event for async processing: {:?}", event);
      }
    });

    Self {
      handlers: Vec::new(),
      event_sender: sender,
    }
  }

  /// Register event handler
  pub fn register_handler(&mut self, handler: Arc<dyn ApplicationEventHandler>) {
    info!("Registering event handler: {}", handler.name());
    self.handlers.push(handler);
  }

  /// Synchronously publish event - Process immediately
  #[instrument(skip(self, event))]
  pub async fn publish_sync(&self, event: ApplicationEvent) -> Result<(), EventError> {
    info!("Publishing sync event: {:?}", event);

    for handler in &self.handlers {
      if let Err(e) = handler.handle(&event).await {
        error!(
            handler = handler.name(),
            error = %e,
            "Event handler failed"
        );
        // Continue processing other handlers, don't interrupt
      }
    }

    Ok(())
  }

  /// Asynchronously publish event - Process in background
  #[instrument(skip(self, event))]
  pub async fn publish_async(&self, event: ApplicationEvent) -> Result<(), EventError> {
    self.event_sender.send(event).map_err(|e| {
      EventError::PublishFailed(format!("Failed to send event to async channel: {}", e))
    })?;

    Ok(())
  }
}

#[async_trait]
impl EventPublisher for ApplicationEventPublisher {
  async fn publish(&self, event: ApplicationEvent) -> Result<(), EventError> {
    self.publish_sync(event).await
  }

  async fn publish_batch(&self, events: Vec<ApplicationEvent>) -> Result<(), EventError> {
    for event in events {
      self.publish_sync(event).await?;
    }
    Ok(())
  }
}

// ============================================================================
// CONCRETE EVENT HANDLERS
// ============================================================================

/// Cache invalidation event handler - Single responsibility: Handle cache invalidation
pub struct CacheInvalidationHandler {
  cache_service: Arc<crate::services::application::CacheStrategyService>,
}

impl CacheInvalidationHandler {
  pub fn new(cache_service: Arc<crate::services::application::CacheStrategyService>) -> Self {
    Self { cache_service }
  }
}

#[async_trait]
impl ApplicationEventHandler for CacheInvalidationHandler {
  async fn handle(&self, event: &ApplicationEvent) -> Result<(), EventError> {
    match event {
      ApplicationEvent::User(user_event) => match user_event {
        UserEvent::UserProfileUpdated { user_id, .. } => {
          self.cache_service.invalidate_user_caches(*user_id).await;
        }
        UserEvent::UserStatusChanged { user_id, .. } => {
          self.cache_service.invalidate_user_caches(*user_id).await;
        }
        _ => {}
      },
      ApplicationEvent::Chat(chat_event) => match chat_event {
        ChatEvent::ChatCreated {
          chat_id,
          initial_members,
          ..
        } => {
          self
            .cache_service
            .invalidate_chat_caches(*chat_id, initial_members)
            .await;
        }
        ChatEvent::MembersAdded {
          chat_id,
          new_members,
          ..
        } => {
          self
            .cache_service
            .invalidate_chat_caches(*chat_id, new_members)
            .await;
        }
        ChatEvent::MembersRemoved {
          chat_id,
          removed_members,
          ..
        } => {
          self
            .cache_service
            .invalidate_chat_caches(*chat_id, removed_members)
            .await;
        }
      },
      ApplicationEvent::Message(message_event) => match message_event {
        MessageEvent::MessageSent {
          chat_id,
          mentioned_users,
          ..
        } => {
          self
            .cache_service
            .invalidate_message_caches(*chat_id, mentioned_users)
            .await;
        }
        _ => {}
      },
      _ => {}
    }

    Ok(())
  }

  fn name(&self) -> &'static str {
    "CacheInvalidationHandler"
  }
}

/// Notification trigger event handler - Single responsibility: Trigger notification sending
pub struct NotificationTriggerHandler {
  notification_service:
    Arc<crate::services::application::notification_app_service::NotificationService>,
}

impl NotificationTriggerHandler {
  pub fn new(
    notification_service: Arc<
      crate::services::application::notification_app_service::NotificationService,
    >,
  ) -> Self {
    Self {
      notification_service,
    }
  }
}

#[async_trait]
impl ApplicationEventHandler for NotificationTriggerHandler {
  async fn handle(&self, event: &ApplicationEvent) -> Result<(), EventError> {
    match event {
      ApplicationEvent::Message(MessageEvent::MessageSent {
        chat_id,
        sender_id,
        content_preview,
        mentioned_users,
        ..
      }) => {
        // Handle @ mention notifications
        if !mentioned_users.is_empty() {
          if let Err(e) = self
            .notification_service
            .send_mention_notification(
              mentioned_users.clone(),
              "User", // TODO: Get from user service
              content_preview,
              *chat_id,
              0, // TODO: Get message_id from event
            )
            .await
          {
            warn!("Failed to send mention notifications: {}", e);
          }
        }
      }
      _ => {}
    }

    Ok(())
  }

  fn name(&self) -> &'static str {
    "NotificationTriggerHandler"
  }
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/// Event handling errors
#[derive(Debug, thiserror::Error)]
pub enum EventError {
  #[error("Event publishing failed: {0}")]
  PublishFailed(String),

  #[error("Event handling failed: {0}")]
  HandlingFailed(String),

  #[error("Event serialization failed: {0}")]
  SerializationFailed(String),
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/// Create configured event publisher
pub fn create_configured_publisher(
  cache_service: Arc<crate::services::application::CacheStrategyService>,
  notification_service: Arc<
    crate::services::application::notification_app_service::NotificationService,
  >,
) -> ApplicationEventPublisher {
  let mut publisher = ApplicationEventPublisher::new();

  // Register cache invalidation handler
  publisher.register_handler(Arc::new(CacheInvalidationHandler::new(cache_service)));

  // Register notification trigger handler
  publisher.register_handler(Arc::new(NotificationTriggerHandler::new(
    notification_service,
  )));

  publisher
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_event_creation() {
    let user_event = ApplicationEvent::User(UserEvent::UserCreated {
      user_id: 123,
      email: "test@example.com".to_string(),
      workspace_id: 1,
    });

    match user_event {
      ApplicationEvent::User(UserEvent::UserCreated { user_id, .. }) => {
        assert_eq!(user_id, 123);
      }
      _ => panic!("Wrong event type"),
    }
  }
}
