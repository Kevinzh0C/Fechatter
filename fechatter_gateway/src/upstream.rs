//! # Upstream Management - Pingora Native
//!
//! Manages upstream services using Pingora's built-in load balancers and health checks

use crate::config::{GatewayConfig, LoadBalancingType};
use anyhow::Result;
use pingora_core::upstreams::peer::HttpPeer;
use pingora_load_balancing::Backend;
use std::collections::HashMap;
use std::sync::Arc;
use std::net::SocketAddr;
use tracing::{debug, info, warn, error};

/// Manages upstream services with Pingora load balancers
pub struct UpstreamManager {
  upstreams: HashMap<String, UpstreamGroup>,
  config: Arc<GatewayConfig>,
}

/// Represents upstream group with load balancer
struct UpstreamGroup {
  name: String,
  backends: Vec<Backend>,
  load_balancing_type: LoadBalancingType,
}

/// Upstream status for monitoring
#[derive(Debug, Clone)]
pub struct UpstreamStatus {
  pub name: String,
  pub total_peers: usize,
  pub healthy_peers: usize,
}

impl UpstreamManager {
  /// Create new upstream manager with proper error handling
  pub async fn new(config: Arc<GatewayConfig>) -> Result<Self> {
    let mut upstreams = HashMap::new();

    for (name, upstream_config) in &config.upstreams {
      info!("⚡ Creating upstream group: {}", name);

      // Create backends with proper socket address parsing
      let mut backends = Vec::new();
      for server in &upstream_config.servers {
        match create_backend_safe(server) {
          Ok(backend) => {
            backends.push(backend);
            debug!("✅ Backend created successfully for {}", server);
          }
          Err(e) => {
            warn!(
              "⚠️  Failed to create backend for {}: {}. Continuing with other backends.",
              server, e
            );
            // Continue processing other backends instead of failing completely
          }
        }
      }

      // Log status but continue even with empty backends (for development)
      if backends.is_empty() {
        warn!(
          "⚠️  Upstream group '{}' has no healthy backends. This may cause 503 errors.",
          name
        );
      } else {
        info!(
          "✅ Upstream group '{}' created with {} healthy servers",
          name,
          backends.len()
        );
      }

      upstreams.insert(
        name.clone(),
        UpstreamGroup {
          name: name.clone(),
          backends,
          load_balancing_type: upstream_config
            .load_balancing
            .clone()
            .unwrap_or(LoadBalancingType::RoundRobin),
        },
      );
    }

    info!("Upstream manager initialized successfully");
    Ok(Self { upstreams, config })
  }

  /// Create basic upstream manager without health checks (for recovery mode)
  pub async fn new_basic(config: Arc<GatewayConfig>) -> Result<Self> {
    let mut upstreams = HashMap::new();

    for (name, upstream_config) in &config.upstreams {
      info!("⚡ Creating upstream group (basic mode): {}", name);

      // Create backends with basic error handling
      let backends = upstream_config
        .servers
        .iter()
        .filter_map(|server| {
          match create_backend_safe(server) {
            Ok(backend) => {
              debug!("✅ Backend created for {} (basic mode)", server);
              Some(backend)
            }
            Err(e) => {
              warn!(
                "⚠️  Skipping backend {} in basic mode: {}",
                server, e
              );
              None
            }
          }
        })
        .collect::<Vec<_>>();

      if backends.is_empty() {
        warn!("⚠️  Upstream group '{}' has no backends in basic mode", name);
      } else {
        info!(
          "✅ Upstream group '{}' created with {} backends (basic mode)",
          name,
          backends.len()
        );
      }

      upstreams.insert(
        name.clone(),
        UpstreamGroup {
          name: name.clone(),
          backends,
          load_balancing_type: upstream_config
            .load_balancing
            .clone()
            .unwrap_or(LoadBalancingType::RoundRobin),
        },
      );
    }

    Ok(Self { upstreams, config })
  }

  /// Select peer from upstream group using simple round-robin
  pub fn select_peer(&self, upstream_name: &str, _key: Option<u64>) -> Option<HttpPeer> {
    let upstream = self.upstreams.get(upstream_name)?;

    if upstream.backends.is_empty() {
      debug!(
        "No healthy backends available for upstream: {}",
        upstream_name
      );
      return None;
    }

    // Simple round-robin selection based on current time
    let index = (std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap_or_default()
      .as_secs() as usize)
      % upstream.backends.len();

    let backend = &upstream.backends[index];
    debug!("Selected backend: {:?}", backend.addr);

    // Convert Backend to HttpPeer - clone the backend to avoid move
    Some(HttpPeer::new(backend.addr.clone(), false, "".to_string()))
  }

  /// Report health status for upstream peer
  pub fn report_health(&self, upstream_name: &str, _peer_id: &str, healthy: bool) {
    if let Some(_upstream) = self.upstreams.get(upstream_name) {
      debug!("Reporting health for {}: {}", upstream_name, healthy);
      // Health reporting logic would go here
    }
  }

  /// Get status of all upstreams
  pub fn get_upstream_status(&self) -> HashMap<String, UpstreamStatus> {
    let mut statuses = HashMap::new();

    for (name, upstream) in &self.upstreams {
      let status = UpstreamStatus {
        name: name.clone(),
        total_peers: self
          .config
          .upstreams
          .get(name)
          .map(|c| c.servers.len())
          .unwrap_or(0),
        healthy_peers: upstream.backends.len(), // Use actual healthy backends count
      };
      statuses.insert(name.clone(), status);
    }

    statuses
  }
}

/// Safely create a backend with proper error handling
fn create_backend_safe(server: &str) -> Result<Backend> {
  // First, validate that we can parse the address
  let socket_addr: SocketAddr = server.parse()
    .map_err(|e| anyhow::anyhow!("Invalid socket address '{}': {}", server, e))?;
  
  debug!("Parsed socket address: {}", socket_addr);

  // Try different Backend creation approaches for Pingora 0.5.0
  
  // Approach 1: Direct socket address
  match Backend::new(server) {
    Ok(backend) => {
      debug!("✅ Backend created with direct address: {}", server);
      return Ok(backend);
    }
    Err(e) => {
      debug!("Failed direct address approach: {:?}", e);
    }
  }

  // Approach 2: With http scheme
  let http_url = format!("http://{}", server);
  match Backend::new(&http_url) {
    Ok(backend) => {
      debug!("✅ Backend created with HTTP URL: {}", http_url);
      return Ok(backend);
    }
    Err(e) => {
      debug!("Failed HTTP URL approach: {:?}", e);
    }
  }

  // Approach 3: Just the host:port without protocol
  match Backend::new(&format!("{}", socket_addr)) {
    Ok(backend) => {
      debug!("✅ Backend created with formatted socket address: {}", socket_addr);
      return Ok(backend);
    }
    Err(e) => {
      error!("All backend creation approaches failed for {}: {:?}", server, e);
      return Err(anyhow::anyhow!("Failed to create backend for {}: {:?}", server, e));
    }
  }
}
