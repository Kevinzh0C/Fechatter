// Adaptive Event Publisher Usage Examples
//
// This file demonstrates how to use the new adaptive event publishing system
// with real-world scenarios and best practices.

#[cfg(test)]
mod examples {
    use std::time::Duration;
    use tokio::time::sleep;
    
    use crate::{
        error::AppError,
        services::infrastructure::event::{
            auto_degradation::{
                adaptive_publisher::{AdaptivePublisher, AdaptivePublisherConfig, PublisherBackend},
                publisher_integration::{
                    PublisherFactory, publish_message_created, publisher_health_check,
                    get_current_backend, switch_publisher_backend, clear_manual_override,
                    publish_message_events_batch,
                },
            },
        },
    };
    
    use fechatter_core::{ChatId, Message, MessageId, UserId, contracts::events::MessageLifecycle};
    use chrono::Utc;

    /// Example 1: Basic adaptive publisher usage
    #[tokio::test]
    #[ignore] // Requires NATS server
    async fn example_basic_adaptive_usage() -> Result<(), AppError> {
        // Connect to NATS
        let nats_client = async_nats::connect("nats://localhost:4222")
            .await
            .map_err(|e| AppError::EventPublishError(format!("NATS connection failed: {}", e)))?;

        // Initialize global adaptive publisher with default config (high-performance preferred)
        PublisherFactory::initialize_global(nats_client, None).await?;

        // Create sample message
        let message = Message {
            id: MessageId(123),
            chat_id: ChatId(456),
            sender_id: UserId(789),
            content: "Hello from adaptive publisher!".to_string(),
            files: None,
            created_at: Utc::now(),
            idempotency_key: None,
        };

        let chat_members = vec![UserId(789), UserId(101112)];

        // Publish using convenience function
        publish_message_created(&message, &chat_members).await?;
        println!("Published message created event using adaptive publisher");

        // Check which backend was used
        let current_backend = get_current_backend().await?;
        println!("Current backend: {}", current_backend);

        // Check health status
        let health = publisher_health_check().await?;
        println!("Publisher health: success_rate={:.2}%, latency={:.2}ms", 
                 health.success_rate * 100.0, health.average_latency_ms);

        Ok(())
    }

    /// Example 2: Manual backend switching and monitoring
    #[tokio::test]
    #[ignore] // Requires NATS server
    async fn example_manual_backend_switching() -> Result<(), AppError> {
        let nats_client = async_nats::connect("nats://localhost:4222")
            .await
            .map_err(|e| AppError::EventPublishError(format!("NATS connection failed: {}", e)))?;

        PublisherFactory::initialize_global(nats_client, None).await?;

        // Check initial backend
        let initial_backend = get_current_backend().await?;
        println!("Initial backend: {}", initial_backend);

        // Manually switch to legacy backend
        switch_publisher_backend(
            PublisherBackend::Legacy, 
            "Manual testing of legacy backend".to_string()
        ).await?;

        let switched_backend = get_current_backend().await?;
        println!("Switched to backend: {}", switched_backend);

        // Publish a message using legacy backend
        let message = Message {
            id: MessageId(124),
            chat_id: ChatId(456),
            sender_id: UserId(789),
            content: "Message from legacy backend".to_string(),
            files: None,
            created_at: Utc::now(),
            idempotency_key: None,
        };

        publish_message_created(&message, &[UserId(789)]).await?;
        println!("Published message using legacy backend");

        // Clear manual override to return to automatic mode
        clear_manual_override().await?;
        println!("Cleared manual override, returning to automatic backend selection");

        Ok(())
    }

    /// Example 3: Production configuration with custom thresholds
    #[tokio::test]
    #[ignore] // Requires NATS server
    async fn example_production_configuration() -> Result<(), AppError> {
        let nats_client = async_nats::connect("nats://localhost:4222")
            .await
            .map_err(|e| AppError::EventPublishError(format!("NATS connection failed: {}", e)))?;

        // Create production-optimized configuration
        let config = AdaptivePublisherConfig::production();
        
        PublisherFactory::initialize_global(nats_client, Some(config)).await?;
        
        println!("Initialized with production configuration:");
        println!("- Preferred backend: high-performance");
        println!("- Max error rate: 2%");
        println!("- Max latency: 500ms");
        println!("- Recovery threshold: 99% success rate");

        // Simulate some message publishing
        for i in 1..=10 {
            let message = Message {
                id: MessageId(i),
                chat_id: ChatId(456),
                sender_id: UserId(789),
                content: format!("Production message {}", i),
                files: None,
                created_at: Utc::now(),
                idempotency_key: None,
            };

            publish_message_created(&message, &[UserId(789)]).await?;
            
            // Small delay to simulate realistic publishing patterns
            sleep(Duration::from_millis(10)).await;
        }

        let health = publisher_health_check().await?;
        println!("Final health status: success_rate={:.2}%, backend={}", 
                 health.success_rate * 100.0, health.current_backend);

        Ok(())
    }

