use anyhow::Result;
use sqlx::PgPool;
use std::path::Path;
use tracing::info;

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
  if let Ok(env_path) = std::env::var("MIGRATIONS_PATH") {
    if !env_path.is_empty() {
      let migration_path = Path::new(&env_path);
      if migration_path.exists() && migration_path.is_dir() {
        info!("Found migrations directory from environment at: {}", env_path);
        return run_migrator(migration_path, pool).await;
      }
    }
  }
  
  let standard_paths = vec!["./migrations", "../migrations"];
  for path_str in standard_paths {
    let migration_path = Path::new(path_str);
    if migration_path.exists() && migration_path.is_dir() {
      info!("Found migrations directory at: {}", path_str);
      return run_migrator(migration_path, pool).await;
    }
  }
  
  Err(anyhow::anyhow!("Could not find migrations directory"))
}

async fn run_migrator(migration_path: &Path, pool: &PgPool) -> Result<()> {
  sqlx::migrate::Migrator::new(migration_path)
    .await?
    .run(pool)
    .await?;
  Ok(())
}
