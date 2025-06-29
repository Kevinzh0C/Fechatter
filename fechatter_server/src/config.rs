//! # Unified Configuration - Single Source of Truth
//!
//! **Responsibility**: Centralized management of all application configuration
//! **Principle**: Single config source, type safety, environment awareness

use anyhow::Result;
use bytes::Bytes;
use fechatter_core::models::jwt::TokenConfigProvider;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, path::PathBuf, time::Duration};
use thiserror::Error;

// ============================================================================
// Core Configuration Structures - Pure data definitions
// ============================================================================

/// Main application configuration - loaded from YAML
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
  pub server: ServerConfig,
  pub auth: AuthConfig,
  pub features: FeatureConfig,
  pub storage: StorageConfig,
}

/// Server and database configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
  pub port: u16,
  pub db_url: String,
  pub base_dir: PathBuf,
  pub max_upload_size: u64,
  pub request_timeout_ms: u64,
  pub cors: Option<CorsConfig>,
  pub analytics: AnalyticsConfig,
}

/// Authentication configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthConfig {
  pub pk: String,
  pub sk: String,
  pub token_expiration: i64,
  pub refresh_token_expiration: i64,
}

/// Feature configurations
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeatureConfig {
  pub cache: CacheConfig,
  pub search: SearchConfig,
  pub messaging: MessagingConfig,
  pub message_service: MessageServiceConfig,
  pub notifications: NotificationConfig,
  pub observability: ObservabilityConfig,
  pub rate_limiting: RateLimitConfig,
}

/// CORS configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CorsConfig {
  pub enabled: bool,
  pub allow_origins: Vec<String>,
  pub allow_methods: Vec<String>,
  pub allow_headers: Vec<String>,
  pub allow_credentials: bool,
  pub max_age: u64,
}

/// Analytics configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalyticsConfig {
  pub enabled: bool,
  pub endpoint: String,
  pub batch_size: usize,
  pub flush_interval_seconds: u64,
  pub timeout_seconds: u64,
  #[serde(default = "default_use_grpc")]
  pub use_grpc: bool, // true for gRPC, false for NATS
}

fn default_use_grpc() -> bool {
  false // Default to NATS
}

// ============================================================================
// Cache Configuration
// ============================================================================

/// Cache configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheConfig {
  // Base config from chat.yml
  pub enabled: bool,
  pub redis_url: String,
  pub key_prefix: String,
  pub default_ttl: u64,
  pub pool_size: u32,
  pub connection_timeout_ms: u64,

  // Middleware extension config (runtime only, not in chat.yml)
  #[serde(skip, default)]
  pub ttl: Duration,
  #[serde(skip, default)]
  pub cache_private: bool,
  #[serde(skip, default)]
  pub user_specific: bool,
  #[serde(skip, default)]
  pub max_size: usize,
  #[serde(skip, default)]
  pub compress: bool,
  #[serde(skip, default)]
  pub variants: Vec<CacheVariant>,
}

/// Cache variant - defines how cache is segmented by request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheVariant {
  /// By user ID
  UserId,
  /// By workspace ID
  WorkspaceId,
  /// By query parameter
  QueryParam(String),
  /// By HTTP header
  Header(String),
  /// By user permissions
  UserPermissions,
}

/// Cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
  /// Cached content
  pub content: Bytes,
  /// HTTP status code
  pub status_code: u16,
  /// HTTP headers
  pub headers: HashMap<String, String>,
  /// Creation timestamp
  pub created_at: u64,
  /// TTL (seconds)
  pub ttl: u64,
  /// Is compressed
  pub compressed: bool,
  /// Content type
  pub content_type: Option<String>,
  /// ETag
  pub etag: Option<String>,
}

impl CacheEntry {
  /// Calculate remaining TTL
  pub fn remaining_ttl(&self) -> u64 {
    let now = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap_or_default()
      .as_secs();
    if now > self.created_at + self.ttl {
      0
    } else {
      self.created_at + self.ttl - now
    }
  }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
  pub entries: u64,
  pub hits: u64,
  pub misses: u64,
  pub memory_usage: u64,
  pub hit_rate: f64,
}