    /// Example 4: Development configuration with relaxed thresholds
    #[tokio::test]
    #[ignore] // Requires NATS server
    async fn example_development_configuration() -> Result<(), AppError> {
        let nats_client = async_nats::connect("nats://localhost:4222")
            .await
            .map_err(|e| AppError::EventPublishError(format!("NATS connection failed: {}", e)))?;

        // Create development-friendly configuration
        let config = AdaptivePublisherConfig::development();
        
        PublisherFactory::initialize_global(nats_client, Some(config)).await?;
        
        println!("Initialized with development configuration:");
        println!("- Preferred backend: high-performance");
        println!("- Max error rate: 10% (more lenient)");
        println!("- Max latency: 2000ms (more lenient)");
        println!("- Recovery threshold: 95% success rate");

        // Development configurations are more forgiving for testing
        let health = publisher_health_check().await?;
        println!("Development health check: healthy={}, backend={}", 
                 health.is_healthy, health.current_backend);

        Ok(())
    }

    /// Example 5: Environment-based configuration
    #[tokio::test]
    #[ignore] // Requires environment variables
    async fn example_environment_configuration() -> Result<(), AppError> {
        // Set environment variables for testing
        std::env::set_var("FECHATTER_PREFERRED_BACKEND", "high_performance");
        std::env::set_var("FECHATTER_ENABLE_AUTO_DEGRADATION", "true");
        std::env::set_var("FECHATTER_MAX_ERROR_RATE", "0.03"); // 3%
        std::env::set_var("FECHATTER_MAX_LATENCY_MS", "800");

        let nats_client = async_nats::connect("nats://localhost:4222")
            .await
            .map_err(|e| AppError::EventPublishError(format!("NATS connection failed: {}", e)))?;

        // Initialize using environment variables
        PublisherFactory::initialize_with_env(nats_client).await?;
        
        println!("Initialized using environment configuration");

        let health = publisher_health_check().await?;
        println!("Environment-configured publisher: backend={}, healthy={}", 
                 health.current_backend, health.is_healthy);

        // Clean up environment variables
        std::env::remove_var("FECHATTER_PREFERRED_BACKEND");
        std::env::remove_var("FECHATTER_ENABLE_AUTO_DEGRADATION");
        std::env::remove_var("FECHATTER_MAX_ERROR_RATE");
        std::env::remove_var("FECHATTER_MAX_LATENCY_MS");

        Ok(())
    }

    /// Example 6: Batch publishing with health monitoring
    #[tokio::test]
    #[ignore] // Requires NATS server
    async fn example_batch_publishing_with_monitoring() -> Result<(), AppError> {
        let nats_client = async_nats::connect("nats://localhost:4222")
            .await
            .map_err(|e| AppError::EventPublishError(format!("NATS connection failed: {}", e)))?;

        PublisherFactory::initialize_global(nats_client, None).await?;

        // Create batch of messages
        let messages = (1..=50).map(|i| {
            let msg = Message {
                id: MessageId(i),
                chat_id: ChatId(456),
                sender_id: UserId(789),
                content: format!("Batch message {}", i),
                files: None,
                created_at: Utc::now(),
                idempotency_key: None,
            };
            (MessageLifecycle::Created, msg, vec![UserId(789)])
        }).collect::<Vec<_>>();

        let start = std::time::Instant::now();
        
        // Publish batch (each message is monitored individually for health)
        publish_message_events_batch(messages).await?;
        
        let duration = start.elapsed();
        println!("Published 50 messages in {:?}", duration);

        // Check final health status
        let health = publisher_health_check().await?;
        println!("Batch publishing health: success_rate={:.2}%, latency={:.2}ms", 
                 health.success_rate * 100.0, health.average_latency_ms);

        Ok(())
    }
}

/// Production deployment examples and patterns
pub mod production_examples {
    use std::time::Duration;
    use crate::{
        error::AppError,
        services::infrastructure::event::{
            auto_degradation::{
                adaptive_publisher::{AdaptivePublisherConfig, PublisherBackend, DegradationThresholds, RecoveryThresholds},
                publisher_integration::{PublisherFactory, publisher_health_check},
            },
        },
    };

