use anyhow::Result;
use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::time::Duration;
use tracing::info;

/// Initialize Prometheus metrics for bot_server
pub async fn init_metrics() -> Result<()> {
    let builder = PrometheusBuilder::new();

    // Register all bot_server metrics first
    register_bot_metrics();

    // Set up Prometheus endpoint on port 9092
    let bind_address = "0.0.0.0:9092";
    
    builder
        .with_http_listener(bind_address.parse::<std::net::SocketAddr>()?)
        .install()?;

    info!("📊 [BOT] Prometheus metrics server started on {}", bind_address);
    Ok(())
}

/// Register all bot_server specific metrics
fn register_bot_metrics() {
    // NATS event processing metrics
    counter!("bot_nats_events_received_total", "event_type" => "message_sent").absolute(0);
    counter!("bot_nats_events_processed_total", "event_type" => "message_sent", "status" => "success").absolute(0);
    counter!("bot_nats_events_failed_total", "event_type" => "message_sent", "error_type" => "processing_error").absolute(0);
    histogram!("bot_nats_event_processing_duration_seconds", "event_type" => "message_sent").record(0.0);

    // Message indexing metrics
    counter!("bot_messages_indexed_total", "status" => "success").absolute(0);
    counter!("bot_messages_index_failed_total", "error_type" => "database_error").absolute(0);
    histogram!("bot_message_indexing_duration_seconds").record(0.0);
    
    // Vector embedding metrics
    counter!("bot_embeddings_generated_total", "model" => "openai").absolute(0);
    counter!("bot_embeddings_failed_total", "model" => "openai", "error_type" => "api_error").absolute(0);
    histogram!("bot_embedding_generation_duration_seconds", "model" => "openai").record(0.0);
    histogram!("bot_embedding_size_dimensions", "model" => "openai").record(0.0);

    // Search indexing metrics
    counter!("bot_search_documents_indexed_total", "index" => "messages").absolute(0);
    counter!("bot_search_index_failed_total", "index" => "messages", "error_type" => "indexing_error").absolute(0);
    histogram!("bot_search_indexing_duration_seconds", "index" => "messages").record(0.0);

    // AI agent metrics
    counter!("bot_ai_requests_total", "agent" => "summary", "status" => "success").absolute(0);
    counter!("bot_ai_requests_failed_total", "agent" => "summary", "error_type" => "timeout").absolute(0);
    histogram!("bot_ai_request_duration_seconds", "agent" => "summary").record(0.0);
    histogram!("bot_ai_token_usage", "agent" => "summary", "type" => "input").record(0.0);

    // Database metrics
    counter!("bot_db_operations_total", "operation" => "insert", "table" => "message_embeddings").absolute(0);
    histogram!("bot_db_operation_duration_seconds", "operation" => "insert", "table" => "message_embeddings").record(0.0);
    gauge!("bot_db_connections_active").set(0.0);

    // Health check metrics
    counter!("bot_health_checks_total", "status" => "healthy").absolute(0);
    histogram!("bot_health_check_duration_seconds").record(0.0);

    info!("[BOT] Metrics registered");
}

/// Metrics collection utilities for bot_server
pub mod collectors {
    use super::*;

    /// NATS event processing metrics
    pub struct NATSEventMetrics;

    impl NATSEventMetrics {
        pub fn event_received(event_type: &str) {
            counter!("bot_nats_events_received_total", 
                    "event_type" => event_type.to_string()).increment(1);
        }

        pub fn event_processed(event_type: &str, duration: Duration, success: bool) {
            let status = if success { "success" } else { "failed" };
            
            counter!("bot_nats_events_processed_total", 
                    "event_type" => event_type.to_string(), 
                    "status" => status.to_string()).increment(1);

            if success {
                histogram!("bot_nats_event_processing_duration_seconds", 
                          "event_type" => event_type.to_string()).record(duration.as_secs_f64());
            }
        }

        pub fn processing_error(event_type: &str, error_type: &str) {
            counter!("bot_nats_events_failed_total", 
                    "event_type" => event_type.to_string(), 
                    "error_type" => error_type.to_string()).increment(1);
        }
    }