impl Default for CacheStats {
  fn default() -> Self {
    Self {
      entries: 0,
      hits: 0,
      misses: 0,
      memory_usage: 0,
      hit_rate: 0.0,
    }
  }
}

impl CacheStats {
  pub fn record_hit(&mut self) {
    self.hits += 1;
    self.update_hit_rate();
  }

  pub fn record_miss(&mut self) {
    self.misses += 1;
    self.update_hit_rate();
  }

  pub fn update_hit_rate(&mut self) {
    let total = self.hits + self.misses;
    self.hit_rate = if total > 0 {
      self.hits as f64 / total as f64
    } else {
      0.0
    };
  }
}

// ============================================================================
// Rate Limiting Configuration
// ============================================================================

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
  /// Enable rate limiting
  pub enabled: bool,
  /// Time window (seconds)
  pub window_seconds: u64,
  /// Max requests per window
  pub max_requests: u32,
  /// Use sliding window
  pub sliding_window: bool,
  /// Rate limiting strategy
  pub strategy: RateLimitStrategy,
}

/// Rate limiting strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitStrategy {
  /// By IP address
  IpBased,
  /// By user ID
  UserBased,
  /// By API key
  ApiKeyBased,
  /// By custom identifier
  Custom(String),
}

impl Default for RateLimitConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      window_seconds: 60,
      max_requests: 100,
      sliding_window: true,
      strategy: RateLimitStrategy::IpBased,
    }
  }
}

impl RateLimitConfig {
  /// Create user-level rate limit config
  pub fn per_user(max_requests: usize, window_secs: u64) -> Self {
    Self {
      enabled: true,
      window_seconds: window_secs,
      max_requests: max_requests as u32,
      sliding_window: true,
      strategy: RateLimitStrategy::UserBased,
    }
  }

  /// Create API-level rate limit config
  pub fn per_api(max_requests: usize, window_secs: u64) -> Self {
    Self {
      enabled: true,
      window_seconds: window_secs,
      max_requests: max_requests as u32,
      sliding_window: true,
      strategy: RateLimitStrategy::ApiKeyBased,
    }
  }

  /// Create strict rate limit config (admin operations)
  pub fn strict() -> Self {
    Self {
      enabled: true,
      window_seconds: 60,
      max_requests: 30,
      sliding_window: true,
      strategy: RateLimitStrategy::UserBased,
    }
  }

  /// Create lenient rate limit config (normal users)
  pub fn lenient() -> Self {
    Self {
      enabled: true,
      window_seconds: 60,
      max_requests: 200,
      sliding_window: true,
      strategy: RateLimitStrategy::UserBased,
    }
  }

  /// Create messaging rate limit config
  pub fn messaging() -> Self {
    Self {
      enabled: true,
      window_seconds: 60,
      max_requests: 100, // 100 messages per minute
      sliding_window: true,
      strategy: RateLimitStrategy::UserBased,
    }
  }

  /// Create file upload rate limit config
  pub fn file_upload() -> Self {
    Self {
      enabled: true,
      window_seconds: 300, // 5 minute window
      max_requests: 20,    // 20 files per 5 minutes
      sliding_window: true,
      strategy: RateLimitStrategy::UserBased,
    }
  }

  /// Create search rate limit config
  pub fn search() -> Self {
    Self {
      enabled: true,
      window_seconds: 60,
      max_requests: 50, // 50 searches per minute
      sliding_window: true,
      strategy: RateLimitStrategy::UserBased,
    }
  }

  /// Create authentication rate limit config (prevent brute force)
  pub fn authentication() -> Self {
    Self {
      enabled: true,
      window_seconds: 300, // 5 minute window
      max_requests: 5,     // 5 attempts per 5 minutes
      sliding_window: true,
      strategy: RateLimitStrategy::IpBased,
    }
  }

  /// Create WebSocket connection rate limit config
  pub fn websocket() -> Self {
    Self {
      enabled: true,
      window_seconds: 60,
      max_requests: 10, // 10 connections per minute
      sliding_window: true,
      strategy: RateLimitStrategy::IpBased,
    }
  }

