//! # Gateway Configuration for Fechatter
//!
//! **Production-ready configuration with CORS preflight support**

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
  pub server: ServerConfig,
  pub upstreams: HashMap<String, UpstreamConfig>,
  pub routes: Vec<RouteConfig>,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
  pub listen_addr: String,
  pub worker_threads: Option<usize>,
  pub max_connections: Option<usize>,
  pub keepalive_timeout: Option<u64>,
  pub request_timeout: Option<u64>,
}

/// Upstream service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamConfig {
  pub servers: Vec<String>,
  pub health_check: Option<HealthCheckConfig>,
  pub load_balancing: Option<LoadBalancingType>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
  pub interval: u64,
  pub timeout: u64,
  pub path: String,
  pub expected_status: Vec<u16>,
  pub healthy_threshold: Option<u32>,
  pub unhealthy_threshold: Option<u32>,
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingType {
  RoundRobin,
  Random,
  LeastConnections,
  WeightedRoundRobin,
  ConsistentHash,
}

/// Route configuration with CORS support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
  pub path: String,
  pub methods: Vec<String>,
  pub upstream: String,
  pub strip_prefix: Option<String>,
  /// Enable CORS preflight for this route
  pub cors_enabled: Option<bool>,
  /// Custom CORS origins for this route
  pub cors_origins: Option<Vec<String>>,
}

impl Default for ServerConfig {
  fn default() -> Self {
    Self {
      listen_addr: "0.0.0.0:8080".to_string(),
      worker_threads: Some(4),
      max_connections: Some(10000),
      keepalive_timeout: Some(60),
      request_timeout: Some(30),
    }
  }
}

impl GatewayConfig {
  /// Load configuration with Docker container support and fallback paths
  pub fn load() -> Result<Self> {
    // Enhanced configuration loading with Docker container support
    let config_attempts = vec![
      // Priority 1: Environment variable override
      std::env::var("GATEWAY_CONFIG").ok().and_then(|path| {
        println!("📍 Loading from GATEWAY_CONFIG: {}", path);
        match Self::from_file(&path) {
          Ok(config) => {
            println!("Gateway configuration loaded successfully from env var!");
            Some(config)
          }
          Err(e) => {
            eprintln!("ERROR: Failed to load config from GATEWAY_CONFIG ({}): {}", path, e);
            None
          }
        }
      }),
      
      // Priority 2: Docker container standard paths
      Self::try_load_from_path("/app/config/gateway.yml"),
      Self::try_load_from_path("/app/config/gateway.yaml"),
      Self::try_load_from_path("/app/gateway.yml"),
      Self::try_load_from_path("/app/fechatter_gateway/gateway.yml"),
      Self::try_load_from_path("/app/fechatter_gateway/gateway.yaml"),
      Self::try_load_from_path("/etc/fechatter/gateway.yml"),
      Self::try_load_from_path("/etc/fechatter/gateway.yaml"),
      
      // Priority 3: Current directory
      Self::try_load_from_path("gateway.yml"),
      Self::try_load_from_path("gateway.yaml"),
      
      // Priority 4: Gateway service directory
      Self::try_load_from_path("fechatter_gateway/gateway.yml"),
      Self::try_load_from_path("fechatter_gateway/gateway.yaml"),
      
      // Priority 5: Binary directory config
      std::env::current_exe().ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .and_then(|exe_dir| {
          Self::try_load_from_path(&exe_dir.join("config").join("gateway.yml").to_string_lossy())
        }),
    ];

    // Try each configuration source in order
    for config_option in config_attempts {
      if let Some(config) = config_option {
        return Ok(config);
      }
    }

    // Show helpful error with search locations
    Self::show_search_error();
    anyhow::bail!("💥 Gateway configuration file not found in any expected location!")
  }

  /// Try to load configuration from a specific path
  fn try_load_from_path(path: &str) -> Option<Self> {
    match Self::from_file(path) {
      Ok(config) => {
        println!("Gateway configuration loaded successfully from: {}", path);
        Some(config)
      }
      Err(_) => None, // File doesn't exist or parse error, continue searching
    }
  }

