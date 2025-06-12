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
    CompleteMessageData, EnhancedEventPublisher, NotifyMessageEvent, NotifyChatMemberEvent,
    NotifyReadReceiptEvent, message_to_complete_data, create_enhanced_publisher_for_notify_server,
};

pub use event_publisher::{
    ChatInfo, ChatMemberJoined, ChatMemberLeft, DynEventPublisher, EventPublisher, MsgLifecycle,
    NatsEventPublisher, RetryConfig, SearchEvent, SearchOp, Signable,
    // Enhanced unified architecture exports
    EnhancedMessageEvent, EnhancedChatMemberEvent, unified_subjects,
};

pub use event_subscriber::{
    CacheEventSubscriber,
    CacheInvalidationConfig,
};

// Re-export core event types
pub use fechatter_core::contracts::events::{DuplicateMessageEvent, MessageEvent};