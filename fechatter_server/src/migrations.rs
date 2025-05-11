use sqlx::PgPool;
use tracing::info;

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Running embedded migrations");
    
    // Initial schema
    info!("Running migration: 0001_initial_schema");
    sqlx::query(include_str!("../migrations/0001_initial_schema.sql"))
        .execute(pool)
        .await?;
    
    // Trigger
    info!("Running migration: 0002_trigger");
    sqlx::query(include_str!("../migrations/0002_trigger.sql"))
        .execute(pool)
        .await?;
    
    // Workspace
    info!("Running migration: 0003_workspace");
    sqlx::query(include_str!("../migrations/0003_workspace.sql"))
        .execute(pool)
        .await?;
    
    // Refresh tokens
    info!("Running migration: 0004_refresh_tokens");
    sqlx::query(include_str!("../migrations/0004_refresh_tokens.sql"))
        .execute(pool)
        .await?;
    
    // Notify trigger
    info!("Running migration: 0005_notify_trigger");
    sqlx::query(include_str!("../migrations/0005_notify_trigger.sql"))
        .execute(pool)
        .await?;
    
    // Add idempotency key
    info!("Running migration: 0006_add_idempotency_key");
    sqlx::query(include_str!("../migrations/0006_add_idempotency_key.sql"))
        .execute(pool)
        .await?;
    
    info!("All migrations completed successfully");
    Ok(())
}