  /// Show helpful error message with search locations
  fn show_search_error() {
    eprintln!("\nSearched for gateway configuration in:");
    eprintln!("   ERROR: Environment: GATEWAY_CONFIG");
    eprintln!("   ERROR: Docker paths: /app/config/gateway.yml, /app/gateway.yml");
    eprintln!("   ERROR: Docker service paths: /app/fechatter_gateway/gateway.yml");
    eprintln!("   ERROR: Container paths: /etc/fechatter/gateway.yml");
    eprintln!("   ERROR: Current directory: gateway.yml");
    eprintln!("   ERROR: Service directory: fechatter_gateway/gateway.yml");
    eprintln!("");
    eprintln!("To fix this:");
    eprintln!("   1. 📋 Set GATEWAY_CONFIG environment variable");
    eprintln!("   2. 📄 Place gateway.yml in current directory");
    eprintln!("   3. 🐳 For Docker: mount config to /app/config/gateway.yml");
  }

  /// Load configuration from YAML file
  pub fn from_file(path: &str) -> Result<Self> {
    let content = std::fs::read_to_string(path)?;
    let mut config: GatewayConfig = serde_yaml::from_str(&content)?;

    // Automatically add OPTIONS method to routes with CORS enabled
    config.normalize_cors_routes();

    config.validate()?;
    Ok(config)
  }

  /// Create test configuration with localhost addresses for testing
  #[cfg(test)]
  pub fn for_testing() -> Self {
    let mut upstreams = HashMap::new();

    // Test upstreams using localhost addresses that don't require DNS resolution
    upstreams.insert(
      "test-server".to_string(),
      UpstreamConfig {
        servers: vec!["127.0.0.1:6688".to_string()],
        health_check: None, // Disable health checks for tests
        load_balancing: Some(LoadBalancingType::RoundRobin),
      },
    );

    upstreams.insert(
      "test-notify".to_string(),
      UpstreamConfig {
        servers: vec!["127.0.0.1:7788".to_string()],
        health_check: None,
        load_balancing: Some(LoadBalancingType::RoundRobin),
      },
    );

    let mut config = Self {
      server: ServerConfig {
        listen_addr: "127.0.0.1:8080".to_string(),
        worker_threads: Some(1),
        max_connections: Some(100),
        keepalive_timeout: Some(10),
        request_timeout: Some(5),
      },
      upstreams,
      routes: vec![
        // Health check
        RouteConfig {
          path: "/health".to_string(),
          methods: vec!["GET".to_string()],
          upstream: "test-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(false),
          cors_origins: None,
        },
        // API routes
        RouteConfig {
          path: "/api/".to_string(),
          methods: vec![
            "GET".to_string(),
            "POST".to_string(),
            "PUT".to_string(),
            "DELETE".to_string(),
          ],
          upstream: "test-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(true),
          cors_origins: None,
        },
        // Notification service
        RouteConfig {
          path: "/events".to_string(),
          methods: vec!["GET".to_string()],
          upstream: "test-notify".to_string(),
          strip_prefix: None,
          cors_enabled: Some(true),
          cors_origins: None,
        },
        // WebSocket
        RouteConfig {
          path: "/ws".to_string(),
          methods: vec!["GET".to_string()],
          upstream: "test-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(true),
          cors_origins: None,
        },
      ],
    };

    // Normalize CORS routes
    config.normalize_cors_routes();
    config
  }

  /// Add OPTIONS method to routes that need CORS preflight support
  fn normalize_cors_routes(&mut self) {
    for route in &mut self.routes {
      // Enable CORS by default for API routes
      if route.cors_enabled.unwrap_or(true) && route.path.starts_with("/api/") {
        // Add OPTIONS method if not already present
        if !route.methods.iter().any(|m| m.to_uppercase() == "OPTIONS") {
          route.methods.push("OPTIONS".to_string());
        }
        route.cors_enabled = Some(true);
      }
    }
  }

  /// Validate configuration
  pub fn validate(&self) -> Result<()> {
    // Validate that all routes reference existing upstreams
    for route in &self.routes {
      if !self.upstreams.contains_key(&route.upstream) {
        return Err(anyhow::anyhow!(
          "Route '{}' references unknown upstream '{}'",
          route.path,
          route.upstream
        ));
      }
    }

    // Validate upstream configurations
    for (name, upstream) in &self.upstreams {
      if upstream.servers.is_empty() {
        return Err(anyhow::anyhow!(
          "Upstream '{}' has no servers configured",
          name
        ));
      }

      // Validate server addresses
      for server in &upstream.servers {
        if server.is_empty() {
          return Err(anyhow::anyhow!(
            "Server in upstream '{}' has empty address",
            name
          ));
        }
      }
    }

    Ok(())
  }

