//! # Fechatter Server - Main Entry Point
//!
//! **Responsibility**: Initializes and runs the Axum web server.

use fechatter_server::{config::AppConfig, error::AppError, get_router, AppState};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Load configuration
    let config = AppConfig::load().expect("Failed to load configuration.");

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(
            &config.features.observability.log_level,
        ))
        .init();

    // Create AppState
    let app_state = AppState::try_new(config.clone()).await?;

    // Get the application router
    let app = get_router(app_state).await?;

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Server listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
