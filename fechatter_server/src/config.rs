use anyhow::{Result, bail};
use fechatter_core::models::jwt::TokenConfigProvider;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
  pub server: ServerConfig,
  pub auth: AuthConfig,
  pub messaging: MessagingConfig,
  pub search: SearchConfig,
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

impl AppConfig {
  pub fn load() -> Result<Self> {
    // Attempt to read from ./chat.yml, /etc/config/chat.yml, or from the environment variable CHAT_CONFIG
    let ret: std::result::Result<AppConfig, serde_yaml::Error> = match (
      File::open("chat.yml"),
      File::open("/etc/config/chat.yml"),
      env::var("CHAT_CONFIG"),
    ) {
      (Ok(reader), _, _) => serde_yaml::from_reader(reader),
      (_, Ok(reader), _) => serde_yaml::from_reader(reader),
      (_, _, Ok(config)) => serde_yaml::from_reader(File::open(config)?),
      _ => bail!("Config file not found"),
    };

    Ok(ret?)
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
