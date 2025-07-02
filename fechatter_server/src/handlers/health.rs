use async_trait::async_trait;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{error, info};
use utoipa::ToSchema;

use crate::{AppError, AppState};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SystemHealth {
    pub status: HealthStatus,
    pub services: Vec<ServiceHealth>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ServiceHealth {
    pub name: String,
    pub status: HealthStatus,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Trait for health checking services
#[async_trait]
pub trait HealthChecker: Send + Sync {
    async fn check_health(&self) -> ServiceHealth;
    fn service_name(&self) -> &'static str;
}

/// Database health checker
pub struct DatabaseChecker {
    pool: sqlx::PgPool,
}

impl DatabaseChecker {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HealthChecker for DatabaseChecker {
    async fn check_health(&self) -> ServiceHealth {
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;

        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => ServiceHealth {
                name: self.service_name().to_string(),
                status: HealthStatus::Healthy,
                latency_ms: Some(latency),
                error: None,
            },
            Err(e) => {
                error!("Database health check failed: {}", e);
                ServiceHealth {
                    name: self.service_name().to_string(),
                    status: HealthStatus::Unhealthy,
                    latency_ms: Some(latency),
                    error: Some(e.to_string()),
                }
            }
        }
    }

    fn service_name(&self) -> &'static str {
        "database"
    }
}

/// NATS health checker
pub struct NatsChecker {
    client: Option<async_nats::Client>,
}

impl NatsChecker {
    pub fn new(client: Option<async_nats::Client>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl HealthChecker for NatsChecker {
    async fn check_health(&self) -> ServiceHealth {
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;

        match &self.client {
            Some(client) => match client.publish("fechatter.health.ping", "ping".into()).await {
                Ok(_) => ServiceHealth {
                    name: self.service_name().to_string(),
                    status: HealthStatus::Healthy,
                    latency_ms: Some(latency),
                    error: None,
                },
                Err(e) => {
                    error!("NATS health check failed: {}", e);
                    ServiceHealth {
                        name: self.service_name().to_string(),
                        status: HealthStatus::Unhealthy,
                        latency_ms: Some(latency),
                        error: Some(e.to_string()),
                    }
                }
            },
            None => ServiceHealth {
                name: self.service_name().to_string(),
                status: HealthStatus::Degraded,
                latency_ms: None,
                error: Some("NATS client not configured".to_string()),
            },
        }
    }

    fn service_name(&self) -> &'static str {
        "nats"
    }
}

/// Search service health checker
pub struct SearchChecker {
    state: AppState,
}

impl SearchChecker {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

/// 新增：Redis缓存健康检查器
pub struct CacheChecker {
    state: AppState,
}

impl CacheChecker {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl HealthChecker for CacheChecker {
    async fn check_health(&self) -> ServiceHealth {
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;

        match self.state.cache_service() {
            Some(cache_service) => {
                // Test actual Redis connectivity with a simple SET/GET operation
                let test_key = "health_check_test";
                let test_value = "ok";

                match cache_service.set(test_key, &test_value, 10).await {
                    Ok(_) => {
                        match cache_service.get::<String>(test_key).await {
                            Ok(Some(value)) if value == test_value => {
                                // Clean up test key
                                let _ = cache_service.del(test_key).await;
                                ServiceHealth {
                                    name: self.service_name().to_string(),
                                    status: HealthStatus::Healthy,
                                    latency_ms: Some(latency),
                                    error: None,
                                }
                            }
                            Ok(_) => ServiceHealth {
                                name: self.service_name().to_string(),
                                status: HealthStatus::Degraded,
                                latency_ms: Some(latency),
                                error: Some(
                                    "Redis read/write test failed: value mismatch".to_string(),
                                ),
                            },
                            Err(e) => ServiceHealth {
                                name: self.service_name().to_string(),
                                status: HealthStatus::Degraded,
                                latency_ms: Some(latency),
                                error: Some(format!("Redis read test failed: {}", e)),
                            },
                        }
                    }
                    Err(e) => ServiceHealth {
                        name: self.service_name().to_string(),
                        status: HealthStatus::Unhealthy,
                        latency_ms: Some(latency),
                        error: Some(format!("Redis write test failed: {}", e)),
                    },
                }
            }
            None => {
                let config_enabled = self.state.inner.config.features.cache.enabled;
                let error_msg = if config_enabled {
                    "Cache service failed to initialize despite being enabled in configuration"
                } else {
                    "Cache service disabled in configuration"
                };

                ServiceHealth {
                    name: self.service_name().to_string(),
                    status: HealthStatus::Degraded,
                    latency_ms: Some(latency),
                    error: Some(error_msg.to_string()),
                }
            }
        }
    }

    fn service_name(&self) -> &'static str {
        "cache"
    }
}

#[async_trait]
impl HealthChecker for SearchChecker {
    async fn check_health(&self) -> ServiceHealth {
        let start = Instant::now();
        let latency = start.elapsed().as_millis() as u64;

        // Check if search service exists
        match self.state.search_service() {
            Some(search_service) => {
                if search_service.is_enabled() {
                    // Get Meilisearch URL from configuration instead of hardcoded localhost
                    let meilisearch_url = &self.state.inner.config.features.search.meilisearch_url;
                    let health_url = format!("{}/health", meilisearch_url);

                    // Create client with proper authorization header
                    let api_key = &self.state.inner.config.features.search.meilisearch_api_key;
                    let client = reqwest::Client::new();
                    let mut request_builder = client.get(&health_url);

                    // Add authorization header if API key is configured
                    if !api_key.is_empty() {
                        request_builder =
                            request_builder.header("Authorization", format!("Bearer {}", api_key));
                    }

                    // Test actual Meilisearch connectivity using configured URL
                    match request_builder.send().await {
                        Ok(response) if response.status().is_success() => ServiceHealth {
                            name: self.service_name().to_string(),
                            status: HealthStatus::Healthy,
                            latency_ms: Some(latency),
                            error: None,
                        },
                        Ok(response) => ServiceHealth {
                            name: self.service_name().to_string(),
                            status: HealthStatus::Degraded,
                            latency_ms: Some(latency),
                            error: Some(format!(
                                "Meilisearch health check failed with status: {} (URL: {})",
                                response.status(),
                                health_url
                            )),
                        },
                        Err(e) => ServiceHealth {
                            name: self.service_name().to_string(),
                            status: HealthStatus::Degraded,
                            latency_ms: Some(latency),
                            error: Some(format!(
                                "Meilisearch connection failed: {} (URL: {})",
                                e, health_url
                            )),
                        },
                    }
                } else {
                    ServiceHealth {
                        name: self.service_name().to_string(),
                        status: HealthStatus::Degraded,
                        latency_ms: Some(latency),
                        error: Some(
                            "Search service exists but is disabled in configuration".to_string(),
                        ),
                    }
                }
            }
            None => {
                // Search service was not initialized - check configuration
                let config_enabled = self.state.inner.config.features.search.enabled;
                let error_msg = if config_enabled {
                    "Search service failed to initialize despite being enabled in configuration"
                } else {
                    "Search service disabled in configuration"
                };

                ServiceHealth {
                    name: self.service_name().to_string(),
                    status: HealthStatus::Degraded,
                    latency_ms: Some(latency),
                    error: Some(error_msg.to_string()),
                }
            }
        }
    }

    fn service_name(&self) -> &'static str {
        "search"
    }
}

impl SystemHealth {
    pub fn new(services: Vec<ServiceHealth>) -> Self {
        let status = Self::calculate_overall_status(&services);

        Self {
            status,
            services,
            timestamp: chrono::Utc::now(),
        }
    }

    fn calculate_overall_status(services: &[ServiceHealth]) -> HealthStatus {
        if services
            .iter()
            .any(|s| matches!(s.status, HealthStatus::Unhealthy))
        {
            HealthStatus::Unhealthy
        } else if services
            .iter()
            .any(|s| matches!(s.status, HealthStatus::Degraded))
        {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "System healthy"),
        (status = 503, description = "System unhealthy")
    ),
    tag = "health"
)]
pub async fn health_check(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    info!("Health check requested");

    // Create health checkers
    let checkers: Vec<Box<dyn HealthChecker>> = vec![
        Box::new(DatabaseChecker::new(state.pool().as_ref().clone())),
        Box::new(NatsChecker::new(state.nats_client())),
        Box::new(SearchChecker::new(state.clone())),
        Box::new(CacheChecker::new(state.clone())),
    ];

    // Run all health checks concurrently
    let mut service_healths = Vec::new();
    for checker in checkers {
        service_healths.push(checker.check_health().await);
    }

    let system_health = SystemHealth::new(service_healths);

    let status_code = match system_health.status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK,
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    info!(
        "Health check completed with status {:?}",
        system_health.status
    );

    Ok((status_code, Json(system_health)))
}

#[utoipa::path(
    get,
    path = "/health/simple",
    responses(
        (status = 200, description = "System operational"),
        (status = 503, description = "System down")
    ),
    tag = "health"
)]
pub async fn simple_health_check(State(state): State<AppState>) -> impl IntoResponse {
    match sqlx::query("SELECT 1").fetch_one(&*state.pool()).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}
