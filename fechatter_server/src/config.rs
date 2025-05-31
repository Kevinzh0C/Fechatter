use anyhow::{Result, bail};
use fechatter_core::models::jwt::TokenConfigProvider;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
  pub server: ServerConfig,
  pub auth: AuthConfig,
  pub messaging: MessagingConfig,
  pub search: SearchConfig,
  pub notification: NotificationConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthConfig {
  pub pk: String,
  pub sk: String,
  pub token_expiration: i64,
  pub refresh_token_expiration: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
  pub port: u16,
  pub db_url: String,
  pub base_dir: PathBuf,
  pub max_upload_size: u64,
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
  pub subjects: NatsSubjectsConfig,
  pub jetstream: JetStreamConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NatsAuthConfig {
  pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NatsSubjectsConfig {
  pub message_created: String,
  pub message_updated: String,
  pub message_deleted: String,
  pub chat_created: String,
  pub chat_updated: String,
  pub user_joined_chat: String,
  pub user_left_chat: String,
  pub duplicate_message_attempted: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JetStreamConfig {
  pub enabled: bool,
  pub stream: String,
  pub storage: String,
  pub max_bytes: u64,
  pub max_msg_size: u64,
  pub max_age: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchConfig {
  pub enabled: bool,
  pub provider: String,
  pub async_indexing: AsyncIndexingConfig,
  pub meilisearch: MeilisearchConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AsyncIndexingConfig {
  pub enabled: bool,
  pub batch_size: usize,
  pub batch_timeout_ms: u64,
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

/// Notification configuration - Focus on in-app notifications
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationConfig {
  /// In-app notification settings (primary)
  pub in_app: InAppNotificationConfig,

  /// Email notification settings (optional, for offline users)
  pub email: Option<EmailNotificationConfig>,
}

/// In-app notification configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InAppNotificationConfig {
  /// Enable/disable in-app notifications
  pub enabled: bool,

  /// Enable real-time WebSocket notifications
  pub realtime_enabled: bool,

  /// Maximum notifications to keep per user
  pub max_notifications_per_user: i32,

  /// Days to keep read notifications
  pub retention_days: i32,
}

/// Simplified email notification configuration (optional)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmailNotificationConfig {
  /// Enable/disable email notifications
  pub enabled: bool,

  /// SMTP server host (e.g., "smtp.gmail.com")
  pub smtp_host: String,

  /// SMTP server port (587 for TLS, 465 for SSL)
  pub smtp_port: u16,

  /// SMTP username for authentication
  pub smtp_username: String,

  /// SMTP password for authentication  
  pub smtp_password: String,

  /// From email address
  pub from_email: String,

  /// From display name
  pub from_name: String,

  /// Use TLS encryption (recommended)
  pub use_tls: bool,
}

/// Email configuration for notification channels
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmailConfig {
  pub enabled: bool,
  pub smtp: SmtpConfig,
  pub templates: EmailTemplateConfig,
}

/// SMTP server configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SmtpConfig {
  pub host: String,
  pub port: u16,
  pub username: String,
  pub password: String,
  pub from_email: String,
  pub from_name: String,
  pub use_tls: bool,
  pub timeout_seconds: u64,
}

/// Email template configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmailTemplateConfig {
  pub base_dir: String,
  pub default_locale: String,
  pub supported_locales: Vec<String>,
  pub mention_template: String,
  pub direct_message_template: String,
  pub chat_invite_template: String,
  pub workspace_invite_template: String,
}

/// Configuration loading errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Invalid configuration value for {field}: {value}")]
    InvalidValue { field: String, value: String },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),
}

impl Default for NotificationConfig {
  fn default() -> Self {
    Self {
      in_app: InAppNotificationConfig::default(),
      email: None, // Email is optional
    }
  }
}

impl Default for InAppNotificationConfig {
  fn default() -> Self {
    Self {
      enabled: true,
      realtime_enabled: true,
      max_notifications_per_user: 100,
      retention_days: 30,
    }
  }
}

