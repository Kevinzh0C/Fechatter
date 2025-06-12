// Adaptive Event Publisher with Auto-Degradation
//
// This publisher automatically switches between high-performance and legacy backends
// based on real-time health monitoring and configurable thresholds.

use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use tokio::sync::{Mutex, RwLock};
use tracing::{error, info, warn, instrument};
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    services::infrastructure::event::{
        high_performance::{
            HighPerformancePublisher, PublisherConfig as HighPerformanceConfig,
            EventData, PublishResult, FastMessageEvent, FastChatMemberEvent,
        },
        legacy::{
            EventPublisher as LegacyEventPublisher, NatsEventPublisher,
        },
        shared::{EventTransport, NatsTransport},
    },
};

use fechatter_core::{
    ChatId, Message, MessageId, UserId,
    contracts::events::{MessageLifecycle, MessageEvent, ChatMemberJoinedEvent, ChatMemberLeftEvent},
};

// Define local duration_serde module to avoid cross-module dependency
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

// =====================================================================================
// PUBLISHER BACKEND ENUM
// =====================================================================================

/// Available publisher backends
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublisherBackend {
    HighPerformance,
    Legacy,
}

impl PublisherBackend {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HighPerformance => "high_performance",
            Self::Legacy => "legacy",
        }
    }
}

/// Reasons for degradation switches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DegradationReason {
    HighErrorRate { error_rate: f64, threshold: f64 },
    HighLatency { avg_latency_ms: f64, threshold_ms: f64 },
    QueueOverflow { queue_size: usize, threshold: usize },
    CircuitBreakerOpen,
    ConnectionFailure { error: String },
    ManualOverride,
    StartupFailure { error: String },
}

impl DegradationReason {
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            Self::CircuitBreakerOpen 
            | Self::ConnectionFailure { .. } 
            | Self::StartupFailure { .. }
        )
    }
}

/// Decision made by the adaptive logic
#[derive(Debug, Clone)]
pub enum SwitchDecision {
    Stay(PublisherBackend),
    Switch { from: PublisherBackend, to: PublisherBackend, reason: DegradationReason },
}

