use anyhow::Result;
use fechatter_server::{AppConfig, AppState, get_router};
use shuttle_runtime::SecretStore;
use sqlx::PgPool;
use tracing::{debug, info};
use tracing_subscriber::{
  EnvFilter, Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt,
};

mod migration;

#[shuttle_runtime::main]
async fn main(
  #[shuttle_shared_db::Postgres] pool: PgPool,
  #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
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

  info!("Running database migrations");
  migration::run_migrations(&pool)
    .await
    .expect("Failed to run migrations");

  // Load app configuration
  let mut config = AppConfig::load().unwrap_or_default();

  config.server.db_url = pool.connect_lazy_options().connection_string().to_string();

  info!("Using Shuttle-provided PostgreSQL database");

  let state = AppState::try_new(config)
    .await
    .expect("Failed to create AppState");
  let app = get_router(state).await.expect("Failed to create router");

  info!("Fechatter server initialized with Shuttle");

  Ok(app.into())
}
