use crate::AppError;
use async_trait::async_trait;

// Storage service trait for different backends
#[async_trait]
pub trait StorageService: Send + Sync {
  /// Upload a file and return its identifier/URL
  async fn upload(&self, file_name: String, data: Vec<u8>) -> Result<String, AppError>;

  /// Download a file by identifier
  async fn download(&self, file_id: &str) -> Result<Vec<u8>, AppError>;

  /// Delete a file by identifier
  async fn delete(&self, file_id: &str) -> Result<(), AppError>;

  /// Get public URL for a file (for downloads)
  async fn get_url(&self, file_id: &str) -> Result<String, AppError>;

  /// Check if a file exists
  async fn exists(&self, file_id: &str) -> Result<bool, AppError>;
}

// Local storage implementation
pub mod local;
pub mod minio;
pub mod s3;

// Re-export for convenience
pub use local::LocalStorage;
pub use minio::MinIOStorage;
pub use s3::S3Storage;

// TODO: Future storage implementations
// pub mod s3;
// pub mod minio;
