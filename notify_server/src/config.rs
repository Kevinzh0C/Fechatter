use anyhow::{Result, bail};
use fechatter_core::models::jwt::TokenConfigProvider;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::analytics::publisher::AnalyticsConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
  pub server: ServerConfig,
  pub auth: AuthConfig,
  pub messaging: MessagingConfig,
  pub search: SearchConfig,
  pub notification: NotificationConfig,
  pub analytics: AnalyticsConfig,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub security: Option<SecurityConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthConfig {
  pub pk: String,
  pub sk: String,
  pub token_expiration: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
  pub port: u16,
  pub db_url: String,
  pub request_timeout_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessagingConfig {
  pub enabled: bool,
  pub provider: String,
  pub nats: NatsConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NatsConfig {
  pub url: String,
  pub auth: NatsAuthConfig,
  pub subscription_subjects: Vec<String>,
  pub jetstream: JetStreamConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NatsAuthConfig {
  pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JetStreamConfig {
  pub enabled: bool,
  pub stream: String,
  pub storage: String,
  pub max_bytes: u64,
  pub max_msg_size: u64,
  pub max_age: u64,
  pub consumers: ConsumersConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConsumersConfig {
  pub notification_processor: ConsumerConfig,
  pub realtime_processor: Option<ConsumerConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConsumerConfig {
  pub name: String,
  pub filter_subjects: Vec<String>,
  pub ack_wait: String,
  pub max_deliver: u32,
  pub max_batch: u32,
  pub idle_heartbeat: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchConfig {
  pub enabled: bool,
  pub provider: String,
  pub meilisearch: MeilisearchConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MeilisearchConfig {
  pub url: String,
  pub api_key: String,
  pub connection_timeout_ms: u64,
  pub request_timeout_ms: u64,
  pub indexes: IndexesConfig,
  pub settings: MeilisearchSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexesConfig {
  pub messages: IndexConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexConfig {
  pub name: String,
  pub primary_key: String,
  pub searchable_fields: Vec<String>,
  pub displayed_fields: Vec<String>,
  pub filterable_fields: Vec<String>,
  pub sortable_fields: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MeilisearchSettings {
  pub pagination_limit: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationConfig {
  pub delivery: DeliveryConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeliveryConfig {
  pub web: WebDeliveryConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebDeliveryConfig {
  pub enabled: bool,
  pub sse_enabled: bool,
  pub connection_timeout_ms: u64,
  pub heartbeat_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
  pub hmac_secret: Option<String>,
  pub verify_signatures: bool,
}

impl AppConfig {
  /// Production-level config loader
  ///
  /// Features:
  /// - Smart path search
  /// - Environment variable overrides
  /// - Config validation
  /// - Default value support
  /// - Detailed error messages
  /// - Security checks
  pub fn load() -> Result<Self> {
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    // 1. Find config file
    let config_path = Self::find_config_file()?;

    // 2. Check file security
    Self::validate_config_security(&config_path)?;

    // 3. Load and parse config
    let mut config = Self::load_from_file(&config_path)?;

    // 4. Apply environment variable overrides
    Self::apply_env_overrides(&mut config)?;

    // 5. Validate config integrity
    Self::validate_config(&config)?;

    // 6. Apply defaults
    Self::apply_defaults(&mut config);

    eprintln!(
      "✓ notify_server: Configuration loaded successfully from: {}",
      config_path.display()
    );

    Ok(config)
  }

  /// Find config file, search by priority
  fn find_config_file() -> Result<PathBuf> {
    use std::path::PathBuf;

    let search_paths = vec![
      // 1. Path specified by env var (highest priority)
      std::env::var("NOTIFY_CONFIG").ok().map(PathBuf::from),
      
      // 2. Docker container standard paths (highest priority for containers)
      Some(PathBuf::from("/app/config/notify.yml")),
      Some(PathBuf::from("/app/config/notify.yaml")),
      Some(PathBuf::from("/app/notify.yml")),
      Some(PathBuf::from("/etc/fechatter/notify.yml")),
      Some(PathBuf::from("/etc/fechatter/notify.yaml")),
      
      // 3. Current working directory
      Some(PathBuf::from("notify.yml")),
      Some(PathBuf::from("notify.yaml")),
      
      // 4. notify_server dir (if running from project root)
      Some(PathBuf::from("notify_server/notify.yml")),
      Some(PathBuf::from("notify_server/notify.yaml")),
      
      // 5. Parent dir (if running inside notify_server dir)
      Some(PathBuf::from("../notify.yml")),
      Some(PathBuf::from("../notify.yaml")),
      
      // 6. Binary directory config (for deployment scenarios)
      std::env::current_exe().ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .map(|exe_dir| exe_dir.join("config").join("notify.yml")),
      std::env::current_exe().ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .map(|exe_dir| exe_dir.join("config").join("notify.yaml")),
    ];

    let valid_paths: Vec<PathBuf> = search_paths.into_iter().filter_map(|p| p).collect();

    for path in &valid_paths {
      if path.exists() && path.is_file() {
        eprintln!("📁 Found notify config: {}", path.display());
        return Ok(path.clone());
      }
    }

    let searched_paths: Vec<String> = valid_paths
      .iter()
      .map(|p| format!("  - {}", p.display()))
      .collect();

    bail!(
      "Configuration file 'notify.yml' not found. Searched in:\n{}\n\n\
      Solutions:\n\
      1. Create notify.yml in the current directory\n\
      2. Set NOTIFY_CONFIG environment variable to the config file path\n\
      3. 🐳 For Docker: mount config to /app/config/notify.yml\n\
      4. Ensure you're running from the correct directory",
      searched_paths.join("\n")
    );
  }

  /// Validate config file security
  fn validate_config_security(path: &std::path::Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::metadata(path)
      .map_err(|e| anyhow::anyhow!("Failed to read config file metadata: {}", e))?;

    let permissions = metadata.permissions();
    let mode = permissions.mode();

    // Check file permissions: should be owner read/write only (600) or owner/group read (640)
    if mode & 0o077 != 0 {
      eprintln!(
        "⚠️  WARNING: Config file {} has overly permissive permissions ({:o}). \
        Consider using 'chmod 600 {}' for better security.",
        path.display(),
        mode & 0o777,
        path.display()
      );
    }

    Ok(())
  }

  /// Load config from file
  fn load_from_file(path: &std::path::Path) -> Result<Self> {
    let file = std::fs::File::open(path)
      .map_err(|e| anyhow::anyhow!("Failed to open config file {}: {}", path.display(), e))?;

    let config: Self = serde_yaml::from_reader(file)
      .map_err(|e| anyhow::anyhow!("Failed to parse config file {}: {}", path.display(), e))?;

    Ok(config)
  }

  /// Apply environment variable overrides
  fn apply_env_overrides(config: &mut Self) -> Result<()> {
    // Database URL override
    if let Ok(db_url) = std::env::var("DATABASE_URL") {
      eprintln!("📝 notify_server: Using DATABASE_URL from environment");
      config.server.db_url = db_url;
    }

    // Port override
    if let Ok(port_str) = std::env::var("NOTIFY_PORT") {
      match port_str.parse::<u16>() {
        Ok(port) => {
          eprintln!(
            "📝 notify_server: Using NOTIFY_PORT from environment: {}",
            port
          );
          config.server.port = port;
        }
        Err(e) => {
          eprintln!(
            "⚠️  WARNING: Invalid NOTIFY_PORT value '{}': {}",
            port_str, e
          );
        }
      }
    }

    // NATS URL override
    if let Ok(nats_url) = std::env::var("NATS_URL") {
      eprintln!("📝 notify_server: Using NATS_URL from environment");
      config.messaging.nats.url = nats_url;
    }

    // Log level
    if let Ok(log_level) = std::env::var("RUST_LOG") {
      eprintln!(
        "📝 notify_server: Using RUST_LOG from environment: {}",
        log_level
      );
    }

    Ok(())
  }

  /// Validate config integrity
  fn validate_config(config: &Self) -> Result<()> {
    // Validate port range
    if config.server.port == 0 || config.server.port > 65535 {
      bail!(
        "Invalid server port: {}. Must be between 1 and 65535",
        config.server.port
      );
    }

    // Validate database URL
    if config.server.db_url.is_empty() {
      bail!("Database URL cannot be empty");
    }

    if !config.server.db_url.starts_with("postgresql://")
      && !config.server.db_url.starts_with("postgres://")
    {
      bail!("Database URL must start with 'postgresql://' or 'postgres://'");
    }

    // Validate JWT keys
    if config.auth.sk.is_empty() || config.auth.pk.is_empty() {
      bail!("JWT private key (sk) and public key (pk) cannot be empty");
    }

    // Validate token expiration
    if config.auth.token_expiration <= 0 {
      bail!(
        "Token expiration must be positive, got: {}",
        config.auth.token_expiration
      );
    }

    // Validate timeout
    if config.server.request_timeout_ms == 0 {
      bail!("Request timeout cannot be zero");
    }

    // Validate NATS config
    if config.messaging.enabled {
      if config.messaging.nats.url.is_empty() {
        bail!("NATS URL cannot be empty when messaging is enabled");
      }

      if !config.messaging.nats.url.starts_with("nats://") {
        bail!("NATS URL must start with 'nats://'");
      }
    }

    Ok(())
  }

  /// Apply default values
  fn apply_defaults(config: &mut Self) {
    // Set reasonable default timeout
    if config.server.request_timeout_ms == 0 {
      config.server.request_timeout_ms = 30000; // 30 seconds
    }

    // Set default token expiration
    if config.auth.token_expiration == 0 {
      config.auth.token_expiration = 1800; // 30 minutes
    }

    // Set JetStream defaults
    if config.messaging.nats.jetstream.max_bytes == 0 {
      config.messaging.nats.jetstream.max_bytes = 1073741824; // 1GB
    }

    if config.messaging.nats.jetstream.max_msg_size == 0 {
      config.messaging.nats.jetstream.max_msg_size = 1048576; // 1MB
    }

    if config.messaging.nats.jetstream.max_age == 0 {
      config.messaging.nats.jetstream.max_age = 86400; // 24 hours
    }

    // Set notification defaults
    if config.notification.delivery.web.connection_timeout_ms == 0 {
      config.notification.delivery.web.connection_timeout_ms = 60000; // 60 seconds
    }

    if config.notification.delivery.web.heartbeat_interval_ms == 0 {
      config.notification.delivery.web.heartbeat_interval_ms = 30000; // 30 seconds
    }
  }

  /// Get config summary (for logging, no sensitive info)
  pub fn get_summary(&self) -> String {
    format!(
      "notify_server config: port={}, messaging={}, search={}, db={}",
      self.server.port,
      if self.messaging.enabled {
        "enabled"
      } else {
        "disabled"
      },
      if self.search.enabled {
        "enabled"
      } else {
        "disabled"
      },
      if self.server.db_url.contains("localhost") {
        "localhost"
      } else {
        "remote"
      }
    )
  }

  /// Validate if config is production-ready
  pub fn validate_production_readiness(&self) -> Result<()> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    // Check for default key
    if self
      .auth
      .sk
      .contains("MC4CAQAwBQYDK2VwBCIEIP/S+etN7RQJctehWKkdjgnrtQ0AUDIMkCnYS4Zk8RFR")
    {
      errors.push("Using default JWT private key in production is insecure");
    }

    // Check database connection
    if self.server.db_url.contains("postgres:postgres") {
      warnings.push("Using default database credentials");
    }

    if self.server.db_url.contains("localhost") {
      warnings.push("Database is localhost - ensure this is intended for production");
    }

    // Check security config
    if let Some(security) = &self.security {
      if !security.verify_signatures {
        warnings.push("Signature verification is disabled");
      }
    } else {
      warnings.push("Security configuration is missing");
    }

    // Print warnings
    for warning in &warnings {
      eprintln!("⚠️  Production Warning: {}", warning);
    }

    // If there are errors, return failure
    if !errors.is_empty() {
      bail!("Production readiness check failed:\n{}", errors.join("\n"));
    }

    if warnings.is_empty() {
      eprintln!("✅ Configuration appears production-ready");
    }

    Ok(())
  }
}

impl TokenConfigProvider for AuthConfig {
  fn get_encoding_key_pem(&self) -> &str {
    &self.sk
  }

  fn get_decoding_key_pem(&self) -> &str {
    &self.pk
  }
}
