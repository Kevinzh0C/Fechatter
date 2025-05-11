use sqlx::PgPool;
use tracing::info;

async fn execute_migration_file(pool: &PgPool, file_name: &str, sql_content: &str) -> Result<(), sqlx::Error> {
    info!("Running migration: {}", file_name);
    
    for statement in sql_content.split(';') {
        let statement = statement.trim();
        if !statement.is_empty() {
            sqlx::query(statement)
                .execute(pool)
                .await?;
        }
    }
    
    Ok(())
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Running embedded migrations");
    
    // Initial schema
    execute_migration_file(
        pool, 
        "0001_initial_schema", 
        include_str!("../../migrations/0001_initial_schema.sql")
    ).await?;
    
    // Trigger
    execute_migration_file(
        pool, 
        "0002_trigger", 
        include_str!("../../migrations/0002_trigger.sql")
    ).await?;
    
    // Workspace
    execute_migration_file(
        pool, 
        "0003_workspace", 
        include_str!("../../migrations/0003_workspace.sql")
    ).await?;
    
    // Refresh tokens
    execute_migration_file(
        pool, 
        "0004_refresh_tokens", 
        include_str!("../../migrations/0004_refresh_tokens.sql")
    ).await?;
    
    // Notify trigger
    execute_migration_file(
        pool, 
        "0005_notify_trigger", 
        include_str!("../../migrations/0005_notify_trigger.sql")
    ).await?;
    
    // Add idempotency key
    execute_migration_file(
        pool, 
        "0006_add_idempotency_key", 
        include_str!("../../migrations/0006_add_idempotency_key.sql")
    ).await?;
    
    info!("All migrations completed successfully");
    Ok(())
}