    /// Message indexing metrics
    pub struct IndexingMetrics;

    impl IndexingMetrics {
        pub fn message_indexed(duration: Duration, success: bool) {
            if success {
                counter!("bot_messages_indexed_total", "status" => "success").increment(1);
                histogram!("bot_message_indexing_duration_seconds").record(duration.as_secs_f64());
            } else {
                counter!("bot_messages_index_failed_total", 
                        "error_type" => "indexing_error").increment(1);
            }
        }

        pub fn search_document_indexed(index: &str, duration: Duration, success: bool) {
            if success {
                counter!("bot_search_documents_indexed_total", 
                        "index" => index.to_string()).increment(1);
                histogram!("bot_search_indexing_duration_seconds", 
                          "index" => index.to_string()).record(duration.as_secs_f64());
            } else {
                counter!("bot_search_index_failed_total", 
                        "index" => index.to_string(), 
                        "error_type" => "indexing_error").increment(1);
            }
        }
    }

    /// Vector embedding metrics
    pub struct EmbeddingMetrics;

    impl EmbeddingMetrics {
        pub fn embedding_generated(model: &str, dimensions: usize, duration: Duration) {
            counter!("bot_embeddings_generated_total", 
                    "model" => model.to_string()).increment(1);
            
            histogram!("bot_embedding_generation_duration_seconds", 
                      "model" => model.to_string()).record(duration.as_secs_f64());
            
            histogram!("bot_embedding_size_dimensions", 
                      "model" => model.to_string()).record(dimensions as f64);
        }

        pub fn embedding_failed(model: &str, error_type: &str) {
            counter!("bot_embeddings_failed_total", 
                    "model" => model.to_string(), 
                    "error_type" => error_type.to_string()).increment(1);
        }
    }

    /// AI agent metrics
    pub struct AIAgentMetrics;

    impl AIAgentMetrics {
        pub fn request_completed(agent: &str, duration: Duration, tokens_used: Option<(u32, u32)>, success: bool) {
            let status = if success { "success" } else { "failed" };
            
            counter!("bot_ai_requests_total", 
                    "agent" => agent.to_string(), 
                    "status" => status.to_string()).increment(1);

            if success {
                histogram!("bot_ai_request_duration_seconds", 
                          "agent" => agent.to_string()).record(duration.as_secs_f64());

                if let Some((input_tokens, output_tokens)) = tokens_used {
                    histogram!("bot_ai_token_usage", 
                              "agent" => agent.to_string(), 
                              "type" => "input").record(input_tokens as f64);
                    histogram!("bot_ai_token_usage", 
                              "agent" => agent.to_string(), 
                              "type" => "output").record(output_tokens as f64);
                }
            }
        }

        pub fn request_failed(agent: &str, error_type: &str) {
            counter!("bot_ai_requests_failed_total", 
                    "agent" => agent.to_string(), 
                    "error_type" => error_type.to_string()).increment(1);
        }
    }

    /// Database metrics
    pub struct DatabaseMetrics;

    impl DatabaseMetrics {
        pub fn operation_completed(operation: &str, table: &str, duration: Duration, success: bool) {
            if success {
                counter!("bot_db_operations_total", 
                        "operation" => operation.to_string(), 
                        "table" => table.to_string()).increment(1);
                
                histogram!("bot_db_operation_duration_seconds", 
                          "operation" => operation.to_string(), 
                          "table" => table.to_string()).record(duration.as_secs_f64());
            }
        }

        pub fn set_active_connections(count: usize) {
            gauge!("bot_db_connections_active").set(count as f64);
        }
    }

    /// Health check metrics
    pub struct HealthMetrics;

    impl HealthMetrics {
        pub fn health_check_completed(duration: Duration, healthy: bool) {
            let status = if healthy { "healthy" } else { "unhealthy" };
            counter!("bot_health_checks_total", "status" => status.to_string()).increment(1);
            histogram!("bot_health_check_duration_seconds").record(duration.as_secs_f64());
        }
    }
}