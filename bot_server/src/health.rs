use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

use crate::AppConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub service: String,
    pub version: String,
    pub checks: HealthChecks,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthChecks {
    pub database: ServiceStatus,
    pub nats: ServiceStatus,
    pub openai: ServiceStatus,
    pub analytics: ServiceStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub status: String,
    pub message: String,
    pub latency_ms: Option<u64>,
    pub details: Option<serde_json::Value>,
}

#[derive(Clone)]
pub struct HealthState {
    pub pool: Arc<PgPool>,
    pub config: Arc<AppConfig>,
    pub nats_client: Option<Arc<async_nats::Client>>,
}

impl HealthState {
    pub fn new(pool: Arc<PgPool>, config: Arc<AppConfig>) -> Self {
        Self {
            pool,
            config,
            nats_client: None,
        }
    }

    pub fn with_nats(mut self, nats_client: Arc<async_nats::Client>) -> Self {
        self.nats_client = Some(nats_client);
        self
    }
}

/// Start HTTP health check server
pub async fn start_health_server(state: HealthState, port: u16) -> Result<()> {
    let app = Router::new()
        .route("/health", get(health_check_handler))
        .route("/ready", get(readiness_check_handler))
        .route("/live", get(liveness_check_handler))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    
    info!("🏥 Health check server listening on {}", addr);
    
    axum::serve(listener, app).await?;
    Ok(())
}

/// Comprehensive health check endpoint
async fn health_check_handler(State(state): State<HealthState>) -> Result<Json<HealthStatus>, (StatusCode, Json<serde_json::Value>)> {
    let start_time = std::time::Instant::now();
    
    // Run all health checks
    let database = check_database(&state).await;
    let nats = check_nats(&state).await;
    let openai = check_openai(&state).await;
    let analytics = check_analytics(&state).await;

    // Determine overall status
    let all_healthy = [&database, &nats, &openai, &analytics]
        .iter()
        .all(|check| check.status == "healthy");

    let overall_status = if all_healthy { "healthy" } else { "unhealthy" };
    let status_code = if all_healthy { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };

    let health_status = HealthStatus {
        status: overall_status.to_string(),
        service: "bot_server".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks: HealthChecks {
            database,
            nats,
            openai,
            analytics,
        },
        timestamp: chrono::Utc::now().timestamp_millis(),
    };

    info!("🏥 Health check completed in {:?}", start_time.elapsed());

    if all_healthy {
        Ok(Json(health_status))
    } else {
        Err((status_code, Json(serde_json::to_value(health_status).unwrap())))
    }
}

/// Readiness check - can accept traffic
async fn readiness_check_handler(State(state): State<HealthState>) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Check critical dependencies
    let database = check_database(&state).await;
    let nats = check_nats(&state).await;

    let ready = database.status == "healthy" && nats.status == "healthy";
    let status_code = if ready { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };

    let response = serde_json::json!({
        "status": if ready { "ready" } else { "not_ready" },
        "checks": {
            "database": database,
            "nats": nats,
        }
    });

    if ready {
        Ok(Json(response))
    } else {
        Err((status_code, Json(response)))
    }
}

/// Liveness check - is the service alive
async fn liveness_check_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "alive",
        "service": "bot_server",
        "timestamp": chrono::Utc::now().timestamp_millis()
    }))
}

/// Check database connectivity
async fn check_database(state: &HealthState) -> ServiceStatus {
    let start = std::time::Instant::now();
    
    match sqlx::query("SELECT 1 as test").fetch_one(state.pool.as_ref()).await {
        Ok(_) => {
            let latency = start.elapsed().as_millis() as u64;
            ServiceStatus {
                status: "healthy".to_string(),
                message: "Database connection successful".to_string(),
                latency_ms: Some(latency),
                details: Some(serde_json::json!({
                    "url": state.config.server.db_url.split('@').last().unwrap_or("***"),
                    "query": "SELECT 1"
                })),
            }
        }
        Err(e) => ServiceStatus {
            status: "unhealthy".to_string(),
            message: format!("Database connection failed: {}", e),
            latency_ms: Some(start.elapsed().as_millis() as u64),
            details: Some(serde_json::json!({
                "error": e.to_string()
            })),
        },
    }
}

