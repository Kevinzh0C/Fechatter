// Health Monitor for Adaptive Publisher
//
// This module provides comprehensive health monitoring and automatic degradation
// detection for the event publishing system.

use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use super::adaptive_publisher::{DegradationReason, PublisherBackend};

// =====================================================================================
// HEALTH MONITORING CONFIGURATION
// =====================================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Health check interval
    pub check_interval: Duration,
    
    /// Time window for metrics calculation
    pub metrics_window: Duration,
    
    /// Maximum number of samples to keep
    pub max_samples: usize,
    
    /// Enable detailed logging
    pub enable_detailed_logging: bool,
    
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Error rate threshold for warnings (0.0-1.0)
    pub warning_error_rate: f64,
    
    /// Error rate threshold for critical alerts (0.0-1.0)
    pub critical_error_rate: f64,
    
    /// Latency threshold for warnings (ms)
    pub warning_latency_ms: f64,
    
    /// Latency threshold for critical alerts (ms)
    pub critical_latency_ms: f64,
    
    /// Queue size threshold for warnings
    pub warning_queue_size: usize,
    
    /// Queue size threshold for critical alerts
    pub critical_queue_size: usize,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(5),
            metrics_window: Duration::from_secs(60),
            max_samples: 1000,
            enable_detailed_logging: false,
            alert_thresholds: AlertThresholds {
                warning_error_rate: 0.01, // 1%
                critical_error_rate: 0.05, // 5%
                warning_latency_ms: 100.0,
                critical_latency_ms: 500.0,
                warning_queue_size: 20_000,
                critical_queue_size: 40_000,
            },
        }
    }
}

// =====================================================================================
// HEALTH METRICS AND STATUS
// =====================================================================================

#[derive(Debug, Clone, Serialize)]
pub struct HealthMetrics {
    /// Success rate over the monitoring window (0.0-1.0)
    pub success_rate: f64,
    
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    
    /// 95th percentile latency
    pub p95_latency_ms: f64,
    
    /// 99th percentile latency
    pub p99_latency_ms: f64,
    
    /// Current queue size (if available)
    pub queue_size: Option<usize>,
    
    /// Number of samples in the current window
    pub sample_count: usize,
    
    /// Operations per second
    pub ops_per_second: f64,
    
    /// Total operations processed
    pub total_operations: u64,
    
    /// Total errors encountered
    pub total_errors: u64,
    
    /// Consecutive failures
    pub consecutive_failures: u32,
    
    /// Consecutive successes
    pub consecutive_successes: u32,
    
    /// Time since last error
    pub time_since_last_error: Option<Duration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

impl HealthStatus {
    pub fn is_degraded(&self) -> bool {
        matches!(self, Self::Critical)
    }
    
    pub fn is_warning(&self) -> bool {
        matches!(self, Self::Warning | Self::Critical)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DegradationTrigger {
    pub reason: DegradationReason,
    pub triggered_at: i64, // Use timestamp instead of Instant for serialization
    pub metrics_snapshot: HealthMetrics,
    pub recommended_action: RecommendedAction,
}

#[derive(Debug, Clone, Serialize)]
pub enum RecommendedAction {
    SwitchToLegacy,
    RestartHighPerformance,
    IncreaseResources,
    CheckNetworkConnectivity,
    ManualIntervention,
}

#[derive(Debug, Clone)]
pub struct DegradationEvent {
    pub reason: DegradationReason,
    pub from_backend: PublisherBackend,
    pub to_backend: PublisherBackend,
    pub triggered_at: i64, // Use timestamp instead of Instant
}

// =====================================================================================
// HEALTH MONITOR IMPLEMENTATION
// =====================================================================================

pub struct HealthMonitor {
    /// Configuration
    config: MonitoringConfig,
    
    /// Health samples (success, latency_ms, timestamp)
    samples: Mutex<Vec<(bool, f64, Instant)>>,
    
    /// Current health status
    current_status: RwLock<HealthStatus>,
    
    /// Last health check time
    last_check: RwLock<Option<Instant>>,
    
    /// Degradation triggers history
    degradation_history: Mutex<Vec<DegradationTrigger>>,
    
    /// Monitoring statistics
    total_operations: AtomicU64,
    total_errors: AtomicU64,
    consecutive_failures: AtomicU64,
    consecutive_successes: AtomicU64,
    
    /// Last error timestamp
    last_error_time: RwLock<Option<Instant>>,
    
    /// Monitoring enabled flag
    enabled: AtomicBool,
}

impl HealthMonitor {
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            config,
            samples: Mutex::new(Vec::new()),
            current_status: RwLock::new(HealthStatus::Unknown),
            last_check: RwLock::new(None),
            degradation_history: Mutex::new(Vec::new()),
            total_operations: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            consecutive_failures: AtomicU64::new(0),
            consecutive_successes: AtomicU64::new(0),
            last_error_time: RwLock::new(None),
            enabled: AtomicBool::new(true),
        }
    }
    