  /// Create permission-based dynamic rate limit config
  pub fn permission_based(is_admin: bool, is_premium: bool) -> Self {
    let max_requests = match (is_admin, is_premium) {
      (true, _) => 1000,     // Admin: 1000 requests per minute
      (false, true) => 500,  // Premium user: 500 requests per minute
      (false, false) => 100, // Normal user: 100 requests per minute
    };

    Self {
      enabled: true,
      window_seconds: 60,
      max_requests,
      sliding_window: true,
      strategy: RateLimitStrategy::UserBased,
    }
  }

  /// Convert to Duration for compatibility
  pub fn window_duration(&self) -> Duration {
    Duration::from_secs(self.window_seconds)
  }
}

// ============================================================================
// Message Service Configuration
// ============================================================================

/// Production-ready message service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageServiceConfig {
  /// Maximum concurrent message sends per chat
  pub max_concurrent_sends_per_chat: usize,
  /// Message send timeout
  pub send_timeout_seconds: u64,
  /// Cache operation timeout
  pub cache_timeout_ms: u64,
  /// Event publishing timeout
  pub event_publish_timeout_seconds: u64,
  /// Stream publish timeout
  pub stream_publish_timeout_seconds: u64,
  /// Enable detailed tracing
  pub enable_detailed_tracing: bool,
  /// Maximum retry attempts for critical operations
  pub max_retry_attempts: u32,
  /// Retry backoff base duration in milliseconds
  pub retry_backoff_base_ms: u64,
}

impl Default for MessageServiceConfig {
  fn default() -> Self {
    Self {
      max_concurrent_sends_per_chat: 10,
      send_timeout_seconds: 30,
      cache_timeout_ms: 500,
      event_publish_timeout_seconds: 5,
      stream_publish_timeout_seconds: 5,
      enable_detailed_tracing: false,
      max_retry_attempts: 3,
      retry_backoff_base_ms: 100,
    }
  }
}

impl MessageServiceConfig {
  /// Convert timeout values to Duration for compatibility
  pub fn send_timeout(&self) -> Duration {
    Duration::from_secs(self.send_timeout_seconds)
  }

  pub fn cache_timeout(&self) -> Duration {
    Duration::from_millis(self.cache_timeout_ms)
  }

  pub fn event_publish_timeout(&self) -> Duration {
    Duration::from_secs(self.event_publish_timeout_seconds)
  }

  pub fn stream_publish_timeout(&self) -> Duration {
    Duration::from_secs(self.stream_publish_timeout_seconds)
  }

  pub fn retry_backoff_base(&self) -> Duration {
    Duration::from_millis(self.retry_backoff_base_ms)
  }

  /// Create development configuration with relaxed timeouts
  pub fn development() -> Self {
    Self {
      max_concurrent_sends_per_chat: 5,
      send_timeout_seconds: 60,
      cache_timeout_ms: 1000,
      event_publish_timeout_seconds: 10,
      stream_publish_timeout_seconds: 10,
      enable_detailed_tracing: true,
      max_retry_attempts: 5,
      retry_backoff_base_ms: 200,
    }
  }

  /// Create production configuration with strict timeouts
  pub fn production() -> Self {
    Self {
      max_concurrent_sends_per_chat: 20,
      send_timeout_seconds: 15,
      cache_timeout_ms: 300,
      event_publish_timeout_seconds: 3,
      stream_publish_timeout_seconds: 3,
      enable_detailed_tracing: false,
      max_retry_attempts: 2,
      retry_backoff_base_ms: 50,
    }
  }

  /// Create high-performance configuration
  pub fn high_performance() -> Self {
    Self {
      max_concurrent_sends_per_chat: 50,
      send_timeout_seconds: 10,
      cache_timeout_ms: 100,
      event_publish_timeout_seconds: 2,
      stream_publish_timeout_seconds: 2,
      enable_detailed_tracing: false,
      max_retry_attempts: 1,
      retry_backoff_base_ms: 25,
    }
  }
}

// ============================================================================
// Other Feature Configurations
// ============================================================================

