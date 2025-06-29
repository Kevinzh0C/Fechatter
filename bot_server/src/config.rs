use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
  pub server: ServerConfig,
  pub messaging: MessagingConfig,
  pub bot: BotConfig,
  pub analytics: AnalyticsConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
  pub db_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessagingConfig {
  pub enabled: bool,
  pub nats: NatsConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NatsConfig {
  pub url: String,
  pub subscription_subjects: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BotConfig {
  pub openai: OpenAIConfig,
  pub vector: VectorConfig,
  pub response_delay_ms: u64,
  pub max_response_length: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAIConfig {
  pub model: String,
  pub embed_model: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub api_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VectorConfig {
  pub size: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalyticsConfig {
  pub enabled: bool,
  pub endpoint: String,
}

impl AppConfig {
  pub fn load() -> Result<Self> {
    // Enhanced configuration loading with Docker container support
    let config_attempts = vec![
      // Priority 1: Environment variable override
      env::var("BOT_CONFIG").ok().and_then(|path| {
        println!("📍 Loading from BOT_CONFIG: {}", path);
        match Self::from_file(&path) {
          Ok(config) => {
            println!("Bot configuration loaded successfully from env var!");
            Some(config)
          }
          Err(e) => {
            eprintln!("ERROR: Failed to load config from BOT_CONFIG ({}): {}", path, e);
            None
          }
        }
      }),
      
      // Priority 2: Docker container standard paths
      Self::try_load_from_path("/app/config/bot.yml"),
      Self::try_load_from_path("/app/config/bot.yaml"),
      Self::try_load_from_path("/app/bot.yml"),
      
      // Priority 2.5: Bot server subdirectory (Docker compatibility)
      Self::try_load_from_path("/app/bot_server/bot.yml"),
      Self::try_load_from_path("/app/bot_server/bot.yaml"),
      
      Self::try_load_from_path("/etc/fechatter/bot.yml"),
      Self::try_load_from_path("/etc/fechatter/bot.yaml"),
      
      // Priority 3: Current directory
      Self::try_load_from_path("bot.yml"),
      Self::try_load_from_path("bot.yaml"),
      
      // Priority 4: Traditional paths
      Self::try_load_from_path("/etc/config/bot.yml"),
      Self::try_load_from_path("/etc/config/bot.yaml"),
      
      // Priority 5: Binary directory config
      env::current_exe().ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .and_then(|exe_dir| {
          Self::try_load_from_path(&exe_dir.join("config").join("bot.yml").to_string_lossy())
        }),
    ];

    // Try each configuration source in order
    for config_option in config_attempts {
      if let Some(mut config) = config_option {
        // Apply environment variable overrides (highest priority)
        config.apply_env_overrides();
        
        // Validate configuration
        config.validate()?;
        return Ok(config);
      }
    }

    // Show helpful error with search locations
    Self::show_search_error();
    bail!("💥 Bot configuration file not found in any expected location!")
  }

  /// Try to load configuration from a specific path
  fn try_load_from_path(path: &str) -> Option<Self> {
    match std::fs::File::open(path) {
      Ok(file) => {
        println!("📁 Found bot config file: {}", path);
        match serde_yaml::from_reader(file) {
          Ok(config) => {
            println!("Bot configuration loaded successfully from: {}", path);
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
    eprintln!("\nSearched for bot configuration in:");
    eprintln!("   ERROR: Environment: BOT_CONFIG");
    eprintln!("   ERROR: Docker paths: /app/config/bot.yml, /app/bot.yml");
    eprintln!("   ERROR: Container paths: /etc/fechatter/bot.yml");
    eprintln!("   ERROR: Current directory: bot.yml");
    eprintln!("   ERROR: System paths: /etc/config/bot.yml");
    eprintln!("");
    eprintln!("To fix this:");
    eprintln!("   1. 📋 Set BOT_CONFIG environment variable");
    eprintln!("   2. 📄 Place bot.yml in current directory");
    eprintln!("   3. 🐳 For Docker: mount config to /app/config/bot.yml");
    eprintln!("");
    eprintln!("🔑 Environment Variables (override yml config):");
    eprintln!("   • OPENAI_API_KEY - OpenAI API key (required)");
    eprintln!("   • OPENAI_MODEL - OpenAI model (optional)");
    eprintln!("   • OPENAI_EMBED_MODEL - OpenAI embedding model (optional)");
  }

  /// Validate configuration integrity
  fn validate(&self) -> Result<()> {
    // Validate database URL
    if self.server.db_url.is_empty() {
      bail!("Database URL cannot be empty");
    }

    if !self.server.db_url.starts_with("postgresql://")
      && !self.server.db_url.starts_with("postgres://")
    {
      bail!("Database URL must start with 'postgresql://' or 'postgres://'");
    }

    // Validate NATS configuration
    if self.messaging.enabled {
      if self.messaging.nats.url.is_empty() {
        bail!("NATS URL cannot be empty when messaging is enabled");
      }

      if !self.messaging.nats.url.starts_with("nats://") {
        bail!("NATS URL must start with 'nats://'");
      }

      if self.messaging.nats.subscription_subjects.is_empty() {
        bail!("At least one NATS subscription subject must be configured");
      }
    }

    // Validate bot configuration
    if self.bot.openai.model.is_empty() {
      bail!("OpenAI model cannot be empty");
    }

    if self.bot.openai.embed_model.is_empty() {
      bail!("OpenAI embedding model cannot be empty");
    }

    // Validate OpenAI API key presence and format
    match &self.bot.openai.api_key {
      Some(key) => {
        if key.is_empty() {
          bail!("OpenAI API key cannot be empty");
        }
        
        // Basic format validation for OpenAI API keys
        if !key.starts_with("sk-") {
          bail!("OpenAI API key must start with 'sk-'");
        }
        
        if key.len() < 20 {
          bail!("OpenAI API key appears to be too short (minimum 20 characters)");
        }
        
        println!("OpenAI API key format validation passed");
      }
      None => {
        bail!("OpenAI API key is required. Set OPENAI_API_KEY environment variable or add 'api_key' to yml config");
      }
    }

    if self.bot.vector.size == 0 {
      bail!("Vector size must be greater than 0");
    }

    if self.bot.max_response_length == 0 {
      bail!("Max response length must be greater than 0");
    }

    Ok(())
  }

  /// Get config summary for logging (exclude sensitive info)
  pub fn get_summary(&self) -> String {
    let api_key_status = match &self.bot.openai.api_key {
      Some(key) => {
        if key.starts_with("sk-") {
          format!("set({}***)", &key[..7]) // Show first 7 chars only
        } else {
          "set(invalid_format)".to_string()
        }
      }
      None => "not_set".to_string(),
    };

    format!(
      "bot_server config: db={}, nats={}, messaging={}, model={}, embed_model={}, api_key={}, vector_size={}",
      if self.server.db_url.contains("localhost") {
        "localhost"
      } else {
        "remote"
      },
      if self.messaging.nats.url.contains("localhost") {
        "localhost"
      } else {
        "remote"
      },
      if self.messaging.enabled {
        "enabled"
      } else {
        "disabled"
      },
      self.bot.openai.model,
      self.bot.openai.embed_model,
      api_key_status,
      self.bot.vector.size
    )
  }

  /// Apply environment variable overrides with highest priority
  fn apply_env_overrides(&mut self) {
    println!("Applying environment variable overrides...");

    // OpenAI API Key - highest priority from environment
    if let Ok(api_key) = env::var("OPENAI_API_KEY") {
      if !api_key.is_empty() {
        println!("Found OPENAI_API_KEY environment variable");
        self.bot.openai.api_key = Some(api_key);
      } else {
        println!("WARNING: OPENAI_API_KEY environment variable is empty");
      }
    } else {
      println!("INFO:  OPENAI_API_KEY environment variable not set, using yml config");
    }

    // Optional: Override other OpenAI settings from environment
    if let Ok(model) = env::var("OPENAI_MODEL") {
      if !model.is_empty() {
        println!("Overriding OpenAI model from environment: {}", model);
        self.bot.openai.model = model;
      }
    }

    if let Ok(embed_model) = env::var("OPENAI_EMBED_MODEL") {
      if !embed_model.is_empty() {
        println!("Overriding OpenAI embed model from environment: {}", embed_model);
        self.bot.openai.embed_model = embed_model;
      }
    }
  }
}

impl OpenAIConfig {
  /// Get the OpenAI API key, preferring environment variable over config file
  pub fn get_api_key(&self) -> Result<&str> {
    match &self.api_key {
      Some(key) => Ok(key.as_str()),
      None => bail!("OpenAI API key not configured. Set OPENAI_API_KEY environment variable or add 'api_key' to yml config"),
    }
  }

  /// Check if API key is configured
  pub fn has_api_key(&self) -> bool {
    self.api_key.is_some() && !self.api_key.as_ref().unwrap().is_empty()
  }
}
