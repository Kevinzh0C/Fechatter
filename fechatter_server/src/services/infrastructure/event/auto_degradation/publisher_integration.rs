// Publisher Integration Layer with Auto-Degradation
//
// This module provides a unified interface for event publishing with automatic
// degradation and intelligent backend switching.

use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;
use tracing::{error, info, instrument, warn};

use crate::{
    error::AppError,
    services::infrastructure::event::auto_degradation::{
        adaptive_publisher::{AdaptiveHealthStatus, AdaptivePublisher, AdaptivePublisherConfig},
        health_monitor::{HealthMonitor, MonitoringConfig},
    },
};

use fechatter_core::{contracts::events::MessageLifecycle, ChatId, Message, MessageId, UserId};

// =====================================================================================
// GLOBAL PUBLISHER INSTANCE
// =====================================================================================

/// Global adaptive publisher instance
static GLOBAL_PUBLISHER: OnceCell<AdaptivePublisher> = OnceCell::const_new();

/// Publisher factory for creating and managing adaptive publisher instances
pub struct PublisherFactory;

impl PublisherFactory {
    /// Initialize the global adaptive publisher instance
    pub async fn initialize_global(
        nats_client: async_nats::Client,
        config: Option<AdaptivePublisherConfig>,
    ) -> Result<(), AppError> {
        let config = config.unwrap_or_else(|| {
            info!("Using default AdaptivePublisherConfig with high-performance preferred");
            AdaptivePublisherConfig::default()
        });

        info!(
            "Initializing global AdaptivePublisher with preferred backend: {:?}",
            config.preferred_backend
        );

        let publisher = AdaptivePublisher::new(nats_client, config).await?;

        GLOBAL_PUBLISHER.set(publisher).map_err(|_| {
            AppError::EventPublishError("Global publisher already initialized".to_string())
        })?;

        info!("Global AdaptivePublisher initialized successfully");
        Ok(())
    }

    /// Get the global publisher instance
    pub fn global() -> Result<&'static AdaptivePublisher, AppError> {
        GLOBAL_PUBLISHER.get().ok_or_else(|| {
            AppError::EventPublishError("Global publisher not initialized".to_string())
        })
    }

    /// Create a standalone adaptive publisher instance
    pub async fn create_standalone(
        nats_client: async_nats::Client,
        config: Option<AdaptivePublisherConfig>,
    ) -> Result<AdaptivePublisher, AppError> {
        let config = config.unwrap_or_default();
        AdaptivePublisher::new(nats_client, config).await
    }

    /// Initialize with environment-based configuration
    pub async fn initialize_with_env(nats_client: async_nats::Client) -> Result<(), AppError> {
        let config = AdaptivePublisherConfig::from_env();
        Self::initialize_global(nats_client, Some(config)).await
    }
}

// =====================================================================================
// PUBLISHER CONFIGURATION EXTENSIONS
// =====================================================================================

