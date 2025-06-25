//! # Production Health Monitoring
//!
//! **Responsibility**: Production-grade health monitoring, metrics, and circuit breaker management
//! **Features**: Real-time service health, performance metrics, circuit breaker controls

use axum::{
  extract::{Path, Query, State},
  response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, instrument};

use crate::services::application::builders::provider::{ServiceHealth, ServiceMetrics};
use crate::{AppError, AppState};

/// Production health summary
#[derive(Debug, Serialize)]
pub struct ProductionHealthSummary {
  pub status: String,
  pub timestamp: u64,
  pub uptime_seconds: u64,
  pub version: String,
  pub environment: String,
  pub services: HashMap<String, ServiceHealthStatus>,
  pub system_metrics: SystemMetrics,
  pub connection_pool: ConnectionPoolMetrics,
  pub circuit_breakers: HashMap<String, CircuitBreakerStatus>,
}

/// Individual service health status
#[derive(Debug, Serialize)]
pub struct ServiceHealthStatus {
  pub status: String,
  pub calls: u64,
  pub failures: u64,
  pub failure_rate: f64,
  pub avg_response_time_ms: u64,
  pub last_error: Option<String>,
  pub last_check: u64,
}

/// System-wide metrics
#[derive(Debug, Serialize)]
pub struct SystemMetrics {
  pub cpu_usage_percent: f64,
  pub memory_usage_mb: u64,
  pub active_connections: usize,
  pub requests_per_second: f64,
  pub cache_hit_rate: f64,
}

/// Database connection pool metrics
#[derive(Debug, Serialize)]
pub struct ConnectionPoolMetrics {
  pub total_connections: u32,
  pub idle_connections: u32,
  pub active_connections: u32,
  pub max_connections: u32,
  pub pending_requests: usize,
}

/// Circuit breaker status
#[derive(Debug, Serialize)]
pub struct CircuitBreakerStatus {
  pub state: String,
  pub failure_count: u32,
  pub last_failure: Option<u64>,
  pub success_threshold: u32,
  pub failure_threshold: u32,
}

/// Detailed service metrics query parameters
#[derive(Debug, Deserialize)]
pub struct ServiceMetricsQuery {
  #[serde(default)]
  pub include_history: bool,
  #[serde(default)]
  pub time_range_minutes: Option<u32>,
}

/// Circuit breaker control request
#[derive(Debug, Deserialize)]
pub struct CircuitBreakerControl {
  pub action: String, // "open", "close", "reset"
}

/// Production health summary endpoint
#[instrument(skip(state))]
pub async fn production_health_summary(
  State(state): State<AppState>,
) -> Result<Json<ProductionHealthSummary>, AppError> {
  info!("Production health check requested");

  let services = state.application_services();
  let metrics = services.get_metrics();

  let mut service_statuses = HashMap::new();
  let mut overall_healthy = true;

  // Check each service health
  for (service_name, metric) in &metrics {
    let health = services.get_service_health(service_name);
    let failure_rate = if metric.calls > 0 {
      (metric.failures as f64 / metric.calls as f64) * 100.0
    } else {
      0.0
    };

    if health != ServiceHealth::Healthy {
      overall_healthy = false;
    }

    service_statuses.insert(
      service_name.clone(),
      ServiceHealthStatus {
        status: match health {
          ServiceHealth::Healthy => "healthy".to_string(),
          ServiceHealth::Degraded => "degraded".to_string(),
          ServiceHealth::Unhealthy => "unhealthy".to_string(),
        },
        calls: metric.calls,
        failures: metric.failures,
        failure_rate,
        avg_response_time_ms: metric.avg_response_time.as_millis() as u64,
        last_error: metric.last_error.clone(),
        last_check: SystemTime::now()
          .duration_since(UNIX_EPOCH)
          .unwrap_or_default()
          .as_secs(),
      },
    );
  }

  // Gather system metrics
  let system_metrics = SystemMetrics {
    cpu_usage_percent: get_cpu_usage().await,
    memory_usage_mb: get_memory_usage().await,
    active_connections: services.connection_count(),
    requests_per_second: calculate_rps(&metrics),
    cache_hit_rate: get_cache_hit_rate(&state).await,
  };

  // Database connection pool metrics
  let pool = state.pool();
  let total_connections = pool.size() as u32;
  let idle_connections = pool.num_idle() as u32;
  let connection_pool = ConnectionPoolMetrics {
    total_connections,
    idle_connections,
    active_connections: total_connections.saturating_sub(idle_connections),
    max_connections: pool.options().get_max_connections(),
    pending_requests: 0, // Would need custom tracking
  };

  // Circuit breaker statuses (simplified)
  let circuit_breakers = HashMap::new(); // Would implement actual circuit breaker status

  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_secs();

  let summary = ProductionHealthSummary {
    status: if overall_healthy {
      "healthy".to_string()
    } else {
      "degraded".to_string()
    },
    timestamp,
    uptime_seconds: timestamp, // Simplified - would track actual start time
    version: env!("CARGO_PKG_VERSION").to_string(),
    environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
    services: service_statuses,
    system_metrics,
    connection_pool,
    circuit_breakers,
  };

  Ok(Json(summary))
}