// =====================================================================================
// ADAPTIVE PUBLISHER CONFIGURATION
// =====================================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptivePublisherConfig {
    /// Preferred backend (default: HighPerformance)
    pub preferred_backend: PublisherBackend,
    
    /// Enable automatic degradation
    pub enable_auto_degradation: bool,
    
    /// Health check interval
    #[serde(with = "duration_serde")]
    pub health_check_interval: Duration,
    
    /// Degradation thresholds
    pub degradation_thresholds: DegradationThresholds,
    
    /// Recovery thresholds for switching back
    pub recovery_thresholds: RecoveryThresholds,
    
    /// Minimum time before attempting recovery
    #[serde(with = "duration_serde")]
    pub recovery_delay: Duration,
    
    /// Configuration for high-performance backend
    pub high_performance_config: HighPerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationThresholds {
    /// Maximum error rate before degradation (0.0-1.0)
    pub max_error_rate: f64,
    
    /// Maximum average latency in milliseconds
    pub max_latency_ms: f64,
    
    /// Maximum queue size
    pub max_queue_size: usize,
    
    /// Number of consecutive failures to trigger degradation
    pub consecutive_failure_threshold: u32,
    
    /// Time window for error rate calculation
    #[serde(with = "duration_serde")]
    pub error_window_duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryThresholds {
    /// Minimum success rate for recovery (0.0-1.0)
    pub min_success_rate: f64,
    
    /// Maximum latency for recovery
    pub max_latency_ms: f64,
    
    /// Number of consecutive successes required for recovery
    pub consecutive_success_threshold: u32,
    
    /// Time window for recovery evaluation
    #[serde(with = "duration_serde")]
    pub recovery_window_duration: Duration,
}

impl Default for AdaptivePublisherConfig {
    fn default() -> Self {
        Self {
            preferred_backend: PublisherBackend::HighPerformance,
            enable_auto_degradation: true,
            health_check_interval: Duration::from_secs(10),
            degradation_thresholds: DegradationThresholds {
                max_error_rate: 0.05, // 5% error rate
                max_latency_ms: 1000.0, // 1 second
                max_queue_size: 40_000,
                consecutive_failure_threshold: 3,
                error_window_duration: Duration::from_secs(60),
            },
            recovery_thresholds: RecoveryThresholds {
                min_success_rate: 0.98, // 98% success rate
                max_latency_ms: 100.0, // 100ms
                consecutive_success_threshold: 10,
                recovery_window_duration: Duration::from_secs(120),
            },
            recovery_delay: Duration::from_secs(30),
            high_performance_config: HighPerformanceConfig::default(),
        }
    }
}

// =====================================================================================
// ADAPTIVE PUBLISHER IMPLEMENTATION
// =====================================================================================

/// Adaptive publisher that automatically switches between backends
pub struct AdaptivePublisher {
    /// Current active backend
    current_backend: Arc<RwLock<PublisherBackend>>,
    
    /// High-performance publisher instance
    high_performance_publisher: Option<HighPerformancePublisher>,
    
    /// Legacy publisher instance
    legacy_publisher: NatsEventPublisher,
    
    /// Configuration
    config: AdaptivePublisherConfig,
    
    /// Health monitoring state
    health_state: Arc<HealthState>,
    
    /// Background monitoring task handle
    _monitor_handle: tokio::task::JoinHandle<()>,
}

/// Internal health monitoring state
struct HealthState {
    /// Recent operation results (success, latency_ms)
    recent_operations: Mutex<Vec<(bool, f64, Instant)>>,
    
    /// Consecutive failures counter
    consecutive_failures: AtomicU32,
    
    /// Consecutive successes counter
    consecutive_successes: AtomicU32,
    
    /// Last degradation time
    last_degradation: RwLock<Option<Instant>>,
    
    /// Degradation count
    degradation_count: AtomicU64,
    
    /// Manual override flag
    manual_override: AtomicBool,
    
    /// Override backend
    override_backend: RwLock<Option<PublisherBackend>>,
}

impl HealthState {
    fn new() -> Self {
        Self {
            recent_operations: Mutex::new(Vec::new()),
            consecutive_failures: AtomicU32::new(0),
            consecutive_successes: AtomicU32::new(0),
            last_degradation: RwLock::new(None),
            degradation_count: AtomicU64::new(0),
            manual_override: AtomicBool::new(false),
            override_backend: RwLock::new(None),
        }
    }
    
    async fn record_operation(&self, success: bool, latency_ms: f64, max_entries: usize) {
        let mut ops = self.recent_operations.lock().await;
        ops.push((success, latency_ms, Instant::now()));
        
        // Keep only recent entries - fix borrow checker issue
        let current_len = ops.len();
        if current_len > max_entries {
            let excess_count = current_len - max_entries;
            ops.drain(0..excess_count);
        }
        
        // Update consecutive counters
        if success {
            self.consecutive_failures.store(0, Ordering::Relaxed);
            self.consecutive_successes.fetch_add(1, Ordering::Relaxed);
        } else {
            self.consecutive_successes.store(0, Ordering::Relaxed);
            self.consecutive_failures.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    async fn calculate_metrics(&self, window: Duration) -> (f64, f64, usize) {
        let ops = self.recent_operations.lock().await;
        let cutoff = Instant::now() - window;
        
        let recent_ops: Vec<_> = ops.iter()
            .filter(|(_, _, timestamp)| *timestamp > cutoff)
            .collect();
        
        if recent_ops.is_empty() {
            return (1.0, 0.0, 0); // No data = assume healthy
        }
        
        let success_count = recent_ops.iter().filter(|(success, _, _)| *success).count();
        let success_rate = success_count as f64 / recent_ops.len() as f64;
        
        let avg_latency = recent_ops.iter()
            .map(|(_, latency, _)| *latency)
            .sum::<f64>() / recent_ops.len() as f64;
        
        (success_rate, avg_latency, recent_ops.len())
    }
}

impl AdaptivePublisher {
    /// Create a new adaptive publisher
    pub async fn new(
        nats_client: async_nats::Client,
        config: AdaptivePublisherConfig,
    ) -> Result<Self, AppError> {
        info!("Initializing AdaptivePublisher with preferred backend: {:?}", config.preferred_backend);
        
        // Create legacy publisher (always available as fallback)
        let legacy_publisher = NatsEventPublisher::new(nats_client.clone());
        
        // Try to create high-performance publisher
        let high_performance_publisher = match HighPerformancePublisher::new(
            nats_client.clone(),
            config.high_performance_config.clone(),
        ) {
            Ok(publisher) => {
                info!("High-performance publisher initialized successfully");
                Some(publisher)
            }
            Err(e) => {
                warn!("Failed to initialize high-performance publisher: {}, falling back to legacy", e);
                None
            }
        };
        
        // Determine initial backend
        let initial_backend = if high_performance_publisher.is_some() && config.preferred_backend == PublisherBackend::HighPerformance {
            PublisherBackend::HighPerformance
        } else {
            PublisherBackend::Legacy
        };
        
        let current_backend = Arc::new(RwLock::new(initial_backend));
        let health_state = Arc::new(HealthState::new());
        
        // Start background health monitoring
        let monitor_handle = Self::start_health_monitoring(
            current_backend.clone(),
            health_state.clone(),
            config.clone(),
        );
        
        info!("AdaptivePublisher initialized with backend: {:?}", initial_backend);
        
        Ok(Self {
            current_backend,
            high_performance_publisher,
            legacy_publisher,
            config,
            health_state,
            _monitor_handle: monitor_handle,
        })
    }
    
    /// Start background health monitoring
    fn start_health_monitoring(
        current_backend: Arc<RwLock<PublisherBackend>>,
        health_state: Arc<HealthState>,
        config: AdaptivePublisherConfig,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.health_check_interval);
            
            loop {
                interval.tick().await;
                
                if !config.enable_auto_degradation {
                    continue;
                }
                
                // Check if manual override is active
                if health_state.manual_override.load(Ordering::Relaxed) {
                    continue;
                }
                
                let decision = Self::evaluate_health(&health_state, &config).await;
                
                match decision {
                    SwitchDecision::Stay(_) => {
                        // No action needed
                    }
                    SwitchDecision::Switch { from, to, reason } => {
                        warn!("Health monitor triggering backend switch: {:?} -> {:?}, reason: {:?}", 
                              from, to, reason);
                        
                        let mut backend = current_backend.write().await;
                        *backend = to;
                        
                        // Update degradation tracking
                        if to == PublisherBackend::Legacy {
                            *health_state.last_degradation.write().await = Some(Instant::now());
                            health_state.degradation_count.fetch_add(1, Ordering::Relaxed);
                        }
                        
                        info!("Backend switched to: {:?}", to);
                    }
                }
            }
        })
    }
    
    /// Evaluate current health and decide on backend switches
    async fn evaluate_health(
        health_state: &HealthState,
        config: &AdaptivePublisherConfig,
    ) -> SwitchDecision {
        let (success_rate, avg_latency, _sample_size) = health_state
            .calculate_metrics(config.degradation_thresholds.error_window_duration)
            .await;
        
        let consecutive_failures = health_state.consecutive_failures.load(Ordering::Relaxed);
        let consecutive_successes = health_state.consecutive_successes.load(Ordering::Relaxed);
        
        // Check if we need to degrade
        if success_rate < (1.0 - config.degradation_thresholds.max_error_rate) {
            return SwitchDecision::Switch {
                from: PublisherBackend::HighPerformance,
                to: PublisherBackend::Legacy,
                reason: DegradationReason::HighErrorRate {
                    error_rate: 1.0 - success_rate,
                    threshold: config.degradation_thresholds.max_error_rate,
                },
            };
        }
        
        if avg_latency > config.degradation_thresholds.max_latency_ms {
            return SwitchDecision::Switch {
                from: PublisherBackend::HighPerformance,
                to: PublisherBackend::Legacy,
                reason: DegradationReason::HighLatency {
                    avg_latency_ms: avg_latency,
                    threshold_ms: config.degradation_thresholds.max_latency_ms,
                },
            };
        }
        
        if consecutive_failures >= config.degradation_thresholds.consecutive_failure_threshold {
            return SwitchDecision::Switch {
                from: PublisherBackend::HighPerformance,
                to: PublisherBackend::Legacy,
                reason: DegradationReason::HighErrorRate {
                    error_rate: 1.0,
                    threshold: config.degradation_thresholds.max_error_rate,
                },
            };
        }
        
        // Check if we can recover
        if let Some(last_degradation) = *health_state.last_degradation.read().await {
            if last_degradation.elapsed() >= config.recovery_delay {
                if success_rate >= config.recovery_thresholds.min_success_rate
                    && avg_latency <= config.recovery_thresholds.max_latency_ms
                    && consecutive_successes >= config.recovery_thresholds.consecutive_success_threshold
                {
                    return SwitchDecision::Switch {
                        from: PublisherBackend::Legacy,
                        to: PublisherBackend::HighPerformance,
                        reason: DegradationReason::ManualOverride, // Recovery reason
                    };
                }
            }
        }
        
        SwitchDecision::Stay(PublisherBackend::HighPerformance) // Default assumption
    }
    
    /// Get current backend
    pub async fn current_backend(&self) -> PublisherBackend {
        *self.current_backend.read().await
    }
    
    /// Manually switch backend
    pub async fn switch_backend(&self, backend: PublisherBackend, reason: String) -> Result<(), AppError> {
        info!("Manual backend switch to {:?}: {}", backend, reason);
        
        let mut current = self.current_backend.write().await;
        *current = backend;
        
        // Set manual override
        self.health_state.manual_override.store(true, Ordering::Relaxed);
        *self.health_state.override_backend.write().await = Some(backend);
        
        Ok(())
    }
    
    /// Clear manual override
    pub async fn clear_manual_override(&self) {
        info!("Clearing manual backend override");
        self.health_state.manual_override.store(false, Ordering::Relaxed);
        *self.health_state.override_backend.write().await = None;
    }
    
    /// Publish message event with automatic backend selection
    #[instrument(skip(self, message, chat_members))]
    pub async fn publish_message_event(
        &self,
        kind: MessageLifecycle,
        message: &Message,
        chat_members: &[UserId],
    ) -> Result<(), AppError> {
        let start = Instant::now();
        let backend = self.current_backend().await;
        
        let result = match backend {
            PublisherBackend::HighPerformance => {
                if let Some(ref hp_publisher) = self.high_performance_publisher {
                    let event = FastMessageEvent {
                        message_id: message.id.0 as u64,
                        chat_id: message.chat_id.0 as u64,
                        user_id: message.sender_id.0 as u64,
                        content: message.content.clone(),
                        event_type: match kind {
                            MessageLifecycle::Created => "created".to_string(),
                            MessageLifecycle::Updated => "updated".to_string(),
                            MessageLifecycle::Deleted => "deleted".to_string(),
                        },
                        timestamp: message.created_at.timestamp() as u64,
                        workspace_id: None,
                    };
                    
                    match hp_publisher.publish(event).await {
                        Ok(publish_result) => {
                            if publish_result.success {
                                Ok(())
                            } else {
                                Err(AppError::EventPublishError(
                                    publish_result.error
                                        .map(|e| e.to_string())
                                        .unwrap_or_else(|| "Unknown high-performance publish error".to_string())
                                ))
                            }
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    // High-performance not available, fall back
                    self.legacy_publisher.publish_message_event(kind, message, chat_members).await
                }
            }
            PublisherBackend::Legacy => {
                self.legacy_publisher.publish_message_event(kind, message, chat_members).await
            }
        };
        
        // Record operation for health monitoring
        let latency_ms = start.elapsed().as_millis() as f64;
        let success = result.is_ok();
        
        self.health_state.record_operation(success, latency_ms, 1000).await;
        
        if let Err(ref e) = result {
            warn!("Publish failed on {:?} backend: {}", backend, e);
        }
        
        result
    }
    
    /// Publish chat member joined event
    #[instrument(skip(self, chat_id, user_id))]
    pub async fn publish_chat_member_joined(
        &self,
        chat_id: &ChatId,
        user_id: &UserId,
    ) -> Result<(), AppError> {
        let start = Instant::now();
        let backend = self.current_backend().await;
        
        let result = match backend {
            PublisherBackend::HighPerformance => {
                if let Some(ref hp_publisher) = self.high_performance_publisher {
                    let event = FastChatMemberEvent {
                        chat_id: chat_id.0 as u64,
                        user_id: user_id.0 as u64,
                        event_type: "joined".to_string(),
                        timestamp: chrono::Utc::now().timestamp() as u64,
                        workspace_id: None,
                    };
                    
                    match hp_publisher.publish(event).await {
                        Ok(publish_result) => {
                            if publish_result.success {
                                Ok(())
                            } else {
                                Err(AppError::EventPublishError(
                                    publish_result.error
                                        .map(|e| e.to_string())
                                        .unwrap_or_else(|| "Unknown high-performance publish error".to_string())
                                ))
                            }
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    self.legacy_publisher.publish_chat_member_joined(chat_id, user_id).await
                }
            }
            PublisherBackend::Legacy => {
                self.legacy_publisher.publish_chat_member_joined(chat_id, user_id).await
            }
        };
        
        // Record operation for health monitoring
        let latency_ms = start.elapsed().as_millis() as f64;
        let success = result.is_ok();
        
        self.health_state.record_operation(success, latency_ms, 1000).await;
        
        result
    }
    
    /// Publish chat member left event
    #[instrument(skip(self, chat_id, user_id))]
    pub async fn publish_chat_member_left(
        &self,
        chat_id: &ChatId,
        user_id: &UserId,
    ) -> Result<(), AppError> {
        let start = Instant::now();
        let backend = self.current_backend().await;
        
        let result = match backend {
            PublisherBackend::HighPerformance => {
                if let Some(ref hp_publisher) = self.high_performance_publisher {
                    let event = FastChatMemberEvent {
                        chat_id: chat_id.0 as u64,
                        user_id: user_id.0 as u64,
                        event_type: "left".to_string(),
                        timestamp: chrono::Utc::now().timestamp() as u64,
                        workspace_id: None,
                    };
                    
                    match hp_publisher.publish(event).await {
                        Ok(publish_result) => {
                            if publish_result.success {
                                Ok(())
                            } else {
                                Err(AppError::EventPublishError(
                                    publish_result.error
                                        .map(|e| e.to_string())
                                        .unwrap_or_else(|| "Unknown high-performance publish error".to_string())
                                ))
                            }
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    self.legacy_publisher.publish_chat_member_left(chat_id, user_id).await
                }
            }
            PublisherBackend::Legacy => {
                self.legacy_publisher.publish_chat_member_left(chat_id, user_id).await
            }
        };
        
        // Record operation for health monitoring
        let latency_ms = start.elapsed().as_millis() as f64;
        let success = result.is_ok();
        
        self.health_state.record_operation(success, latency_ms, 1000).await;
        
        result
    }
    
    /// Get health status and metrics
    pub async fn health_status(&self) -> AdaptiveHealthStatus {
        let backend = self.current_backend().await;
        let (success_rate, avg_latency, sample_size) = self.health_state
            .calculate_metrics(self.config.degradation_thresholds.error_window_duration)
            .await;
        
        let consecutive_failures = self.health_state.consecutive_failures.load(Ordering::Relaxed);
        let consecutive_successes = self.health_state.consecutive_successes.load(Ordering::Relaxed);
        let degradation_count = self.health_state.degradation_count.load(Ordering::Relaxed);
        let manual_override = self.health_state.manual_override.load(Ordering::Relaxed);
        
        AdaptiveHealthStatus {
            current_backend: backend,
            success_rate,
            avg_latency_ms: avg_latency,
            sample_size,
            consecutive_failures,
            consecutive_successes,
            total_degradations: degradation_count,
            manual_override,
            is_healthy: success_rate >= (1.0 - self.config.degradation_thresholds.max_error_rate)
                && avg_latency <= self.config.degradation_thresholds.max_latency_ms,
        }
    }
}

// =====================================================================================
// HEALTH STATUS TYPES
// =====================================================================================

#[derive(Debug, Clone, Serialize)]
pub struct AdaptiveHealthStatus {
    pub current_backend: PublisherBackend,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub sample_size: usize,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
    pub total_degradations: u64,
    pub manual_override: bool,
    pub is_healthy: bool,
}

use std::sync::atomic::AtomicU32;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_degradation_reason_criticality() {
        assert!(DegradationReason::CircuitBreakerOpen.is_critical());
        assert!(DegradationReason::ConnectionFailure { error: "test".to_string() }.is_critical());
        assert!(!DegradationReason::HighLatency { avg_latency_ms: 100.0, threshold_ms: 50.0 }.is_critical());
    }
    
    #[test]
    fn test_publisher_backend_string_conversion() {
        assert_eq!(PublisherBackend::HighPerformance.as_str(), "high_performance");
        assert_eq!(PublisherBackend::Legacy.as_str(), "legacy");
    }
    
    #[test]
    fn test_adaptive_config_defaults() {
        let config = AdaptivePublisherConfig::default();
        assert_eq!(config.preferred_backend, PublisherBackend::HighPerformance);
        assert!(config.enable_auto_degradation);
        assert!(config.degradation_thresholds.max_error_rate > 0.0);
        assert!(config.recovery_thresholds.min_success_rate > 0.0);
    }
}