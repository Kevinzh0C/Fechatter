use anyhow::Result;
use fechatter_server::{AppConfig, AppState, get_router};
use shuttle_runtime::SecretStore;
use sqlx::PgPool;
use tracing::{debug, info};

mod migration;

#[shuttle_runtime::main]
async fn main(
  #[shuttle_shared_db::Postgres] pool: PgPool,
  #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
  
  info!("Running database migrations");
  migration::run_migrations(&pool)
    .await
    .expect("Failed to run migrations");

  // Load app configuration
  let mut config = AppConfig::load().expect("Failed to load configuration");
  
  if let Ok(db_url) = std::env::var("DATABASE_URL") {
    config.server.db_url = db_url;
    info!("Using Shuttle-provided PostgreSQL database");
  } else {
    info!("Using configured database from config file");
  }

  let state = AppState::try_new(config)
    .await
    .expect("Failed to create AppState");
  let app = get_router(state).await.expect("Failed to create router");

  info!("Fechatter server initialized with Shuttle");

  Ok(app.into())
}