/// Search configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchConfig {
  pub enabled: bool,
  pub provider: String,
  pub meilisearch_url: String,
  pub meilisearch_api_key: String,
  pub async_indexing: bool,
  pub batch_size: usize,
  #[serde(default)]
  pub indexer: Option<AsyncIndexingConfig>,
}

/// Async indexing configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AsyncIndexingConfig {
  pub batch_size: usize,
  pub batch_timeout_ms: u64,
}

/// Messaging configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessagingConfig {
  pub enabled: bool,
  pub provider: String,
  pub nats_url: String,
  pub jetstream_enabled: bool,
}

/// Notification configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationConfig {
  pub in_app_enabled: bool,
  pub realtime_enabled: bool,
  pub max_per_user: i32,
  pub retention_days: i32,
  pub email_enabled: bool,
  pub smtp_host: String,
  pub smtp_port: u16,
  pub smtp_username: String,
  pub smtp_password: String,
  pub from_email: String,
  pub from_name: String,
  pub use_tls: bool,
}

/// Observability configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObservabilityConfig {
  pub log_level: String,
  pub log_format: String,
  pub log_to_file: bool,
  pub log_file_path: String,
  pub metrics_enabled: bool,
  pub metrics_bind_address: String,
  pub tracing_enabled: bool,
  pub service_name: String,
  pub service_version: String,
  pub environment: String,
}

/// Storage configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageConfig {
  pub path: String,
  pub url_prefix: String,
}

// ============================================================================
// Configuration Implementations
// ============================================================================

impl Default for CacheConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      redis_url: "redis://:fechatter_redis_pass@localhost:6379".to_string(),
      key_prefix: "fechatter".to_string(),
      default_ttl: 300,
      pool_size: 10,
      connection_timeout_ms: 5000,
      ttl: Duration::from_secs(300),
      cache_private: false,
      user_specific: false,
      max_size: 100 * 1024 * 1024, // 100MB
      compress: true,
      variants: vec![],
    }
  }
}

impl CacheConfig {
  /// Initialize runtime config fields
  pub fn init_runtime_fields(mut self) -> Self {
    self.ttl = Duration::from_secs(self.default_ttl);
    self
  }

  /// Create public content cache config
  pub fn public(ttl_secs: u64) -> Self {
    Self {
      default_ttl: ttl_secs,
      ttl: Duration::from_secs(ttl_secs),
      cache_private: false,
      user_specific: false,
      ..Default::default()
    }
  }

  /// Create user-specific cache config
  pub fn user_specific(ttl_secs: u64) -> Self {
    Self {
      default_ttl: ttl_secs,
      ttl: Duration::from_secs(ttl_secs),
      cache_private: true,
      user_specific: true,
      variants: vec![CacheVariant::UserId],
      ..Default::default()
    }
  }

  /// Create workspace-level cache config
  pub fn workspace_level(ttl_secs: u64) -> Self {
    Self {
      default_ttl: ttl_secs,
      ttl: Duration::from_secs(ttl_secs),
      cache_private: true,
      user_specific: false,
      variants: vec![CacheVariant::WorkspaceId],
      ..Default::default()
    }
  }

  /// Add cache variant
  pub fn with_variant(mut self, variant: CacheVariant) -> Self {
    self.variants.push(variant);
    self
  }

  /// Enable compression
  pub fn with_compression(mut self) -> Self {
    self.compress = true;
    self
  }

  /// Set max cache size
  pub fn with_max_size(mut self, max_size: usize) -> Self {
    self.max_size = max_size;
    self
  }

  /// Set cache key prefix
  pub fn with_key_prefix(mut self, prefix: String) -> Self {
    self.key_prefix = prefix;
    self
  }
}

// ============================================================================
// Configuration Loading - Production-Grade
// ============================================================================

