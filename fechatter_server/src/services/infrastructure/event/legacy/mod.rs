// Legacy Event Publishing System
//
// This module contains the traditional event publishing implementation
// that serves as a reliable fallback when the high-performance system
// encounters issues or during gradual migration.

pub mod analytics_publisher;
pub mod enhanced_publisher;
pub mod event_publisher;
pub mod event_subscriber;

// Re-export for backward compatibility
pub use analytics_publisher::{
    AnalyticsConfig, AnalyticsEventPublisher, AnalyticsTracking, NatsAnalyticsPublisher,
};

pub use enhanced_publisher::{
    create_enhanced_publisher_for_notify_server, message_to_complete_data, CompleteMessageData,
    EnhancedEventPublisher, NotifyChatMemberEvent, NotifyMessageEvent, NotifyReadReceiptEvent,
};

pub use event_publisher::{
    unified_subjects,
    ChatInfo,
    ChatMemberJoined,
    ChatMemberLeft,
    DynEventPublisher,
    EnhancedChatMemberEvent,
    // Enhanced unified architecture exports
    EnhancedMessageEvent,
    EventPublisher,
    MsgLifecycle,
    NatsEventPublisher,
    RetryConfig,
    SearchEvent,
    SearchOp,
    Signable,
};

pub use event_subscriber::{CacheEventSubscriber, CacheInvalidationConfig};

// Re-export core event types
pub use fechatter_core::contracts::events::{DuplicateMessageEvent, MessageEvent};
