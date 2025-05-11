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
  
  config.server.db_url = pool.connect_opts().get_database()
    .map(|db| format!("postgres://{}", db))
    .unwrap_or_else(|| pool.to_string());

  info!("Using Shuttle-provided PostgreSQL database: {}", config.server.db_url);

  let state = AppState::try_new(config)
    .await
    .expect("Failed to create AppState");
  let app = get_router(state).await.expect("Failed to create router");

  info!("Fechatter server initialized with Shuttle");

  Ok(app.into())
}