    /// Record an operation result
    pub async fn record_operation(&self, success: bool, latency_ms: f64) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }
        
        let now = Instant::now();
        
        // Update samples
        let mut samples = self.samples.lock().await;
        samples.push((success, latency_ms, now));
        
        // Keep only recent samples
        let current_len = samples.len();
        if current_len > self.config.max_samples {
            let excess_count = current_len - self.config.max_samples;
            samples.drain(0..excess_count);
        }
        
        // Update counters
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        
        if success {
            self.consecutive_failures.store(0, Ordering::Relaxed);
            self.consecutive_successes.fetch_add(1, Ordering::Relaxed);
        } else {
            self.total_errors.fetch_add(1, Ordering::Relaxed);
            self.consecutive_successes.store(0, Ordering::Relaxed);
            self.consecutive_failures.fetch_add(1, Ordering::Relaxed);
            *self.last_error_time.write().await = Some(now);
        }
        
        if self.config.enable_detailed_logging {
            debug!("Health monitor recorded operation: success={}, latency={}ms", success, latency_ms);
        }
    }
    
    /// Record queue size (for high-performance publisher)
    pub async fn record_queue_size(&self, size: usize) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }
        
        // Check for queue overflow alerts
        if size >= self.config.alert_thresholds.critical_queue_size {
            warn!("Critical queue size detected: {} (threshold: {})", 
                  size, self.config.alert_thresholds.critical_queue_size);
        } else if size >= self.config.alert_thresholds.warning_queue_size {
            warn!("Warning queue size detected: {} (threshold: {})", 
                  size, self.config.alert_thresholds.warning_queue_size);
        }
    }
    
    /// Calculate current health metrics
    pub async fn calculate_metrics(&self) -> HealthMetrics {
        let samples = self.samples.lock().await;
        let cutoff = Instant::now() - self.config.metrics_window;
        
        let recent_samples: Vec<_> = samples.iter()
            .filter(|(_, _, timestamp)| *timestamp > cutoff)
            .collect();
        
        if recent_samples.is_empty() {
            return HealthMetrics {
                success_rate: 1.0, // Assume healthy if no data
                avg_latency_ms: 0.0,
                p95_latency_ms: 0.0,
                p99_latency_ms: 0.0,
                queue_size: None,
                sample_count: 0,
                ops_per_second: 0.0,
                total_operations: self.total_operations.load(Ordering::Relaxed),
                total_errors: self.total_errors.load(Ordering::Relaxed),
                consecutive_failures: self.consecutive_failures.load(Ordering::Relaxed) as u32,
                consecutive_successes: self.consecutive_successes.load(Ordering::Relaxed) as u32,
                time_since_last_error: self.last_error_time.read().await.map(|t| t.elapsed()),
            };
        }
        
        // Calculate success rate
        let success_count = recent_samples.iter().filter(|(success, _, _)| *success).count();
        let success_rate = success_count as f64 / recent_samples.len() as f64;
        
        // Calculate latency metrics
        let mut latencies: Vec<f64> = recent_samples.iter().map(|(_, latency, _)| *latency).collect();
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let avg_latency_ms = latencies.iter().sum::<f64>() / latencies.len() as f64;
        let p95_latency_ms = percentile(&latencies, 0.95);
        let p99_latency_ms = percentile(&latencies, 0.99);
        
        // Calculate operations per second
        let window_seconds = self.config.metrics_window.as_secs_f64();
        let ops_per_second = recent_samples.len() as f64 / window_seconds;
        
        HealthMetrics {
            success_rate,
            avg_latency_ms,
            p95_latency_ms,
            p99_latency_ms,
            queue_size: None, // Will be updated externally
            sample_count: recent_samples.len(),
            ops_per_second,
            total_operations: self.total_operations.load(Ordering::Relaxed),
            total_errors: self.total_errors.load(Ordering::Relaxed),
            consecutive_failures: self.consecutive_failures.load(Ordering::Relaxed) as u32,
            consecutive_successes: self.consecutive_successes.load(Ordering::Relaxed) as u32,
            time_since_last_error: self.last_error_time.read().await.map(|t| t.elapsed()),
        }
    }
    
    /// Evaluate current health status
    pub async fn evaluate_health(&self) -> HealthStatus {
        let metrics = self.calculate_metrics().await;
        let thresholds = &self.config.alert_thresholds;
        
        // Check for critical conditions
        if metrics.success_rate < (1.0 - thresholds.critical_error_rate) {
            return HealthStatus::Critical;
        }
        
        if metrics.avg_latency_ms > thresholds.critical_latency_ms {
            return HealthStatus::Critical;
        }
        
        if let Some(queue_size) = metrics.queue_size {
            if queue_size >= thresholds.critical_queue_size {
                return HealthStatus::Critical;
            }
        }
        
        // Check for warning conditions
        if metrics.success_rate < (1.0 - thresholds.warning_error_rate) {
            return HealthStatus::Warning;
        }
        
        if metrics.avg_latency_ms > thresholds.warning_latency_ms {
            return HealthStatus::Warning;
        }
        
        if let Some(queue_size) = metrics.queue_size {
            if queue_size >= thresholds.warning_queue_size {
                return HealthStatus::Warning;
            }
        }
        
        HealthStatus::Healthy
    }
    
    /// Check if degradation should be triggered
    pub async fn check_degradation_triggers(&self) -> Option<DegradationTrigger> {
        let metrics = self.calculate_metrics().await;
        let status = self.evaluate_health().await;
        
        if status.is_degraded() {
            let reason = self.determine_degradation_reason(&metrics).await;
            let recommended_action = self.recommend_action(&reason);
            
            let trigger = DegradationTrigger {
                reason,
                triggered_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
                metrics_snapshot: metrics,
                recommended_action,
            };
            
            // Record trigger
            self.degradation_history.lock().await.push(trigger.clone());
            
            return Some(trigger);
        }
        
        None
    }
    
    /// Determine the primary reason for degradation
    async fn determine_degradation_reason(&self, metrics: &HealthMetrics) -> DegradationReason {
        let thresholds = &self.config.alert_thresholds;
        
        // Check error rate first (most critical)
        if metrics.success_rate < (1.0 - thresholds.critical_error_rate) {
            return DegradationReason::HighErrorRate {
                error_rate: 1.0 - metrics.success_rate,
                threshold: thresholds.critical_error_rate,
            };
        }
        
        // Check latency
        if metrics.avg_latency_ms > thresholds.critical_latency_ms {
            return DegradationReason::HighLatency {
                avg_latency_ms: metrics.avg_latency_ms,
                threshold_ms: thresholds.critical_latency_ms,
            };
        }
        
        // Check queue overflow
        if let Some(queue_size) = metrics.queue_size {
            if queue_size >= thresholds.critical_queue_size {
                return DegradationReason::QueueOverflow {
                    queue_size,
                    threshold: thresholds.critical_queue_size,
                };
            }
        }
        
        // Check consecutive failures
        if metrics.consecutive_failures >= 5 {
            return DegradationReason::HighErrorRate {
                error_rate: 1.0,
                threshold: 0.0,
            };
        }
        
        // Default fallback
        DegradationReason::ManualOverride
    }
    
    /// Recommend action based on degradation reason
    fn recommend_action(&self, reason: &DegradationReason) -> RecommendedAction {
        match reason {
            DegradationReason::HighErrorRate { .. } => RecommendedAction::SwitchToLegacy,
            DegradationReason::HighLatency { .. } => RecommendedAction::SwitchToLegacy,
            DegradationReason::QueueOverflow { .. } => RecommendedAction::IncreaseResources,
            DegradationReason::CircuitBreakerOpen => RecommendedAction::SwitchToLegacy,
            DegradationReason::ConnectionFailure { .. } => RecommendedAction::CheckNetworkConnectivity,
            DegradationReason::StartupFailure { .. } => RecommendedAction::RestartHighPerformance,
            DegradationReason::ManualOverride => RecommendedAction::ManualIntervention,
        }
    }
    
    /// Update current health status
    pub async fn update_status(&self) {
        let new_status = self.evaluate_health().await;
        let mut current_status = self.current_status.write().await;
        
        if *current_status != new_status {
            info!("Health status changed: {:?} -> {:?}", *current_status, new_status);
            *current_status = new_status;
        }
        
        *self.last_check.write().await = Some(Instant::now());
    }
    
    /// Get current health status
    pub async fn current_status(&self) -> HealthStatus {
        *self.current_status.read().await
    }
    
    /// Get degradation history
    pub async fn degradation_history(&self) -> Vec<DegradationTrigger> {
        self.degradation_history.lock().await.clone()
    }
    
    /// Reset monitoring state
    pub async fn reset(&self) {
        info!("Resetting health monitor state");
        
        self.samples.lock().await.clear();
        *self.current_status.write().await = HealthStatus::Unknown;
        *self.last_check.write().await = None;
        self.degradation_history.lock().await.clear();
        
        self.total_operations.store(0, Ordering::Relaxed);
        self.total_errors.store(0, Ordering::Relaxed);
        self.consecutive_failures.store(0, Ordering::Relaxed);
        self.consecutive_successes.store(0, Ordering::Relaxed);
        *self.last_error_time.write().await = None;
    }
    
    /// Enable/disable monitoring
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
        if enabled {
            info!("Health monitoring enabled");
        } else {
            info!("Health monitoring disabled");
        }
    }
    
    /// Check if monitoring is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
}

