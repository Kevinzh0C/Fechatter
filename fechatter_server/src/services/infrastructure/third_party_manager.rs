use std::sync::Arc;

use crate::{
  error::AppError,
  services::{
    ai::{CohereClient, HuggingFaceClient, OpenAIClient},
    infrastructure::storage::{/*MinIOStorage, S3Storage,*/ StorageService},
  },
};
use fechatter_core::AIService;

/// OpenAI configuration
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
  pub api_key: String,
  pub organization: Option<String>,
  pub base_url: Option<String>,
  pub default_model: String,
  pub temperature: f32,
  pub max_tokens: u32,
  pub timeout_seconds: u64,
}

impl OpenAIConfig {
  pub fn new(api_key: String) -> Self {
    Self {
      api_key,
      organization: None,
      base_url: Some("https://api.openai.com/v1".to_string()),
      default_model: "gpt-3.5-turbo".to_string(),
      temperature: 0.7,
      max_tokens: 1000,
      timeout_seconds: 30,
    }
  }

  pub fn validate(&self) -> Result<(), AppError> {
    if self.api_key.is_empty() {
      return Err(AppError::InvalidInput(
        "OpenAI API key cannot be empty".to_string(),
      ));
    }
    if self.temperature < 0.0 || self.temperature > 2.0 {
      return Err(AppError::InvalidInput(
        "OpenAI temperature must be between 0.0 and 2.0".to_string(),
      ));
    }
    Ok(())
  }

  pub fn from_env() -> Result<Self, AppError> {
    let api_key = std::env::var("OPENAI_API_KEY")
      .map_err(|_| AppError::InvalidInput("OPENAI_API_KEY not set".to_string()))?;

    let base_url = std::env::var("OPENAI_BASE_URL")
      .ok()
      .or_else(|| Some("https://api.openai.com/v1".to_string()));

    let default_model =
      std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string());

    let temperature = std::env::var("OPENAI_TEMPERATURE")
      .unwrap_or_else(|_| "0.7".to_string())
      .parse::<f32>()
      .unwrap_or(0.7);

    let max_tokens = std::env::var("OPENAI_MAX_TOKENS")
      .unwrap_or_else(|_| "1000".to_string())
      .parse::<u32>()
      .unwrap_or(1000);

    let timeout_seconds = std::env::var("OPENAI_TIMEOUT")
      .unwrap_or_else(|_| "30".to_string())
      .parse::<u64>()
      .unwrap_or(30);

    Ok(Self {
      api_key,
      organization: std::env::var("OPENAI_ORGANIZATION").ok(),
      base_url,
      default_model,
      temperature,
      max_tokens,
      timeout_seconds,
    })
  }
}

/// Configuration for third-party services
#[derive(Debug, Clone)]
pub struct ThirdPartyConfig {
  // AI Services
  pub openai_config: Option<OpenAIConfig>,
  pub cohere_api_key: Option<String>,
  pub huggingface_api_key: Option<String>,

  // Storage Services
  pub aws_config: Option<AwsConfig>,
  pub cloudflare_r2_config: Option<CloudflareR2Config>,
  pub minio_config: Option<MinIOConfig>,
}

#[derive(Debug, Clone)]
pub struct AwsConfig {
  pub region: String,
  pub bucket: String,
  // AWS SDK will use env vars or IAM role for credentials
}

#[derive(Debug, Clone)]
pub struct CloudflareR2Config {
  pub account_id: String,
  pub bucket: String,
  pub access_key_id: String,
  pub secret_access_key: String,
}

#[derive(Debug, Clone)]
pub struct MinIOConfig {
  pub endpoint: String,
  pub bucket: String,
  pub access_key: String,
  pub secret_key: String,
}

/// Third-party service manager
/// Provides lazy initialization and centralized access to all external services
pub struct ThirdPartyManager {
  config: ThirdPartyConfig,

  // AI Services (lazy initialized)
  openai_client: tokio::sync::OnceCell<Arc<OpenAIClient>>,
  cohere_client: tokio::sync::OnceCell<Arc<CohereClient>>,
  huggingface_client: tokio::sync::OnceCell<Arc<HuggingFaceClient>>,

  // Storage Services (lazy initialized) - Temporarily disabled
  // s3_storage: tokio::sync::OnceCell<Arc<S3Storage>>,
  // minio_storage: tokio::sync::OnceCell<Arc<MinIOStorage>>,
}

impl ThirdPartyManager {
  pub fn new(config: ThirdPartyConfig) -> Self {
    Self {
      config,
      openai_client: tokio::sync::OnceCell::new(),
      cohere_client: tokio::sync::OnceCell::new(),
      huggingface_client: tokio::sync::OnceCell::new(),
      // s3_storage: tokio::sync::OnceCell::new(),
      // minio_storage: tokio::sync::OnceCell::new(),
    }
  }

  /// Get OpenAI client (for chat features)
  pub async fn openai(&self) -> Result<Arc<OpenAIClient>, AppError> {
    self
      .openai_client
      .get_or_try_init(|| async {
        let openai_config =
          self.config.openai_config.as_ref().ok_or_else(|| {
            AppError::InvalidInput("OpenAI configuration not provided".to_string())
          })?;

        let client = OpenAIClient::new(openai_config.clone())?;
        Ok(Arc::new(client))
      })
      .await
      .map(|client| client.clone())
  }

