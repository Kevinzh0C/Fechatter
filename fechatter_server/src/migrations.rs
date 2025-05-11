use sqlx::PgPool;
use tracing::info;
use anyhow::Result;

sqlx::migrate!("../../migrations");

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Running embedded migrations using sqlx::migrate! macro");
    
    match MIGRATIONS.run(pool).await {
        Ok(_) => {
            info!("All migrations completed successfully");
            Ok(())
        },
        Err(err) => {
            info!("Error running migrations: {}", err);
            Ok(())
        }
    }
}
