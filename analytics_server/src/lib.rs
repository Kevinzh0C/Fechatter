mod config;
mod error;
mod events;
mod extractors;
mod handlers;
pub mod nats_subscriber;
pub mod observability;
mod openapi;
pub mod pb;

pub use config::*;
use dashmap::DashMap;
pub use error::*;
pub use events::*;

use anyhow::Context;
use clickhouse::Client;
use handlers::{create_event_handler, create_batch_events_handler, health_check_handler};
use openapi::OpenApiRouter as _;
use std::{fmt, ops::Deref, sync::Arc, time::Duration};
use tokio::fs;
use tower_http::{
  compression::CompressionLayer,
  cors::{self, CorsLayer},
  trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use axum::{
  Router,
  extract::State,
  http::{Method, StatusCode},
  middleware,
  response::Json,
  routing::{get, post},
};

pub use config::AppConfig;

#[derive(Debug, Clone)]
pub struct AppState {
  inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
  pub config: AppConfig,
  pub(crate) client: Client,
  pub(crate) sessions: Arc<DashMap<String, (String, i64)>>,
  pub(crate) metrics: Arc<Metrics>,
}

#[derive(Debug, Default)]
pub struct Metrics {
  pub events_received: std::sync::atomic::AtomicU64,
  pub events_processed: std::sync::atomic::AtomicU64,
  pub events_failed: std::sync::atomic::AtomicU64,
  pub database_errors: std::sync::atomic::AtomicU64,
}

impl Metrics {
  pub fn increment_events_received(&self) {
    self
      .events_received
      .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
  }

  pub fn increment_events_processed(&self) {
    self
      .events_processed
      .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
  }

  pub fn increment_events_failed(&self) {
    self
      .events_failed
      .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
  }

  pub fn increment_database_errors(&self) {
    self
      .database_errors
      .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
  }
}

pub async fn get_router(state: AppState) -> Result<Router, AppError> {
  let cors = CorsLayer::new()
    .allow_methods([
      Method::GET,
      Method::POST,
      Method::PATCH,
      Method::DELETE,
      Method::PUT,
      Method::OPTIONS,
    ])
    .allow_origin(cors::Any)
    .allow_headers(cors::Any)
    .max_age(Duration::from_secs(3600));

  let trace_layer = TraceLayer::new_for_http()
    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
    .on_request(DefaultOnRequest::new().level(Level::INFO))
    .on_response(DefaultOnResponse::new().level(Level::INFO));

  let api = Router::new()
    .route("/event", post(create_event_handler))
    .route("/batch", post(create_batch_events_handler))
    .layer(CompressionLayer::new())
    .layer(trace_layer)
    .layer(cors)
    .layer(middleware::from_fn(metrics_middleware));

  let health = Router::new()
    .route("/health", get(health_check_handler))
    .route("/metrics", get(metrics_handler))
    .route("/ready", get(readiness_check_handler))
    .route("/live", get(liveness_check_handler));

  let app = Router::new()
    .openapi()
    .nest("/api", api)
    .merge(health)
    .with_state(state)
    .fallback(not_found_handler);

  Ok(app)
}

/// Metrics middleware to track request counts and performance
async fn metrics_middleware(
  req: axum::http::Request<axum::body::Body>,
  next: axum::middleware::Next,
) -> axum::response::Response {
  let start = std::time::Instant::now();
  let method = req.method().clone();
  let uri = req.uri().clone();

  let response = next.run(req).await;

  let duration = start.elapsed();
  let status = response.status();

  tracing::info!(
      method = %method,
      uri = %uri,
      status = %status,
      duration_ms = %duration.as_millis(),
      "Request completed"
  );

  response
}

/// Metrics endpoint exposing Prometheus-style metrics
async fn metrics_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
  use std::sync::atomic::Ordering;

  Json(serde_json::json!({
      "events_received_total": state.metrics.events_received.load(Ordering::Relaxed),
      "events_processed_total": state.metrics.events_processed.load(Ordering::Relaxed),
      "events_failed_total": state.metrics.events_failed.load(Ordering::Relaxed),
      "database_errors_total": state.metrics.database_errors.load(Ordering::Relaxed),
      "active_sessions": state.sessions.len(),
      "timestamp": chrono::Utc::now(),
      "uptime_seconds": std::time::SystemTime::now()
          .duration_since(std::time::UNIX_EPOCH)
          .unwrap_or_default()
          .as_secs(),
  }))
}