impl AdaptivePublisherConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Read environment variables for configuration
        if let Ok(backend_str) = std::env::var("FECHATTER_PREFERRED_BACKEND") {
            match backend_str.to_lowercase().as_str() {
                "legacy" => {
                    config.preferred_backend = super::adaptive_publisher::PublisherBackend::Legacy
                }
                "high_performance" | "hp" => {
                    config.preferred_backend =
                        super::adaptive_publisher::PublisherBackend::HighPerformance
                }
                _ => warn!("Unknown backend preference: {}, using default", backend_str),
            }
        }

        if let Ok(auto_degrade_str) = std::env::var("FECHATTER_ENABLE_AUTO_DEGRADATION") {
            config.enable_auto_degradation = auto_degrade_str.parse().unwrap_or(true);
        }

        if let Ok(health_interval_str) = std::env::var("FECHATTER_HEALTH_CHECK_INTERVAL_SECONDS") {
            if let Ok(seconds) = health_interval_str.parse::<u64>() {
                config.health_check_interval = Duration::from_secs(seconds);
            }
        }

        if let Ok(max_error_rate_str) = std::env::var("FECHATTER_MAX_ERROR_RATE") {
            if let Ok(rate) = max_error_rate_str.parse::<f64>() {
                config.degradation_thresholds.max_error_rate = rate.clamp(0.0, 1.0);
            }
        }

        if let Ok(max_latency_str) = std::env::var("FECHATTER_MAX_LATENCY_MS") {
            if let Ok(latency) = max_latency_str.parse::<f64>() {
                config.degradation_thresholds.max_latency_ms = latency;
            }
        }

        info!("AdaptivePublisherConfig loaded from environment: preferred_backend={:?}, auto_degradation={}", 
              config.preferred_backend, config.enable_auto_degradation);

        config
    }

    /// Create a production-optimized configuration
    pub fn production() -> Self {
        Self {
            preferred_backend: super::adaptive_publisher::PublisherBackend::HighPerformance,
            enable_auto_degradation: true,
            health_check_interval: Duration::from_secs(5),
            degradation_thresholds: super::adaptive_publisher::DegradationThresholds {
                max_error_rate: 0.02,  // 2% error rate
                max_latency_ms: 500.0, // 500ms
                max_queue_size: 50_000,
                consecutive_failure_threshold: 3,
                error_window_duration: Duration::from_secs(120),
            },
            recovery_thresholds: super::adaptive_publisher::RecoveryThresholds {
                min_success_rate: 0.99, // 99% success rate
                max_latency_ms: 100.0,  // 100ms
                consecutive_success_threshold: 20,
                recovery_window_duration: Duration::from_secs(300),
            },
            recovery_delay: Duration::from_secs(60),
            high_performance_config:
                crate::services::infrastructure::event::high_performance::PublisherConfig {
                    buffer_size: 100_000,
                    max_concurrent: 500,
                    batch_size: 100,
                    batch_timeout: Duration::from_millis(5),
                    ..Default::default()
                },
        }
    }

    /// Create a development configuration
    pub fn development() -> Self {
        Self {
            preferred_backend: super::adaptive_publisher::PublisherBackend::HighPerformance,
            enable_auto_degradation: true,
            health_check_interval: Duration::from_secs(10),
            degradation_thresholds: super::adaptive_publisher::DegradationThresholds {
                max_error_rate: 0.1,    // 10% error rate (more lenient)
                max_latency_ms: 2000.0, // 2 seconds
                max_queue_size: 10_000,
                consecutive_failure_threshold: 5,
                error_window_duration: Duration::from_secs(60),
            },
            recovery_thresholds: super::adaptive_publisher::RecoveryThresholds {
                min_success_rate: 0.95, // 95% success rate
                max_latency_ms: 200.0,  // 200ms
                consecutive_success_threshold: 5,
                recovery_window_duration: Duration::from_secs(120),
            },
            recovery_delay: Duration::from_secs(10),
            high_performance_config: Default::default(),
        }
    }
}

// =====================================================================================
// CONVENIENCE FUNCTIONS FOR COMMON OPERATIONS
// =====================================================================================

/// Publish a message created event using the global adaptive publisher
#[instrument(skip(message, chat_members))]
pub async fn publish_message_created(
    message: &Message,
    chat_members: &[UserId],
) -> Result<(), AppError> {
    let publisher = PublisherFactory::global()?;
    publisher
        .publish_message_event(MessageLifecycle::Created, message, chat_members)
        .await
}

/// Publish a message updated event using the global adaptive publisher
#[instrument(skip(message, chat_members))]
pub async fn publish_message_updated(
    message: &Message,
    chat_members: &[UserId],
) -> Result<(), AppError> {
    let publisher = PublisherFactory::global()?;
    publisher
        .publish_message_event(MessageLifecycle::Updated, message, chat_members)
        .await
}

/// Publish a message deleted event using the global adaptive publisher
#[instrument(skip(message, chat_members))]
pub async fn publish_message_deleted(
    message: &Message,
    chat_members: &[UserId],
) -> Result<(), AppError> {
    let publisher = PublisherFactory::global()?;
    publisher
        .publish_message_event(MessageLifecycle::Deleted, message, chat_members)
        .await
}

/// Publish chat member joined event using the global adaptive publisher
#[instrument(skip(chat_id, user_id))]
pub async fn publish_chat_member_joined(
    chat_id: &ChatId,
    user_id: &UserId,
) -> Result<(), AppError> {
    let publisher = PublisherFactory::global()?;
    publisher.publish_chat_member_joined(chat_id, user_id).await
}

/// Publish chat member left event using the global adaptive publisher
#[instrument(skip(chat_id, user_id))]
pub async fn publish_chat_member_left(chat_id: &ChatId, user_id: &UserId) -> Result<(), AppError> {
    let publisher = PublisherFactory::global()?;
    publisher.publish_chat_member_left(chat_id, user_id).await
}

/// Batch publish multiple message events using the global adaptive publisher
#[instrument(skip(events))]
pub async fn publish_message_events_batch(
    events: Vec<(MessageLifecycle, Message, Vec<UserId>)>,
) -> Result<(), AppError> {
    let publisher = PublisherFactory::global()?;

    // Process each event individually through the adaptive publisher
    // This allows each event to be monitored and contribute to health metrics
    for (kind, message, chat_members) in events {
        publisher
            .publish_message_event(kind, &message, &chat_members)
            .await?;
    }

    Ok(())
}

// =====================================================================================
// HEALTH AND MONITORING
// =====================================================================================

