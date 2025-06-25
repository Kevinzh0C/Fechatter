//! # Notification Flow
//!
//! **Responsibilities**: In-app notifications and user alerts
//! **Goal**: User-facing notification experience (WebSocket, Email, Push, In-app)
//! **Features**:
//! - Multi-channel notification delivery (WebSocket, Email, Push, Database)
//! - Intelligent routing and priority handling
//! - Notification filtering based on user preferences
//! - Retry and failure handling mechanisms
//! **Delivery Semantics**: best-effort with fallback
//! **Storage**: Multi-layer storage (Database, Cache, Real-time transport)

use crate::domains::notification::{
  Notification, NotificationChannel, NotificationContent, NotificationPriority, NotificationType,
};
use crate::error::AppError;
use crate::services::infrastructure::event::{LegacyEventPublisher, NatsTransport, Signable, EventTransport};
use crate::services::infrastructure::notification::{
  NotificationChannelFactory, NotificationDeliveryService,
};
use async_trait::async_trait;
use chrono::Utc;
use fechatter_core::{ChatId, UserId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};

// -- Re-export existing types for backward compatibility --
// These types are already imported above for internal use and will be re-exported through mod.rs

// -- Notification flow service based on unified event infrastructure --

/// Notification flow event - stream events related to notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationFlowEvent {
  /// Notification send request
  NotificationSendRequested {
    notification_id: i64,
    user_id: i64,
    notification_type: String, // "mention", "direct_message", etc.
    channels: Vec<String>,     // "websocket", "email", "push", "database"
    priority: String,          // "low", "normal", "high", "urgent"
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },

  /// Notification sent successfully
  NotificationSent {
    notification_id: i64,
    user_id: i64,
    channel: String, // "websocket", "email", "push", "database"
    sent_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },

  /// Notification send failed
  NotificationFailed {
    notification_id: i64,
    user_id: i64,
    channel: String,
    error: String,
    retry_count: u32,
    failed_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },

  /// Notification read
  NotificationRead {
    notification_id: i64,
    user_id: i64,
    read_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },
}

// Implement Signable trait to support event signatures
impl Signable for NotificationFlowEvent {
  fn set_signature(&mut self, sig: Option<String>) {
    match self {
      NotificationFlowEvent::NotificationSendRequested { sig: event_sig, .. }
      | NotificationFlowEvent::NotificationSent { sig: event_sig, .. }
      | NotificationFlowEvent::NotificationFailed { sig: event_sig, .. }
      | NotificationFlowEvent::NotificationRead { sig: event_sig, .. } => {
        *event_sig = sig;
      }
    }
  }

  fn get_signature(&self) -> &Option<String> {
    match self {
      NotificationFlowEvent::NotificationSendRequested { sig, .. }
      | NotificationFlowEvent::NotificationSent { sig, .. }
      | NotificationFlowEvent::NotificationFailed { sig, .. }
      | NotificationFlowEvent::NotificationRead { sig, .. } => sig,
    }
  }
}

/// Notification flow service - built on mature notification infrastructure
///
/// **After refactor**: Integrates existing domains/notification and infrastructure/notification
/// **Advantages**:
/// - Reuses robust multi-channel notification delivery mechanisms
/// - Leverages existing email templates, retry logic, etc.
/// - Publishes stream events based on LegacyEventPublisher<T: EventTransport>
/// - Supports intelligent routing and fallback strategies
pub struct NotificationFlowService<T: EventTransport> {
  event_publisher: Arc<LegacyEventPublisher<T>>,
  delivery_service: Arc<NotificationDeliveryService>,
  flow_subjects: NotificationFlowSubjects,
}

/// Notification flow subject configuration
#[derive(Debug, Clone)]
pub struct NotificationFlowSubjects {
  pub notification_events: String,
  pub user_preferences: String,
  pub delivery_status: String,
}

impl Default for NotificationFlowSubjects {
  fn default() -> Self {
    Self {
      notification_events: "fechatter.notification.events".to_string(),
      user_preferences: "fechatter.notification.preferences".to_string(),
      delivery_status: "fechatter.notification.delivery".to_string(),
    }
  }
}

