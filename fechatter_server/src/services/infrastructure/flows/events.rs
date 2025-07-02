//! # Domain Events Stream
//!
//! **Responsibility**: Reliable domain event propagation and persistence
//! **Target**: All downstream services (search-svc, analytics-svc, audit-svc, etc.)
//! **Features**:
//! - Complete domain events for message creation/edit/delete
//! - Chat management events (creation, member changes, etc.)
//! - User activity events (login, status change, etc.)
//! - Search index update triggers
//! - Data analytics event collection
//! **Delivery Semantics**: at-least-once (must be delivered and persisted)

use crate::error::AppError;
use fechatter_core::models::message::MessageCreatedEvent;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

// ── Domain Event Type Definitions ──────────────────────────────────────

/// Application Domain Event - for business integrity and downstream services
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
    },

    /// Message deleted
    MessageDeleted {
        message_id: i64,
        chat_id: i64,
        deleted_by: i64,
        deleted_at: String,
        deletion_reason: Option<String>,
        soft_delete: bool, // soft delete vs hard delete
    },

    /// Message reaction (emoji reactions)
    MessageReactionAdded {
        message_id: i64,
        chat_id: i64,
        user_id: i64,
        reaction: String, // emoji
        added_at: String,
    },

    /// Message reaction removed
    MessageReactionRemoved {
        message_id: i64,
        chat_id: i64,
        user_id: i64,
        reaction: String,
        removed_at: String,
    },
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
    },

    /// Chat member added
    ChatMemberAdded {
        chat_id: i64,
        added_by: i64,
        new_member_id: i64,
        added_at: String,
        invitation_context: Option<String>,
    },

    /// Chat member removed
    ChatMemberRemoved {
        chat_id: i64,
        removed_by: i64,
        removed_member_id: i64,
        removed_at: String,
        removal_reason: Option<String>,
    },

    /// Chat settings updated
    ChatSettingsUpdated {
        chat_id: i64,
        updated_by: i64,
        updated_at: String,
        changes: serde_json::Value, // details of settings changes
    },
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
    },

    /// User status changed
    UserStatusChanged {
        user_id: i64,
        old_status: String,
        new_status: String,
        changed_at: String,
        status_message: Option<String>,
    },

    /// User profile updated
    UserProfileUpdated {
        user_id: i64,
        updated_fields: Vec<String>,
        updated_at: String,
        updated_by: i64, // could be updated by admin
    },
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
    },

    /// Cache invalidation required
    CacheInvalidationRequired {
        cache_keys: Vec<String>,
        invalidation_reason: String,
        timestamp: String,
    },

    /// Analytics event generated
    AnalyticsEventGenerated {
        event_type: String,
        entity_data: serde_json::Value,
        user_id: Option<i64>,
        workspace_id: i64,
        timestamp: String,
    },
}

// ── Domain Event Publisher ─────────────────────────────────────────────

/// Domain event publisher - focused on reliability and integrity
#[async_trait::async_trait]
pub trait DomainEventPublisher: Send + Sync {
    /// Publish a domain event to the persistent stream
    async fn publish_domain_event(&self, event: DomainEvent) -> Result<(), AppError>;

    /// Batch publish domain events
    async fn publish_domain_events(&self, events: Vec<DomainEvent>) -> Result<(), AppError>;
}

/// NATS JetStream domain event publisher
pub struct NatsDomainEventPublisher {
    jetstream: Option<async_nats::jetstream::Context>,
    stream_name: String,
}

impl NatsDomainEventPublisher {
    pub fn new(jetstream: Option<async_nats::jetstream::Context>) -> Self {
        Self {
            jetstream,
            stream_name: "FECHATTER_DOMAIN_EVENTS".to_string(),
        }
    }

    pub fn with_stream_name(mut self, stream_name: String) -> Self {
        self.stream_name = stream_name;
        self
    }
}

#[async_trait::async_trait]
impl DomainEventPublisher for NatsDomainEventPublisher {
    async fn publish_domain_event(&self, event: DomainEvent) -> Result<(), AppError> {
        if let Some(js) = &self.jetstream {
            let subject = match &event {
                DomainEvent::Message(_) => format!("{}.message", self.stream_name),
                DomainEvent::Chat(_) => format!("{}.chat", self.stream_name),
                DomainEvent::User(_) => format!("{}.user", self.stream_name),
                DomainEvent::System(_) => format!("{}.system", self.stream_name),
            };

            let payload = serde_json::to_vec(&event).map_err(|e| {
                AppError::Internal(format!("Failed to serialize domain event: {}", e))
            })?;

            // JetStream persistent publish - ensure reliability
            let ack = js
                .publish(subject.clone(), payload.into())
                .await
                .map_err(|e| {
                    AppError::Internal(format!("Failed to publish domain event: {}", e))
                })?;

            // Wait for ack
            ack.await.map_err(|e| {
                AppError::Internal(format!("Failed to get domain event ack: {}", e))
            })?;

            info!("Published domain event to JetStream: {}", subject);
        }
        Ok(())
    }

