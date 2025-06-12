//! # Domain Events Stream
//!
//! **Responsibility**: Reliable propagation and persistence of business domain events
//! **Target**: All downstream microservices (search-svc, analytics-svc, audit-svc, workflow-engine, etc.)
//! **Features**:
//! - Rich domain events with complete business context
//! - Message lifecycle events (create/update/delete with full context)
//! - Chat management events (chat management with member changes)
//! - User activity events (user activities with business meaning)
//! - System integration events (search indexing, analytics triggers)
//! - Audit trail events (audit trails for compliance)
//! **Delivery Semantics**: at-least-once (must be delivered, reliability guaranteed)
//! **Storage**: persistent NATS JetStream (persistent, replayable, ordered)

use crate::error::AppError;
use crate::services::infrastructure::event::{LegacyEventPublisher as EventPublisher, EventTransport, Signable};
use fechatter_core::models::message::MessageCreatedEvent;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

// ---- Domain Event Type Definitions ------------------------------------------

/// Application domain events - for business integrity and downstream services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
  /// Message domain event
  Message(MessageDomainEvent),
  /// Chat domain event
  Chat(ChatDomainEvent),
  /// User domain event
  User(UserDomainEvent),
  /// System domain event
  System(SystemDomainEvent),
}

// Implement Signable trait to support event signatures
impl Signable for DomainEvent {
  fn set_signature(&mut self, sig: Option<String>) {
    match self {
      DomainEvent::Message(event) => event.set_signature(sig),
      DomainEvent::Chat(event) => event.set_signature(sig),
      DomainEvent::User(event) => event.set_signature(sig),
      DomainEvent::System(event) => event.set_signature(sig),
    }
  }

  fn get_signature(&self) -> &Option<String> {
    match self {
      DomainEvent::Message(event) => event.get_signature(),
      DomainEvent::Chat(event) => event.get_signature(),
      DomainEvent::User(event) => event.get_signature(),
      DomainEvent::System(event) => event.get_signature(),
    }
  }
}

/// Message domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageDomainEvent {
  /// Message created - full business context
  MessageCreated {
    message_id: i64,
    chat_id: i64,
    sender_id: i64,
    content: String,
    files: Vec<String>,
    created_at: String,
    chat_members: Vec<i64>,
    mentions: Vec<i64>,
    sequence_number: Option<i64>,
    /// Business metadata
    workspace_id: i64,
    sender_name: String,
    chat_name: String,
    message_type: String, // "text", "file", "image", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },

  /// Message updated
  MessageUpdated {
    message_id: i64,
    chat_id: i64,
    editor_id: i64,
    old_content: String,
    new_content: String,
    updated_at: String,
    edit_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },

  /// Message deleted
  MessageDeleted {
    message_id: i64,
    chat_id: i64,
    deleted_by: i64,
    deleted_at: String,
    deletion_reason: Option<String>,
    soft_delete: bool, // soft delete vs hard delete
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },

  /// Message reaction (emoji reactions)
  MessageReactionAdded {
    message_id: i64,
    chat_id: i64,
    user_id: i64,
    reaction: String, // emoji
    added_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },

  /// Message reaction removed
  MessageReactionRemoved {
    message_id: i64,
    chat_id: i64,
    user_id: i64,
    reaction: String,
    removed_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },
}

impl Signable for MessageDomainEvent {
  fn set_signature(&mut self, sig: Option<String>) {
    match self {
      MessageDomainEvent::MessageCreated { sig: event_sig, .. }
      | MessageDomainEvent::MessageUpdated { sig: event_sig, .. }
      | MessageDomainEvent::MessageDeleted { sig: event_sig, .. }
      | MessageDomainEvent::MessageReactionAdded { sig: event_sig, .. }
      | MessageDomainEvent::MessageReactionRemoved { sig: event_sig, .. } => {
        *event_sig = sig;
      }
    }
  }