#[derive(Debug, Error)]
pub enum ConfigError {
  #[error("Missing required environment variable: {0}")]
  MissingEnvVar(String),
  #[error("Invalid value for {field}: {value}")]
  InvalidValue { field: String, value: String },
  #[error("IO error: {0}")]
  Io(#[from] std::io::Error),
  #[error("YAML parsing error: {0}")]
  YamlParse(#[from] serde_yaml::Error),
  #[error("TOML parsing error: {0}")]
  TomlParse(#[from] toml::de::Error),
  #[error("JSON parsing error: {0}")]
  JsonParse(#[from] serde_json::Error),
}

impl AppConfig {
  /// Production-grade configuration loading - enhanced search only
  pub fn load() -> Result<Self> {
    // Phase 1: Environment variable override (highest priority)
    if let Ok(config_path) = env::var("FECHATTER_CONFIG") {
      println!("📍 Loading from FECHATTER_CONFIG: {}", config_path);
      match Self::from_file(&config_path) {
        Ok(config) => {
          println!("Configuration loaded successfully!");
          Self::print_config_summary(&config);
          return Ok(config);
        }
        Err(e) => {
          return Err(anyhow::anyhow!(
            "ERROR: Failed to load config from FECHATTER_CONFIG ({}): {}",
            config_path,
            e
          ));
        }
      }
    }

    // Phase 2: Enhanced search in multiple locations
    let search_locations = Self::get_production_search_locations();

    for (desc, path) in &search_locations {
      if path.exists() {
        println!("📁 Found config: {} ({})", path.display(), desc);
        match Self::from_file(path.to_str().unwrap()) {
          Ok(config) => {
            println!("Configuration loaded successfully!");
            Self::print_config_summary(&config);
            return Ok(config);
          }
          Err(e) => {
            println!("WARNING: Parse error in {}: {}", path.display(), e);
            continue;
          }
        }
      }
    }

    // Phase 3: Show helpful error with existing file suggestions only
    Self::show_helpful_error(&search_locations);
    anyhow::bail!("💥 No valid configuration file found!")
  }

  /// Get production-focused search locations - enhanced with Docker support
  fn get_production_search_locations() -> Vec<(String, PathBuf)> {
    let mut locations = Vec::new();

    // Priority 0: Docker container standard paths (highest priority after env var)
    locations.push((
      "Docker app config".to_string(),
      PathBuf::from("/app/config/chat.yml"),
    ));
    locations.push((
      "Docker app config (yaml)".to_string(),
      PathBuf::from("/app/config/chat.yaml"),
    ));
    locations.push((
      "Docker app root".to_string(),
      PathBuf::from("/app/chat.yml"),
    ));
    locations.push((
      "Docker etc config".to_string(),
      PathBuf::from("/etc/fechatter/chat.yml"),
    ));
    locations.push((
      "Docker etc config (yaml)".to_string(),
      PathBuf::from("/etc/fechatter/chat.yaml"),
    ));

    if let Ok(cwd) = env::current_dir() {
      // Priority 1: Current crate directory - check if we're in fechatter_server or if it exists
      let current_crate_dir =
        if cwd.file_name().and_then(|n| n.to_str()) == Some("fechatter_server") {
          // We're already in fechatter_server directory
          cwd.clone()
        } else {
          // Look for fechatter_server subdirectory
          cwd.join("fechatter_server")
        };

      if current_crate_dir.exists() {
        locations.push((
          "Current crate (fechatter_server)".to_string(),
          current_crate_dir.join("chat.yml"),
        ));
        locations.push((
          "Current crate alt (fechatter_server)".to_string(),
          current_crate_dir.join("fechatter.yml"),
        ));
      }

      // Priority 2: Current working directory (fallback)
      locations.push(("Current directory".to_string(), cwd.join("chat.yml")));
      locations.push((
        "Current directory alt".to_string(),
        cwd.join("fechatter.yml"),
      ));

      // Priority 3: Binary's directory analysis for deployment scenarios
      if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
          // Check if binary is in target directory structure
          let mut search_base = exe_dir;

          // Walk up to find project root containing fechatter_server/
          for _ in 0..5 {
            let potential_crate_dir = search_base.join("fechatter_server");
            if potential_crate_dir.exists() {
              locations.push((
                "Detected project: fechatter_server".to_string(),
                potential_crate_dir.join("chat.yml"),
              ));
              break;
            }
            if let Some(parent) = search_base.parent() {
              search_base = parent;
            } else {
              break;
            }
          }

          // Also check for config directory relative to binary
          locations.push((
            "Binary config directory".to_string(),
            exe_dir.join("config").join("chat.yml"),
          ));
          locations.push((
            "Binary config directory (yaml)".to_string(),
            exe_dir.join("config").join("chat.yaml"),
          ));
        }
      }

      // Priority 4: Other common project structure locations (lower priority)
      let other_subdirs = ["config", "configs", "fechatter_tests"];
      for subdir in &other_subdirs {
        let subdir_path = cwd.join(subdir);
        if subdir_path.exists() {
          locations.push((
            format!("Project structure ({})", subdir),
            subdir_path.join("chat.yml"),
          ));
          locations.push((
            format!("Project structure ({}) alt", subdir),
            subdir_path.join("fechatter.yml"),
          ));
        }
      }

      // Priority 5: Parent directory (deployment pattern)
      if let Some(parent) = cwd.parent() {
        locations.push(("Parent directory".to_string(), parent.join("chat.yml")));
      }
    }

    // Priority 6: System-wide configuration (lowest priority)
    if let Ok(home) = env::var("HOME") {
      locations.push((
        "User home".to_string(),
        PathBuf::from(home).join(".config/fechatter/chat.yml"),
      ));
    }

    locations.push((
      "System config".to_string(),
      PathBuf::from("/etc/fechatter/chat.yml"),
    ));

    locations
  }

  /// Load configuration from file (supports YAML, TOML, and JSON)
  pub fn from_file(path: &str) -> Result<Self, ConfigError> {
    let content = std::fs::read_to_string(path)?;

    if path.ends_with(".yml") || path.ends_with(".yaml") {
      Ok(serde_yaml::from_str(&content)?)
    } else if path.ends_with(".toml") {
      Ok(toml::from_str(&content)?)
    } else if path.ends_with(".json") {
      Ok(serde_json::from_str(&content)?)
    } else {
      // Try YAML first, then TOML, then JSON
      serde_yaml::from_str(&content)
        .map_err(ConfigError::YamlParse)
        .or_else(|_| toml::from_str(&content).map_err(ConfigError::TomlParse))
        .or_else(|_| serde_json::from_str(&content).map_err(ConfigError::JsonParse))
    }
  }

  /// Print configuration summary
  fn print_config_summary(config: &Self) {
    println!("Configuration Summary:");
    println!(
      "   Server: Port {}, DB: {}",
      config.server.port,
      Self::mask_db_url(&config.server.db_url)
    );
    println!(
      "   Cache: {}",
      if config.features.cache.enabled {
        "Enabled"
      } else {
        "ERROR: Disabled"
      }
    );
    println!(
      "   Search: {}",
      if config.features.search.enabled {
        "Enabled"
      } else {
        "ERROR: Disabled"
      }
    );
    println!(
      "   Messaging: {}",
      if config.features.messaging.enabled {
        "Enabled"
      } else {
        "ERROR: Disabled"
      }
    );
  }

  /// Mask sensitive parts of database URL
  fn mask_db_url(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
      if let Some(colon_pos) = url[..at_pos].rfind(':') {
        format!("{}:***@{}", &url[..colon_pos], &url[at_pos + 1..])
      } else {
        url.to_string()
      }
    } else {
      url.to_string()
    }
  }

  /// Show helpful error message when no config is found (simplified)
  fn show_helpful_error(search_locations: &[(String, PathBuf)]) {
    println!("\nSearched for configuration in:");
    for (desc, path) in search_locations {
      println!("   ERROR: {}: {}", desc, path.display());
    }

    println!("\nTo fix this:");
    println!("   1. 📋 Set FECHATTER_CONFIG environment variable to your config file path");
    println!("   2. 📄 Use supported formats: .yml, .yaml, .toml, .json");

    // Enhanced suggestions based on discovered existing files only
    if let Ok(cwd) = env::current_dir() {
      // Check for existing config files in subdirectories
      let mut found_configs = Vec::new();
      for subdir in ["fechatter_server", "fechatter_tests", "fixtures", "config"] {
        let config_path = cwd.join(subdir).join("chat.yml");
        if config_path.exists() {
          found_configs.push(config_path);
        }
        let alt_config_path = cwd.join(subdir).join("fechatter.yml");
        if alt_config_path.exists() {
          found_configs.push(alt_config_path);
        }
      }

      if !found_configs.is_empty() {
        println!("\n✨ Found existing config files to copy:");
        for config in &found_configs {
          println!("   📁 cp {} ./chat.yml", config.display());
        }

        println!("\nQuick Fix Command:");
        if let Some(first_config) = found_configs.first() {
          println!("   cp {} ./chat.yml", first_config.display());
        }
      } else {
        println!("\nWARNING: No existing config files found in common locations.");
        println!("   Please ensure you have a valid chat.yml file.");
      }
    }
  }
}

// ============================================================================
// Token Configuration Provider Implementation
// ============================================================================

impl TokenConfigProvider for AuthConfig {
  fn get_encoding_key_pem(&self) -> &str {
    &self.sk
  }

