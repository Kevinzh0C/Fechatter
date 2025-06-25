use async_trait::async_trait;
use aws_config::Region;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::{Client, Config};
use std::path::Path;

use super::StorageService;
use crate::error::AppError;

/// S3-compatible storage service
/// Works with AWS S3, Cloudflare R2, and other S3-compatible services
pub struct S3Storage {
  client: Client,
  bucket: String,
  endpoint_url: Option<String>,
  public_url_base: Option<String>,
}

impl S3Storage {
  /// Create new S3 storage for AWS S3
  pub async fn new_aws(bucket: String, region: String) -> Result<Self, AppError> {
    let config = aws_config::from_env()
      .region(Region::new(region))
      .load()
      .await;

    let client = Client::new(&config);

    Ok(Self {
      client,
      bucket,
      endpoint_url: None,
      public_url_base: None,
    })
  }

  /// Create new S3 storage for Cloudflare R2
  pub async fn new_r2(
    bucket: String,
    account_id: String,
    access_key_id: String,
    secret_access_key: String,
  ) -> Result<Self, AppError> {
    let endpoint_url = format!("https://{}.r2.cloudflarestorage.com", account_id);

    let credentials =
      aws_credential_types::Credentials::new(access_key_id, secret_access_key, None, None, "r2");

    let config = Config::builder()
      .endpoint_url(&endpoint_url)
      .credentials_provider(credentials)
      .region(Region::new("auto"))
      .build();

    let client = Client::from_conf(config);

    Ok(Self {
      client,
      bucket,
      endpoint_url: Some(endpoint_url.clone()),
      public_url_base: Some(format!("https://pub-{}.r2.dev", account_id)),
    })
  }

  /// Create new S3 storage with custom endpoint (for MinIO, etc.)
  pub async fn new_custom(
    bucket: String,
    endpoint_url: String,
    access_key_id: String,
    secret_access_key: String,
    region: String,
  ) -> Result<Self, AppError> {
    let credentials = aws_credential_types::Credentials::new(
      access_key_id,
      secret_access_key,
      None,
      None,
      "custom",
    );

    let config = Config::builder()
            .endpoint_url(&endpoint_url)
            .credentials_provider(credentials)
            .region(Region::new(region))
            .force_path_style(true) // Required for MinIO
            .build();

    let client = Client::from_conf(config);

    Ok(Self {
      client,
      bucket,
      endpoint_url: Some(endpoint_url.clone()),
      public_url_base: Some(endpoint_url),
    })
  }

  fn generate_key(&self, file_name: &str) -> String {
    let timestamp = chrono::Utc::now().timestamp();
    let extension = Path::new(file_name)
      .extension()
      .and_then(|ext| ext.to_str())
      .unwrap_or("bin");

    format!(
      "files/{}/{}.{}",
      chrono::Utc::now().format("%Y/%m/%d"),
      timestamp,
      extension
    )
  }
}

#[async_trait]
impl StorageService for S3Storage {
  async fn upload(&self, file_name: String, data: Vec<u8>) -> Result<String, AppError> {
    let key = self.generate_key(&file_name);
    let content_type = mime_guess::from_path(&file_name)
      .first_or_octet_stream()
      .to_string();

    self
      .client
      .put_object()
      .bucket(&self.bucket)
      .key(&key)
      .body(ByteStream::from(data))
      .content_type(content_type)
      .send()
      .await
      .map_err(|e| AppError::ExternalServiceError(format!("S3 upload error: {}", e)))?;

    // Return public URL if available, otherwise return the key
    if let Some(base) = &self.public_url_base {
      Ok(format!("{}/{}", base, key))
    } else {
      Ok(key)
    }
  }

  async fn download(&self, file_id: &str) -> Result<Vec<u8>, AppError> {
    // Extract key from URL if needed
    let key = if file_id.starts_with("http") {
      file_id.split('/').last().unwrap_or(file_id)
    } else {
      file_id
    };

    let response = self
      .client
      .get_object()
      .bucket(&self.bucket)
      .key(key)
      .send()
      .await
      .map_err(|e| AppError::ExternalServiceError(format!("S3 download error: {}", e)))?;

    let data = response
      .body
      .collect()
      .await
      .map_err(|e| AppError::ExternalServiceError(format!("Failed to read S3 object: {}", e)))?;

    Ok(data.into_bytes().to_vec())
  }

  async fn delete(&self, file_id: &str) -> Result<(), AppError> {
    // Extract key from URL if needed
    let key = if file_id.starts_with("http") {
      file_id.split('/').last().unwrap_or(file_id)
    } else {
      file_id
    };

    self
      .client
      .delete_object()
      .bucket(&self.bucket)
      .key(key)
      .send()
      .await
      .map_err(|e| AppError::ExternalServiceError(format!("S3 delete error: {}", e)))?;

    Ok(())
  }

  async fn get_url(&self, file_id: &str) -> Result<String, AppError> {
    // For public buckets, return the public URL
    if let Some(base) = &self.public_url_base {
      let key = if file_id.starts_with("http") {
        file_id.split('/').last().unwrap_or(file_id)
      } else {
        file_id
      };
      return Ok(format!("{}/{}", base, key));
    }

    // For private buckets, generate presigned URL
    let presigned_request = self
      .client
      .get_object()
      .bucket(&self.bucket)
      .key(file_id)
      .presigned(
        aws_sdk_s3::presigning::PresigningConfig::expires_in(std::time::Duration::from_secs(3600))
          .map_err(|e| AppError::ExternalServiceError(format!("Presigning error: {}", e)))?,
      )
      .await
      .map_err(|e| AppError::ExternalServiceError(format!("Failed to generate presigned URL: {}", e)))?;

    Ok(presigned_request.uri().to_string())
  }

  async fn exists(&self, file_id: &str) -> Result<bool, AppError> {
    let key = if file_id.starts_with("http") {
      file_id.split('/').last().unwrap_or(file_id)
    } else {
      file_id
    };

    match self
      .client
      .head_object()
      .bucket(&self.bucket)
      .key(key)
      .send()
      .await
    {
      Ok(_) => Ok(true),
      Err(_) => Ok(false), // Object doesn't exist
    }
  }
}
