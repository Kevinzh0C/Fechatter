pub mod metrics;

use crate::NotifyError;

/// Initialize observability (metrics and tracing)
pub async fn init_observability() -> Result<(), NotifyError> {
    // Initialize Prometheus metrics
    metrics::init_metrics().await?;
    
    tracing::info!("Observability initialized for notify_server");
    Ok(())
}

/// Shutdown observability
pub async fn shutdown_observability() {
    tracing::info!("Shutting down observability");
}