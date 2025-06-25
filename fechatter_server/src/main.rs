use anyhow::Result;
use fechatter_server::services::infrastructure::observability::{
  init_telemetry, shutdown_telemetry,
};
use fechatter_server::{AppConfig, AppState, get_router};
use tokio::net::TcpListener;
use tracing::{debug, info};

#[tokio::main]
async fn main() -> Result<()> {
  // Load app configuration FIRST
  let config = AppConfig::load()?;

  // Initialize global OpenTelemetry tracing and metrics
  init_telemetry(&config.features.observability)
    .await
    .expect("Failed to initialize OpenTelemetry");

  debug!("OpenTelemetry and tracing initialized");

  let addr = format!("0.0.0.0:{}", config.server.port);

  let state = AppState::try_new(config).await?;
  let app = get_router(state).await?;
  let listener = TcpListener::bind(&addr).await?;
  info!("Fechatter server starting on: {}", addr);

  // Graceful shutdown handling
  let result = axum::serve(
    listener,
    app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
  )
  .await;

  // Shutdown OpenTelemetry
  shutdown_telemetry().await;
  info!("Server shutdown completed");

  result?;
  Ok(())
}
