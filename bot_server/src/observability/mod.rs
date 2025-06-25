pub mod metrics;

use anyhow::Result;

/// Initialize observability components (metrics)
pub async fn init_observability() -> Result<()> {
    // Initialize Prometheus metrics on port 9092
    metrics::init_metrics().await?;
    
    tracing::info!("📊 Observability initialized for bot_server");
    Ok(())
}

/// Shutdown observability
pub async fn shutdown_observability() {
    tracing::info!("📊 Shutting down observability");
}