  fn get_signature(&self) -> &Option<String> {
    match self {
      MessageDomainEvent::MessageCreated { sig, .. }
      | MessageDomainEvent::MessageUpdated { sig, .. }
      | MessageDomainEvent::MessageDeleted { sig, .. }
      | MessageDomainEvent::MessageReactionAdded { sig, .. }
      | MessageDomainEvent::MessageReactionRemoved { sig, .. } => sig,
    }
  }
}

/// Chat domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatDomainEvent {
  /// Chat created
  ChatCreated {
    chat_id: i64,
    creator_id: i64,
    chat_name: String,
    chat_type: String, // "direct", "group", "channel"
    workspace_id: i64,
    initial_members: Vec<i64>,
    created_at: String,
    privacy_settings: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },

  /// Chat updated
  ChatUpdated {
    chat_id: i64,
    updated_by: i64,
    chat_name: String,
    updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },

  /// Chat deleted
  ChatDeleted {
    chat_id: i64,
    deleted_by: i64,
    deleted_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },

  /// Ownership transferred
  OwnershipTransferred {
    chat_id: i64,
    old_owner: i64,
    new_owner: i64,
    transferred_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },

  /// Chat member added
  ChatMemberAdded {
    chat_id: i64,
    added_by: i64,
    new_member_id: i64,
    added_at: String,
    invitation_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },

  /// Chat member removed
  ChatMemberRemoved {
    chat_id: i64,
    removed_by: i64,
    removed_member_id: i64,
    removed_at: String,
    removal_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },

  /// Chat settings updated
  ChatSettingsUpdated {
    chat_id: i64,
    updated_by: i64,
    updated_at: String,
    changes: serde_json::Value, // details of settings changes
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },
}

impl Signable for ChatDomainEvent {
  fn set_signature(&mut self, sig: Option<String>) {
    match self {
      ChatDomainEvent::ChatCreated { sig: event_sig, .. }
      | ChatDomainEvent::ChatUpdated { sig: event_sig, .. }
      | ChatDomainEvent::ChatDeleted { sig: event_sig, .. }
      | ChatDomainEvent::OwnershipTransferred { sig: event_sig, .. }
      | ChatDomainEvent::ChatMemberAdded { sig: event_sig, .. }
      | ChatDomainEvent::ChatMemberRemoved { sig: event_sig, .. }
      | ChatDomainEvent::ChatSettingsUpdated { sig: event_sig, .. } => {
        *event_sig = sig;
      }
    }
  }

  fn get_signature(&self) -> &Option<String> {
    match self {
      ChatDomainEvent::ChatCreated { sig, .. }
      | ChatDomainEvent::ChatUpdated { sig, .. }
      | ChatDomainEvent::ChatDeleted { sig, .. }
      | ChatDomainEvent::OwnershipTransferred { sig, .. }
      | ChatDomainEvent::ChatMemberAdded { sig, .. }
      | ChatDomainEvent::ChatMemberRemoved { sig, .. }
      | ChatDomainEvent::ChatSettingsUpdated { sig, .. } => sig,
    }
  }
}

/// User domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserDomainEvent {
  /// User registered
  UserRegistered {
    user_id: i64,
    email: String,
    workspace_id: i64,
    registered_at: String,
    registration_source: String, // "invite", "signup", "sso"
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },

  /// User status changed
  UserStatusChanged {
    user_id: i64,
    old_status: String,
    new_status: String,
    changed_at: String,
    status_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },

  /// User profile updated
  UserProfileUpdated {
    user_id: i64,
    updated_fields: Vec<String>,
    updated_at: String,
    updated_by: i64, // could be updated by admin
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },
}

impl Signable for UserDomainEvent {
  fn set_signature(&mut self, sig: Option<String>) {
    match self {
      UserDomainEvent::UserRegistered { sig: event_sig, .. }
      | UserDomainEvent::UserStatusChanged { sig: event_sig, .. }
      | UserDomainEvent::UserProfileUpdated { sig: event_sig, .. } => {
        *event_sig = sig;
      }
    }
  }

  fn get_signature(&self) -> &Option<String> {
    match self {
      UserDomainEvent::UserRegistered { sig, .. }
      | UserDomainEvent::UserStatusChanged { sig, .. }
      | UserDomainEvent::UserProfileUpdated { sig, .. } => sig,
    }
  }
}

