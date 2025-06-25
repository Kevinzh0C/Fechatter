// Event Publishing System - Organized Architecture
//
// This module provides a comprehensive event publishing system with multiple
// backend implementations and intelligent auto-degradation capabilities.

// Submodules organized by functionality
pub mod auto_degradation;  // Adaptive publisher with auto-degradation
pub mod high_performance;  // High-performance zero-cost abstraction publisher
pub mod legacy;           // Traditional reliable publisher (fallback)
pub mod shared;           // Shared components (transport, utilities)

// Re-export main components for easy access

// Auto-degradation system (recommended for production)
pub use auto_degradation::{
    AdaptivePublisher, AdaptivePublisherConfig, PublisherBackend,
    HealthMonitor, HealthStatus, DegradationReason,
    PublisherFactory, PublisherMetrics, PublisherHealthStatus,
    
    // Convenience functions
    publish_message_created, publish_message_updated, publish_message_deleted,
    publish_chat_member_joined, publish_chat_member_left, publish_message_events_batch,
    get_publisher_metrics, publisher_health_check,
    
    // Backend switching
    switch_publisher_backend, clear_manual_override, get_current_backend,
    
    // Initialization helpers
    initialize_default_publisher, initialize_production_publisher, 
    initialize_development_publisher,
    
    // Migration config (legacy compatibility)
    EventPublisherMigrationConfig, HighPerformanceConfig as MigrationHighPerformanceConfig,
    MigrationSafetyConfig, PublisherMigrationManager, MigrationConfigLoader, 
    MigrationStatus, initialize_with_migration_config, validate_migration_config,
};

// High-performance system (for direct usage)
pub use high_performance::{
    HighPerformancePublisher, PublisherConfig, EventData, EventPriority,
    FastMessageEvent, FastChatMemberEvent, PublishResult,
    CircuitBreakerConfig, RetryConfig, BackpressureConfig,
    PublisherMetrics as HighPerformanceMetrics, EventMetadata,
};

// Legacy system (for fallback and compatibility)
pub use legacy::{
    // Event publisher
    EventPublisher as LegacyEventPublisher, NatsEventPublisher,
    ChatInfo, ChatMemberJoined, ChatMemberLeft, DynEventPublisher,
    MsgLifecycle, RetryConfig as LegacyRetryConfig, SearchEvent, SearchOp, Signable,
    EnhancedMessageEvent, EnhancedChatMemberEvent, unified_subjects,
    
    // Analytics and enhanced publishers
    AnalyticsConfig, AnalyticsEventPublisher, AnalyticsTracking, NatsAnalyticsPublisher,
    CompleteMessageData, EnhancedEventPublisher, NotifyMessageEvent, NotifyChatMemberEvent,
    NotifyReadReceiptEvent, message_to_complete_data, create_enhanced_publisher_for_notify_server,
    
    // Event subscriber
    CacheEventSubscriber, CacheInvalidationConfig,
    
    // Core event types
    DuplicateMessageEvent, MessageEvent,
};

// Shared components
pub use shared::{
    // Transport abstractions
    EventTransport, InMemoryTransport, KafkaConfig, KafkaSecurityConfig, KafkaTransport,
    NatsTransport, TransportConfig, TransportFactory,
    
};

// Re-export error types
pub use crate::error::EventTransportError;

// Re-export core event types from fechatter_core
pub use fechatter_core::contracts::events::{
    MessageLifecycle, EventVersion, HmacSha256Verifier, SignatureVerifier,
    ChatMemberJoinedEvent, ChatMemberLeftEvent, subjects,
};

// Deprecated unified publisher (use auto_degradation instead)
#[deprecated(note = "Use auto_degradation::AdaptivePublisher instead for better reliability")]
pub use shared::unified_publisher::*;

// =====================================================================================
// SYSTEM RECOMMENDATIONS AND BEST PRACTICES
// =====================================================================================

/// Recommended initialization for production systems
/// 
/// This function sets up the adaptive publisher with production-optimized settings:
/// - High-performance backend preferred
/// - Automatic degradation enabled
/// - Aggressive recovery thresholds
/// - Comprehensive health monitoring
pub async fn initialize_recommended_production(
    nats_client: async_nats::Client,
) -> Result<(), crate::error::AppError> {
    initialize_production_publisher(nats_client).await
}

/// Recommended initialization for development systems
/// 
/// This function sets up the adaptive publisher with development-friendly settings:
/// - High-performance backend preferred but more lenient thresholds
/// - Automatic degradation enabled with relaxed triggers
/// - Faster recovery for development iteration
pub async fn initialize_recommended_development(
    nats_client: async_nats::Client,
) -> Result<(), crate::error::AppError> {
    initialize_development_publisher(nats_client).await
}

