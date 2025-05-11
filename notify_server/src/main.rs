use anyhow::Result;
use notify_server::{AppConfig, get_router};
use sqlx::PgPool;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
  Layer::{self as _},
  fmt::Layer,
  layer::SubscriberExt,
  util::SubscriberInitExt,
};

#[shuttle_runtime::main]
async fn main(
  #[shuttle_shared_db::Postgres] pool: PgPool,
) -> shuttle_axum::ShuttleAxum {
  // Initialize tracing for logging
  let layer = Layer::new().with_filter(LevelFilter::INFO);
  tracing_subscriber::registry().with(layer).init();

  let config = AppConfig::load().expect("Failed to load config");
  
  let app = get_router(config).await.expect("Failed to create router");
  
  info!("Notify server initialized with Shuttle");
  
  Ok(app.into())
}