    async fn publish_domain_events(&self, events: Vec<DomainEvent>) -> Result<(), AppError> {
        for event in events {
            self.publish_domain_event(event).await?;
        }
        Ok(())
    }
}

// ── Domain Event Service ───────────────────────────────────────────────

/// Domain event service - unified entry for domain event publishing
pub struct DomainEventService {
    domain_publisher: Arc<dyn DomainEventPublisher>,
    cache_service: Arc<crate::services::application::CacheStrategyService>,
}

impl DomainEventService {
    pub fn new(
        domain_publisher: Arc<dyn DomainEventPublisher>,
        cache_service: Arc<crate::services::application::CacheStrategyService>,
    ) -> Self {
        Self {
            domain_publisher,
            cache_service,
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
        });

        // Publish domain event
        self.domain_publisher
            .publish_domain_event(domain_event.clone())
            .await?;

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
        });

        self.domain_publisher
            .publish_domain_event(domain_event.clone())
            .await?;
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
        });

        self.domain_publisher
            .publish_domain_event(domain_event.clone())
            .await?;
        self.trigger_downstream_processing(&domain_event).await?;

        Ok(())
    }

    /// Trigger downstream processing - search index, cache invalidation, analytics, etc.
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
                        let search_event =
                            DomainEvent::System(SystemDomainEvent::SearchIndexUpdateRequired {
                                entity_type: "message".to_string(),
                                entity_id: *message_id,
                                operation: "create".to_string(),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            });
                        self.domain_publisher
                            .publish_domain_event(search_event)
                            .await?;

                        // Trigger cache invalidation
                        self.cache_service
                            .invalidate_message_caches(*chat_id, chat_members)
                            .await;

                        // Trigger analytics event
                        let analytics_event =
                            DomainEvent::System(SystemDomainEvent::AnalyticsEventGenerated {
                                event_type: "message_sent".to_string(),
                                entity_data: serde_json::json!({
                                  "message_id": message_id,
                                  "chat_id": chat_id,
                                  "member_count": chat_members.len()
                                }),
                                user_id: Some(*message_id), // TODO: fix to sender_id
                                workspace_id: 0,            // TODO: get from context
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            });
                        self.domain_publisher
                            .publish_domain_event(analytics_event)
                            .await?;
                    }

                    MessageDomainEvent::MessageUpdated { message_id, .. } => {
                        // Search index update
                        let search_event =
                            DomainEvent::System(SystemDomainEvent::SearchIndexUpdateRequired {
                                entity_type: "message".to_string(),
                                entity_id: *message_id,
                                operation: "update".to_string(),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            });
                        self.domain_publisher
                            .publish_domain_event(search_event)
                            .await?;
                    }

                    MessageDomainEvent::MessageDeleted { message_id, .. } => {
                        // Search index delete
                        let search_event =
                            DomainEvent::System(SystemDomainEvent::SearchIndexUpdateRequired {
                                entity_type: "message".to_string(),
                                entity_id: *message_id,
                                operation: "delete".to_string(),
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            });
                        self.domain_publisher
                            .publish_domain_event(search_event)
                            .await?;
                    }

                    _ => {}
                }
            }
            _ => {}
        }

        Ok(())
    }
}

// ── Factory Function ──────────────────────────────────────────────────

/// Create domain event service
pub fn create_domain_event_service(
    jetstream: Option<async_nats::jetstream::Context>,
    cache_service: Arc<crate::services::application::CacheStrategyService>,
) -> Arc<DomainEventService> {
    let domain_publisher = Arc::new(NatsDomainEventPublisher::new(jetstream));

    Arc::new(DomainEventService::new(domain_publisher, cache_service))
}

// ── Simplified Version (Backward Compatibility) ───────────────────────

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

/// Simplified event publisher (backward compatibility)
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
                self.cache_service
                    .invalidate_message_caches(chat_id, &user_ids)
                    .await;
                Ok(())
            }
            CacheInvalidationEvent::ChatUpdated { chat_id, user_ids } => {
                info!("Invalidating cache for chat {} update", chat_id);
                self.cache_service
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

/// Create simplified event publisher
pub fn create_simplified_publisher(
    cache_service: Arc<crate::services::application::CacheStrategyService>,
) -> Arc<SimplifiedEventPublisher> {
    Arc::new(SimplifiedEventPublisher::new(cache_service))
}

// TechniqueUsed: RustTechnique::EventSourcing | DesignPrinciple: DomainDrivenDesign