impl<T: EventTransport> NotificationFlowService<T> {
  pub fn new(
    event_publisher: Arc<LegacyEventPublisher<T>>,
    delivery_service: Arc<NotificationDeliveryService>,
  ) -> Self {
    Self {
      event_publisher,
      delivery_service,
      flow_subjects: NotificationFlowSubjects::default(),
    }
  }

  pub fn with_subjects(mut self, subjects: NotificationFlowSubjects) -> Self {
    self.flow_subjects = subjects;
    self
  }

  /// Publish notification flow event - reuse unified event infrastructure
  pub async fn publish_notification_flow_event(
    &self,
    event: NotificationFlowEvent,
  ) -> Result<(), AppError> {
    let subject = self.get_subject_for_event(&event);

    self
      .event_publisher
      .publish_event(&subject, event, "notification_flow_event")
      .await
  }

  /// Determine subject based on event type
  fn get_subject_for_event(&self, event: &NotificationFlowEvent) -> String {
    match event {
      NotificationFlowEvent::NotificationSendRequested { .. }
      | NotificationFlowEvent::NotificationSent { .. }
      | NotificationFlowEvent::NotificationFailed { .. } => {
        self.flow_subjects.delivery_status.clone()
      }
      NotificationFlowEvent::NotificationRead { .. } => {
        self.flow_subjects.notification_events.clone()
      }
    }
  }

  /// Send mention notification - integrates existing infrastructure
  pub async fn send_mention_notification(
    &self,
    mentioned_user_ids: Vec<i64>,
    message_id: i64,
    chat_id: i64,
    sender_name: &str,
    content_preview: &str,
  ) -> Result<Vec<i64>, AppError> {
    let mut notification_ids = Vec::new();

    for user_id in mentioned_user_ids {
      // Create notification entity
      let notification = Notification {
        id: 0, // Generated by database
        user_id: UserId::new(user_id),
        notification_type: NotificationType::Mention,
        title: format!("You were mentioned by {}", sender_name),
        content: NotificationContent {
          message: content_preview.to_string(),
          chat_id: Some(ChatId::new(chat_id)),
          sender_id: None, // TODO: Add sender ID
          metadata: Some(format!("message_id:{}", message_id)),
        },
        priority: NotificationPriority::Normal,
        is_read: false,
        created_at: Utc::now(),
        read_at: None,
      };

      // Publish send request event
      let send_event = NotificationFlowEvent::NotificationSendRequested {
        notification_id: notification.id,
        user_id,
        notification_type: "mention".to_string(),
        channels: vec!["websocket".to_string(), "database".to_string()], // Default channels
        priority: "normal".to_string(),
        sig: None,
      };

      self.publish_notification_flow_event(send_event).await?;

      // Send notification to multiple channels
      let channels = vec![
        NotificationChannel::WebSocket,
        NotificationChannel::Database,
      ];
      let results = self
        .delivery_service
        .send_via_multiple_channels(&notification, &channels)
        .await;

      // Handle send results
      for (i, result) in results.into_iter().enumerate() {
        let channel_name = match channels.get(i) {
          Some(NotificationChannel::WebSocket) => "websocket",
          Some(NotificationChannel::Database) => "database",
          Some(NotificationChannel::Email) => "email",
          Some(NotificationChannel::Push) => "push",
          None => "unknown",
        };

        match result {
          Ok(()) => {
            let success_event = NotificationFlowEvent::NotificationSent {
              notification_id: notification.id,
              user_id,
              channel: channel_name.to_string(),
              sent_at: Utc::now().to_rfc3339(),
              sig: None,
            };
            let _ = self.publish_notification_flow_event(success_event).await;
          }
          Err(e) => {
            warn!(
              "Failed to send mention notification via {}: {}",
              channel_name, e
            );
            let failure_event = NotificationFlowEvent::NotificationFailed {
              notification_id: notification.id,
              user_id,
              channel: channel_name.to_string(),
              error: e,
              retry_count: 0,
              failed_at: Utc::now().to_rfc3339(),
              sig: None,
            };
            let _ = self.publish_notification_flow_event(failure_event).await;
          }
        }
      }

      notification_ids.push(notification.id);
    }

    Ok(notification_ids)
  }