  /// Get CORS origins for a specific route
  pub fn get_cors_origins(&self, route_path: &str) -> Vec<String> {
    // Find the route and return its CORS origins, or default ones
    for route in &self.routes {
      if route.path == route_path {
        if let Some(origins) = &route.cors_origins {
          return origins.clone();
        }
        break;
      }
    }

    // Default CORS origins
    vec![
      "http://localhost:1420".to_string(),
      "http://localhost:3000".to_string(),
      "http://localhost:5173".to_string(),
      "http://127.0.0.1:1420".to_string(),
      "https://fechatter.com".to_string(),
    ]
  }

  /// Check if route has CORS enabled
  pub fn is_cors_enabled(&self, route_path: &str) -> bool {
    for route in &self.routes {
      if route.path == route_path {
        return route.cors_enabled.unwrap_or(true);
      }
    }
    true // Default to enabled
  }
}

impl Default for GatewayConfig {
  fn default() -> Self {
    let mut upstreams = HashMap::new();

    // Add default upstreams with proper CORS support
    upstreams.insert(
      "fechatter-server".to_string(),
      UpstreamConfig {
        servers: vec!["fechatter-server:6688".to_string()],
        health_check: Some(HealthCheckConfig {
          interval: 30,
          timeout: 5,
          path: "/health".to_string(),
          expected_status: vec![200],
          healthy_threshold: Some(2),
          unhealthy_threshold: Some(3),
        }),
        load_balancing: Some(LoadBalancingType::RoundRobin),
      },
    );

    upstreams.insert(
      "notify-server".to_string(),
      UpstreamConfig {
        servers: vec!["notify-server:7788".to_string()],
        health_check: Some(HealthCheckConfig {
          interval: 30,
          timeout: 5,
          path: "/sse/health".to_string(), // Use notify server's health endpoint
          expected_status: vec![200],
          healthy_threshold: Some(2),
          unhealthy_threshold: Some(3),
        }),
        load_balancing: Some(LoadBalancingType::RoundRobin),
      },
    );

    upstreams.insert(
      "bot-server".to_string(),
      UpstreamConfig {
        servers: vec!["bot-server:9988".to_string()],
        health_check: Some(HealthCheckConfig {
          interval: 30,
          timeout: 5,
          path: "/health".to_string(),
          expected_status: vec![200],
          healthy_threshold: Some(2),
          unhealthy_threshold: Some(3),
        }),
        load_balancing: Some(LoadBalancingType::RoundRobin),
      },
    );

    let mut config = Self {
      server: ServerConfig::default(),
      upstreams,
      routes: vec![
        // Health check endpoints
        RouteConfig {
          path: "/health".to_string(),
          methods: vec!["GET".to_string()],
          upstream: "fechatter-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(false),
          cors_origins: None,
        },
        // Root path for fechatter-server (index page)
        RouteConfig {
          path: "/".to_string(),
          methods: vec!["GET".to_string()],
          upstream: "fechatter-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(false),
          cors_origins: None,
        },
        // Health check variations
        RouteConfig {
          path: "/health/simple".to_string(),
          methods: vec!["GET".to_string()],
          upstream: "fechatter-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(false),
          cors_origins: None,
        },
        // Authentication routes (fechatter-server)
        RouteConfig {
          path: "/api/signin".to_string(),
          methods: vec!["POST".to_string()],
          upstream: "fechatter-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(true),
          cors_origins: None,
        },
        RouteConfig {
          path: "/api/signup".to_string(),
          methods: vec!["POST".to_string()],
          upstream: "fechatter-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(true),
          cors_origins: None,
        },
        RouteConfig {
          path: "/api/refresh".to_string(),
          methods: vec!["POST".to_string()],
          upstream: "fechatter-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(true),
          cors_origins: None,
        },
        RouteConfig {
          path: "/api/logout".to_string(),
          methods: vec!["POST".to_string()],
          upstream: "fechatter-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(true),
          cors_origins: None,
        },
        RouteConfig {
          path: "/api/logout-all".to_string(),
          methods: vec!["POST".to_string()],
          upstream: "fechatter-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(true),
          cors_origins: None,
        },
        // Debug routes (temporary)
        RouteConfig {
          path: "/api/debug-signin".to_string(),
          methods: vec!["POST".to_string()],
          upstream: "fechatter-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(true),
          cors_origins: None,
        },
        // Chat and workspace API routes (fechatter-server)
        RouteConfig {
          path: "/api/".to_string(),
          methods: vec![
            "GET".to_string(),
            "POST".to_string(),
            "PUT".to_string(),
            "PATCH".to_string(),
            "DELETE".to_string(),
          ],
          upstream: "fechatter-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(true),
          cors_origins: None,
        },
        // Notification service routes
        RouteConfig {
          path: "/events".to_string(),
          methods: vec!["GET".to_string()],
          upstream: "notify-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(true),
          cors_origins: None,
        },
        RouteConfig {
          path: "/online-users".to_string(),
          methods: vec!["GET".to_string()],
          upstream: "notify-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(true),
          cors_origins: None,
        },
        RouteConfig {
          path: "/sse/health".to_string(),
          methods: vec!["GET".to_string()],
          upstream: "notify-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(false),
          cors_origins: None,
        },
        // Bot service routes
        RouteConfig {
          path: "/api/bot/".to_string(),
          methods: vec!["GET".to_string(), "POST".to_string()],
          upstream: "bot-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(true),
          cors_origins: None,
        },
        // WebSocket endpoint - NOTE: fechatter-server doesn't have WebSocket implementation yet
        // This is for future compatibility when WebSocket is implemented
        RouteConfig {
          path: "/ws".to_string(),
          methods: vec!["GET".to_string()],
          upstream: "fechatter-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(true),
          cors_origins: None,
        },
      ],
    };

    // Normalize CORS routes to add OPTIONS methods
    config.normalize_cors_routes();
    config
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_default_config_creation() {
    let config = GatewayConfig::default();
    assert!(!config.upstreams.is_empty());
    assert!(!config.routes.is_empty());
    assert_eq!(config.server.listen_addr, "0.0.0.0:8080");
  }

  #[test]
  fn test_config_validation_passes() {
    let config = GatewayConfig::default();
    assert!(config.validate().is_ok());
  }

  #[test]
  fn test_for_testing_config() {
    let config = GatewayConfig::for_testing();
    assert!(config.validate().is_ok());
    assert_eq!(config.server.listen_addr, "127.0.0.1:8080");
    assert!(config.upstreams.contains_key("test-server"));
    assert!(config.upstreams.contains_key("test-notify"));
  }
}

/// Testing utilities for integration tests
/// This module provides test configurations that can be used by integration tests
pub mod testing {
  use super::*;

