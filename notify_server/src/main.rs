use anyhow::Result;
use notify_server::{get_router, set_up_pg_listener};
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

  let addr = "0.0.0.0:6687";

  set_up_pg_listener().await?;

  let app = get_router();
  let listener = TcpListener::bind(&addr).await?;

  info!("Listening on: {}", addr);

  axum::serve(listener, app.into_make_service()).await?;

  Ok(())
}
