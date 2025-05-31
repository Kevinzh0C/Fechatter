use super::{s3::S3Storage, StorageService};
use crate::error::AppError;
use async_trait::async_trait;

/// MinIO storage service
/// MinIO is S3-compatible, so we just wrap S3Storage with MinIO-specific defaults
pub struct MinIOStorage {
  inner: S3Storage,
}

impl MinIOStorage {
  /// Create new MinIO storage
  pub async fn new(
    endpoint_url: String,
    bucket: String,
    access_key_id: String,
    secret_access_key: String,
  ) -> Result<Self, AppError> {
    // MinIO typically uses "us-east-1" as the default region
    let inner = S3Storage::new_custom(
      bucket,
      endpoint_url,
      access_key_id,
      secret_access_key,
      "us-east-1".to_string(),
    )
    .await?;

    Ok(Self { inner })
  }

  /// Create MinIO storage from environment variables
  pub async fn from_env() -> Result<Self, AppError> {
    let endpoint_url =
      std::env::var("MINIO_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string());
    let bucket = std::env::var("MINIO_BUCKET")
      .map_err(|_| AppError::InvalidInput("MINIO_BUCKET not set".to_string()))?;
    let access_key = std::env::var("MINIO_ACCESS_KEY")
      .map_err(|_| AppError::InvalidInput("MINIO_ACCESS_KEY not set".to_string()))?;
    let secret_key = std::env::var("MINIO_SECRET_KEY")
      .map_err(|_| AppError::InvalidInput("MINIO_SECRET_KEY not set".to_string()))?;

    Self::new(endpoint_url, bucket, access_key, secret_key).await
  }

  /// Get the underlying S3Storage for direct access if needed
  pub fn inner(&self) -> &S3Storage {
    &self.inner
  }
}

#[async_trait]
impl StorageService for MinIOStorage {
  async fn upload(&self, file_name: String, data: Vec<u8>) -> Result<String, AppError> {
    self.inner.upload(file_name, data).await
  }

  async fn download(&self, file_id: &str) -> Result<Vec<u8>, AppError> {
    self.inner.download(file_id).await
  }

  async fn delete(&self, file_id: &str) -> Result<(), AppError> {
    self.inner.delete(file_id).await
  }

  async fn get_url(&self, file_id: &str) -> Result<String, AppError> {
    self.inner.get_url(file_id).await
  }

  async fn exists(&self, file_id: &str) -> Result<bool, AppError> {
    self.inner.exists(file_id).await
  }
}

// Delegate all StorageService methods to the inner S3Storage
impl std::ops::Deref for MinIOStorage {
  type Target = S3Storage;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl std::ops::DerefMut for MinIOStorage {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.inner
  }
}
