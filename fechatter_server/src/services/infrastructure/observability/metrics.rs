use crate::AppError;
use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::time::Duration;
use tracing::info;

pub struct MetricsGuard;

/// Initialize Prometheus metrics
pub async fn init_metrics(
  config: &crate::config::ObservabilityConfig,
) -> Result<MetricsGuard, AppError> {
  let builder = PrometheusBuilder::new();

  // Register all application metrics first
  register_application_metrics();

  // Use metrics_bind_address from config
  let bind_address = &config.metrics_bind_address;

  // Set up Prometheus endpoint
  builder
    .with_http_listener(bind_address.parse::<std::net::SocketAddr>().map_err(|e| {
      AppError::Internal(format!(
        "Invalid metrics bind address '{}': {}",
        bind_address, e
      ))
    })?)
    .install()
    .map_err(|e| AppError::Internal(format!("Failed to install Prometheus exporter: {}", e)))?;

  tracing::info!("Prometheus metrics server started on {}", bind_address);

  Ok(MetricsGuard)
}

/// Register all application-specific metrics
fn register_application_metrics() {
  // Note: In modern metrics crate, metrics are registered automatically when first used
  // We'll initialize some basic metrics to ensure they exist

  // Request metrics
  counter!("fechatter_http_requests_total", "method" => "GET", "route" => "/health", "status" => "200").absolute(0);
  histogram!("fechatter_http_request_duration_seconds", "method" => "GET", "route" => "/health")
    .record(0.0);
  counter!("fechatter_http_errors_total", "method" => "GET", "route" => "/health", "status" => "500").absolute(0);

  // Database metrics
  histogram!("fechatter_db_query_duration_seconds", "operation" => "select", "table" => "users")
    .record(0.0);
  counter!("fechatter_db_queries_total", "operation" => "select", "table" => "users").absolute(0);
  counter!("fechatter_db_errors_total", "operation" => "select", "table" => "users").absolute(0);
  gauge!("fechatter_db_connections_active").set(0.0);

  // Cache metrics
  counter!("fechatter_cache_operations_total", "operation" => "get").absolute(0);
  counter!("fechatter_cache_hits_total", "operation" => "get").absolute(0);
  counter!("fechatter_cache_misses_total", "operation" => "get").absolute(0);
  histogram!("fechatter_cache_operation_duration_seconds", "operation" => "get").record(0.0);

  // Chat metrics
  counter!("fechatter_chats_created_total", "type" => "private").absolute(0);
  counter!("fechatter_messages_sent_total", "chat_type" => "private", "has_files" => "false")
    .absolute(0);
  gauge!("fechatter_active_users").set(0.0);
  gauge!("fechatter_active_chats").set(0.0);

  // WebSocket metrics
  gauge!("fechatter_websocket_connections").set(0.0);
  counter!("fechatter_websocket_messages_total", "type" => "text").absolute(0);

  // File upload metrics
  counter!("fechatter_files_uploaded_total", "type" => "image", "success" => "true").absolute(0);
  histogram!("fechatter_file_upload_duration_seconds", "type" => "image").record(0.0);
  histogram!("fechatter_file_size_bytes", "type" => "image").record(0.0);

  info!("Application metrics registered");
}

/// Metrics collection utilities
pub mod collectors {
  use super::*;

  /// HTTP request metrics collector
  pub struct HttpMetrics;

  impl HttpMetrics {
    pub fn record_request(method: &str, route: &str, status: u16, duration: Duration) {
      counter!("fechatter_http_requests_total", 
                "method" => method.to_string(), 
                "route" => route.to_string(), 
                "status" => status.to_string())
      .increment(1);

      histogram!("fechatter_http_request_duration_seconds",
                "method" => method.to_string(),
                "route" => route.to_string())
      .record(duration.as_secs_f64());

      if status >= 400 {
        counter!("fechatter_http_errors_total",
                    "method" => method.to_string(),
                    "route" => route.to_string(),
                    "status" => status.to_string())
        .increment(1);
      }
    }
  }

  /// Database metrics collector
  pub struct DatabaseMetrics;

  impl DatabaseMetrics {
    pub fn record_query(operation: &str, table: &str, duration: Duration, success: bool) {
      counter!("fechatter_db_queries_total",
                "operation" => operation.to_string(),
                "table" => table.to_string())
      .increment(1);

      histogram!("fechatter_db_query_duration_seconds",
                "operation" => operation.to_string(),
                "table" => table.to_string())
      .record(duration.as_secs_f64());

      if !success {
        counter!("fechatter_db_errors_total",
                    "operation" => operation.to_string(),
                    "table" => table.to_string())
        .increment(1);
      }
    }

    pub fn set_active_connections(count: i64) {
      gauge!("fechatter_db_connections_active").set(count as f64);
    }
  }

  /// Cache metrics collector
  pub struct CacheMetrics;

  impl CacheMetrics {
    pub fn record_operation(operation: &str, hit: bool, duration: Duration) {
      counter!("fechatter_cache_operations_total",
                "operation" => operation.to_string())
      .increment(1);

      if hit {
        counter!("fechatter_cache_hits_total",
                    "operation" => operation.to_string())
        .increment(1);
      } else {
        counter!("fechatter_cache_misses_total",
                    "operation" => operation.to_string())
        .increment(1);
      }

      histogram!("fechatter_cache_operation_duration_seconds",
                "operation" => operation.to_string())
      .record(duration.as_secs_f64());
    }
  }

  /// Chat metrics collector
  pub struct ChatMetrics;

  impl ChatMetrics {
    pub fn record_chat_created(chat_type: &str) {
      counter!("fechatter_chats_created_total",
                "type" => chat_type.to_string())
      .increment(1);
    }

    pub fn record_message_sent(chat_type: &str, has_files: bool) {
      counter!("fechatter_messages_sent_total",
                "chat_type" => chat_type.to_string(),
                "has_files" => has_files.to_string())
      .increment(1);
    }

    pub fn set_active_users(count: i64) {
      gauge!("fechatter_active_users").set(count as f64);
    }

    pub fn set_active_chats(count: i64) {
      gauge!("fechatter_active_chats").set(count as f64);
    }
  }

  /// WebSocket metrics collector
  pub struct WebSocketMetrics;

  impl WebSocketMetrics {
    pub fn connection_opened() {
      gauge!("fechatter_websocket_connections").increment(1.0);
    }

    pub fn connection_closed() {
      gauge!("fechatter_websocket_connections").decrement(1.0);
    }

    pub fn message_sent(message_type: &str) {
      counter!("fechatter_websocket_messages_total",
                "type" => message_type.to_string())
      .increment(1);
    }
  }

  /// File upload metrics collector
  pub struct FileMetrics;

  impl FileMetrics {
    pub fn record_upload(file_type: &str, size_bytes: u64, duration: Duration, success: bool) {
      counter!("fechatter_files_uploaded_total",
                "type" => file_type.to_string(),
                "success" => success.to_string())
      .increment(1);

      if success {
        histogram!("fechatter_file_upload_duration_seconds",
                    "type" => file_type.to_string())
        .record(duration.as_secs_f64());

        histogram!("fechatter_file_size_bytes",
                    "type" => file_type.to_string())
        .record(size_bytes as f64);
      }
    }
  }
}
