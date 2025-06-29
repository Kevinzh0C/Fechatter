use analytics_server::{AppConfig, AppState, get_router};
use anyhow::Result;
use tokio::net::TcpListener;
use tracing::{error, info, warn};
use tracing_subscriber::{
  EnvFilter, Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt,
};

#[tokio::main]
async fn main() -> Result<()> {
  // Initialize structured logging based on configuration
  let env_filter = EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| EnvFilter::new("info,analytics_server=debug"));

  let layer = Layer::new()
    .with_target(true)
    .with_thread_ids(true)
    .with_file(true)
    .with_line_number(true)
    .with_filter(env_filter);

  tracing_subscriber::registry().with(layer).init();

  info!("Starting Analytics Server v{}", env!("CARGO_PKG_VERSION"));

  // Initialize observability (Prometheus metrics)
  let metrics_addr = "0.0.0.0:7778"; // Default metrics port for analytics_server
  if let Err(e) = analytics_server::observability::init_observability(metrics_addr).await {
    error!("Failed to initialize observability: {}", e);
    warn!("Continuing without Prometheus metrics export");
  } else {
    info!("Prometheus metrics available at: http://{}/metrics", metrics_addr);
  }

  // Load and validate configuration
  let config = AppConfig::load().map_err(|e| {
    error!("Failed to load configuration: {}", e);
    e
  })?;

  let addr = format!("0.0.0.0:{}", config.server.port);
  
  info!("Analytics Server will bind to: {}", addr);
  info!("ClickHouse URL: {}", config.get_database_url());
  info!("Database: {}", config.get_database_name());

  // Initialize application state with comprehensive error handling
  let state = AppState::try_new(config).await.map_err(|e| {
    error!("Failed to initialize application state: {}", e);
    e
  })?;

  info!("Application state initialized successfully");
  info!("Connected to ClickHouse database");

  // Start background maintenance tasks
  state.start_session_cleanup_task();
  info!("Session cleanup task started");

  // Start NATS subscriber if enabled in configuration
  if state.config.is_nats_enabled() {
    info!("NATS messaging is enabled, starting subscriber...");
    let nats_url = state.config.nats().nats_url.clone();
    let state_clone = state.clone();
    
    // Start NATS subscriber in background
    tokio::spawn(async move {
      if let Err(e) = analytics_server::nats_subscriber::start_nats_subscriber(
        nats_url, 
        state_clone.into_arc()
      ).await {
        error!("Failed to start NATS subscriber: {}", e);
      }
    });
    
    info!("SUBSCRIPTION: NATS subscriber started for URL: {} with stream: {}", 
          state.config.nats().nats_url,
          state.config.jetstream().stream_name);
  } else {
    warn!("WARNING: NATS messaging is disabled in configuration");
  }

  // Build application router with all middleware
  let app = get_router(state.clone()).await.map_err(|e| {
    error!("Failed to build application router: {}", e);
    e
  })?;

  info!("Application router built successfully");

  // Start HTTP server with proper error handling
  let listener = TcpListener::bind(&addr).await.map_err(|e| {
    error!("Failed to bind HTTP server to address {}: {}", addr, e);
    e
  })?;

  info!("Analytics Server listening on: {}", addr);
  info!("Health check available at: http://{}/health", addr);
  info!("Metrics available at: http://{}/metrics", addr);
  info!("OpenAPI docs available at: http://{}/swagger-ui/", addr);
  
  // Log available event ingestion methods
  info!("Event Ingestion Methods:");
  info!("  - HTTP POST: {}/api/event (protobuf)", addr);
  info!("  - HTTP POST: {}/api/batch (protobuf batch)", addr);
  if state.config.is_nats_enabled() {
    info!("  - NATS: {} (JSON events)", state.config.nats().nats_url);
  }

  // Create graceful shutdown channel
  let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel(1);
  
  // Spawn signal handler for graceful shutdown
  tokio::spawn(async move {
    tokio::signal::ctrl_c()
      .await
      .expect("Failed to install CTRL+C signal handler");
    info!("Shutdown signal received, initiating graceful shutdown...");
    let _ = shutdown_tx.send(());
  });

  // Start the HTTP server with graceful shutdown
  let http_server = axum::serve(listener, app.into_make_service())
    .with_graceful_shutdown(async move {
      let _ = shutdown_rx.recv().await;
      info!("HTTP server shutting down...");
    });

  // Run the server with proper error handling
  if let Err(e) = http_server.await {
    error!("HTTP server error: {}", e);
  } else {
    info!("HTTP server stopped gracefully");
  }

  info!("Analytics Server shutdown complete");
  Ok(())
}
