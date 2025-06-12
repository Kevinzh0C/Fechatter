use anyhow::Result;

use notify_server::{AppConfig, get_router};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
  Layer::{self as _},
  fmt::Layer,
  layer::SubscriberExt,
  util::SubscriberInitExt,
};

#[tokio::main]
async fn main() -> Result<()> {
  // Initialize tracing for logging
  let layer = Layer::new().with_filter(LevelFilter::INFO);
  tracing_subscriber::registry().with(layer).init();

  // Initialize observability (metrics)
  if let Err(e) = notify_server::observability::init_observability().await {
    eprintln!("❌ Failed to initialize observability: {}", e);
    std::process::exit(1);
  }

  // Load configuration for production
  let config = match AppConfig::load() {
    Ok(config) => {
      // Print configuration summary (excluding sensitive information)
      info!("{}", config.get_summary());

      // Validate configuration in production environment
      if std::env::var("ENVIRONMENT").unwrap_or_default() == "production" {
        if let Err(e) = config.validate_production_readiness() {
          eprintln!("❌ Production readiness check failed: {}", e);
          std::process::exit(1);
        }
      }

      config
    }
    Err(e) => {
      eprintln!("❌ Failed to load configuration: {}", e);
      eprintln!("\n💡 Quick fix suggestions:");
      eprintln!("   1. Copy notify.yml.example to notify.yml");
      eprintln!("   2. Set NOTIFY_CONFIG=/path/to/your/config.yml");
      eprintln!("   3. Check file permissions (should be 600 or 640)");
      std::process::exit(1);
    }
  };

  let addr = format!("0.0.0.0:{}", config.server.port);
  let app = get_router(config).await?;

  let listener = TcpListener::bind(&addr).await?;

  info!("🚀 notify_server listening on: {}", addr);

  axum::serve(listener, app.into_make_service()).await?;

  Ok(())
}