    /// Example: Initialize for high-availability production deployment
    pub async fn initialize_ha_production(nats_url: &str) -> Result<(), AppError> {
        let nats_client = async_nats::connect(nats_url)
            .await
            .map_err(|e| AppError::EventPublishError(format!("NATS connection failed: {}", e)))?;

        // Production configuration with aggressive monitoring
        let config = AdaptivePublisherConfig {
            preferred_backend: PublisherBackend::HighPerformance,
            enable_auto_degradation: true,
            health_check_interval: Duration::from_secs(2), // Very frequent checks
            degradation_thresholds: DegradationThresholds {
                max_error_rate: 0.01, // 1% error rate threshold
                max_latency_ms: 200.0, // 200ms max latency
                max_queue_size: 100_000,
                consecutive_failure_threshold: 2, // Aggressive degradation
                error_window_duration: Duration::from_secs(30),
            },
            recovery_thresholds: RecoveryThresholds {
                min_success_rate: 0.995, // 99.5% success rate for recovery
                max_latency_ms: 50.0, // 50ms max latency for recovery
                consecutive_success_threshold: 30,
                recovery_window_duration: Duration::from_secs(120),
            },
            recovery_delay: Duration::from_secs(30),
            high_performance_config: crate::services::infrastructure::event::high_performance::PublisherConfig {
                buffer_size: 200_000,
                max_concurrent: 1000,
                batch_size: 200,
                batch_timeout: Duration::from_millis(2),
                ..Default::default()
            },
        };

        PublisherFactory::initialize_global(nats_client, Some(config)).await?;
        
        println!("High-availability production publisher initialized");
        Ok(())
    }

    /// Example: Monitor and alert on publisher health
    pub async fn health_monitoring_loop() -> Result<(), AppError> {
        loop {
            let health = publisher_health_check().await?;
            
            // Log health status
            println!("Publisher Health: backend={}, success_rate={:.2}%, latency={:.2}ms, degradations={}", 
                     health.current_backend,
                     health.success_rate * 100.0,
                     health.average_latency_ms,
                     health.total_degradations);
            
            // Alert on critical conditions
            if !health.is_healthy {
                eprintln!("ALERT: Publisher unhealthy! Backend: {}, Success Rate: {:.2}%", 
                         health.current_backend, health.success_rate * 100.0);
            }
            
            if health.total_degradations > 100 {
                eprintln!("ALERT: High degradation count: {}", health.total_degradations);
            }
            
            if health.manual_override {
                println!("WARNING: Manual override active");
            }
            
            // Wait before next check
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    }
}

/// Integration patterns with existing systems
pub mod integration_patterns {
    use std::time::Duration;
    use fechatter_core::{ChatId, Message, MessageId, UserId};
    use chrono::Utc;
    use crate::{
        error::AppError,
        services::infrastructure::event::{
            auto_degradation::{
                adaptive_publisher::{AdaptivePublisherConfig, PublisherBackend},
                publisher_integration::{PublisherFactory, publish_message_created, switch_publisher_backend, clear_manual_override},
            },
        },
    };

    /// Example: Gradual migration from legacy to adaptive
    pub async fn gradual_migration_pattern(nats_client: async_nats::Client) -> Result<(), AppError> {
        // Phase 1: Start with legacy preferred for safety
        let mut config = AdaptivePublisherConfig::default();
        config.preferred_backend = PublisherBackend::Legacy;
        config.enable_auto_degradation = false; // Disable auto-switching initially
        
        PublisherFactory::initialize_global(nats_client, Some(config)).await?;
        println!("Phase 1: Started with legacy backend, no auto-degradation");
        
        // Simulate some load
        for i in 1..=20 {
            let message = Message {
                id: MessageId(i),
                chat_id: ChatId(456),
                sender_id: UserId(789),
                content: format!("Migration test message {}", i),
                files: None,
                created_at: Utc::now(),
                idempotency_key: None,
            };
            publish_message_created(&message, &[UserId(789)]).await?;
        }
        
        // Phase 2: Enable auto-degradation but keep legacy preferred
        // (In production, this would be a configuration update)
        println!("Phase 2: Would enable auto-degradation with legacy preferred");
        
        // Phase 3: Switch to high-performance preferred
        switch_publisher_backend(
            PublisherBackend::HighPerformance,
            "Migrating to high-performance backend".to_string()
        ).await?;
        println!("Phase 3: Manually switched to high-performance backend");
        
        // Phase 4: Enable auto-degradation for production safety
        clear_manual_override().await?;
        println!("Phase 4: Enabled automatic backend selection");
        
        Ok(())
    }
}