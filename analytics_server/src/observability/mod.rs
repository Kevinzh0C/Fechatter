pub mod metrics;

use crate::error::AppError;
use std::net::SocketAddr;

/// Initialize observability components (metrics)
pub async fn init_observability(bind_address: &str) -> Result<(), AppError> {
    // Initialize Prometheus metrics
    let addr: SocketAddr = bind_address.parse()
        .map_err(|e| AppError::AnyError(anyhow::anyhow!("Invalid metrics address: {}", e)))?;
    
    metrics::init_metrics(addr).await?;
    
    tracing::info!("Observability initialized for analytics_server");
    Ok(())
}

/// Shutdown observability
pub async fn shutdown_observability() {
    tracing::info!("Shutting down observability");
}