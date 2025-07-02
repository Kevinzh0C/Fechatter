// Auto-Degradation Event Publishing System
//
// This module provides intelligent switching between high-performance and legacy
// publishers based on real-time health monitoring and failure detection.

pub mod adaptive_publisher;
pub mod health_monitor;
pub mod migration_config;
pub mod publisher_integration;
pub mod usage_example;

// Re-export main components
pub use adaptive_publisher::{
    AdaptivePublisher, AdaptivePublisherConfig, DegradationReason, PublisherBackend, SwitchDecision,
};

pub use health_monitor::{
    DegradationTrigger, HealthMetrics, HealthMonitor, HealthStatus, MonitoringConfig,
};

pub use publisher_integration::{
    clear_manual_override, get_current_backend, get_publisher_metrics,
    initialize_default_publisher, initialize_development_publisher,
    initialize_production_publisher, publish_chat_member_joined, publish_chat_member_left,
    publish_message_created, publish_message_deleted, publish_message_events_batch,
    publish_message_updated, publisher_health_check, switch_publisher_backend, PublisherFactory,
    PublisherHealthStatus, PublisherMetrics,
};

pub use migration_config::{
    initialize_with_migration_config, validate_migration_config, EventPublisherMigrationConfig,
    HighPerformanceConfig, MigrationConfigLoader, MigrationSafetyConfig, MigrationStatus,
    PublisherMigrationManager,
};