  /// Create test configuration with localhost addresses for integration testing
  /// This is separate from the #[cfg(test)] method to allow integration tests to use it
  pub fn create_test_config() -> GatewayConfig {
    let mut upstreams = HashMap::new();

    // Test upstreams using localhost addresses that don't require DNS resolution
    upstreams.insert(
      "test-server".to_string(),
      UpstreamConfig {
        servers: vec!["127.0.0.1:6688".to_string()],
        health_check: None, // Disable health checks for tests
        load_balancing: Some(LoadBalancingType::RoundRobin),
      },
    );

    upstreams.insert(
      "test-notify".to_string(),
      UpstreamConfig {
        servers: vec!["127.0.0.1:7788".to_string()],
        health_check: None,
        load_balancing: Some(LoadBalancingType::RoundRobin),
      },
    );

    let mut config = GatewayConfig {
      server: ServerConfig {
        listen_addr: "127.0.0.1:8080".to_string(),
        worker_threads: Some(1),
        max_connections: Some(100),
        keepalive_timeout: Some(10),
        request_timeout: Some(5),
      },
      upstreams,
      routes: vec![
        // Health check
        RouteConfig {
          path: "/health".to_string(),
          methods: vec!["GET".to_string()],
          upstream: "test-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(false),
          cors_origins: None,
        },
        // API routes
        RouteConfig {
          path: "/api/".to_string(),
          methods: vec![
            "GET".to_string(),
            "POST".to_string(),
            "PUT".to_string(),
            "DELETE".to_string(),
          ],
          upstream: "test-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(true),
          cors_origins: None,
        },
        // Notification service
        RouteConfig {
          path: "/events".to_string(),
          methods: vec!["GET".to_string()],
          upstream: "test-notify".to_string(),
          strip_prefix: None,
          cors_enabled: Some(true),
          cors_origins: None,
        },
        // WebSocket
        RouteConfig {
          path: "/ws".to_string(),
          methods: vec!["GET".to_string()],
          upstream: "test-server".to_string(),
          strip_prefix: None,
          cors_enabled: Some(true),
          cors_origins: None,
        },
      ],
    };

    // Normalize CORS routes
    config.normalize_cors_routes();
    config
  }
}