/// System domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemDomainEvent {
  /// Search index update required
  SearchIndexUpdateRequired {
    entity_type: String, // "message", "chat", "user"
    entity_id: i64,
    operation: String, // "create", "update", "delete"
    timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },

  /// Cache invalidation required
  CacheInvalidationRequired {
    cache_keys: Vec<String>,
    invalidation_reason: String,
    timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },

  /// Analytics event generated
  AnalyticsEventGenerated {
    event_type: String,
    entity_data: serde_json::Value,
    user_id: Option<i64>,
    workspace_id: i64,
    timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sig: Option<String>,
  },
}

impl Signable for SystemDomainEvent {
  fn set_signature(&mut self, sig: Option<String>) {
    match self {
      SystemDomainEvent::SearchIndexUpdateRequired { sig: event_sig, .. }
      | SystemDomainEvent::CacheInvalidationRequired { sig: event_sig, .. }
      | SystemDomainEvent::AnalyticsEventGenerated { sig: event_sig, .. } => {
        *event_sig = sig;
      }
    }
  }

  fn get_signature(&self) -> &Option<String> {
    match self {
      SystemDomainEvent::SearchIndexUpdateRequired { sig, .. }
      | SystemDomainEvent::CacheInvalidationRequired { sig, .. }
      | SystemDomainEvent::AnalyticsEventGenerated { sig, .. } => sig,
    }
  }
}

// ---- Domain Event Service Based on Unified Event Infrastructure -------------

/// Domain event service - unified entry for publishing domain events
///
/// **After refactor**: Built on EventPublisher<T: EventTransport>
/// **Advantages**:
/// - Reuse transport layer abstraction (NATS, Kafka, etc.)
/// - Unified retry, signature, and error handling
/// - Pluggable transport layer support
pub struct DomainEventService<T: EventTransport> {
  event_publisher: Arc<EventPublisher<T>>,
  cache_service: Arc<crate::services::application::CacheStrategyService>,
  stream_subjects: DomainEventSubjects,
}

/// Domain event subject configuration
#[derive(Debug, Clone)]
pub struct DomainEventSubjects {
  pub message_events: String,
  pub chat_events: String,
  pub user_events: String,
  pub system_events: String,
}

impl Default for DomainEventSubjects {
  fn default() -> Self {
    Self {
      message_events: "fechatter.domain.message".to_string(),
      chat_events: "fechatter.domain.chat".to_string(),
      user_events: "fechatter.domain.user".to_string(),
      system_events: "fechatter.domain.system".to_string(),
    }
  }
}

impl<T: EventTransport> DomainEventService<T> {
  pub fn new(
    event_publisher: Arc<EventPublisher<T>>,
    cache_service: Arc<crate::services::application::CacheStrategyService>,
  ) -> Self {
    Self {
      event_publisher,
      cache_service,
      stream_subjects: DomainEventSubjects::default(),
    }
  }

  pub fn with_subjects(mut self, subjects: DomainEventSubjects) -> Self {
    self.stream_subjects = subjects;
    self
  }

  /// Publish a domain event - reuse unified event infrastructure
  pub async fn publish_domain_event(&self, event: DomainEvent) -> Result<(), AppError> {
    let subject = self.get_subject_for_event(&event);

    self
      .event_publisher
      .publish_event(&subject, event, "domain_event")
      .await
  }

  /// Publish multiple domain events in batch
  pub async fn publish_domain_events(&self, events: Vec<DomainEvent>) -> Result<(), AppError> {
    for event in events {
      self.publish_domain_event(event).await?;
    }
    Ok(())
  }

  /// Determine subject based on event type
  fn get_subject_for_event(&self, event: &DomainEvent) -> String {
    match event {
      DomainEvent::Message(_) => self.stream_subjects.message_events.clone(),
      DomainEvent::Chat(_) => self.stream_subjects.chat_events.clone(),
      DomainEvent::User(_) => self.stream_subjects.user_events.clone(),
      DomainEvent::System(_) => self.stream_subjects.system_events.clone(),
    }
  }

