use anyhow::{Result, bail};
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
    // Attempt to read from ./app.yml, /etc/config/app.yml, or from the environment variable CHAT_CONFIG
    let ret = match (
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