  fn get_decoding_key_pem(&self) -> &str {
    &self.pk
  }
}

// ============================================================================
// Environment and Extended Configuration Types
// ============================================================================

/// Application environment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Environment {
  Development,
  Testing,
  Staging,
  Production,
}

impl Default for Environment {
  fn default() -> Self {
    Environment::Development
  }
}

impl Environment {
  /// Load environment from environment variable
  pub fn from_env() -> Self {
    match env::var("FECHATTER_ENV")
      .unwrap_or_default()
      .to_lowercase()
      .as_str()
    {
      "production" | "prod" => Environment::Production,
      "staging" | "stage" => Environment::Staging,
      "testing" | "test" => Environment::Testing,
      _ => Environment::Development,
    }
  }

  /// Check if production environment
  pub fn is_production(&self) -> bool {
    matches!(self, Environment::Production)
  }

  /// Check if development environment
  pub fn is_development(&self) -> bool {
    matches!(self, Environment::Development)
  }

  /// Check if testing environment
  pub fn is_testing(&self) -> bool {
    matches!(self, Environment::Testing)
  }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
  /// Database URL
  pub url: String,
  /// Maximum connections
  pub max_connections: u32,
  /// Minimum connections
  pub min_connections: u32,
  /// Connection timeout (milliseconds)
  pub connect_timeout_ms: u64,
  /// Idle timeout (milliseconds)
  pub idle_timeout_ms: u64,
  /// Maximum lifetime (milliseconds)
  pub max_lifetime_ms: u64,
  /// Enable SSL
  pub ssl_enabled: bool,
  /// SSL certificate path
  pub ssl_cert_path: Option<String>,
  /// Enable query logging
  pub query_logging: bool,
  /// Slow query threshold (milliseconds)
  pub slow_query_threshold_ms: u64,
  /// Enable pool monitoring
  pub pool_monitoring: bool,
  /// Database name
  pub database_name: String,
  /// Auto migrate
  pub auto_migrate: bool,
}

