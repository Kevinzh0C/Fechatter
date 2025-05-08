use anyhow::Result;
use fechatter_core::utils::jwt::AuthConfig;
use notify_server::{app_state::NotifyState, get_router, set_up_pg_listener};
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

  // Initialize notify state
  let db_url = "postgres://postgres:postgres@localhost:5432/fechatter";
  let state = NotifyState::new(db_url, &auth_config).await?;

  let app = get_router(state);
  let listener = TcpListener::bind(&addr).await?;

  info!("Listening on: {}", addr);

  axum::serve(listener, app.into_make_service()).await?;

  Ok(())
}
