use anyhow::Result;
use sqlx::PgPool;
use std::path::Path;
use tracing::info;

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
  let possible_paths = vec![
    "./migrations",
    "../migrations",
    &std::env::var("MIGRATIONS_PATH").unwrap_or_default(),
  ];
  
  for path_str in possible_paths {
    if path_str.is_empty() {
      continue;
    }
    
    let migration_path = Path::new(path_str);
    if migration_path.exists() && migration_path.is_dir() {
      info!("Found migrations directory at: {}", path_str);
      
      sqlx::migrate::Migrator::new(migration_path)
        .await?
        .run(pool)
        .await?;
        
      return Ok(());
    }
  }
  
  Err(anyhow::anyhow!("Could not find migrations directory"))
}