  /// Send direct message notification
  pub async fn send_direct_message_notification(
    &self,
    user_id: i64,
    message_id: i64,
    chat_id: i64,
    sender_name: &str,
    content_preview: &str,
  ) -> Result<i64, AppError> {
    let notification = Notification {
      id: 0,
      user_id: UserId::new(user_id),
      notification_type: NotificationType::DirectMessage,
      title: format!("New message from {}", sender_name),
      content: NotificationContent {
        message: content_preview.to_string(),
        chat_id: Some(ChatId::new(chat_id)),
        sender_id: None,
        metadata: Some(format!("message_id:{}", message_id)),
      },
      priority: NotificationPriority::Normal,
      is_read: false,
      created_at: Utc::now(),
      read_at: None,
    };

    // Send to appropriate channels (WebSocket for real-time + Database for persistence)
    let channels = vec![
      NotificationChannel::WebSocket,
      NotificationChannel::Database,
    ];
    let _results = self
      .delivery_service
      .send_via_multiple_channels(&notification, &channels)
      .await;

    // Publish event
    let send_event = NotificationFlowEvent::NotificationSendRequested {
      notification_id: notification.id,
      user_id,
      notification_type: "direct_message".to_string(),
      channels: vec!["websocket".to_string(), "database".to_string()],
      priority: "normal".to_string(),
      sig: None,
    };

    self.publish_notification_flow_event(send_event).await?;

    Ok(notification.id)
  }

  /// Send chat invite notification
  pub async fn send_chat_invite_notification(
    &self,
    user_id: i64,
    chat_id: i64,
    chat_name: &str,
    inviter_name: &str,
  ) -> Result<i64, AppError> {
    let notification = Notification {
      id: 0,
      user_id: UserId::new(user_id),
      notification_type: NotificationType::ChatInvite,
      title: format!("Invitation to join '{}'", chat_name),
      content: NotificationContent {
        message: format!("{} invited you to join '{}'", inviter_name, chat_name),
        chat_id: Some(ChatId::new(chat_id)),
        sender_id: None,
        metadata: Some(format!("inviter:{}", inviter_name)),
      },
      priority: NotificationPriority::High,
      is_read: false,
      created_at: Utc::now(),
      read_at: None,
    };

    // Chat invite notifications have higher priority and are sent to more channels
    let channels = vec![
      NotificationChannel::WebSocket,
      NotificationChannel::Database,
      NotificationChannel::Email, // Important notifications are also sent via email
    ];

    let _results = self
      .delivery_service
      .send_via_multiple_channels(&notification, &channels)
      .await;

    let send_event = NotificationFlowEvent::NotificationSendRequested {
      notification_id: notification.id,
      user_id,
      notification_type: "chat_invite".to_string(),
      channels: vec![
        "websocket".to_string(),
        "database".to_string(),
        "email".to_string(),
      ],
      priority: "high".to_string(),
      sig: None,
    };

    self.publish_notification_flow_event(send_event).await?;

    Ok(notification.id)
  }

  /// Mark notification as read
  pub async fn mark_notification_as_read(
    &self,
    notification_id: i64,
    user_id: i64,
  ) -> Result<(), AppError> {
    // TODO: Integrate actual notification repository to update status

    let read_event = NotificationFlowEvent::NotificationRead {
      notification_id,
      user_id,
      read_at: Utc::now().to_rfc3339(),
      sig: None,
    };

    self.publish_notification_flow_event(read_event).await?;

    info!(
      "Marked notification {} as read for user {}",
      notification_id, user_id
    );
    Ok(())
  }
}

// -- Factory functions --

/// Create a notification flow service based on NATS
pub fn create_notification_flow_service_with_nats(
  nats_client: async_nats::Client,
) -> Arc<NotificationFlowService<crate::services::infrastructure::event::NatsTransport>> {
  use crate::services::infrastructure::event::{LegacyEventPublisher, NatsTransport};

  let transport = NatsTransport::new(nats_client);
  let event_publisher = Arc::new(LegacyEventPublisher::with_transport(transport));

  // Create notification delivery service (using default configuration)
  let channels = NotificationChannelFactory::create_all_channels();
  let delivery_service = Arc::new(NotificationDeliveryService::new(channels));

  Arc::new(NotificationFlowService::new(
    event_publisher,
    delivery_service,
  ))
}

