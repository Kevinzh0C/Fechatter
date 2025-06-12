use anyhow::Result;
use bot_server::{AppConfig, HealthState, setup_nats_subscriber, start_health_server};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
  // Initialize tracing for logging
  let layer = Layer::new().with_filter(LevelFilter::INFO);
  tracing_subscriber::registry().with(layer).init();

  // Initialize observability (metrics)
  if let Err(e) = bot_server::observability::init_observability().await {
    eprintln!("❌ Failed to initialize observability: {}", e);
    eprintln!("   Continuing without Prometheus metrics");
  } else {
    info!("📊 Prometheus metrics available at: http://0.0.0.0:9092/metrics");
  }

  // Load configuration
  let config = match AppConfig::load() {
    Ok(config) => {
      info!("✅ {}", config.get_summary());
      config
    }
    Err(e) => {
      eprintln!("❌ Failed to load bot_server configuration: {}", e);
      eprintln!("\n💡 Quick fix suggestions:");
      eprintln!("   1. Copy bot.yml.example to bot.yml");
      eprintln!("   2. Set BOT_CONFIG=/path/to/your/config.yml");
      eprintln!("   3. Check NATS and database connectivity");
      std::process::exit(1);
    }
  };

  info!("🤖 Starting bot_server with NATS event processing...");

  // Connect to database for health checks
  let pool = PgPoolOptions::new()
    .connect(&config.server.db_url)
    .await?;
  let pool = Arc::new(pool);

  // Create NATS client at main level for both health checks and functionality
  let nats_client = if config.messaging.enabled {
    info!("🚀 Connecting to NATS: {}", config.messaging.nats.url);
    
    let client = async_nats::ConnectOptions::new()
      .connection_timeout(std::time::Duration::from_secs(10))
      .ping_interval(std::time::Duration::from_secs(60))
      .max_reconnects(Some(5))
      .reconnect_delay_callback(|attempts: usize| {
        std::time::Duration::from_secs(std::cmp::min(2u64.pow(attempts as u32), 10))
      })
      .connect(&config.messaging.nats.url)
      .await?;

    info!("✅ Connected to NATS: {}", config.messaging.nats.url);
    Some(Arc::new(client))
  } else {
    info!("⚠️ NATS messaging disabled in configuration");
    None
  };

  // Setup health check state with NATS client
  let config_arc = Arc::new(config.clone());
  let health_state = if let Some(nats_client) = &nats_client {
    HealthState::new(pool.clone(), config_arc.clone()).with_nats(nats_client.clone())
  } else {
    HealthState::new(pool.clone(), config_arc.clone())
  };

  // Start health check server in background
  let health_port = 6686; // Default health check port for bot_server
  let health_state_clone = health_state.clone();
  let health_handle = tokio::spawn(async move {
    if let Err(e) = start_health_server(health_state_clone, health_port).await {
      tracing::error!("Health server failed: {}", e);
    }
  });

  info!("🏥 Health check server started on port {}", health_port);
  info!("   GET http://localhost:{}/health - Full health check", health_port);
  info!("   GET http://localhost:{}/ready - Readiness check", health_port);
  info!("   GET http://localhost:{}/live - Liveness check", health_port);

  // Setup NATS subscriber for event processing with existing client
  setup_nats_subscriber(&config, nats_client).await?;

  // Keep the process running
  info!("🎯 Bot server is running and listening for NATS events");
  info!(
    "📡 Subscribed to: {:?}",
    config.messaging.nats.subscription_subjects
  );

  // Wait for health server to complete (it runs forever)
  let _ = health_handle.await;

  Ok(())
}
