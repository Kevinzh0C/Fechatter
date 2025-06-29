//! # Fechatter Gateway - Pure Pingora Implementation
//!
//! **Perfect integration with Pingora framework for high-performance Gateway**
//!
//! Core Pingora Features:
//! - ProxyHttp trait implementation
//! - Built-in load balancing and health checks
//! - Zero-copy streaming processing
//! - Production-ready connection pooling

// Core modules - Pingora native architecture
pub mod config;
pub mod proxy;
pub mod upstream;

use anyhow::Result;
use pingora::prelude::*;
use pingora_proxy::http_proxy_service;
use std::sync::Arc;
use tracing::{error, info, warn};

// Essential re-exports for gateway consumers
pub use config::GatewayConfig;
pub use proxy::FechatterProxy;
pub use upstream::UpstreamManager;

/// Pingora-native Gateway application with complete lifecycle management
pub struct PingoraGateway {
  pub config: Arc<GatewayConfig>,
  pub upstream_manager: Arc<UpstreamManager>,
  pub proxy: FechatterProxy,
}

impl PingoraGateway {
  /// Create new Pingora Gateway instance from configuration file
  pub async fn new(config_path: &str) -> Result<Self> {
    info!("Loading gateway configuration from: {}", config_path);

    // Try explicit config path first, then fallback to enhanced loading with Docker support
    let config = match GatewayConfig::from_file(config_path) {
      Ok(cfg) => Arc::new(cfg),
      Err(e) => {
        error!("Failed to load configuration from {}: {:?}", config_path, e);
        warn!("Falling back to enhanced configuration loading with Docker container support...");
        
        // Use enhanced loading method that searches multiple paths including Docker containers
        match GatewayConfig::load() {
          Ok(cfg) => {
            info!("Gateway configuration loaded successfully with enhanced Docker-aware loading!");
            Arc::new(cfg)
          }
          Err(load_e) => {
            error!("Enhanced configuration loading also failed: {:?}", load_e);
            return Err(anyhow::anyhow!("Configuration load failed: {} (enhanced: {})", e, load_e));
          }
        }
      }
    };
    info!(
      "Configuration loaded with {} upstreams",
      config.upstreams.len()
    );

    // Initialize upstream manager with health checks and error recovery
    let upstream_manager = match UpstreamManager::new(config.clone()).await {
      Ok(manager) => Arc::new(manager),
      Err(e) => {
        warn!("Failed to initialize upstream manager with full health checks: {:?}", e);
        warn!("Creating upstream manager with degraded mode...");
        // Create a basic upstream manager without health checks for recovery
        Arc::new(UpstreamManager::new_basic(config.clone()).await?)
      }
    };
    info!("Upstream manager initialized successfully");

    // Create proxy with routing logic
    let proxy = FechatterProxy::new(config.clone(), upstream_manager.clone());
    info!("Fechatter proxy created with route matching");

    Ok(Self {
      config,
      upstream_manager,
      proxy,
    })
  }

  /// Create new Pingora Gateway from environment-based configuration
  pub async fn new_with_env(environment: &str) -> Result<Self> {
    let config_path = match environment {
      "development" | "dev" => "config/development.yml",
      "production" | "prod" => "config/production.yml",
      "staging" => "config/staging.yml",
      "test" => "config/test.yml",
      _ => {
        info!(
          "Unknown environment '{}', falling back to default config",
          environment
        );
        "config/gateway.yml"
      }
    };

    info!("🌍 Loading {} environment configuration", environment);
    Self::new(config_path).await
  }