/// Create a notification flow service supporting any transport layer
pub fn create_notification_flow_service<T: EventTransport>(
  event_publisher: Arc<LegacyEventPublisher<T>>,
  delivery_service: Option<Arc<NotificationDeliveryService>>,
) -> Arc<NotificationFlowService<T>> {
  let delivery_service = delivery_service.unwrap_or_else(|| {
    let channels = NotificationChannelFactory::create_all_channels();
    Arc::new(NotificationDeliveryService::new(channels))
  });

  Arc::new(NotificationFlowService::new(
    event_publisher,
    delivery_service,
  ))
}

// -- Backward-compatible simplified interface --

/// Simplified notification type for backward compatibility
#[derive(Debug, Clone)]
pub enum SimpleNotificationType {
  /// Mention notification
  Mention {
    message_id: i64,
    chat_id: i64,
    sender_name: String,
    content_preview: String,
  },
  /// Direct message notification
  DirectMessage {
    message_id: i64,
    chat_id: i64,
    sender_name: String,
    content_preview: String,
  },
  /// Chat invite notification
  ChatInvite {
    chat_id: i64,
    chat_name: String,
    inviter_name: String,
  },
}

/// Backward-compatible notification service trait
#[async_trait]
pub trait NotificationServiceTrait: Send + Sync {
  /// Send notification to user
  async fn send_notification(
    &self,
    user_id: i64,
    notification: SimpleNotificationType,
  ) -> Result<i64, AppError>;

  /// Send mention notification
  async fn send_mention_notification(
    &self,
    mentioned_user_ids: Vec<i64>,
    mentioned_by_name: &str,
    message_preview: &str,
    chat_id: i64,
    message_id: i64,
  ) -> Result<Vec<i64>, AppError>;

  /// Mark notification as read
  async fn mark_notification_as_read(
    &self,
    notification_id: i64,
    user_id: i64,
  ) -> Result<(), AppError>;
}

/// Backward-compatible notification service implementation
pub struct NotificationService<T: EventTransport> {
  flow_service: Arc<NotificationFlowService<T>>,
}

impl<T: EventTransport> NotificationService<T> {
  pub fn new(flow_service: Arc<NotificationFlowService<T>>) -> Self {
    Self { flow_service }
  }
}

#[async_trait]
impl<T: EventTransport> NotificationServiceTrait for NotificationService<T> {
  async fn send_notification(
    &self,
    user_id: i64,
    notification: SimpleNotificationType,
  ) -> Result<i64, AppError> {
    match notification {
      SimpleNotificationType::Mention {
        message_id,
        chat_id,
        sender_name,
        content_preview,
      } => {
        let results = self
          .flow_service
          .send_mention_notification(
            vec![user_id],
            message_id,
            chat_id,
            &sender_name,
            &content_preview,
          )
          .await?;
        Ok(results.first().copied().unwrap_or(0))
      }
      SimpleNotificationType::DirectMessage {
        message_id,
        chat_id,
        sender_name,
        content_preview,
      } => {
        self
          .flow_service
          .send_direct_message_notification(
            user_id,
            message_id,
            chat_id,
            &sender_name,
            &content_preview,
          )
          .await
      }
      SimpleNotificationType::ChatInvite {
        chat_id,
        chat_name,
        inviter_name,
      } => {
        self
          .flow_service
          .send_chat_invite_notification(user_id, chat_id, &chat_name, &inviter_name)
          .await
      }
    }
  }

  async fn send_mention_notification(
    &self,
    mentioned_user_ids: Vec<i64>,
    mentioned_by_name: &str,
    message_preview: &str,
    chat_id: i64,
    message_id: i64,
  ) -> Result<Vec<i64>, AppError> {
    self
      .flow_service
      .send_mention_notification(
        mentioned_user_ids,
        message_id,
        chat_id,
        mentioned_by_name,
        message_preview,
      )
      .await
  }

  async fn mark_notification_as_read(
    &self,
    notification_id: i64,
    user_id: i64,
  ) -> Result<(), AppError> {
    self
      .flow_service
      .mark_notification_as_read(notification_id, user_id)
      .await
  }
}

/// Create backward-compatible notification service
pub fn create_notification_service<T: EventTransport>(
  flow_service: Arc<NotificationFlowService<T>>,
) -> Arc<NotificationService<T>> {
  Arc::new(NotificationService::new(flow_service))
}

// TechniqueUsed: RustTechnique::CompositionOverInheritance | DesignPrinciple: DependencyInversion
// Architecture: UnifiedEventInfrastructure | Pattern: AdapterPattern
