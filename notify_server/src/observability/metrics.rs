use crate::NotifyError;
use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::time::Duration;
use tracing::info;

/// Initialize Prometheus metrics for notify_server
pub async fn init_metrics() -> Result<(), NotifyError> {
    let builder = PrometheusBuilder::new();

    // Register all notify_server metrics first
    register_notify_metrics();

    // Set up Prometheus endpoint on port 9091 (different from main server)
    let bind_address = "0.0.0.0:9091";
    
    builder
        .with_http_listener(bind_address.parse::<std::net::SocketAddr>()
            .map_err(|e| NotifyError::Internal(format!("Invalid metrics bind address: {}", e)))?)
        .install()
        .map_err(|e| NotifyError::Internal(format!("Failed to install Prometheus exporter: {}", e)))?;

    info!("📊 [NOTIFY] Prometheus metrics server started on {}", bind_address);
    Ok(())
}

/// Register all notify_server specific metrics
fn register_notify_metrics() {
    // SSE Connection metrics
    gauge!("notify_sse_connections_active").set(0.0);
    counter!("notify_sse_connections_total", "status" => "connected").absolute(0);
    counter!("notify_sse_connections_total", "status" => "disconnected").absolute(0);
    histogram!("notify_sse_connection_duration_seconds").record(0.0);

    // NATS metrics
    counter!("notify_nats_messages_received_total", "subject" => "chat.events").absolute(0);
    counter!("notify_nats_messages_processed_total", "subject" => "chat.events", "status" => "success").absolute(0);
    counter!("notify_nats_messages_failed_total", "subject" => "chat.events", "error_type" => "parse_error").absolute(0);
    histogram!("notify_nats_message_processing_duration_seconds", "subject" => "chat.events").record(0.0);

    // Online users metrics
    gauge!("notify_online_users_count").set(0.0);
    histogram!("notify_online_users_query_duration_seconds").record(0.0);

    // Event broadcasting metrics
    counter!("notify_events_broadcast_total", "event_type" => "message").absolute(0);
    counter!("notify_events_broadcast_failed_total", "event_type" => "message", "error_type" => "connection_lost").absolute(0);
    histogram!("notify_event_broadcast_duration_seconds", "event_type" => "message").record(0.0);

    // Health check metrics
    counter!("notify_health_checks_total", "status" => "healthy").absolute(0);
    histogram!("notify_health_check_duration_seconds").record(0.0);

    info!("[NOTIFY] Metrics registered");
}

/// Metrics collection utilities for notify_server
pub mod collectors {
    use super::*;

    /// SSE connection metrics collector
    pub struct SSEMetrics;

    impl SSEMetrics {
        pub fn connection_opened() {
            gauge!("notify_sse_connections_active").increment(1.0);
            counter!("notify_sse_connections_total", "status" => "connected").increment(1);
        }

        pub fn connection_closed(duration: Duration) {
            gauge!("notify_sse_connections_active").decrement(1.0);
            counter!("notify_sse_connections_total", "status" => "disconnected").increment(1);
            histogram!("notify_sse_connection_duration_seconds").record(duration.as_secs_f64());
        }

        pub fn record_active_connections(count: usize) {
            gauge!("notify_sse_connections_active").set(count as f64);
        }
    }

    /// NATS message processing metrics
    pub struct NATSMetrics;

    impl NATSMetrics {
        pub fn message_received(subject: &str) {
            counter!("notify_nats_messages_received_total", 
                    "subject" => subject.to_string()).increment(1);
        }

        pub fn message_processed(subject: &str, duration: Duration, success: bool) {
            if success {
                counter!("notify_nats_messages_processed_total", 
                        "subject" => subject.to_string(), 
                        "status" => "success").increment(1);
            } else {
                counter!("notify_nats_messages_failed_total", 
                        "subject" => subject.to_string(), 
                        "error_type" => "processing_error").increment(1);
            }

            histogram!("notify_nats_message_processing_duration_seconds", 
                      "subject" => subject.to_string()).record(duration.as_secs_f64());
        }

        pub fn parse_error(subject: &str) {
            counter!("notify_nats_messages_failed_total", 
                    "subject" => subject.to_string(), 
                    "error_type" => "parse_error").increment(1);
        }
    }

    /// Online users metrics
    pub struct OnlineUsersMetrics;

    impl OnlineUsersMetrics {
        pub fn set_count(count: usize) {
            gauge!("notify_online_users_count").set(count as f64);
        }

        pub fn query_completed(duration: Duration) {
            histogram!("notify_online_users_query_duration_seconds").record(duration.as_secs_f64());
        }
    }

    /// Event broadcasting metrics
    pub struct EventMetrics;

    impl EventMetrics {
        pub fn broadcast_event(event_type: &str, duration: Duration, success: bool) {
            if success {
                counter!("notify_events_broadcast_total", 
                        "event_type" => event_type.to_string()).increment(1);
            } else {
                counter!("notify_events_broadcast_failed_total", 
                        "event_type" => event_type.to_string(), 
                        "error_type" => "broadcast_failed").increment(1);
            }

            histogram!("notify_event_broadcast_duration_seconds", 
                      "event_type" => event_type.to_string()).record(duration.as_secs_f64());
        }

        pub fn connection_lost_during_broadcast(event_type: &str) {
            counter!("notify_events_broadcast_failed_total", 
                    "event_type" => event_type.to_string(), 
                    "error_type" => "connection_lost").increment(1);
        }
    }

    /// Health check metrics
    pub struct HealthMetrics;

    impl HealthMetrics {
        pub fn health_check_completed(duration: Duration, healthy: bool) {
            let status = if healthy { "healthy" } else { "unhealthy" };
            counter!("notify_health_checks_total", "status" => status.to_string()).increment(1);
            histogram!("notify_health_check_duration_seconds").record(duration.as_secs_f64());
        }
    }
}