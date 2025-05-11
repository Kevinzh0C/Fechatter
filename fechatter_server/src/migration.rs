use anyhow::Result;
use sqlx::PgPool;
use std::path::Path;

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
  let migration_path = Path::new("./migrations");
  sqlx::migrate::Migrator::new(migration_path)
    .await?
    .run(pool)
    .await?;
  Ok(())
}
