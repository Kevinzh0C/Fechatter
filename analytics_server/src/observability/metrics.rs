use crate::error::AppError;
use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use std::time::Duration;
use tracing::info;

/// Initialize Prometheus metrics for analytics_server
pub async fn init_metrics(bind_address: SocketAddr) -> Result<(), AppError> {
    let builder = PrometheusBuilder::new();

    // Register all analytics_server metrics first
    register_analytics_metrics();

    builder
        .with_http_listener(bind_address)
        .install()
        .map_err(|e| AppError::AnyError(anyhow::anyhow!("Failed to install Prometheus exporter: {}", e)))?;

    info!("📊 [ANALYTICS] Prometheus metrics server started on {}", bind_address);
    Ok(())
}

/// Register all analytics_server specific metrics
fn register_analytics_metrics() {
    // HTTP API metrics
    counter!("analytics_http_requests_total", "method" => "POST", "endpoint" => "/api/event", "status" => "200").absolute(0);
    counter!("analytics_http_requests_total", "method" => "POST", "endpoint" => "/api/batch", "status" => "200").absolute(0);
    histogram!("analytics_http_request_duration_seconds", "method" => "POST", "endpoint" => "/api/event").record(0.0);
    histogram!("analytics_batch_size", "endpoint" => "/api/batch").record(0.0);

    // ClickHouse metrics
    counter!("analytics_clickhouse_operations_total", "operation" => "insert", "table" => "events", "status" => "success").absolute(0);
    counter!("analytics_clickhouse_operations_total", "operation" => "query", "table" => "events", "status" => "success").absolute(0);
    histogram!("analytics_clickhouse_operation_duration_seconds", "operation" => "insert", "table" => "events").record(0.0);
    gauge!("analytics_clickhouse_connections_active").set(0.0);
    counter!("analytics_clickhouse_connection_errors_total").absolute(0);

    // NATS subscription metrics
    counter!("analytics_nats_messages_received_total", "subject" => "chat.events", "type" => "message").absolute(0);
    counter!("analytics_nats_messages_processed_total", "subject" => "chat.events", "status" => "success").absolute(0);
    counter!("analytics_nats_messages_failed_total", "subject" => "chat.events", "error_type" => "parse_error").absolute(0);
    histogram!("analytics_nats_message_processing_duration_seconds", "subject" => "chat.events").record(0.0);
    gauge!("analytics_nats_subscription_active", "subject" => "chat.events").set(0.0);

    // Event processing metrics
    counter!("analytics_events_received_total", "event_type" => "message_sent", "source" => "http").absolute(0);
    counter!("analytics_events_stored_total", "event_type" => "message_sent", "status" => "success").absolute(0);
    counter!("analytics_events_failed_total", "event_type" => "message_sent", "error_type" => "validation_error").absolute(0);
    histogram!("analytics_event_processing_duration_seconds", "event_type" => "message_sent").record(0.0);
    histogram!("analytics_event_size_bytes", "event_type" => "message_sent").record(0.0);

    // Session management metrics
    gauge!("analytics_active_sessions").set(0.0);
    counter!("analytics_sessions_created_total").absolute(0);
    counter!("analytics_sessions_expired_total").absolute(0);
    histogram!("analytics_session_duration_seconds").record(0.0);

    // Query metrics
    counter!("analytics_queries_total", "query_type" => "user_activity", "status" => "success").absolute(0);
    histogram!("analytics_query_duration_seconds", "query_type" => "user_activity").record(0.0);
    histogram!("analytics_query_result_size", "query_type" => "user_activity").record(0.0);

    info!("[ANALYTICS] Metrics registered");
}

/// Metrics collection utilities for analytics_server
pub mod collectors {
    use super::*;

    /// HTTP API metrics collector
    pub struct HttpMetrics;

    impl HttpMetrics {
        pub fn record_request(method: &str, endpoint: &str, status: u16, duration: Duration) {
            counter!("analytics_http_requests_total", 
                    "method" => method.to_string(), 
                    "endpoint" => endpoint.to_string(), 
                    "status" => status.to_string())
            .increment(1);

            histogram!("analytics_http_request_duration_seconds",
                      "method" => method.to_string(),
                      "endpoint" => endpoint.to_string())
            .record(duration.as_secs_f64());
        }

        pub fn record_batch_size(endpoint: &str, size: usize) {
            histogram!("analytics_batch_size", "endpoint" => endpoint.to_string())
                .record(size as f64);
        }
    }

