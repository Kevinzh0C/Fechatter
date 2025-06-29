use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
  pub server: ServerConfig,
  #[serde(default)]
  pub features: FeaturesConfig,
  #[serde(default)]
  pub integrations: IntegrationsConfig,
  #[serde(default)]
  pub observability: ObservabilityConfig,
  #[serde(default)]
  pub security: SecurityConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
  pub port: u16,
  pub clickhouse: ClickHouseConfig,
  pub base_dir: PathBuf,
  #[serde(default = "default_request_timeout")]
  pub request_timeout_ms: u64,
  #[serde(default = "default_max_concurrent_requests")]
  pub max_concurrent_requests: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClickHouseConfig {
  pub host: String,
  pub port: u16,
  pub database: String,
  pub user: String,
  #[serde(default)]
  pub password: String,
  pub http_url: String,
  pub native_url: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FeaturesConfig {
  #[serde(default)]
  pub analytics: AnalyticsFeatureConfig,
  #[serde(default)]
  pub streaming: StreamingFeatureConfig,
  #[serde(default)]
  pub aggregation: AggregationFeatureConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsFeatureConfig {
  #[serde(default = "default_true")]
  pub enabled: bool,
  #[serde(default = "default_batch_size")]
  pub batch_size: usize,
  #[serde(default = "default_flush_interval")]
  pub flush_interval_seconds: u64,
  #[serde(default = "default_retry_attempts")]
  pub max_retry_attempts: u32,
  #[serde(default = "default_retry_backoff")]
  pub retry_backoff_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamingFeatureConfig {
  #[serde(default = "default_true")]
  pub enabled: bool,
  #[serde(default = "default_buffer_size")]
  pub buffer_size: usize,
  #[serde(default = "default_flush_threshold")]
  pub flush_threshold: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AggregationFeatureConfig {
  #[serde(default = "default_true")]
  pub enabled: bool,
  #[serde(default = "default_aggregation_interval")]
  pub interval_seconds: u64,
  #[serde(default = "default_batch_size")]
  pub batch_size: usize,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct IntegrationsConfig {
  #[serde(default)]
  pub messaging: MessagingConfig,
  #[serde(default)]
  pub cache: CacheConfig,
  #[serde(default)]
  pub postgres: PostgresConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessagingConfig {
  #[serde(default)]
  pub enabled: bool,
  #[serde(default)]
  pub nats_url: String,
  #[serde(default)]
  pub jetstream_enabled: bool,
  #[serde(default)]
  pub jetstream: JetStreamConfig,
  #[serde(default)]
  pub subjects: MessagingSubjects,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JetStreamConfig {
  pub stream_name: String,
  pub max_bytes: u64,      // Stream storage limit in bytes
  pub max_messages: u64,   // Maximum number of messages
  pub max_age_hours: u64,  // Maximum age in hours
  pub storage_type: String,
  pub num_replicas: u8,
  pub retention_policy: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MessagingSubjects {
  #[serde(default)]
  pub analytics_events: String,
  #[serde(default)]
  pub analytics_batch: String,
  #[serde(default)]
  pub user_events: String,
  #[serde(default)]
  pub chat_events: String,
  #[serde(default)]
  pub message_events: String,
  #[serde(default)]
  pub bot_events: String,
  #[serde(default)]
  pub error_events: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheConfig {
  #[serde(default)]
  pub enabled: bool,
  #[serde(default)]
  pub redis_url: String,
  #[serde(default)]
  pub key_prefix: String,
  #[serde(default = "default_cache_ttl")]
  pub default_ttl: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostgresConfig {
  #[serde(default)]
  pub enabled: bool,
  #[serde(default)]
  pub database_url: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ObservabilityConfig {
  #[serde(default = "default_log_level")]
  pub log_level: String,
  #[serde(default = "default_log_format")]
  pub log_format: String,
  #[serde(default)]
  pub log_to_file: bool,
  #[serde(default)]
  pub metrics: MetricsConfig,
  #[serde(default)]
  pub health_check: HealthCheckConfig,
  #[serde(default)]
  pub performance: PerformanceConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsConfig {
  #[serde(default = "default_true")]
  pub enabled: bool,
  #[serde(default = "default_metrics_port")]
  pub port: u16,
  #[serde(default = "default_metrics_path")]
  pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckConfig {
  #[serde(default = "default_true")]
  pub enabled: bool,
  #[serde(default = "default_health_path")]
  pub path: String,
  #[serde(default = "default_health_timeout")]
  pub timeout_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceConfig {
  #[serde(default = "default_true")]
  pub track_query_performance: bool,
  #[serde(default = "default_slow_query_threshold")]
  pub slow_query_threshold_ms: u64,
  #[serde(default)]
  pub enable_profiling: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SecurityConfig {
  #[serde(default)]
  pub auth: AuthConfig,
  #[serde(default)]
  pub cors: CorsConfig,
  #[serde(default)]
  pub rate_limiting: RateLimitingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
  #[serde(default)]
  pub enabled: bool,
  #[serde(default)]
  pub jwt_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CorsConfig {
  #[serde(default = "default_true")]
  pub enabled: bool,
  #[serde(default)]
  pub allow_origins: Vec<String>,
  #[serde(default)]
  pub allow_methods: Vec<String>,
  #[serde(default)]
  pub allow_headers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RateLimitingConfig {
  #[serde(default = "default_true")]
  pub enabled: bool,
  #[serde(default = "default_requests_per_minute")]
  pub requests_per_minute: u32,
  #[serde(default = "default_burst_size")]
  pub burst_size: u32,
}

// Default value functions for serde
fn default_true() -> bool { true }
fn default_request_timeout() -> u64 { 30000 }
fn default_max_concurrent_requests() -> usize { 100 }
fn default_batch_size() -> usize { 100 }
fn default_flush_interval() -> u64 { 30 }
fn default_retry_attempts() -> u32 { 3 }
fn default_retry_backoff() -> u64 { 1000 }
fn default_buffer_size() -> usize { 1000 }
fn default_flush_threshold() -> usize { 500 }
fn default_aggregation_interval() -> u64 { 300 }
fn default_cache_ttl() -> u64 { 3600 }
fn default_log_level() -> String { "info".to_string() }
fn default_log_format() -> String { "json".to_string() }
fn default_metrics_port() -> u16 { 9091 }
fn default_metrics_path() -> String { "/metrics".to_string() }
fn default_health_path() -> String { "/health".to_string() }
fn default_health_timeout() -> u64 { 5000 }
fn default_slow_query_threshold() -> u64 { 1000 }
fn default_requests_per_minute() -> u32 { 1000 }
fn default_burst_size() -> u32 { 100 }

// Implement Default for configs that need custom defaults
impl Default for AnalyticsFeatureConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      batch_size: 100,
      flush_interval_seconds: 30,
      max_retry_attempts: 3,
      retry_backoff_ms: 1000,
    }
  }
}

impl Default for StreamingFeatureConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      buffer_size: 1000,
      flush_threshold: 500,
    }
  }
}

impl Default for AggregationFeatureConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      interval_seconds: 300,
      batch_size: 1000,
    }
  }
}

impl Default for MessagingConfig {
  fn default() -> Self {
    Self {
      enabled: false,
      nats_url: "nats://localhost:4222".to_string(),
      jetstream_enabled: true,
      jetstream: JetStreamConfig::default(),
      subjects: MessagingSubjects::default(),
    }
  }
}

impl Default for CacheConfig {
  fn default() -> Self {
    Self {
      enabled: false,
      redis_url: "redis://:fechatter_redis_pass@localhost:6379".to_string(),
      key_prefix: "fechatter:analytics".to_string(),
      default_ttl: 3600,
    }
  }
}

impl Default for PostgresConfig {
  fn default() -> Self {
    Self {
      enabled: false,
      database_url: String::new(),
    }
  }
}

impl Default for MetricsConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      port: 9091,
      path: "/metrics".to_string(),
    }
  }
}

impl Default for HealthCheckConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      path: "/health".to_string(),
      timeout_ms: 5000,
    }
  }
}

impl Default for PerformanceConfig {
  fn default() -> Self {
    Self {
      track_query_performance: true,
      slow_query_threshold_ms: 1000,
      enable_profiling: false,
    }
  }
}

impl Default for AuthConfig {
  fn default() -> Self {
    Self {
      enabled: false,
      jwt_secret: String::new(),
    }
  }
}

impl Default for CorsConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      allow_origins: vec![
        "http://localhost:1420".to_string(),
        "http://127.0.0.1:1420".to_string(),
        "http://localhost:3000".to_string(),
      ],
      allow_methods: vec![
        "GET".to_string(),
        "POST".to_string(),
        "PUT".to_string(),
        "DELETE".to_string(),
        "OPTIONS".to_string(),
      ],
      allow_headers: vec![
        "Content-Type".to_string(),
        "Authorization".to_string(),
      ],
    }
  }
}