impl Default for DatabaseConfig {
  fn default() -> Self {
    Self {
      url: "postgresql://localhost/fechatter".to_string(),
      max_connections: 20,
      min_connections: 5,
      connect_timeout_ms: 30000,
      idle_timeout_ms: 600000,  // 10 minutes
      max_lifetime_ms: 3600000, // 1 hour
      ssl_enabled: false,
      ssl_cert_path: None,
      query_logging: false,
      slow_query_threshold_ms: 1000, // 1 second
      pool_monitoring: true,
      database_name: "fechatter".to_string(),
      auto_migrate: true,
    }
  }
}

impl DatabaseConfig {
  /// Create development configuration
  pub fn development() -> Self {
    Self {
      url: "postgresql://localhost/fechatter_dev".to_string(),
      max_connections: 10,
      min_connections: 2,
      query_logging: true,
      slow_query_threshold_ms: 500,
      database_name: "fechatter_dev".to_string(),
      auto_migrate: true,
      ..Default::default()
    }
  }

  /// Create production configuration
  pub fn production() -> Self {
    Self {
      max_connections: 50,
      min_connections: 10,
      connect_timeout_ms: 10000,
      idle_timeout_ms: 300000, // 5 minutes
      ssl_enabled: true,
      query_logging: false,
      slow_query_threshold_ms: 2000, // 2 seconds
      auto_migrate: false,
      ..Default::default()
    }
  }

