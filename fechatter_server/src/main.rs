use anyhow::Result;
use fechatter_server::{AppConfig, get_router};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
  // Initialize tracing for logging
  let layer = Layer::new().with_filter(LevelFilter::INFO);
  tracing_subscriber::registry().with(layer).init();

  // Load app configuration
  let config = AppConfig::load()?;

  let addr = format!("0.0.0.0:{}", config.server.port);
  let app = get_router(config).await?;
  let listener = TcpListener::bind(&addr).await?;
  info!("Listening on: {}", addr);

  axum::serve(listener, app.into_make_service()).await?;

  Ok(())
}