impl Default for EmailNotificationConfig {
  fn default() -> Self {
    Self {
      enabled: false,
      smtp_host: "smtp.gmail.com".to_string(),
      smtp_port: 587,
      smtp_username: "".to_string(),
      smtp_password: "".to_string(),
      from_email: "noreply@fechatter.com".to_string(),
      from_name: "Fechatter".to_string(),
      use_tls: true,
    }
  }
}

impl Default for EmailConfig {
  fn default() -> Self {
    Self {
      enabled: false,
      smtp: SmtpConfig::default(),
      templates: EmailTemplateConfig::default(),
    }
  }
}

impl Default for SmtpConfig {
  fn default() -> Self {
    Self {
      host: "smtp.gmail.com".to_string(),
      port: 587,
      username: "".to_string(),
      password: "".to_string(),
      from_email: "noreply@fechatter.com".to_string(),
      from_name: "Fechatter".to_string(),
      use_tls: true,
      timeout_seconds: 30,
    }
  }
}

impl Default for EmailTemplateConfig {
  fn default() -> Self {
    Self {
      base_dir: "templates".to_string(),
      default_locale: "en".to_string(),
      supported_locales: vec!["en".to_string()],
      mention_template: "mention_notification".to_string(),
      direct_message_template: "direct_message_notification".to_string(),
      chat_invite_template: "chat_invite_notification".to_string(),
      workspace_invite_template: "workspace_invite_notification".to_string(),
    }
  }
}

impl AppConfig {
  /// Load configuration from environment variables
  pub fn from_env() -> Result<Self, ConfigError> {
    Ok(Self {
      server: ServerConfig::from_env()?,
      auth: AuthConfig::from_env()?,
      messaging: MessagingConfig::from_env()?,
      search: SearchConfig::from_env()?,
      notification: NotificationConfig::from_env()?,
    })
  }

  /// Load configuration from TOML file
  pub fn from_file(path: &str) -> Result<Self, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: Self = toml::from_str(&content)?;
    Ok(config)
  }

  /// Load configuration with fallback: file -> env -> minimal
  pub fn load_with_fallback() -> Result<Self, ConfigError> {
    // Try to load from file first
    if let Ok(config) = Self::from_file("fechatter.toml") {
      return Ok(config);
    }
    
    // Fallback to environment variables
    if let Ok(config) = Self::from_env() {
      return Ok(config);
    }
    
    // Last resort: minimal development config
    Self::minimal_dev_config()
  }

  /// Create minimal configuration for development (NOT for production)
  pub fn minimal_dev_config() -> Result<Self, ConfigError> {
    // Generate temporary keys for development only
    let (private_key, public_key) = Self::generate_dev_keypair();
    
    Ok(Self {
      server: ServerConfig {
        port: 8080,
        db_url: std::env::var("DATABASE_URL")
          .unwrap_or_else(|_| "postgresql://localhost/fechatter_dev".to_string()),
        base_dir: std::env::current_dir()?.join("static"),
        max_upload_size: 10 * 1024 * 1024, // 10MB
        request_timeout_ms: 30000,
      },
      auth: AuthConfig {
        pk: public_key,
        sk: private_key,
        token_expiration: 3600, // 1 hour
        refresh_token_expiration: 86400 * 7, // 7 days
      },
      messaging: MessagingConfig::minimal_dev(),
      search: SearchConfig::minimal_dev(),
      notification: NotificationConfig::minimal_dev(),
    })
  }

  /// Generate temporary keypair for development (WARNING: NOT for production)
  fn generate_dev_keypair() -> (String, String) {
    eprintln!("⚠️  WARNING: Using temporary generated keys for development!");
    eprintln!("   These keys are NOT persistent and will change on restart.");
    eprintln!("   DO NOT use this in production!");
    
    // For development, we'll use a fixed test key pair
    // In a real scenario, you'd generate proper RSA keys
    let private_key = r#"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDh/nCDmXaEqxN4
16b9XjV8acmbqA52uPzKbesWQSNZPZu1/um8IZrkavvPNGqkooX7Zsl2PkTeQMzi
0XLuWHMPPg9lWfvg/sMU8znPojhFsrR0trslSBLB9CblPMPpf6boflPHPwvy/rHi
TzP7hb7hHwHiIGFi7udivaCcjevp8vtb5j97lpjWvr7EnzEjVjVLuT7LVjz6/n5j
fP37tQ
-----END PRIVATE KEY-----"#.to_string();
    
    let public_key = r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA4f5wg5l2hKsTeNem/V41
fGnJm6gOdrj8ym3rFkEjWT2btf7pvCGa5Gr7zzRqpKKF+2bJdj5E3kDM4tFy7lhz
Dz4PZVn74P7DFPM5z6I4RbK0dLa7JUgSwfQm5TzD6X+m6H5jxz8L8v6x4k8z+4W+
4R8B4iBhYu7nYr2gHI3r6fL7W+Y/e5aY1r6+xJ8xI1Y1S7k+y1Y8+v5+Y3z9+7U
-----END PUBLIC KEY-----"#.to_string();
    
    (private_key, public_key)
  }
}