// =====================================================================================
// UTILITY FUNCTIONS
// =====================================================================================

/// Calculate percentile of a sorted slice
fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    
    if p <= 0.0 {
        return sorted_values[0];
    }
    
    if p >= 1.0 {
        return sorted_values[sorted_values.len() - 1];
    }
    
    let index = (sorted_values.len() - 1) as f64 * p;
    let lower = index.floor() as usize;
    let upper = index.ceil() as usize;
    
    if lower == upper {
        sorted_values[lower]
    } else {
        let weight = index - lower as f64;
        sorted_values[lower] * (1.0 - weight) + sorted_values[upper] * weight
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_percentile_calculation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        assert_eq!(percentile(&values, 0.0), 1.0);
        assert_eq!(percentile(&values, 0.5), 3.0);
        assert_eq!(percentile(&values, 1.0), 5.0);
        
        // Test empty slice
        assert_eq!(percentile(&[], 0.5), 0.0);
    }
    
    #[test]
    fn test_health_status_methods() {
        assert!(HealthStatus::Critical.is_degraded());
        assert!(!HealthStatus::Healthy.is_degraded());
        
        assert!(HealthStatus::Warning.is_warning());
        assert!(HealthStatus::Critical.is_warning());
        assert!(!HealthStatus::Healthy.is_warning());
    }
    
    #[tokio::test]
    async fn test_health_monitor_basic_operations() {
        let config = MonitoringConfig::default();
        let monitor = HealthMonitor::new(config);
        
        // Initially unknown
        assert_eq!(monitor.current_status().await, HealthStatus::Unknown);
        
        // Record some successful operations
        for _ in 0..10 {
            monitor.record_operation(true, 50.0).await;
        }
        
        let metrics = monitor.calculate_metrics().await;
        assert_eq!(metrics.success_rate, 1.0);
        assert_eq!(metrics.avg_latency_ms, 50.0);
        assert_eq!(metrics.sample_count, 10);
    }
    
    #[tokio::test]
    async fn test_degradation_detection() {
        let mut config = MonitoringConfig::default();
        config.alert_thresholds.critical_error_rate = 0.2; // 20%
        
        let monitor = HealthMonitor::new(config);
        
        // Record mostly failed operations
        for _ in 0..8 {
            monitor.record_operation(false, 1000.0).await;
        }
        for _ in 0..2 {
            monitor.record_operation(true, 100.0).await;
        }
        
        let status = monitor.evaluate_health().await;
        assert_eq!(status, HealthStatus::Critical);
        
        let trigger = monitor.check_degradation_triggers().await;
        assert!(trigger.is_some());
    }
}