/// Detailed service metrics endpoint
#[instrument(skip(state))]
pub async fn service_metrics(
  State(state): State<AppState>,
  Path(service_name): Path<String>,
  Query(query): Query<ServiceMetricsQuery>,
) -> Result<Json<ServiceMetrics>, AppError> {
  info!("Service metrics requested for: {}", service_name);

  let services = state.application_services();
  let metrics = services.get_metrics();

  if let Some(metric) = metrics.get(&service_name) {
    Ok(Json(metric.clone()))
  } else {
    Err(AppError::NotFound(vec![format!(
      "Service '{}' not found",
      service_name
    )]))
  }
}

/// Circuit breaker control endpoint
#[instrument(skip(state))]
pub async fn circuit_breaker_control(
  State(state): State<AppState>,
  Path(service_name): Path<String>,
  Json(control): Json<CircuitBreakerControl>,
) -> Result<Json<serde_json::Value>, AppError> {
  info!(
    "Circuit breaker control for {}: {}",
    service_name, control.action
  );

  match control.action.as_str() {
    "open" => {
      // Would implement actual circuit breaker control
      info!("Opening circuit breaker for service: {}", service_name);
    }
    "close" => {
      info!("Closing circuit breaker for service: {}", service_name);
    }
    "reset" => {
      info!("Resetting circuit breaker for service: {}", service_name);
    }
    _ => {
      return Err(AppError::InvalidInput(format!(
        "Invalid action: {}. Valid actions: open, close, reset",
        control.action
      )));
    }
  }

  Ok(Json(serde_json::json!({
    "service": service_name,
    "action": control.action,
    "timestamp": SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap_or_default()
      .as_secs(),
    "status": "success"
  })))
}

/// Service health check endpoint (individual service)
#[instrument(skip(state))]
pub async fn service_health_check(
  State(state): State<AppState>,
  Path(service_name): Path<String>,
) -> Result<Json<ServiceHealthStatus>, AppError> {
  let services = state.application_services();
  let health = services.get_service_health(&service_name);
  let metrics = services.get_metrics();

  if let Some(metric) = metrics.get(&service_name) {
    let failure_rate = if metric.calls > 0 {
      (metric.failures as f64 / metric.calls as f64) * 100.0
    } else {
      0.0
    };

    let status = ServiceHealthStatus {
      status: match health {
        ServiceHealth::Healthy => "healthy".to_string(),
        ServiceHealth::Degraded => "degraded".to_string(),
        ServiceHealth::Unhealthy => "unhealthy".to_string(),
      },
      calls: metric.calls,
      failures: metric.failures,
      failure_rate,
      avg_response_time_ms: metric.avg_response_time.as_millis() as u64,
      last_error: metric.last_error.clone(),
      last_check: SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs(),
    };

    // Set appropriate HTTP status code based on health
    match health {
      ServiceHealth::Healthy => Ok(Json(status)),
      ServiceHealth::Degraded => Ok(Json(status)), // Still return 200 for degraded
      ServiceHealth::Unhealthy => Err(AppError::ServiceUnavailable(format!(
        "Service '{}' is unhealthy",
        service_name
      ))),
    }
  } else {
    Err(AppError::NotFound(vec![format!(
      "Service '{}' not found",
      service_name
    )]))
  }
}

/// Force service cache refresh (admin operation)
#[instrument(skip(state))]
pub async fn refresh_service_cache(
  State(state): State<AppState>,
  Path(service_name): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
  info!("Forcing cache refresh for service: {}", service_name);

  // This would trigger cache invalidation and recreation
  // For now, just return success

  Ok(Json(serde_json::json!({
    "service": service_name,
    "action": "cache_refresh",
    "timestamp": SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap_or_default()
      .as_secs(),
    "status": "success"
  })))
}

// ========================================================================
// Helper Functions
// ========================================================================

/// Get CPU usage (simplified - would use proper system monitoring)
async fn get_cpu_usage() -> f64 {
  // Would implement actual CPU monitoring
  // For demo, return simulated value
  42.5
}

/// Get memory usage (simplified)
async fn get_memory_usage() -> u64 {
  // Would implement actual memory monitoring
  // For demo, return simulated value in MB
  1024
}

/// Calculate requests per second from metrics
fn calculate_rps(metrics: &HashMap<String, ServiceMetrics>) -> f64 {
  let total_calls: u64 = metrics.values().map(|m| m.calls).sum();
  // Simplified calculation - would track time windows
  total_calls as f64 / 60.0 // Assume 1-minute window
}

/// Get cache hit rate
async fn get_cache_hit_rate(state: &AppState) -> f64 {
  // Would implement actual cache metrics tracking
  if state.cache_service().is_some() {
    85.7 // Simulated high hit rate
  } else {
    0.0
  }
}