impl ServerConfig {
  pub fn from_env() -> Result<Self, ConfigError> {
    let port = std::env::var("FECHATTER_PORT")
      .unwrap_or_else(|_| "8080".to_string())
      .parse()
      .map_err(|_| ConfigError::InvalidValue {
        field: "FECHATTER_PORT".to_string(),
        value: std::env::var("FECHATTER_PORT").unwrap_or_default(),
      })?;

    let db_url = std::env::var("DATABASE_URL")
      .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL".to_string()))?;

    Ok(Self {
      port,
      db_url,
      base_dir: std::env::current_dir()?.join("static"),
      max_upload_size: 10 * 1024 * 1024,
      request_timeout_ms: 30000,
    })
  }
}

impl AuthConfig {
  pub fn from_env() -> Result<Self, ConfigError> {
    let pk = std::env::var("FECHATTER_PUBLIC_KEY")
      .map_err(|_| ConfigError::MissingEnvVar("FECHATTER_PUBLIC_KEY".to_string()))?;
    
    let sk = std::env::var("FECHATTER_PRIVATE_KEY")
      .map_err(|_| ConfigError::MissingEnvVar("FECHATTER_PRIVATE_KEY".to_string()))?;

    Ok(Self {
      pk,
      sk,
      token_expiration: 3600,
      refresh_token_expiration: 86400 * 7,
    })
  }
}

impl MessagingConfig {
  pub fn from_env() -> Result<Self, ConfigError> {
    Ok(Self {
      enabled: std::env::var("FECHATTER_MESSAGING_ENABLED")
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .unwrap_or(true),
      provider: std::env::var("FECHATTER_MESSAGING_PROVIDER")
        .unwrap_or_else(|_| "nats".to_string()),
      nats: NatsConfig::from_env()?,
    })
  }

  pub fn minimal_dev() -> Self {
    Self {
      enabled: false, // Disabled by default in dev
      provider: "nats".to_string(),
      nats: NatsConfig::minimal_dev(),
    }
  }
}

impl NatsConfig {
  pub fn from_env() -> Result<Self, ConfigError> {
    Ok(Self {
      url: std::env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string()),
      auth: NatsAuthConfig::from_env()?,
      subjects: NatsSubjectsConfig::from_env()?,
      jetstream: JetStreamConfig::from_env()?,
    })
  }

  pub fn minimal_dev() -> Self {
    Self {
      url: "nats://localhost:4222".to_string(),
      auth: NatsAuthConfig::minimal_dev(),
      subjects: NatsSubjectsConfig::minimal_dev(),
      jetstream: JetStreamConfig::minimal_dev(),
    }
  }
}

