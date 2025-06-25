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
    AdaptivePublisher, AdaptivePublisherConfig, PublisherBackend, 
    DegradationReason, SwitchDecision,
};

pub use health_monitor::{
    HealthMonitor, HealthStatus, HealthMetrics,
    MonitoringConfig, DegradationTrigger,
};

pub use publisher_integration::{
    PublisherFactory, PublisherMetrics, PublisherHealthStatus,
    publish_message_created, publish_message_updated, publish_message_deleted,
    publish_chat_member_joined, publish_chat_member_left, publish_message_events_batch,
    get_publisher_metrics, publisher_health_check,
    switch_publisher_backend, clear_manual_override, get_current_backend,
    initialize_default_publisher, initialize_production_publisher, initialize_development_publisher,
};

pub use migration_config::{
    EventPublisherMigrationConfig, HighPerformanceConfig, MigrationSafetyConfig,
    PublisherMigrationManager, MigrationConfigLoader, MigrationStatus,
    initialize_with_migration_config, validate_migration_config,
};