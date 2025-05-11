use anyhow::Result;
use sqlx::PgPool;
use tracing::info;

sqlx::migrate!("../migrations");

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
  info!("Running embedded database migrations");
  
  MIGRATIONS.run(pool).await?;
  
  info!("Database migrations completed successfully");
  Ok(())
}
