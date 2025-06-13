use anyhow::Result;
use fechatter_server::{AppConfig, AppState, get_router};
use tokio::net::TcpListener;
use tracing::{debug, info};
use tracing_subscriber::{EnvFilter, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
  let fmt_layer = Layer::new();

  let filter_layer = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
    // 默认显示INFO级别，但是对于auth和中间件相关模块使用DEBUG级别
    EnvFilter::new(
      "info,fechatter_server::middlewares=debug,fechatter_server::handlers::auth=debug",
    )
  });

  tracing_subscriber::registry()
    .with(filter_layer)
    .with(fmt_layer)
    .init();

  debug!("Debug logging enabled");

  // Load app configuration
  let config = AppConfig::load()?;
  let addr = format!("0.0.0.0:{}", config.server.port);

  let state = AppState::try_new(config).await?;
  let app = get_router(state).await?;
  let listener = TcpListener::bind(&addr).await?;
  info!("Listening on: {}", addr);

  axum::serve(listener, app.into_make_service()).await?;

  Ok(())
}
