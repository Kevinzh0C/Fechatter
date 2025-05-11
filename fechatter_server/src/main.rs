use fechatter_server::{AppConfig, AppState, get_router};
use shuttle_runtime::SecretStore;
use sqlx::PgPool;
use tracing::info;

mod migrations;

#[shuttle_runtime::main]
async fn main(
  #[shuttle_shared_db::Postgres] pool: PgPool,
  #[shuttle_runtime::Secrets] _secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
  
  info!("Running database migrations");
  if let Err(err) = migrations::run_migrations(&pool).await {
    info!("Migration error: {}", err);
  } else {
    info!("Database migrations completed successfully");
  }

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
