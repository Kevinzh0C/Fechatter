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
  
  let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
    info!("DATABASE_URL not found, using configured database");
    config.server.db_url.clone()
  });
  
  config.server.db_url = db_url;
  info!("Using database connection from environment");
  
  let state = AppState::new_with_pool(config, pool.clone())
    .expect("Failed to create AppState with provided pool");
  
  let app = get_router(state).await.expect("Failed to create router");

  info!("Fechatter server initialized with Shuttle");

  Ok(app.into())
}