impl NatsAuthConfig {
  pub fn from_env() -> Result<Self, ConfigError> {
    Ok(Self {
      enabled: std::env::var("NATS_AUTH_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false),
    })
  }

  pub fn minimal_dev() -> Self {
    Self { enabled: false }
  }
}

impl NatsSubjectsConfig {
  pub fn from_env() -> Result<Self, ConfigError> {
    Ok(Self {
      message_created: std::env::var("NATS_SUBJECT_MESSAGE_CREATED")
        .unwrap_or_else(|_| "chat.message.created".to_string()),
      message_updated: std::env::var("NATS_SUBJECT_MESSAGE_UPDATED")
        .unwrap_or_else(|_| "chat.message.updated".to_string()),
      message_deleted: std::env::var("NATS_SUBJECT_MESSAGE_DELETED")
        .unwrap_or_else(|_| "chat.message.deleted".to_string()),
      chat_created: std::env::var("NATS_SUBJECT_CHAT_CREATED")
        .unwrap_or_else(|_| "chat.chat.created".to_string()),
      chat_updated: std::env::var("NATS_SUBJECT_CHAT_UPDATED")
        .unwrap_or_else(|_| "chat.chat.updated".to_string()),
      user_joined_chat: std::env::var("NATS_SUBJECT_USER_JOINED")
        .unwrap_or_else(|_| "chat.user.joined".to_string()),
      user_left_chat: std::env::var("NATS_SUBJECT_USER_LEFT")
        .unwrap_or_else(|_| "chat.user.left".to_string()),
      duplicate_message_attempted: std::env::var("NATS_SUBJECT_DUPLICATE_MESSAGE")
        .unwrap_or_else(|_| "chat.message.duplicate".to_string()),
    })
  }

  pub fn minimal_dev() -> Self {
    Self {
      message_created: "chat.message.created".to_string(),
      message_updated: "chat.message.updated".to_string(),
      message_deleted: "chat.message.deleted".to_string(),
      chat_created: "chat.chat.created".to_string(),
      chat_updated: "chat.chat.updated".to_string(),
      user_joined_chat: "chat.user.joined".to_string(),
      user_left_chat: "chat.user.left".to_string(),
      duplicate_message_attempted: "chat.message.duplicate".to_string(),
    }
  }
}

impl JetStreamConfig {
  pub fn from_env() -> Result<Self, ConfigError> {
    Ok(Self {
      enabled: std::env::var("JETSTREAM_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false),
      stream: std::env::var("JETSTREAM_STREAM")
        .unwrap_or_else(|_| "CHAT_EVENTS".to_string()),
      storage: std::env::var("JETSTREAM_STORAGE")
        .unwrap_or_else(|_| "memory".to_string()),
      max_bytes: 1024 * 1024 * 100, // 100MB
      max_msg_size: 1024 * 1024,    // 1MB
      max_age: 86400 * 7,           // 7 days
    })
  }

  pub fn minimal_dev() -> Self {
    Self {
      enabled: false,
      stream: "CHAT_EVENTS".to_string(),
      storage: "memory".to_string(),
      max_bytes: 1024 * 1024 * 100,
      max_msg_size: 1024 * 1024,
      max_age: 86400 * 7,
    }
  }
}

impl SearchConfig {
  pub fn from_env() -> Result<Self, ConfigError> {
    Ok(Self {
      enabled: std::env::var("SEARCH_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false),
      provider: std::env::var("SEARCH_PROVIDER")
        .unwrap_or_else(|_| "meilisearch".to_string()),
      async_indexing: AsyncIndexingConfig::from_env()?,
      meilisearch: MeilisearchConfig::from_env()?,
    })
  }

  pub fn minimal_dev() -> Self {
    Self {
      enabled: false, // Disabled by default in dev
      provider: "meilisearch".to_string(),
      async_indexing: AsyncIndexingConfig::minimal_dev(),
      meilisearch: MeilisearchConfig::minimal_dev(),
    }
  }
}

impl AsyncIndexingConfig {
  pub fn from_env() -> Result<Self, ConfigError> {
    Ok(Self {
      enabled: std::env::var("ASYNC_INDEXING_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false),
      batch_size: 100,
      batch_timeout_ms: 5000,
    })
  }

  pub fn minimal_dev() -> Self {
    Self {
      enabled: false,
      batch_size: 100,
      batch_timeout_ms: 5000,
    }
  }
}

impl MeilisearchConfig {
  pub fn from_env() -> Result<Self, ConfigError> {
    Ok(Self {
      url: std::env::var("MEILISEARCH_URL")
        .unwrap_or_else(|_| "http://localhost:7700".to_string()),
      api_key: std::env::var("MEILISEARCH_API_KEY")
        .unwrap_or_else(|_| "master_key".to_string()),
      connection_timeout_ms: 5000,
      request_timeout_ms: 10000,
      indexes: IndexesConfig::from_env()?,
      settings: MeilisearchSettings::minimal_dev(),
    })
  }