  /// Publish message created event - convert from MessageCreatedEvent
  pub async fn publish_message_created(
    &self,
    message_event: MessageCreatedEvent,
    workspace_id: i64,
    sender_name: String,
    chat_name: String,
  ) -> Result<(), AppError> {
    // Convert to full domain event
    let domain_event = DomainEvent::Message(MessageDomainEvent::MessageCreated {
      message_id: message_event.message_id,
      chat_id: message_event.chat_id,
      sender_id: message_event.sender_id,
      content: message_event.content,
      files: message_event.files,
      created_at: message_event.created_at,
      chat_members: message_event.chat_members,
      mentions: message_event.mentions,
      sequence_number: message_event.sequence_number,
      workspace_id,
      sender_name,
      chat_name,
      message_type: "text".to_string(), // TODO: infer type from content
      sig: None,
    });

    // Publish domain event
    self.publish_domain_event(domain_event.clone()).await?;

    // Trigger downstream processing
    self.trigger_downstream_processing(&domain_event).await?;

    Ok(())
  }

  /// Publish message updated event
  pub async fn publish_message_updated(
    &self,
    message_id: i64,
    chat_id: i64,
    editor_id: i64,
    old_content: String,
    new_content: String,
  ) -> Result<(), AppError> {
    let domain_event = DomainEvent::Message(MessageDomainEvent::MessageUpdated {
      message_id,
      chat_id,
      editor_id,
      old_content,
      new_content,
      updated_at: chrono::Utc::now().to_rfc3339(),
      edit_reason: None,
      sig: None,
    });

    self.publish_domain_event(domain_event.clone()).await?;
    self.trigger_downstream_processing(&domain_event).await?;

    Ok(())
  }

  /// Publish message deleted event
  pub async fn publish_message_deleted(
    &self,
    message_id: i64,
    chat_id: i64,
    deleted_by: i64,
    soft_delete: bool,
  ) -> Result<(), AppError> {
    let domain_event = DomainEvent::Message(MessageDomainEvent::MessageDeleted {
      message_id,
      chat_id,
      deleted_by,
      deleted_at: chrono::Utc::now().to_rfc3339(),
      deletion_reason: None,
      soft_delete,
      sig: None,
    });

    self.publish_domain_event(domain_event.clone()).await?;
    self.trigger_downstream_processing(&domain_event).await?;

    Ok(())
  }

  /// Trigger downstream processing - search indexing, cache invalidation, analytics, etc.
  async fn trigger_downstream_processing(&self, event: &DomainEvent) -> Result<(), AppError> {
    match event {
      DomainEvent::Message(msg_event) => {
        match msg_event {
          MessageDomainEvent::MessageCreated {
            chat_id,
            chat_members,
            message_id,
            ..
          } => {
            // Trigger search index update
            let search_event = DomainEvent::System(SystemDomainEvent::SearchIndexUpdateRequired {
              entity_type: "message".to_string(),
              entity_id: *message_id,
              operation: "create".to_string(),
              timestamp: chrono::Utc::now().to_rfc3339(),
              sig: None,
            });
            self.publish_domain_event(search_event).await?;

            // Trigger cache invalidation
            self
              .cache_service
              .invalidate_message_caches(*chat_id, chat_members)
              .await;

            // Trigger analytics event
            let analytics_event = DomainEvent::System(SystemDomainEvent::AnalyticsEventGenerated {
              event_type: "message_sent".to_string(),
              entity_data: serde_json::json!({
                "message_id": message_id,
                "chat_id": chat_id,
                "member_count": chat_members.len()
              }),
              user_id: Some(*message_id), // TODO: fix to sender_id
              workspace_id: 0,            // TODO: get from context
              timestamp: chrono::Utc::now().to_rfc3339(),
              sig: None,
            });
            self.publish_domain_event(analytics_event).await?;
          }

          MessageDomainEvent::MessageUpdated { message_id, .. } => {
            // Search index update
            let search_event = DomainEvent::System(SystemDomainEvent::SearchIndexUpdateRequired {
              entity_type: "message".to_string(),
              entity_id: *message_id,
              operation: "update".to_string(),
              timestamp: chrono::Utc::now().to_rfc3339(),
              sig: None,
            });
            self.publish_domain_event(search_event).await?;
          }

          MessageDomainEvent::MessageDeleted { message_id, .. } => {
            // Search index delete
            let search_event = DomainEvent::System(SystemDomainEvent::SearchIndexUpdateRequired {
              entity_type: "message".to_string(),
              entity_id: *message_id,
              operation: "delete".to_string(),
              timestamp: chrono::Utc::now().to_rfc3339(),
              sig: None,
            });
            self.publish_domain_event(search_event).await?;
          }

          _ => {}
        }
      }
      _ => {}
    }

    Ok(())
  }
}

