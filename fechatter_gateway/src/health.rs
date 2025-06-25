//! # Health Check Module for Gateway
//!
//! Provides health check endpoints for monitoring gateway status

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: u64,
    pub version: String,
    pub uptime_seconds: u64,
    pub upstream_status: UpstreamHealthStatus,
}

/// Upstream health status
#[derive(Debug, Serialize, Deserialize)]
pub struct UpstreamHealthStatus {
    pub total_upstreams: usize,
    pub healthy_upstreams: usize,
    pub degraded_upstreams: usize,
    pub unhealthy_upstreams: usize,
}

/// Application state for health checks
#[derive(Clone)]
pub struct HealthState {
    pub start_time: SystemTime,
    pub version: String,
}

impl HealthState {
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Calculate uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time
            .elapsed()
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

/// Create health check router
pub fn health_router(state: HealthState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/live", get(liveness_check))
        .route("/health/ready", get(readiness_check))
        .with_state(Arc::new(state))
}

/// Main health check endpoint
async fn health_check(State(state): State<Arc<HealthState>>) -> impl IntoResponse {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp,
        version: state.version.clone(),
        uptime_seconds: state.uptime_seconds(),
        upstream_status: UpstreamHealthStatus {
            total_upstreams: 4, // Placeholder - should be from actual upstream manager
            healthy_upstreams: 3,
            degraded_upstreams: 1,
            unhealthy_upstreams: 0,
        },
    };

    (StatusCode::OK, Json(response))
}

/// Kubernetes liveness probe endpoint
async fn liveness_check() -> impl IntoResponse {
    // Simple liveness check - gateway process is running
    (StatusCode::OK, Json(serde_json::json!({ "status": "alive" })))
}

/// Kubernetes readiness probe endpoint
async fn readiness_check(State(state): State<Arc<HealthState>>) -> impl IntoResponse {
    // Check if gateway has been running for at least 5 seconds
    let uptime = state.uptime_seconds();
    
    if uptime < 5 {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ 
                "status": "not_ready",
                "reason": "warming_up",
                "uptime_seconds": uptime
            }))
        );
    }

    // In production, check actual upstream health here
    (StatusCode::OK, Json(serde_json::json!({ 
        "status": "ready",
        "uptime_seconds": uptime
    })))
}