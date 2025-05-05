use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
  pub server: ServerConfig,
  pub auth: AuthConfig,
  // ...
}

pub use fechatter_core::utils::jwt::AuthConfig;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
  pub port: u16,
  pub db_url: String,
  pub base_dir: PathBuf,
  // ...
}

impl AppConfig {
  pub fn load() -> Result<Self> {
    // Attempt to read from ./app.yml, /etc/config/app.yml, or from the environment variable CHAT_CONFIG
    let ret: std::result::Result<AppConfig, serde_yaml::Error> = match (
      File::open("app.yml"),
      File::open("/etc/config/app.yml"),
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