  /// Get Cohere client (for search enhancement)
  pub async fn cohere(&self) -> Result<Arc<CohereClient>, AppError> {
    self
      .cohere_client
      .get_or_try_init(|| async {
        let api_key = self
          .config
          .cohere_api_key
          .as_ref()
          .ok_or_else(|| AppError::InvalidInput("Cohere API key not configured".to_string()))?;
        Ok(Arc::new(CohereClient::new(api_key.clone())))
      })
      .await
      .map(|client| client.clone())
  }

  /// Get HuggingFace client (for specialized AI tasks)
  pub async fn huggingface(&self) -> Result<Arc<HuggingFaceClient>, AppError> {
    self
      .huggingface_client
      .get_or_try_init(|| async {
        let api_key = self.config.huggingface_api_key.as_ref().ok_or_else(|| {
          AppError::InvalidInput("HuggingFace API key not configured".to_string())
        })?;
        Ok(Arc::new(HuggingFaceClient::new(api_key.clone())))
      })
      .await
      .map(|client| client.clone())
  }

  /// Get storage service based on configuration priority
  /* Temporarily disabled - depends on S3Storage and MinIOStorage
  pub async fn storage(&self) -> Result<Arc<dyn StorageService>, AppError> {
    // Priority: MinIO > AWS S3 > Cloudflare R2
    if let Some(minio_config) = &self.config.minio_config {
      let storage = self
        // .minio_storage
        .get_or_try_init(|| async {
          let storage = MinIOStorage::new(
            minio_config.endpoint.clone(),
            minio_config.bucket.clone(),
            minio_config.access_key.clone(),
            minio_config.secret_key.clone(),
          )
          .await?;
          Ok::<Arc<MinIOStorage>, AppError>(Arc::new(storage))
        })
        .await?;
      return Ok(storage.clone() as Arc<dyn StorageService>);
    }

    if let Some(aws_config) = &self.config.aws_config {
      let storage = self
        // .s3_storage
        .get_or_try_init(|| async {
          let storage =
            S3Storage::new_aws(aws_config.bucket.clone(), aws_config.region.clone()).await?;
          Ok::<Arc<S3Storage>, AppError>(Arc::new(storage))
        })
        .await?;
      return Ok(storage.clone() as Arc<dyn StorageService>);
    }

    if let Some(r2_config) = &self.config.cloudflare_r2_config {
      let storage = self
        // .s3_storage
        .get_or_try_init(|| async {
          let storage = S3Storage::new_r2(
            r2_config.bucket.clone(),
            r2_config.account_id.clone(),
            r2_config.access_key_id.clone(),
            r2_config.secret_access_key.clone(),
          )
          .await?;
          Ok::<Arc<S3Storage>, AppError>(Arc::new(storage))
        })
        .await?;
      return Ok(storage.clone() as Arc<dyn StorageService>);
    }

    Err(AppError::InvalidInput(
      "No storage service configured".to_string(),
    ))
  }
  */

  /// Get AI service for general use (OpenAI as default)
  pub async fn ai_service(&self) -> Result<Arc<dyn AIService>, AppError> {
    // TODO: Ensure OpenAIClient implements AIService trait properly
    Err(AppError::InvalidInput(
      "AI service not yet fully implemented".to_string(),
    ))
  }
}

// Helper function to create config from environment
impl ThirdPartyConfig {
  pub fn from_env() -> Self {
    Self {
      // AI Services
      openai_config: OpenAIConfig::from_env().ok(),
      cohere_api_key: std::env::var("COHERE_API_KEY").ok(),
      huggingface_api_key: std::env::var("HUGGINGFACE_API_KEY").ok(),

      // Storage Services
      aws_config: Self::aws_config_from_env(),
      cloudflare_r2_config: Self::r2_config_from_env(),
      minio_config: Self::minio_config_from_env(),
    }
  }

  fn aws_config_from_env() -> Option<AwsConfig> {
    let bucket = std::env::var("AWS_S3_BUCKET").ok()?;
    let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());

    Some(AwsConfig { region, bucket })
  }

  fn r2_config_from_env() -> Option<CloudflareR2Config> {
    Some(CloudflareR2Config {
      account_id: std::env::var("R2_ACCOUNT_ID").ok()?,
      bucket: std::env::var("R2_BUCKET").ok()?,
      access_key_id: std::env::var("R2_ACCESS_KEY_ID").ok()?,
      secret_access_key: std::env::var("R2_SECRET_ACCESS_KEY").ok()?,
    })
  }

  fn minio_config_from_env() -> Option<MinIOConfig> {
    Some(MinIOConfig {
      endpoint: std::env::var("MINIO_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:9000".to_string()),
      bucket: std::env::var("MINIO_BUCKET").ok()?,
      access_key: std::env::var("MINIO_ACCESS_KEY").ok()?,
      secret_key: std::env::var("MINIO_SECRET_KEY").ok()?,
    })
  }
}
