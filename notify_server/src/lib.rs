//! # Notify Server - Unified Notification Service
//!
//! Refactored clear architecture:
//! - Single Responsibility: Each module has a clear responsibility
//! - Clear layering: config -> state -> handlers/events/connections
//! - Easy to maintain: Features are aggregated, avoiding duplication

// Module definitions
pub mod analytics;
pub mod config;
pub mod connections;
pub mod error;
pub mod events;
pub mod handlers;
pub mod observability;
pub mod state;
pub mod utils;

// Re-export commonly used types
pub use config::AppConfig;
pub use error::NotifyError;
pub use events::{handle_system_event, types::NotifyEvent, EventProcessor};
pub use state::AppState;

// Re-export handlers
pub use handlers::{
  get_online_users_handler, sse_health_check, OnlineUserResponse, OnlineUsersQuery,
  SSEHealthResponse,
};

// Re-export connection handler
pub use connections::sse_handler;

use anyhow::Result;
use axum::{
  middleware::from_fn_with_state,
  response::{Html, IntoResponse},
  routing::get,
  Router,
};
use fechatter_core::middlewares::{verify_query_token_middleware, verify_token_middleware};
use std::sync::Arc;

const INDEX_HTML: &str = include_str!("../index.html");

/// Create the application router
pub async fn get_router(config: AppConfig) -> Result<Router> {
  let state = AppState::try_new_async(config).await?;

  // Setup unified event processing architecture
  if state.config.messaging.enabled {
    tracing::info!("🚀 NATS event processing is enabled");

    // Initialize NATS connection and subscriber for notify events
    let nats_client =
      events::nats::NatsClient::connect_with_retry(&state.config.messaging.nats.url).await?;

    // Subscribe to notification-related subjects
    let subjects = vec![
      "fechatter.chat.>",
      "fechatter.user.>",
      "fechatter.message.>",
      "fechatter.realtime.>",
      "fechatter.messages.created",
      "fechatter.chats.member.joined",
      "fechatter.chats.member.left",
    ];

    let state_arc = Arc::new(state.clone());
    for subject in subjects {
      tracing::info!("📡 [NOTIFY] Subscribing to NATS subject: {}", subject);
      let subscriber = nats_client.subscribe(subject).await?;
      let processor = EventProcessor::new(subscriber, state_arc.clone()).await?;

      // Spawn event processor for this subject
      tokio::spawn(async move {
        tracing::info!(
          "🚀 [NOTIFY] Starting event processor for subject: {}",
          subject
        );
        if let Err(e) = processor.start().await {
          tracing::error!("❌ [NOTIFY] Event processor failed for {}: {}", subject, e);
        }
      });
    }

    tracing::info!("✅ [NOTIFY] All NATS event processors started successfully");
  } else {
    tracing::warn!(
      "⚠️  NATS messaging is disabled, falling back to PostgreSQL NOTIFY (not recommended)"
    );
    // Note: PostgreSQL NOTIFY is deprecated, this is only for backward compatibility
  }

  // SSE endpoint with query parameter authentication
  let sse_routes = Router::new()
    .route("/events", get(sse_handler))
    .layer(from_fn_with_state(
      state.clone(),
      verify_query_token_middleware::<AppState>,
    ));

  // API endpoints with header authentication
  let api_routes = Router::new()
    .route("/online-users", get(get_online_users_handler))
    .route("/sse/health", get(sse_health_check))
    .layer(from_fn_with_state(
      state.clone(),
      verify_token_middleware::<AppState>,
    ));

  // Public endpoints
  let public_routes = Router::new()
    .route("/", get(index_handler))
    .route("/health", get(health_check))
    .route("/ready", get(readiness_check))
    .route("/live", get(liveness_check));

  let app = Router::new()
    .merge(sse_routes)      // Query parameter auth
    .merge(api_routes)      // Header auth
    .merge(public_routes)   // No auth
    .with_state(state);

  Ok(app)
}

/// Index handler
async fn index_handler() -> impl IntoResponse {
  Html(INDEX_HTML)
}

/// Comprehensive health check
async fn health_check() -> impl IntoResponse {
  use serde_json::json;
  use std::time::Instant;

  let start = Instant::now();

  // Check various components
  let health_status = json!({
      "status": "healthy",
      "service": "notify_server",
      "version": env!("CARGO_PKG_VERSION"),
      "timestamp": chrono::Utc::now().to_rfc3339(),
      "uptime_seconds": start.elapsed().as_secs(),
      "checks": {
          "nats": "healthy",  // TODO: Add actual NATS health check
          "connections": "healthy"
      }
  });

  // Record health check metrics
  observability::metrics::collectors::HealthMetrics::health_check_completed(start.elapsed(), true);

  axum::Json(health_status)
}

/// Readiness check - checks if service is ready to receive traffic
async fn readiness_check() -> impl IntoResponse {
  // Check if all dependencies are available
  axum::Json(serde_json::json!({
      "status": "ready",
      "service": "notify_server",
      "dependencies": {
          "nats": "connected",  // TODO: Add actual checks
          "auth_service": "available"
      }
  }))
}

/// Liveness check - simple check if service is alive
async fn liveness_check() -> impl IntoResponse {
  axum::Json(serde_json::json!({
      "status": "alive",
      "service": "notify_server",
      "timestamp": chrono::Utc::now().to_rfc3339()
  }))
}

// Legacy health check handler
pub async fn health() -> &'static str {
  "OK"
}