/// Initialize with environment variables
/// 
/// Reads configuration from environment variables:
/// - `FECHATTER_PREFERRED_BACKEND`: "high_performance" or "legacy"
/// - `FECHATTER_ENABLE_AUTO_DEGRADATION`: "true" or "false"
/// - `FECHATTER_HEALTH_CHECK_INTERVAL_SECONDS`: interval in seconds
/// - `FECHATTER_MAX_ERROR_RATE`: maximum error rate (0.0-1.0)
/// - `FECHATTER_MAX_LATENCY_MS`: maximum latency in milliseconds
pub async fn initialize_from_environment(
    nats_client: async_nats::Client,
) -> Result<(), crate::error::AppError> {
    PublisherFactory::initialize_with_env(nats_client).await
}

// =====================================================================================
// MIGRATION UTILITIES
// =====================================================================================

/// Migrate from legacy publisher setup to adaptive publisher
/// 
/// This function helps migrate existing code that uses legacy publishers
/// to the new adaptive system while maintaining compatibility.
pub async fn migrate_from_legacy(
    nats_client: async_nats::Client,
    preferred_backend: Option<PublisherBackend>,
) -> Result<(), crate::error::AppError> {
    let mut config = AdaptivePublisherConfig::default();
    
    if let Some(backend) = preferred_backend {
        config.preferred_backend = backend;
    }
    
    // Enable auto-degradation for safety during migration
    config.enable_auto_degradation = true;
    
    // Use more conservative thresholds during migration
    config.degradation_thresholds.max_error_rate = 0.01; // 1%
    config.recovery_delay = std::time::Duration::from_secs(120); // 2 minutes
    
    PublisherFactory::initialize_global(nats_client, Some(config)).await
}

// =====================================================================================
// HEALTH CHECK ENDPOINTS
// =====================================================================================

/// Get comprehensive system health status
/// 
/// Returns detailed health information for monitoring and alerting systems
pub async fn system_health_status() -> Result<SystemHealthStatus, crate::error::AppError> {
    let publisher_health = publisher_health_check().await?;
    let current_backend = get_current_backend().await?;
    let metrics = get_publisher_metrics().await?;
    
    // Clone publisher_health to avoid borrow after move
    let health_for_recommendations = publisher_health.clone();
    
    Ok(SystemHealthStatus {
        overall_healthy: publisher_health.is_healthy,
        current_backend: current_backend.clone(),
        backend_details: publisher_health,
        performance_metrics: metrics,
        recommendations: generate_recommendations(&health_for_recommendations).await,
    })
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SystemHealthStatus {
    pub overall_healthy: bool,
    pub current_backend: String,
    pub backend_details: PublisherHealthStatus,
    pub performance_metrics: Option<PublisherMetrics>,
    pub recommendations: Vec<String>,
}

async fn generate_recommendations(health: &PublisherHealthStatus) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    if health.success_rate < 0.95 {
        recommendations.push("Consider investigating the cause of publish failures".to_string());
    }
    
    if health.average_latency_ms > 500.0 {
        recommendations.push("High latency detected, consider scaling NATS resources".to_string());
    }
    
    if health.consecutive_failures > 5 {
        recommendations.push("Multiple consecutive failures detected, manual intervention may be needed".to_string());
    }
    
    if health.total_degradations > 10 {
        recommendations.push("Frequent degradations occurring, review system stability".to_string());
    }
    
    if health.manual_override {
        recommendations.push("Manual override is active, consider returning to automatic mode".to_string());
    }
    
    if recommendations.is_empty() {
        recommendations.push("System is operating normally".to_string());
    }
    
    recommendations
}

// =====================================================================================
// VERSION AND COMPATIBILITY INFO
// =====================================================================================

/// Get version information about the event publishing system
pub fn version_info() -> VersionInfo {
    VersionInfo {
        system_version: "2.0.0".to_string(),
        adaptive_publisher_version: "1.0.0".to_string(),
        high_performance_version: "1.0.0".to_string(),
        legacy_version: "1.5.0".to_string(),
        features: vec![
            "adaptive_backend_switching".to_string(),
            "auto_degradation".to_string(),
            "health_monitoring".to_string(),
            "zero_cost_abstractions".to_string(),
            "batch_processing".to_string(),
            "circuit_breaker".to_string(),
            "backpressure_control".to_string(),
        ],
        recommended_backend: "adaptive".to_string(),
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VersionInfo {
    pub system_version: String,
    pub adaptive_publisher_version: String,
    pub high_performance_version: String,
    pub legacy_version: String,
    pub features: Vec<String>,
    pub recommended_backend: String,
}