  /// Create testing configuration
  pub fn testing() -> Self {
    Self {
      url: "postgresql://localhost/fechatter_test".to_string(),
      max_connections: 5,
      min_connections: 1,
      connect_timeout_ms: 5000,
      query_logging: true,
      slow_query_threshold_ms: 100,
      database_name: "fechatter_test".to_string(),
      auto_migrate: true,
      ..Default::default()
    }
  }

  /// Convert timeouts to Duration for compatibility
  pub fn connect_timeout(&self) -> Duration {
    Duration::from_millis(self.connect_timeout_ms)
  }

  pub fn idle_timeout(&self) -> Duration {
    Duration::from_millis(self.idle_timeout_ms)
  }

  pub fn max_lifetime(&self) -> Duration {
    Duration::from_millis(self.max_lifetime_ms)
  }

  pub fn slow_query_threshold(&self) -> Duration {
    Duration::from_millis(self.slow_query_threshold_ms)
  }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
  /// Log level
  pub level: String,
  /// Enable structured logging
  pub structured: bool,
  /// Log output format
  pub format: LogFormat,
  /// Log file path
  pub file_path: Option<String>,
  /// Enable log rotation
  pub rotation_enabled: bool,
  /// Maximum file size
  pub max_file_size: usize,
  /// Maximum number of files
  pub max_files: usize,
}

/// Log format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
  Json,
  Pretty,
  Compact,
}

impl Default for LoggingConfig {
  fn default() -> Self {
    Self {
      level: "info".to_string(),
      structured: false,
      format: LogFormat::Pretty,
      file_path: None,
      rotation_enabled: false,
      max_file_size: 0,
      max_files: 0,
    }
  }
}

impl LoggingConfig {
  /// Create development configuration
  pub fn development() -> Self {
    Self {
      level: "debug".to_string(),
      structured: false,
      format: LogFormat::Pretty,
      file_path: None,
      rotation_enabled: false,
      max_file_size: 0,
      max_files: 0,
    }
  }

  /// Create production configuration
  pub fn production() -> Self {
    Self {
      level: "warn".to_string(),
      structured: true,
      format: LogFormat::Json,
      file_path: Some("/var/log/fechatter/app.log".to_string()),
      rotation_enabled: true,
      max_file_size: 500 * 1024 * 1024, // 500MB
      max_files: 30,
    }
  }
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
  /// Enable metrics collection
  pub metrics_enabled: bool,
  /// Metrics port
  pub metrics_port: u16,
  /// Enable health check
  pub health_check_enabled: bool,
  /// Health check path
  pub health_check_path: String,
  /// Enable distributed tracing
  pub tracing_enabled: bool,
  /// Tracing sample rate
  pub tracing_sample_rate: f64,
}

impl Default for MonitoringConfig {
  fn default() -> Self {
    Self {
      metrics_enabled: true,
      metrics_port: 9090,
      health_check_enabled: true,
      health_check_path: "/health".to_string(),
      tracing_enabled: false,
      tracing_sample_rate: 0.01,
    }
  }
}

impl MonitoringConfig {
  /// Create development configuration
  pub fn development() -> Self {
    Self {
      metrics_enabled: true,
      metrics_port: 9090,
      health_check_enabled: true,
      health_check_path: "/health".to_string(),
      tracing_enabled: true,
      tracing_sample_rate: 1.0,
    }
  }

  /// Create production configuration
  pub fn production() -> Self {
    Self {
      metrics_enabled: true,
      metrics_port: 9090,
      health_check_enabled: true,
      health_check_path: "/health".to_string(),
      tracing_enabled: true,
      tracing_sample_rate: 0.01,
    }
  }
}
