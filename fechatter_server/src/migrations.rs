use sqlx::PgPool;
use tracing::info;

async fn execute_migration_file(pool: &PgPool, file_name: &str, sql_content: &str) -> Result<(), sqlx::Error> {
    info!("Running migration: {}", file_name);
    
    if !sql_content.trim().is_empty() {
        sqlx::query(sql_content)
            .execute(pool)
            .await?;
    }
    
    Ok(())
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Running embedded migrations");
    
    
    // Initial schema
    if let Err(err) = execute_migration_file(
        pool, 
        "0001_initial_schema", 
        include_str!("../../migrations/0001_initial_schema.sql")
    ).await {
        info!("Error running 0001_initial_schema: {}", err);
    }
    
    // Trigger
    if let Err(err) = execute_migration_file(
        pool, 
        "0002_trigger", 
        include_str!("../../migrations/0002_trigger.sql")
    ).await {
        info!("Error running 0002_trigger: {}", err);
    }
    
    // Workspace
    if let Err(err) = execute_migration_file(
        pool, 
        "0003_workspace", 
        include_str!("../../migrations/0003_workspace.sql")
    ).await {
        info!("Error running 0003_workspace: {}", err);
    }
    
    // Refresh tokens
    if let Err(err) = execute_migration_file(
        pool, 
        "0004_refresh_tokens", 
        include_str!("../../migrations/0004_refresh_tokens.sql")
    ).await {
        info!("Error running 0004_refresh_tokens: {}", err);
    }
    
    // Notify trigger
    if let Err(err) = execute_migration_file(
        pool, 
        "0005_notify_trigger", 
        include_str!("../../migrations/0005_notify_trigger.sql")
    ).await {
        info!("Error running 0005_notify_trigger: {}", err);
    }
    
    // Add idempotency key
    if let Err(err) = execute_migration_file(
        pool, 
        "0006_add_idempotency_key", 
        include_str!("../../migrations/0006_add_idempotency_key.sql")
    ).await {
        info!("Error running 0006_add_idempotency_key: {}", err);
    }
    
    info!("Migrations completed");
    Ok(())
}
