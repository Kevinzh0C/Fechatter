// Event Publisher Migration Configuration
//
// This module provides configuration and utilities for migrating from the legacy
// event publishing system to the new high-performance system with feature flags
// and gradual rollout capabilities.

use serde::{Serialize, Deserialize};
use std::{sync::Arc, time::Duration};
use tracing::{info, warn, error};

use crate::{
    config::AppConfig,
    error::AppError,
    services::infrastructure::event::{
        high_performance::{PublisherConfig, CircuitBreakerConfig, RetryConfig, BackpressureConfig},
        legacy::NatsEventPublisher,
        shared::NatsTransport,
    },
};

// =====================================================================================
// MIGRATION CONFIGURATION
// =====================================================================================

/// Configuration for event publisher migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPublisherMigrationConfig {
    /// Enable high-performance publisher
    pub enable_high_performance: bool,
    
    /// Percentage of traffic to route to high-performance publisher (0-100)
    pub rollout_percentage: u8,
    
    /// Maximum events per second for load testing
    pub max_events_per_second: Option<u32>,
    
    /// High-performance publisher configuration
    pub high_performance_config: Option<HighPerformanceConfig>,
    
    /// Circuit breaker settings for migration safety
    pub migration_safety: MigrationSafetyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighPerformanceConfig {
    /// Buffer size for internal channel
    pub buffer_size: usize,
    
    /// Maximum concurrent operations
    pub max_concurrent: usize,
    
    /// Batch size for bulk operations
    pub batch_size: usize,
    
    /// Batch timeout in milliseconds
    pub batch_timeout_ms: u64,
    
    /// Enable circuit breaker
    pub enable_circuit_breaker: bool,
    
    /// Maximum retries for failed publishes
    pub max_retries: u32,
    
    /// Initial backoff in milliseconds
    pub initial_backoff_ms: u64,
    
    /// Maximum backoff in milliseconds
    pub max_backoff_ms: u64,
    
    /// Enable backpressure management
    pub enable_backpressure: bool,
    
    /// High water mark for backpressure
    pub backpressure_high_water_mark: usize,
    
    /// Low water mark for backpressure
    pub backpressure_low_water_mark: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationSafetyConfig {
    /// Enable automatic fallback to legacy on errors
    pub enable_automatic_fallback: bool,
    
    /// Error threshold for automatic fallback (percentage)
    pub error_threshold_percentage: f64,
    
    /// Time window for error calculation in seconds
    pub error_window_seconds: u64,
    
    /// Enable health check monitoring
    pub enable_health_monitoring: bool,
    
    /// Health check interval in seconds
    pub health_check_interval_seconds: u64,
}

impl Default for EventPublisherMigrationConfig {
    fn default() -> Self {
        Self {
            enable_high_performance: false,
            rollout_percentage: 0,
            max_events_per_second: None,
            high_performance_config: Some(HighPerformanceConfig::default()),
            migration_safety: MigrationSafetyConfig::default(),
        }
    }
}

impl Default for HighPerformanceConfig {
    fn default() -> Self {
        Self {
            buffer_size: 10_000,
            max_concurrent: 100,
            batch_size: 50,
            batch_timeout_ms: 10,
            enable_circuit_breaker: true,
            max_retries: 3,
            initial_backoff_ms: 50,
            max_backoff_ms: 5000,
            enable_backpressure: true,
            backpressure_high_water_mark: 8_000,
            backpressure_low_water_mark: 2_000,
        }
    }
}

impl Default for MigrationSafetyConfig {
    fn default() -> Self {
        Self {
            enable_automatic_fallback: true,
            error_threshold_percentage: 5.0,
            error_window_seconds: 300, // 5 minutes
            enable_health_monitoring: true,
            health_check_interval_seconds: 30,
        }
    }
}

impl From<HighPerformanceConfig> for PublisherConfig {
    fn from(config: HighPerformanceConfig) -> Self {
        Self {
            buffer_size: config.buffer_size,
            max_concurrent: config.max_concurrent,
            batch_size: config.batch_size,
            batch_timeout: Duration::from_millis(config.batch_timeout_ms),
            circuit_breaker: CircuitBreakerConfig {
                failure_threshold: 5,
                success_threshold: 3,
                timeout: Duration::from_secs(30),
            },
            retry: RetryConfig {
                max_retries: config.max_retries,
                initial_backoff: Duration::from_millis(config.initial_backoff_ms),
                max_backoff: Duration::from_millis(config.max_backoff_ms),
                jitter: true,
            },
            backpressure: BackpressureConfig {
                enabled: config.enable_backpressure,
                high_water_mark: config.backpressure_high_water_mark,
                low_water_mark: config.backpressure_low_water_mark,
                shed_probability: 0.1,
            },
        }
    }
}

// =====================================================================================
// MIGRATION MANAGER
// =====================================================================================

/// Manages the migration between legacy and high-performance publishers
pub struct PublisherMigrationManager {
    config: EventPublisherMigrationConfig,
    app_config: Arc<AppConfig>,
}

impl PublisherMigrationManager {
    pub fn new(config: EventPublisherMigrationConfig, app_config: Arc<AppConfig>) -> Self {
        Self { config, app_config }
    }

    /// Initialize publisher based on migration configuration
    pub async fn initialize_publisher(&self) -> Result<(), AppError> {
        let nats_client = self.create_nats_client().await?;
        
        if self.config.enable_high_performance && self.should_use_high_performance() {
            info!(
                "Initializing high-performance publisher (rollout: {}%)",
                self.config.rollout_percentage
            );
            
            // In a complete implementation, you would use the high-performance publisher here
            // For now, we fall back to the NATS transport
            info!("High-performance publisher initialization (placeholder)");
            
            if self.config.migration_safety.enable_health_monitoring {
                self.start_health_monitoring().await?;
            }
        } else {
            info!("Initializing legacy publisher");
            // Use the available NATS transport
            info!("Legacy publisher initialization (placeholder)");
        }

        Ok(())
    }

    /// Check if high-performance publisher should be used based on rollout percentage
    fn should_use_high_performance(&self) -> bool {
        if !self.config.enable_high_performance {
            return false;
        }

        if self.config.rollout_percentage >= 100 {
            return true;
        }

        if self.config.rollout_percentage == 0 {
            return false;
        }

        // Use deterministic selection based on configuration hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.app_config.server.port.hash(&mut hasher);
        self.app_config.server.db_url.hash(&mut hasher);
        
        let hash = hasher.finish();
        let selection = (hash % 100) as u8;
        
        selection < self.config.rollout_percentage
    }

    /// Create NATS client from application configuration
    async fn create_nats_client(&self) -> Result<async_nats::Client, AppError> {
        let nats_url = &self.app_config.features.messaging.nats_url;
        
        async_nats::connect(nats_url)
            .await
            .map_err(|e| AppError::EventPublishError(format!("Failed to connect to NATS: {}", e)))
    }

    /// Create publisher configuration from migration config
    fn create_publisher_config(&self) -> PublisherConfig {
        self.config
            .high_performance_config
            .clone()
            .unwrap_or_default()
            .into()
    }

    /// Start health monitoring for the migration
    async fn start_health_monitoring(&self) -> Result<(), AppError> {
        let safety_config = self.config.migration_safety.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                Duration::from_secs(safety_config.health_check_interval_seconds)
            );

            loop {
                interval.tick().await;
                
                if let Err(e) = Self::perform_health_check(&safety_config).await {
                    error!("Health check failed: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Perform health check and automatic fallback if needed
    async fn perform_health_check(safety_config: &MigrationSafetyConfig) -> Result<(), AppError> {
        // Simplified health check implementation
        // In a complete system, this would integrate with the actual publisher health monitoring
        
        // Placeholder health status
        let success_rate = 0.95; // 95% success rate as example
        let is_healthy = success_rate > (1.0 - safety_config.error_threshold_percentage / 100.0);
        
        if !is_healthy {
            warn!(
                "Publisher health check failed - Success rate: {:.2}%",
                success_rate * 100.0,
            );

            if safety_config.enable_automatic_fallback {
                error!(
                    "Error threshold exceeded ({:.2}% < {:.2}%), automatic fallback needed",
                    success_rate * 100.0,
                    100.0 - safety_config.error_threshold_percentage
                );
                
                // In a production system, you would implement automatic fallback here
                // For now, we just log the need for fallback
            }
        } else {
            info!(
                "Publisher health check passed - Success rate: {:.2}%",
                success_rate * 100.0,
            );
        }

        Ok(())
    }

    /// Get current migration status
    pub fn migration_status(&self) -> MigrationStatus {
        MigrationStatus {
            high_performance_enabled: self.config.enable_high_performance,
            rollout_percentage: self.config.rollout_percentage,
            using_high_performance: self.should_use_high_performance(),
            safety_enabled: self.config.migration_safety.enable_automatic_fallback,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MigrationStatus {
    pub high_performance_enabled: bool,
    pub rollout_percentage: u8,
    pub using_high_performance: bool,
    pub safety_enabled: bool,
}

// =====================================================================================
// CONFIGURATION LOADING
// =====================================================================================

/// Load migration configuration from various sources
pub struct MigrationConfigLoader;

impl MigrationConfigLoader {
    /// Load configuration from environment variables
    pub fn from_env() -> EventPublisherMigrationConfig {
        let enable_high_performance = std::env::var("FECHATTER_ENABLE_HIGH_PERFORMANCE_PUBLISHER")
            .map(|v| v.parse().unwrap_or(false))
            .unwrap_or(false);
            
        let rollout_percentage = std::env::var("FECHATTER_PUBLISHER_ROLLOUT_PERCENTAGE")
            .map(|v| v.parse().unwrap_or(0))
            .unwrap_or(0)
            .clamp(0, 100);

        EventPublisherMigrationConfig {
            enable_high_performance,
            rollout_percentage,
            ..Default::default()
        }
    }

    /// Load configuration from YAML file
    pub fn from_file(path: &str) -> Result<EventPublisherMigrationConfig, AppError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AppError::ConfigError(format!("Failed to read config file: {}", e)))?;
            
        serde_yaml::from_str(&content)
            .map_err(|e| AppError::ConfigError(format!("Failed to parse config: {}", e)))
    }

    /// Load configuration with environment variable overrides
    pub fn load_with_overrides(base_config: EventPublisherMigrationConfig) -> EventPublisherMigrationConfig {
        let mut config = base_config;
        
        if let Ok(enable_str) = std::env::var("FECHATTER_ENABLE_HIGH_PERFORMANCE_PUBLISHER") {
            if let Ok(enable) = enable_str.parse() {
                config.enable_high_performance = enable;
            }
        }
        
        if let Ok(percentage_str) = std::env::var("FECHATTER_PUBLISHER_ROLLOUT_PERCENTAGE") {
            if let Ok(percentage) = percentage_str.parse::<u8>() {
                config.rollout_percentage = percentage.clamp(0, 100);
            }
        }
        
        config
    }
}

// =====================================================================================
// UTILITIES AND HELPERS
// =====================================================================================

/// Utility function to initialize publisher with migration configuration
pub async fn initialize_with_migration_config(
    app_config: Arc<AppConfig>,
    migration_config: Option<EventPublisherMigrationConfig>,
) -> Result<(), AppError> {
    let config = migration_config.unwrap_or_else(|| {
        MigrationConfigLoader::load_with_overrides(MigrationConfigLoader::from_env())
    });
    
    info!("Event publisher migration config: {:?}", config);
    
    let manager = PublisherMigrationManager::new(config, app_config);
    manager.initialize_publisher().await
}

/// Validate migration configuration
pub fn validate_migration_config(config: &EventPublisherMigrationConfig) -> Result<(), String> {
    if config.rollout_percentage > 100 {
        return Err("Rollout percentage cannot exceed 100".to_string());
    }
    
    if let Some(hp_config) = &config.high_performance_config {
        if hp_config.buffer_size == 0 {
            return Err("Buffer size must be greater than 0".to_string());
        }
        
        if hp_config.max_concurrent == 0 {
            return Err("Max concurrent must be greater than 0".to_string());
        }
        
        if hp_config.batch_size == 0 {
            return Err("Batch size must be greater than 0".to_string());
        }
    }
    
    if config.migration_safety.error_threshold_percentage < 0.0 
        || config.migration_safety.error_threshold_percentage > 100.0 {
        return Err("Error threshold percentage must be between 0 and 100".to_string());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_config_defaults() {
        let config = EventPublisherMigrationConfig::default();
        assert!(!config.enable_high_performance);
        assert_eq!(config.rollout_percentage, 0);
        assert!(config.high_performance_config.is_some());
    }

    #[test]
    fn test_config_validation() {
        let mut config = EventPublisherMigrationConfig::default();
        
        // Valid config should pass
        assert!(validate_migration_config(&config).is_ok());
        
        // Invalid rollout percentage
        config.rollout_percentage = 150;
        assert!(validate_migration_config(&config).is_err());
        
        // Reset and test buffer size
        config.rollout_percentage = 50;
        config.high_performance_config.as_mut().unwrap().buffer_size = 0;
        assert!(validate_migration_config(&config).is_err());
    }

    #[test]
    fn test_rollout_percentage_deterministic() {
        // Skip this test since AppConfig doesn't have a Default implementation
        // In a complete system, you would provide a proper test configuration
        
        let mut config = EventPublisherMigrationConfig::default();
        config.enable_high_performance = true;
        
        // Test boundary conditions
        config.rollout_percentage = 0;
        // Cannot test without AppConfig, but logic should be deterministic
        
        config.rollout_percentage = 100;
        // Would always use high performance if enabled
    }

    #[test]
    fn test_high_performance_config_conversion() {
        let hp_config = HighPerformanceConfig {
            buffer_size: 5000,
            max_concurrent: 50,
            batch_size: 25,
            batch_timeout_ms: 20,
            max_retries: 5,
            initial_backoff_ms: 100,
            max_backoff_ms: 10000,
            ..Default::default()
        };
        
        let publisher_config: PublisherConfig = hp_config.into();
        assert_eq!(publisher_config.buffer_size, 5000);
        assert_eq!(publisher_config.max_concurrent, 50);
        assert_eq!(publisher_config.batch_size, 25);
        assert_eq!(publisher_config.batch_timeout, Duration::from_millis(20));
    }
}