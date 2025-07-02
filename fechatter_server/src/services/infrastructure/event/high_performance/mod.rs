// High-Performance Event Publishing System
//
// This module contains the new high-performance, zero-cost abstraction
// event publishing implementation optimized for production workloads.

pub mod high_performance_publisher;

// Re-export main components
pub use high_performance_publisher::{
    BackpressureConfig, CircuitBreakerConfig, EventData, EventMetadata, EventPriority,
    FastChatMemberEvent, FastMessageEvent, HighPerformancePublisher, PublishResult,
    PublisherConfig, PublisherMetrics, RetryConfig,
};