/// Check NATS connectivity
async fn check_nats(state: &HealthState) -> ServiceStatus {
    let start = std::time::Instant::now();
    
    if !state.config.messaging.enabled {
        return ServiceStatus {
            status: "disabled".to_string(),
            message: "NATS messaging is disabled in configuration".to_string(),
            latency_ms: None,
            details: Some(serde_json::json!({
                "enabled": false
            })),
        };
    }

    match &state.nats_client {
        Some(client) => {
            // Check connection state first
            match client.connection_state() {
                async_nats::connection::State::Connected => {
                    // Try to publish a test message to verify connectivity
                    match client.publish("test.health.check", "ping".into()).await {
                        Ok(_) => {
                            let latency = start.elapsed().as_millis() as u64;
                            ServiceStatus {
                                status: "healthy".to_string(),
                                message: "NATS connection successful".to_string(),
                                latency_ms: Some(latency),
                                details: Some(serde_json::json!({
                                    "url": state.config.messaging.nats.url,
                                    "subjects": state.config.messaging.nats.subscription_subjects,
                                    "connection_state": "Connected"
                                })),
                            }
                        }
                        Err(e) => ServiceStatus {
                            status: "unhealthy".to_string(),
                            message: format!("NATS publish failed: {}", e),
                            latency_ms: Some(start.elapsed().as_millis() as u64),
                            details: Some(serde_json::json!({
                                "error": e.to_string(),
                                "connection_state": "Connected"
                            })),
                        },
                    }
                }
                other_state => {
                    ServiceStatus {
                        status: "unhealthy".to_string(),
                        message: format!("NATS connection state: {:?}", other_state),
                        latency_ms: Some(start.elapsed().as_millis() as u64),
                        details: Some(serde_json::json!({
                            "connection_state": format!("{:?}", other_state),
                            "url": state.config.messaging.nats.url
                        })),
                    }
                }
            }
        }
        None => {
            // NATS client not initialized - check if this is expected
            if state.config.messaging.enabled {
                // This is an error - messaging is enabled but client is not initialized
                ServiceStatus {
                    status: "unhealthy".to_string(),
                    message: "NATS client not initialized despite messaging being enabled".to_string(),
                    latency_ms: Some(start.elapsed().as_millis() as u64),
                    details: Some(serde_json::json!({
                        "messaging_enabled": true,
                        "client_initialized": false,
                        "url": state.config.messaging.nats.url,
                        "error": "This indicates a bug in the bot_server initialization process"
                    })),
                }
            } else {
                // This is expected - messaging is disabled
                ServiceStatus {
                    status: "disabled".to_string(),
                    message: "NATS messaging is disabled".to_string(),
                    latency_ms: None,
                    details: Some(serde_json::json!({
                        "messaging_enabled": false,
                        "client_initialized": false
                    })),
                }
            }
        }
    }
}

/// Check OpenAI API connectivity
async fn check_openai(state: &HealthState) -> ServiceStatus {
    let start = std::time::Instant::now();
    
    // Create minimal OpenAI client test
    let client = reqwest::Client::new();
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
    
    if api_key.is_empty() {
        return ServiceStatus {
            status: "unhealthy".to_string(),
            message: "OPENAI_API_KEY environment variable not set".to_string(),
            latency_ms: Some(start.elapsed().as_millis() as u64),
            details: Some(serde_json::json!({
                "error": "Missing API key"
            })),
        };
    }

    // Test with a minimal request to OpenAI models endpoint
    match client
        .get("https://api.openai.com/v1/models")
        .header("Authorization", format!("Bearer {}", api_key))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) => {
            let latency = start.elapsed().as_millis() as u64;
            if response.status().is_success() {
                ServiceStatus {
                    status: "healthy".to_string(),
                    message: "OpenAI API connection successful".to_string(),
                    latency_ms: Some(latency),
                    details: Some(serde_json::json!({
                        "model": state.config.bot.openai.model,
                        "embed_model": state.config.bot.openai.embed_model
                    })),
                }
            } else {
                ServiceStatus {
                    status: "unhealthy".to_string(),
                    message: format!("OpenAI API returned status: {}", response.status()),
                    latency_ms: Some(latency),
                    details: Some(serde_json::json!({
                        "status": response.status().as_u16()
                    })),
                }
            }
        }
        Err(e) => ServiceStatus {
            status: "unhealthy".to_string(),
            message: format!("OpenAI API connection failed: {}", e),
            latency_ms: Some(start.elapsed().as_millis() as u64),
            details: Some(serde_json::json!({
                "error": e.to_string()
            })),
        },
    }
}

/// Check analytics service connectivity
async fn check_analytics(state: &HealthState) -> ServiceStatus {
    let start = std::time::Instant::now();
    
    if !state.config.analytics.enabled {
        return ServiceStatus {
            status: "disabled".to_string(),
            message: "Analytics is disabled in configuration".to_string(),
            latency_ms: None,
            details: Some(serde_json::json!({
                "enabled": false
            })),
        };
    }

    let client = reqwest::Client::new();
    let health_url = format!("{}/health", state.config.analytics.endpoint.trim_end_matches('/'));

    match client
        .get(&health_url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
    {
        Ok(response) => {
            let latency = start.elapsed().as_millis() as u64;
            if response.status().is_success() {
                ServiceStatus {
                    status: "healthy".to_string(),
                    message: "Analytics service connection successful".to_string(),
                    latency_ms: Some(latency),
                    details: Some(serde_json::json!({
                        "endpoint": state.config.analytics.endpoint
                    })),
                }
            } else {
                ServiceStatus {
                    status: "unhealthy".to_string(),
                    message: format!("Analytics service returned status: {}", response.status()),
                    latency_ms: Some(latency),
                    details: Some(serde_json::json!({
                        "endpoint": state.config.analytics.endpoint,
                        "status": response.status().as_u16()
                    })),
                }
            }
        }
        Err(e) => ServiceStatus {
            status: "unhealthy".to_string(),
            message: format!("Analytics service connection failed: {}", e),
            latency_ms: Some(start.elapsed().as_millis() as u64),
            details: Some(serde_json::json!({
                "endpoint": state.config.analytics.endpoint,
                "error": e.to_string()
            })),
        },
    }
} 