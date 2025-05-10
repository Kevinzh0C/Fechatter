use anyhow::{Result, bail};
use fechatter_core::models::jwt::TokenConfigProvider;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
  pub server: ServerConfig,
  pub auth: AuthConfig,
  // ...
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthConfig {
  pub pk: String,
  pub sk: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
  pub port: u16,
  pub db_url: String,
  // ...
}

impl AppConfig {
  pub fn load() -> Result<Self> {
    // Attempt to read from ./notify.yml, /etc/config/notify.yml, or from the environment variable NOTIFY_CONFIG
    let ret: std::result::Result<AppConfig, serde_yaml::Error> = match (
      File::open("notify.yml"),
      File::open("/etc/config/notify.yml"),
      env::var("NOTIFY_CONFIG"),
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
