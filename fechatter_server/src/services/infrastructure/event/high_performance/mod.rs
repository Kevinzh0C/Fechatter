// High-Performance Event Publishing System
//
// This module contains the new high-performance, zero-cost abstraction
// event publishing implementation optimized for production workloads.

pub mod high_performance_publisher;

// Re-export main components
pub use high_performance_publisher::{
    HighPerformancePublisher, PublisherConfig, EventData, EventPriority,
    FastMessageEvent, FastChatMemberEvent, PublishResult,
    CircuitBreakerConfig, RetryConfig, BackpressureConfig,
    PublisherMetrics, EventMetadata,
};