// ---- Factory Functions ------------------------------------------------------

/// Create a NATS-based domain event service
pub fn create_domain_event_service_with_nats(
  nats_client: async_nats::Client,
  cache_service: Arc<crate::services::application::CacheStrategyService>,
) -> Arc<DomainEventService<crate::services::infrastructure::event::NatsTransport>> {
  use crate::services::infrastructure::event::{LegacyEventPublisher as EventPublisher, NatsTransport};

  let transport = NatsTransport::new(nats_client);
  let event_publisher = Arc::new(EventPublisher::with_transport(transport));

  Arc::new(DomainEventService::new(event_publisher, cache_service))
}

/// Create a domain event service supporting any transport layer
pub fn create_domain_event_service<T: EventTransport>(
  event_publisher: Arc<EventPublisher<T>>,
  cache_service: Arc<crate::services::application::CacheStrategyService>,
) -> Arc<DomainEventService<T>> {
  Arc::new(DomainEventService::new(event_publisher, cache_service))
}

// ---- Simplified Version (Backward Compatibility) ----------------------------

/// Simplified cache invalidation event
#[derive(Debug, Clone)]
pub enum CacheInvalidationEvent {
  /// Invalidate cache after message sent
  MessageSent { chat_id: i64, user_ids: Vec<i64> },
  /// Invalidate cache after chat updated
  ChatUpdated { chat_id: i64, user_ids: Vec<i64> },
  /// Invalidate cache after user info updated
  UserUpdated { user_id: i64 },
}

/// Simplified event publisher (backward compatible)
pub struct SimplifiedEventPublisher {
  cache_service: Arc<crate::services::application::CacheStrategyService>,
}

impl SimplifiedEventPublisher {
  pub fn new(cache_service: Arc<crate::services::application::CacheStrategyService>) -> Self {
    Self { cache_service }
  }

  /// Publish cache invalidation event
  pub async fn publish_cache_invalidation(
    &self,
    event: CacheInvalidationEvent,
  ) -> Result<(), AppError> {
    match event {
      CacheInvalidationEvent::MessageSent { chat_id, user_ids } => {
        info!("Invalidating cache for message sent in chat {}", chat_id);
        self
          .cache_service
          .invalidate_message_caches(chat_id, &user_ids)
          .await;
        Ok(())
      }
      CacheInvalidationEvent::ChatUpdated { chat_id, user_ids } => {
        info!("Invalidating cache for chat {} update", chat_id);
        self
          .cache_service
          .invalidate_chat_caches(chat_id, &user_ids)
          .await;
        Ok(())
      }
      CacheInvalidationEvent::UserUpdated { user_id } => {
        info!("Invalidating cache for user {} update", user_id);
        self.cache_service.invalidate_user_caches(user_id).await;
        Ok(())
      }
    }
  }
}

/// Create a simplified event publisher
pub fn create_simplified_publisher(
  cache_service: Arc<crate::services::application::CacheStrategyService>,
) -> Arc<SimplifiedEventPublisher> {
  Arc::new(SimplifiedEventPublisher::new(cache_service))
}

// TechniqueUsed: RustTechnique::EventSourcing | DesignPrinciple: DomainDrivenDesign
// Architecture: UnifiedEventInfrastructure | Pattern: StrategyPattern
