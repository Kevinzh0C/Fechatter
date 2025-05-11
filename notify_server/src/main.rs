use anyhow::Result;
use notify_server::{AppConfig, get_router};
use sqlx::PgPool;
use tracing::info;

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
  
  let mut config = AppConfig::load().expect("Failed to load config");
  

  let app = get_router(config).await.expect("Failed to create router");

  info!("Notify server initialized with Shuttle");

  Ok(app.into())
}