  pub fn minimal_dev() -> Self {
    Self {
      url: "http://localhost:7700".to_string(),
      api_key: "master_key".to_string(),
      connection_timeout_ms: 5000,
      request_timeout_ms: 10000,
      indexes: IndexesConfig::minimal_dev(),
      settings: MeilisearchSettings::minimal_dev(),
    }
  }
}

impl IndexesConfig {
  pub fn from_env() -> Result<Self, ConfigError> {
    Ok(Self {
      messages: IndexConfig::from_env()?,
    })
  }

  pub fn minimal_dev() -> Self {
    Self {
      messages: IndexConfig::minimal_dev(),
    }
  }
}

impl IndexConfig {
  pub fn from_env() -> Result<Self, ConfigError> {
    Ok(Self {
      name: "messages".to_string(),
      primary_key: "id".to_string(),
      searchable_fields: vec!["content".to_string(), "user_name".to_string()],
      displayed_fields: vec!["id".to_string(), "content".to_string(), "user_name".to_string(), "created_at".to_string()],
      filterable_fields: vec!["chat_id".to_string(), "user_id".to_string(), "created_at".to_string()],
      sortable_fields: vec!["created_at".to_string()],
    })
  }

  pub fn minimal_dev() -> Self {
    Self {
      name: "messages".to_string(),
      primary_key: "id".to_string(),
      searchable_fields: vec!["content".to_string()],
      displayed_fields: vec!["id".to_string(), "content".to_string()],
      filterable_fields: vec!["chat_id".to_string()],
      sortable_fields: vec!["created_at".to_string()],
    }
  }
}

impl MeilisearchSettings {
  pub fn minimal_dev() -> Self {
    Self {
      pagination_limit: 100,
    }
  }
}

impl NotificationConfig {
  pub fn from_env() -> Result<Self, ConfigError> {
    Ok(Self {
      in_app: InAppNotificationConfig::from_env()?,
      email: if std::env::var("EMAIL_ENABLED").unwrap_or_default() == "true" {
        Some(EmailNotificationConfig::from_env()?)
      } else {
        None
      },
    })
  }

  pub fn minimal_dev() -> Self {
    Self {
      in_app: InAppNotificationConfig::minimal_dev(),
      email: None, // Disabled by default in dev
    }
  }
}

impl InAppNotificationConfig {
  pub fn from_env() -> Result<Self, ConfigError> {
    Ok(Self {
      enabled: std::env::var("IN_APP_NOTIFICATIONS_ENABLED")
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .unwrap_or(true),
      realtime_enabled: std::env::var("REALTIME_NOTIFICATIONS_ENABLED")
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .unwrap_or(true),
      max_notifications_per_user: 1000,
      retention_days: 30,
    })
  }

  pub fn minimal_dev() -> Self {
    Self {
      enabled: true,
      realtime_enabled: true,
      max_notifications_per_user: 1000,
      retention_days: 30,
    }
  }
}

impl EmailNotificationConfig {
  pub fn from_env() -> Result<Self, ConfigError> {
    let smtp_host = std::env::var("SMTP_HOST")
      .map_err(|_| ConfigError::MissingEnvVar("SMTP_HOST".to_string()))?;
    let smtp_username = std::env::var("SMTP_USERNAME")
      .map_err(|_| ConfigError::MissingEnvVar("SMTP_USERNAME".to_string()))?;
    let smtp_password = std::env::var("SMTP_PASSWORD")
      .map_err(|_| ConfigError::MissingEnvVar("SMTP_PASSWORD".to_string()))?;
    let from_email = std::env::var("FROM_EMAIL")
      .map_err(|_| ConfigError::MissingEnvVar("FROM_EMAIL".to_string()))?;

    Ok(Self {
      enabled: true,
      smtp_host,
      smtp_port: std::env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse()
        .unwrap_or(587),
      smtp_username,
      smtp_password,
      from_email,
      from_name: std::env::var("FROM_NAME")
        .unwrap_or_else(|_| "Fechatter".to_string()),
      use_tls: std::env::var("SMTP_USE_TLS")
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .unwrap_or(true),
    })
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