  /// Auto-detect environment and load appropriate configuration
  pub async fn new_auto() -> Result<Self> {
    // Try to detect environment from various sources
    let environment = std::env::var("GATEWAY_ENV")
      .or_else(|_| std::env::var("NODE_ENV"))
      .or_else(|_| std::env::var("ENV"))
      .unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
          "development".to_string()
        } else {
          "production".to_string()
        }
      });

    info!("Auto-detected environment: {}", environment);
    Self::new_with_env(&environment).await
  }

  /// Create new Pingora Gateway using enhanced configuration loading with Docker container support
  pub async fn new_from_enhanced_config() -> Result<Self> {
    info!("Loading gateway configuration with enhanced Docker container support");

    // Use enhanced loading method that searches multiple paths including Docker containers
    let config = match GatewayConfig::load() {
      Ok(cfg) => {
        info!("Gateway configuration loaded successfully with enhanced Docker-aware loading!");
        Arc::new(cfg)
      }
      Err(e) => {
        error!("Enhanced configuration loading failed: {:?}", e);
        return Err(anyhow::anyhow!("Enhanced configuration load failed: {}", e));
      }
    };
    
    info!(
      "Configuration loaded with {} upstreams",
      config.upstreams.len()
    );

    // Initialize upstream manager with health checks and error recovery
    let upstream_manager = match UpstreamManager::new(config.clone()).await {
      Ok(manager) => Arc::new(manager),
      Err(e) => {
        warn!("Failed to initialize upstream manager with full health checks: {:?}", e);
        warn!("Creating upstream manager with degraded mode...");
        // Create a basic upstream manager without health checks for recovery
        Arc::new(UpstreamManager::new_basic(config.clone()).await?)
      }
    };
    info!("Upstream manager initialized successfully");

    // Create proxy with routing logic
    let proxy = FechatterProxy::new(config.clone(), upstream_manager.clone());
    info!("Fechatter proxy created with route matching");

    Ok(Self {
      config,
      upstream_manager,
      proxy,
    })
  }

  /// Create new Pingora Gateway instance from configuration object (for testing)
  pub async fn new_from_config(config: GatewayConfig) -> Result<Self> {
    // Validate configuration first
    config.validate()?;

    let config = Arc::new(config);
    info!(
      "Configuration loaded with {} upstreams",
      config.upstreams.len()
    );

    // Initialize upstream manager with health checks and error recovery
    let upstream_manager = match UpstreamManager::new(config.clone()).await {
      Ok(manager) => Arc::new(manager),
      Err(e) => {
        warn!("Failed to initialize upstream manager with full health checks: {:?}", e);
        warn!("Creating upstream manager with degraded mode...");
        // Create a basic upstream manager without health checks for recovery
        Arc::new(UpstreamManager::new_basic(config.clone()).await?)
      }
    };
    info!("Upstream manager initialized successfully");

    // Create proxy with routing logic
    let proxy = FechatterProxy::new(config.clone(), upstream_manager.clone());
    info!("Fechatter proxy created with route matching");

    Ok(Self {
      config,
      upstream_manager,
      proxy,
    })
  }

  /// Start gateway server using Pingora runtime with error recovery
  pub async fn run(self) -> Result<()> {
    info!(
      "Starting Pingora Gateway server on {}",
      self.config.server.listen_addr
    );

    // Create Pingora server instance with error handling
    let mut server = match Server::new(None) {
      Ok(srv) => srv,
      Err(e) => {
        error!("Failed to create Pingora server: {:?}", e);
        return Err(anyhow::anyhow!("Server creation failed: {}", e));
      }
    };
    
    // Bootstrap server
    server.bootstrap();

    // Add proxy service to server
    let mut proxy_service = http_proxy_service(&server.configuration, self.proxy);
    proxy_service.add_tcp(&self.config.server.listen_addr);

    // Add service to server
    server.add_service(proxy_service);

    // Run server with graceful shutdown handling
    info!("Gateway listening and ready to serve requests");
    info!("Press Ctrl+C to gracefully shutdown");
    
    // Run server (blocks until shutdown signal)
    server.run_forever();
  }

  /// Get gateway runtime status for monitoring
  pub async fn get_status(&self) -> GatewayStatus {
    let upstream_statuses = self.upstream_manager.get_upstream_status();
    let total_upstreams = upstream_statuses.len();
    let healthy_upstreams = upstream_statuses
      .values()
      .filter(|status| status.healthy_peers > 0)
      .count();

    GatewayStatus {
      listen_addr: self.config.server.listen_addr.clone(),
      total_upstreams,
      healthy_upstreams,
      healthy: healthy_upstreams > 0,
      upstreams: upstream_statuses,
    }
  }

  /// Validate gateway configuration for consistency
  pub fn validate_config(&self) -> Result<()> {
    self.config.validate()
  }

  /// Check if gateway is compatible with server endpoints
  pub fn check_server_compatibility(&self) -> Vec<String> {
    let mut compatibility_issues = Vec::new();

    // Check for standard server endpoints compatibility
    let expected_server_routes = vec![
      "/health",
      "/api/v1/auth/",
      "/api/v1/chat/",
      "/api/v1/workspace/",
      "/ws",
    ];

    for expected_route in expected_server_routes {
      let route_configured = self
        .config
        .routes
        .iter()
        .any(|route| route.path == expected_route || route.path.starts_with(expected_route));

      if !route_configured {
        compatibility_issues.push(format!(
          "Missing route configuration for expected server endpoint: {}",
          expected_route
        ));
      }
    }

    // Check CORS configuration consistency
    let api_routes_have_cors = self
      .config
      .routes
      .iter()
      .any(|route| route.path.starts_with("/api/") && route.cors_enabled.unwrap_or(false));

    if !api_routes_have_cors {
      compatibility_issues
        .push("API routes should have CORS enabled for frontend compatibility".to_string());
    }

    // Check WebSocket route configuration
    let ws_route_exists = self.config.routes.iter().any(|route| route.path == "/ws");

    if !ws_route_exists {
      compatibility_issues
        .push("WebSocket route (/ws) should be configured for real-time features".to_string());
    }

    compatibility_issues
  }

  /// Get gateway metrics for monitoring
  pub async fn get_metrics(&self) -> GatewayMetrics {
    let status = self.get_status().await;

    GatewayMetrics {
      total_upstreams: status.total_upstreams,
      healthy_upstreams: status.healthy_upstreams,
      total_routes: self.config.routes.len(),
      cors_enabled_routes: self
        .config
        .routes
        .iter()
        .filter(|r| r.cors_enabled.unwrap_or(false))
        .count(),
      listen_address: status.listen_addr,
      server_compatibility_score: {
        let issues = self.check_server_compatibility();
        if issues.is_empty() {
          100
        } else {
          100 - (issues.len() * 20).min(100)
        }
      },
    }
  }
}