    /// ClickHouse metrics collector
    pub struct ClickHouseMetrics;

    impl ClickHouseMetrics {
        pub fn record_operation(operation: &str, table: &str, duration: Duration, success: bool) {
            let status = if success { "success" } else { "failed" };
            
            counter!("analytics_clickhouse_operations_total",
                    "operation" => operation.to_string(),
                    "table" => table.to_string(),
                    "status" => status.to_string())
            .increment(1);

            if success {
                histogram!("analytics_clickhouse_operation_duration_seconds",
                          "operation" => operation.to_string(),
                          "table" => table.to_string())
                .record(duration.as_secs_f64());
            }
        }

        pub fn set_active_connections(count: usize) {
            gauge!("analytics_clickhouse_connections_active").set(count as f64);
        }

        pub fn connection_error() {
            counter!("analytics_clickhouse_connection_errors_total").increment(1);
        }
    }

    /// NATS metrics collector
    pub struct NATSMetrics;

    impl NATSMetrics {
        pub fn message_received(subject: &str, message_type: &str) {
            counter!("analytics_nats_messages_received_total", 
                    "subject" => subject.to_string(),
                    "type" => message_type.to_string())
            .increment(1);
        }

        pub fn message_processed(subject: &str, duration: Duration, success: bool) {
            let status = if success { "success" } else { "failed" };
            
            counter!("analytics_nats_messages_processed_total", 
                    "subject" => subject.to_string(), 
                    "status" => status.to_string())
            .increment(1);

            if success {
                histogram!("analytics_nats_message_processing_duration_seconds", 
                          "subject" => subject.to_string())
                .record(duration.as_secs_f64());
            }
        }

        pub fn parse_error(subject: &str) {
            counter!("analytics_nats_messages_failed_total", 
                    "subject" => subject.to_string(), 
                    "error_type" => "parse_error".to_string())
            .increment(1);
        }

        pub fn set_subscription_active(subject: &str, active: bool) {
            gauge!("analytics_nats_subscription_active", "subject" => subject.to_string())
                .set(if active { 1.0 } else { 0.0 });
        }
    }

    /// Event processing metrics
    pub struct EventMetrics;

    impl EventMetrics {
        pub fn event_received(event_type: &str, source: &str, size_bytes: usize) {
            counter!("analytics_events_received_total", 
                    "event_type" => event_type.to_string(),
                    "source" => source.to_string())
            .increment(1);

            histogram!("analytics_event_size_bytes", 
                      "event_type" => event_type.to_string())
            .record(size_bytes as f64);
        }

        pub fn event_processed(event_type: &str, duration: Duration, success: bool) {
            if success {
                counter!("analytics_events_stored_total", 
                        "event_type" => event_type.to_string(),
                        "status" => "success".to_string())
                .increment(1);
            } else {
                counter!("analytics_events_failed_total", 
                        "event_type" => event_type.to_string(),
                        "error_type" => "processing_error".to_string())
                .increment(1);
            }

            histogram!("analytics_event_processing_duration_seconds", 
                      "event_type" => event_type.to_string())
            .record(duration.as_secs_f64());
        }

        pub fn validation_error(event_type: &str) {
            counter!("analytics_events_failed_total", 
                    "event_type" => event_type.to_string(),
                    "error_type" => "validation_error".to_string())
            .increment(1);
        }
    }

    /// Session metrics
    pub struct SessionMetrics;

    impl SessionMetrics {
        pub fn set_active_sessions(count: usize) {
            gauge!("analytics_active_sessions").set(count as f64);
        }

        pub fn session_created() {
            counter!("analytics_sessions_created_total").increment(1);
        }

        pub fn session_expired(duration: Duration) {
            counter!("analytics_sessions_expired_total").increment(1);
            histogram!("analytics_session_duration_seconds").record(duration.as_secs_f64());
        }
    }

    /// Query metrics
    pub struct QueryMetrics;

    impl QueryMetrics {
        pub fn query_executed(query_type: &str, duration: Duration, result_size: usize, success: bool) {
            let status = if success { "success" } else { "failed" };
            
            counter!("analytics_queries_total", 
                    "query_type" => query_type.to_string(),
                    "status" => status.to_string())
            .increment(1);

            if success {
                histogram!("analytics_query_duration_seconds", 
                          "query_type" => query_type.to_string())
                .record(duration.as_secs_f64());

                histogram!("analytics_query_result_size", 
                          "query_type" => query_type.to_string())
                .record(result_size as f64);
            }
        }
    }
}