/// Readiness check - service is ready to receive traffic
async fn readiness_check_handler(
  State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
  // Check ClickHouse connectivity
  match tokio::time::timeout(
    Duration::from_secs(2),
    state.client.query("SELECT 1").execute(),
  )
  .await
  {
    Ok(Ok(_)) => Ok(Json(serde_json::json!({
        "status": "ready",
        "checks": {
            "database": "ok"
        }
    }))),
    Ok(Err(e)) => Err((
      StatusCode::SERVICE_UNAVAILABLE,
      Json(serde_json::json!({
          "status": "not_ready",
          "checks": {
              "database": "fail",
              "error": e.to_string()
          }
      })),
    )),
    Err(_) => Err((
      StatusCode::SERVICE_UNAVAILABLE,
      Json(serde_json::json!({
          "status": "not_ready",
          "checks": {
              "database": "timeout"
          }
      })),
    )),
  }
}

/// Liveness check - service is alive and not deadlocked
async fn liveness_check_handler() -> Json<serde_json::Value> {
  Json(serde_json::json!({
      "status": "alive",
      "timestamp": chrono::Utc::now()
  }))
}

/// 404 handler
async fn not_found_handler() -> (StatusCode, Json<serde_json::Value>) {
  (
    StatusCode::NOT_FOUND,
    Json(serde_json::json!({
        "error": "Not Found",
        "message": "The requested resource was not found"
    })),
  )
}

// 当我调用 state.config => state.inner.config
impl Deref for AppState {
  type Target = AppStateInner;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl AppState {
  pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
    fs::create_dir_all(&config.server.base_dir)
      .await
      .context("Failed to create base directory")?;

    // Create ClickHouse client using new configuration structure
    let mut client = Client::default()
      .with_url(config.get_database_url())
      .with_database(config.get_database_name());

    // Set user if provided
    if let Some(user) = config.get_database_user() {
      client = client.with_user(user);
    }

    // Set password if provided  
    if let Some(password) = config.get_database_password() {
      client = client.with_password(password);
    }

    // Test database connection with timeout
    tokio::time::timeout(
      std::time::Duration::from_secs(10),
      client.query("SELECT 1").execute()
    )
    .await
    .context("Database connection timed out")?
    .context("Failed to connect to ClickHouse - ensure ClickHouse is running and accessible")?;

    // Initialize session storage for client session tracking
    // TODO: Consider persisting sessions to Redis for multi-instance deployments
    let sessions = Arc::new(DashMap::new());

    // Initialize metrics collection
    let metrics = Arc::new(Metrics::default());

    Ok(Self {
      inner: Arc::new(AppStateInner {
        config,
        client,
        sessions,
        metrics,
      }),
    })
  }

  /// Convert AppState to Arc<AppState> for functions that require it
  pub fn into_arc(self) -> Arc<Self> {
    Arc::new(self)
  }

  /// Cleanup expired sessions periodically
  pub fn start_session_cleanup_task(&self) {
    let sessions = Arc::clone(&self.sessions);
    let cleanup_interval = Duration::from_secs(300); // 5 minutes
    let session_timeout = Duration::from_secs(600); // 10 minutes

    tokio::spawn(async move {
      let mut interval = tokio::time::interval(cleanup_interval);

      loop {
        interval.tick().await;

        let now = chrono::Utc::now().timestamp_millis();
        let mut expired_keys = Vec::new();

        // Find expired sessions
        for entry in sessions.iter() {
          let (_, last_ts) = entry.value();
          if now - last_ts > session_timeout.as_millis() as i64 {
            expired_keys.push(entry.key().clone());
          }
        }

        // Remove expired sessions
        for key in expired_keys {
          sessions.remove(&key);
        }

        tracing::debug!(
          "Session cleanup completed, active sessions: {}",
          sessions.len()
        );
      }
    });
  }
}

impl fmt::Debug for AppStateInner {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("AppStateInner")
      .field("config", &self.config)
      .field("sessions_count", &self.sessions.len())
      .finish()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_metrics_increment() {
    let metrics = Metrics::default();
    metrics.increment_events_received();
    metrics.increment_events_processed();

    assert_eq!(
      metrics
        .events_received
        .load(std::sync::atomic::Ordering::Relaxed),
      1
    );
    assert_eq!(
      metrics
        .events_processed
        .load(std::sync::atomic::Ordering::Relaxed),
      1
    );
  }
}