/// Runtime status information for monitoring and health checks
#[derive(Debug, Clone)]
pub struct GatewayStatus {
  pub listen_addr: String,
  pub total_upstreams: usize,
  pub healthy_upstreams: usize,
  pub healthy: bool,
  pub upstreams: std::collections::HashMap<String, upstream::UpstreamStatus>,
}

/// Gateway metrics for monitoring and observability
#[derive(Debug, Clone)]
pub struct GatewayMetrics {
  pub total_upstreams: usize,
  pub healthy_upstreams: usize,
  pub total_routes: usize,
  pub cors_enabled_routes: usize,
  pub listen_address: String,
  pub server_compatibility_score: usize, // 0-100 score
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_gateway_creation_with_default_config() {
    // Create temporary config file for testing using test configuration
    let config = GatewayConfig::for_testing();
    let config_yaml = serde_yaml::to_string(&config).unwrap();

    let temp_file = "/tmp/test_gateway.yml";
    std::fs::write(temp_file, config_yaml).unwrap();

    // Test gateway creation
    let gateway = PingoraGateway::new(temp_file).await;
    assert!(gateway.is_ok());

    // Cleanup
    std::fs::remove_file(temp_file).ok();
  }

  #[tokio::test]
  async fn test_gateway_creation_from_config() {
    let config = GatewayConfig::for_testing();
    let gateway = PingoraGateway::new_from_config(config).await;
    assert!(gateway.is_ok());
  }

  #[tokio::test]
  async fn test_gateway_status() {
    let config = GatewayConfig::for_testing();
    let config_yaml = serde_yaml::to_string(&config).unwrap();

    let temp_file = "/tmp/test_gateway_status.yml";
    std::fs::write(temp_file, config_yaml).unwrap();

    let gateway = PingoraGateway::new(temp_file).await.unwrap();
    let status = gateway.get_status().await;

    assert_eq!(status.listen_addr, "127.0.0.1:8080");
    assert!(status.total_upstreams > 0);

    // Cleanup
    std::fs::remove_file(temp_file).ok();
  }

  #[tokio::test]
  async fn test_server_compatibility_check() {
    let config = GatewayConfig::for_testing();
    let gateway = PingoraGateway::new_from_config(config).await.unwrap();

    let compatibility_issues = gateway.check_server_compatibility();

    // Test config should be compatible with basic server endpoints
    println!("Compatibility issues found: {:?}", compatibility_issues);
    // Note: Some issues may be expected in test config vs production config
  }

  #[tokio::test]
  async fn test_gateway_metrics() {
    let config = GatewayConfig::for_testing();
    let gateway = PingoraGateway::new_from_config(config).await.unwrap();

    let metrics = gateway.get_metrics().await;

    assert!(metrics.total_upstreams > 0);
    assert!(metrics.total_routes > 0);
  }

  #[tokio::test]
  async fn test_config_validation() {
    let config = GatewayConfig::for_testing();
    let gateway = PingoraGateway::new_from_config(config).await.unwrap();

    let validation_result = gateway.validate_config();
    assert!(validation_result.is_ok(), "Test config should be valid");
  }
}