impl Default for RateLimitingConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      requests_per_minute: 1000,
      burst_size: 100,
    }
  }
}

impl Default for JetStreamConfig {
  fn default() -> Self {
    Self {
      stream_name: "ANALYTICS".to_string(),
      max_bytes: 1 * 1024 * 1024 * 1024, // 1GB
      max_messages: 10_000_000,           // 10M messages
      max_age_hours: 24 * 30,             // 30 days
      storage_type: "file".to_string(),
      num_replicas: 1,
      retention_policy: "limits".to_string(),
    }
  }
}

impl AppConfig {
  /// Load configuration from file with fallback locations and environment override
  pub fn load() -> Result<Self> {
    // Enhanced configuration loading with Docker container support
    let config_attempts = vec![
      // Priority 1: Environment variable override
      env::var("ANALYTICS_CONFIG").ok().and_then(|path| {
        println!("📍 Loading from ANALYTICS_CONFIG: {}", path);
        match Self::from_file(&path) {
          Ok(config) => {
            println!("Configuration loaded successfully from env var!");
            Some(config)
          }
          Err(e) => {
            eprintln!("ERROR: Failed to load config from ANALYTICS_CONFIG ({}): {}", path, e);
            None
          }
        }
      }),
      
      // Priority 2: Docker container standard paths
      Self::try_load_from_path("/app/config/analytics.yml"),
      Self::try_load_from_path("/app/config/analytics.yaml"),
      Self::try_load_from_path("/app/analytics.yml"),
      
      // Priority 2.5: Analytics server subdirectory (Docker compatibility)
      Self::try_load_from_path("/app/analytics_server/analytics.yml"),
      Self::try_load_from_path("/app/analytics_server/analytics.yaml"),
      
      Self::try_load_from_path("/etc/fechatter/analytics.yml"),
      Self::try_load_from_path("/etc/fechatter/analytics.yaml"),
      
      // Priority 3: Current directory
      Self::try_load_from_path("analytics.yml"),
      Self::try_load_from_path("analytics.yaml"),
      
      // Priority 4: Traditional paths
      Self::try_load_from_path("/etc/config/analytics.yml"),
      Self::try_load_from_path("/etc/config/analytics.yaml"),
      
      // Priority 5: Binary directory config
      env::current_exe().ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .and_then(|exe_dir| {
          Self::try_load_from_path(&exe_dir.join("config").join("analytics.yml").to_string_lossy())
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
    bail!("💥 Analytics configuration file not found in any expected location!")
  }

  /// Try to load configuration from a specific path
  fn try_load_from_path(path: &str) -> Option<Self> {
    match std::fs::File::open(path) {
      Ok(file) => {
        println!("📁 Found config file: {}", path);
        match serde_yaml::from_reader(file) {
          Ok(config) => {
            println!("Analytics configuration loaded successfully from: {}", path);
            Some(config)
          }
          Err(e) => {
            eprintln!("WARNING: Parse error in {}: {}", path, e);
            None
          }
        }
      }
      Err(_) => None, // File doesn't exist, continue searching
    }
  }

  /// Load configuration from a file reader (legacy compatibility)
  fn from_file(path: &str) -> Result<Self> {
    let file = std::fs::File::open(path)?;
    Ok(serde_yaml::from_reader(file)?)
  }

  /// Show helpful error message with search locations
  fn show_search_error() {
    eprintln!("\nSearched for analytics configuration in:");
    eprintln!("   ERROR: Environment: ANALYTICS_CONFIG");
    eprintln!("   ERROR: Docker paths: /app/config/analytics.yml, /app/analytics.yml");
    eprintln!("   ERROR: Container paths: /etc/fechatter/analytics.yml");
    eprintln!("   ERROR: Current directory: analytics.yml");
    eprintln!("   ERROR: System paths: /etc/config/analytics.yml");
    eprintln!("");
    eprintln!("To fix this:");
    eprintln!("   1. 📋 Set ANALYTICS_CONFIG environment variable");
    eprintln!("   2. 📄 Place analytics.yml in current directory");
    eprintln!("   3. 🐳 For Docker: mount config to /app/config/analytics.yml");
  }

  /// Get NATS configuration
  pub fn nats(&self) -> &MessagingConfig {
    &self.integrations.messaging
  }

  /// Get JetStream configuration
  pub fn jetstream(&self) -> &JetStreamConfig {
    &self.integrations.messaging.jetstream
  }

  /// Get analytics subjects from configuration
  pub fn analytics_subjects(&self) -> Vec<String> {
    let subjects = &self.integrations.messaging.subjects;
    vec![
      subjects.analytics_events.clone(),
      subjects.analytics_batch.clone(),
      subjects.user_events.clone(),
      subjects.chat_events.clone(),
      subjects.message_events.clone(),
      subjects.bot_events.clone(),
      subjects.error_events.clone(),
    ].into_iter()
     .filter(|s| !s.is_empty())
     .collect()
  }

  /// Check if NATS messaging is enabled
  pub fn is_nats_enabled(&self) -> bool {
    self.integrations.messaging.enabled && self.integrations.messaging.jetstream_enabled
  }

  /// Get database URL for ClickHouse connection
  pub fn get_database_url(&self) -> &str {
    &self.server.clickhouse.http_url
  }

  /// Get database name
  pub fn get_database_name(&self) -> &str {
    &self.server.clickhouse.database
  }

  /// Get database user
  pub fn get_database_user(&self) -> Option<&str> {
    if self.server.clickhouse.user.is_empty() {
      None
    } else {
      Some(&self.server.clickhouse.user)
    }
  }

  /// Get database password
  pub fn get_database_password(&self) -> Option<&str> {
    if self.server.clickhouse.password.is_empty() {
      None
    } else {
      Some(&self.server.clickhouse.password)
    }
  }
}