/// Get current publisher health status and metrics
pub async fn publisher_health_check() -> Result<PublisherHealthStatus, AppError> {
    let publisher = PublisherFactory::global()?;
    let health_status = publisher.health_status().await;

    Ok(PublisherHealthStatus {
        backend_type: health_status.current_backend.as_str().to_string(),
        is_healthy: health_status.is_healthy,
        success_rate: health_status.success_rate,
        average_latency_ms: health_status.avg_latency_ms,
        queue_size: 0,      // Not directly available in adaptive publisher
        total_published: 0, // Would need to be tracked separately
        total_failed: 0,
        current_backend: health_status.current_backend.as_str().to_string(),
        consecutive_failures: health_status.consecutive_failures,
        consecutive_successes: health_status.consecutive_successes,
        total_degradations: health_status.total_degradations,
        manual_override: health_status.manual_override,
    })
}

/// Get simplified publisher metrics
pub async fn get_publisher_metrics() -> Result<Option<PublisherMetrics>, AppError> {
    let publisher = PublisherFactory::global()?;
    let health_status = publisher.health_status().await;

    Ok(Some(PublisherMetrics {
        total_published: 0, // Would need to be tracked in the adaptive publisher
        total_failed: 0,
        total_retries: 0,
        average_latency_us: (health_status.avg_latency_ms * 1000.0) as u64,
        queue_size: 0,
        batch_count: 0,
        success_rate: health_status.success_rate,
        current_backend: health_status.current_backend.as_str().to_string(),
    }))
}

/// Switch publisher backend manually
pub async fn switch_publisher_backend(
    backend: super::adaptive_publisher::PublisherBackend,
    reason: String,
) -> Result<(), AppError> {
    let publisher = PublisherFactory::global()?;
    publisher.switch_backend(backend, reason).await
}

/// Clear manual override and return to automatic backend selection
pub async fn clear_manual_override() -> Result<(), AppError> {
    let publisher = PublisherFactory::global()?;
    publisher.clear_manual_override().await;
    Ok(())
}

/// Get current backend
pub async fn get_current_backend() -> Result<String, AppError> {
    let publisher = PublisherFactory::global()?;
    let backend = publisher.current_backend().await;
    Ok(backend.as_str().to_string())
}

// =====================================================================================
// TYPES FOR COMPATIBILITY
// =====================================================================================

#[derive(Debug, Clone, Serialize)]
pub struct PublisherHealthStatus {
    pub backend_type: String,
    pub is_healthy: bool,
    pub success_rate: f64,
    pub average_latency_ms: f64,
    pub queue_size: usize,
    pub total_published: u64,
    pub total_failed: u64,
    pub current_backend: String,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
    pub total_degradations: u64,
    pub manual_override: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublisherMetrics {
    pub total_published: u64,
    pub total_failed: u64,
    pub total_retries: u64,
    pub average_latency_us: u64,
    pub queue_size: usize,
    pub batch_count: u64,
    pub success_rate: f64,
    pub current_backend: String,
}

impl PublisherMetrics {
    pub fn success_rate(&self) -> f64 {
        self.success_rate
    }

    pub fn average_latency_ms(&self) -> f64 {
        self.average_latency_us as f64 / 1000.0
    }
}

// =====================================================================================
// INITIALIZATION HELPER
// =====================================================================================

/// Initialize publisher with default high-performance configuration
pub async fn initialize_default_publisher(nats_client: async_nats::Client) -> Result<(), AppError> {
    let config = AdaptivePublisherConfig::default();
    PublisherFactory::initialize_global(nats_client, Some(config)).await
}

/// Initialize publisher for production environment
pub async fn initialize_production_publisher(
    nats_client: async_nats::Client,
) -> Result<(), AppError> {
    let config = AdaptivePublisherConfig::production();
    PublisherFactory::initialize_global(nats_client, Some(config)).await
}

/// Initialize publisher for development environment
pub async fn initialize_development_publisher(
    nats_client: async_nats::Client,
) -> Result<(), AppError> {
    let config = AdaptivePublisherConfig::development();
    PublisherFactory::initialize_global(nats_client, Some(config)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_config_from_env() {
        // Test default values when no env vars are set
        let config = AdaptivePublisherConfig::from_env();
        assert_eq!(
            config.preferred_backend,
            super::super::adaptive_publisher::PublisherBackend::HighPerformance
        );
        assert!(config.enable_auto_degradation);
    }

    #[test]
    fn test_production_config() {
        let config = AdaptivePublisherConfig::production();
        assert_eq!(
            config.preferred_backend,
            super::super::adaptive_publisher::PublisherBackend::HighPerformance
        );
        assert_eq!(config.degradation_thresholds.max_error_rate, 0.02);
        assert_eq!(config.recovery_thresholds.min_success_rate, 0.99);
    }

    #[test]
    fn test_development_config() {
        let config = AdaptivePublisherConfig::development();
        assert_eq!(config.degradation_thresholds.max_error_rate, 0.1);
        assert_eq!(config.recovery_thresholds.min_success_rate, 0.95);
    }
}
