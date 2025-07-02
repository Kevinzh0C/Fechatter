//! # Application Flows
//!
//! ## Module Responsibilities
//! - **realtime_stream** - Real-time stream: Low-latency message delivery and state sync for user experience
//! - **domain_events** - Domain event stream: Reliable event propagation and persistence for downstream services
//! - **notifications** - Notification stream: In-app notifications and user alerts
//!
//! ## Three-Stream Architecture After Refactoring (Based on Unified Event Infrastructure)
//!
//! **Core Improvements**:
//! - All streams are built on `EventPublisher<T: EventTransport>`
//! - Reuse transport layer abstraction (NATS, Kafka, custom transports)
//! - Unified retry, signature, and error handling mechanisms
//! - Support for pluggable transport layers
//!
//! **Three Streams with Distinct Responsibilities**:
//! - **Real-time stream** focuses on user experience (WebSocket, presence, typing indicators)
//! - **Domain event stream** focuses on business integrity (search indexing, analytics, audit)
//! - **Notification stream** focuses on user alerts (mentions, direct messages, invites)

/// Real-time stream module - User experience and low latency
pub mod realtime_stream;

/// Domain event stream module - Business integrity and reliability
pub mod domain_events;

/// Notification stream module - User alerts and notifications
pub mod notifications;

/// Events module - Legacy compatibility
pub mod events;

/// Typing indicator module - Real-time typing status
pub mod typing_indicator;

// ── Unified Exports (Based on EventTransport Abstraction) ────────────────────────────────────

// Real-time stream exports
pub use realtime_stream::{
    create_instant_messaging_service,
    create_messaging_service,
    create_realtime_stream_service,
    create_realtime_stream_service_with_nats,
    InstantMessagingService,
    MessagingService,
    // Backward compatibility
    NatsRealtimeStreamPublisher,
    RealtimeStreamEvent,
    RealtimeStreamPublisher,
    RealtimeStreamService,
    RealtimeStreamSubjects,
};

// Domain event stream exports
pub use domain_events::{
    create_domain_event_service,
    create_domain_event_service_with_nats,
    create_simplified_publisher,
    // Backward compatibility
    CacheInvalidationEvent,
    ChatDomainEvent,
    DomainEvent,
    DomainEventService,
    DomainEventSubjects,
    MessageDomainEvent,
    SimplifiedEventPublisher,
    SystemDomainEvent,
    UserDomainEvent,
};

// Notification stream exports
pub use notifications::{
    create_notification_flow_service,
    create_notification_flow_service_with_nats,
    create_notification_service,
    // New notification stream based on event infrastructure
    NotificationFlowEvent,
    NotificationFlowService,
    NotificationFlowSubjects,
    // Backward compatibility
    NotificationService,
    NotificationServiceTrait,
    SimpleNotificationType,
};

// Re-export notification types directly from domain (public access)
pub use crate::domains::notification::{NotificationPriority, NotificationType};

// Typing indicator exports
pub use typing_indicator::{create_typing_indicator_service, TypingIndicatorService, TypingUser};

// Events module exports (legacy compatibility)
pub use events::{
    create_domain_event_service as events_create_domain_event_service,
    create_simplified_publisher as events_create_simplified_publisher,
    CacheInvalidationEvent as EventsCacheInvalidationEvent,
    ChatDomainEvent as EventsChatDomainEvent, DomainEvent as EventsDomainEvent,
    DomainEventPublisher, DomainEventService as EventsDomainEventService,
    MessageDomainEvent as EventsMessageDomainEvent, NatsDomainEventPublisher,
    SimplifiedEventPublisher as EventsSimplifiedEventPublisher,
    SystemDomainEvent as EventsSystemDomainEvent, UserDomainEvent as EventsUserDomainEvent,
};

// ── Refactoring Notes ────────────────────────────────────────────────────────────

/// # Refactoring Summary: Flows Built on Events
///
/// ## Problem Analysis
///
/// **Original Issues**:
/// - Duplicate event publishing implementations in `flows` and `event` modules
/// - Duplicate transport layer abstractions (flows using NATS directly, event having EventTransport)
/// - Inconsistent architectural design
///
/// ## Solution
///
/// **Unified Architecture**:
/// ```rust
/// // All stream services are based on unified event infrastructure
/// pub struct DomainEventService<T: EventTransport> {
///   event_publisher: Arc<EventPublisher<T>>,
///   // ...
/// }
///
/// pub struct RealtimeStreamService<T: EventTransport> {
///   event_publisher: Arc<EventPublisher<T>>,
///   // ...
/// }
/// ```
///
/// **Benefits**:
/// 1. **Unified Transport Layer**: All streams use `EventTransport` abstraction
/// 2. **Pluggable Design**: Support for NATS, Kafka, custom transport layers
/// 3. **Feature Reuse**: Unified retry, signature, and error handling mechanisms
/// 4. **Backward Compatibility**: Existing APIs remain unchanged
/// 5. **Architectural Consistency**: Clear layering of domain concepts + infrastructure
///
/// ## Usage Example
///
/// ```rust
/// // Using NATS transport
/// let domain_service = create_domain_event_service_with_nats(nats_client, cache_service);
/// let realtime_service = create_realtime_stream_service_with_nats(domain_service, nats_client);
///
/// // Using any transport
/// let event_publisher = Arc::new(EventPublisher::with_transport(custom_transport));
/// let domain_service = create_domain_event_service(event_publisher, cache_service);
/// ```
///
/// **Design Principles**:
/// - DRY (Don't Repeat Yourself): Eliminate duplicate implementations
/// - SRP (Single Responsibility Principle): Each stream focuses on specific responsibilities
/// - OCP (Open/Closed Principle): Support for extending new transport layers
/// - DIP (Dependency Inversion Principle): Depend on abstractions not implementations
pub mod architecture_notes {
    //! Architecture refactoring technical notes
}

// Architecture: UnifiedEventInfrastructure | Pattern: StrategyPattern
// TechniqueUsed: RustTechnique::GenericAbstraction | DesignPrinciple: DomainDrivenDesign
