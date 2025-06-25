use super::StorageService;
use crate::AppError;
use async_trait::async_trait;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct LocalStorage {
  base_dir: PathBuf,
  url_prefix: String,
}

impl LocalStorage {
  pub fn new<P: AsRef<Path>>(base_dir: P, url_prefix: &str) -> Result<Self, AppError> {
    let base_dir = base_dir.as_ref().to_path_buf();

    // Create base directory if it doesn't exist
    std::fs::create_dir_all(&base_dir)
      .map_err(|e| AppError::ChatFileError(format!("Failed to create storage directory: {}", e)))?;

    Ok(Self {
      base_dir,
      url_prefix: url_prefix.to_string(),
    })
  }

  /// Generate file path based on hash for better distribution
  fn hash_to_path(&self, hash: &str, extension: &str) -> PathBuf {
    let (part1, part2) = hash.split_at(3);
    let (part2, part3) = part2.split_at(3);

    self
      .base_dir
      .join(part1)
      .join(part2)
      .join(format!("{}.{}", part3, extension))
  }

  /// Calculate SHA256 hash of data
  fn calculate_hash(&self, data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
  }

  /// Extract extension from filename
  fn extract_extension(&self, filename: &str) -> String {
    Path::new(filename)
      .extension()
      .and_then(|ext| ext.to_str())
      .unwrap_or("bin")
      .to_string()
  }
}

#[async_trait]
impl StorageService for LocalStorage {
  async fn upload(&self, file_name: String, data: Vec<u8>) -> Result<String, AppError> {
    let hash = self.calculate_hash(&data);
    let extension = self.extract_extension(&file_name);
    let file_path = self.hash_to_path(&hash, &extension);

    // Create parent directories
    if let Some(parent) = file_path.parent() {
      fs::create_dir_all(parent)
        .await
        .map_err(|e| AppError::ChatFileError(format!("Failed to create directory: {}", e)))?;
    }

    // Check if file already exists (deduplication)
    if file_path.exists() {
      return Ok(format!("{}/{}.{}", self.url_prefix, hash, extension));
    }

    // Write file
    fs::write(&file_path, data)
      .await
      .map_err(|e| AppError::ChatFileError(format!("Failed to write file: {}", e)))?;

    Ok(format!("{}/{}.{}", self.url_prefix, hash, extension))
  }

  async fn download(&self, file_id: &str) -> Result<Vec<u8>, AppError> {
    // Extract hash and extension from file_id (format: hash.ext)
    let parts: Vec<&str> = file_id.split('.').collect();
    if parts.len() != 2 {
      return Err(AppError::NotFound(vec![
        "Invalid file identifier".to_string(),
      ]));
    }

    let hash = parts[0];
    let extension = parts[1];
    let file_path = self.hash_to_path(hash, extension);

    fs::read(&file_path).await.map_err(|e| {
      if e.kind() == std::io::ErrorKind::NotFound {
        AppError::NotFound(vec![format!("File not found: {}", file_id)])
      } else {
        AppError::ChatFileError(format!("Failed to read file: {}", e))
      }
    })
  }

  async fn delete(&self, file_id: &str) -> Result<(), AppError> {
    // Extract hash and extension from file_id
    let parts: Vec<&str> = file_id.split('.').collect();
    if parts.len() != 2 {
      return Err(AppError::NotFound(vec![
        "Invalid file identifier".to_string(),
      ]));
    }

    let hash = parts[0];
    let extension = parts[1];
    let file_path = self.hash_to_path(hash, extension);

    fs::remove_file(&file_path).await.map_err(|e| {
      if e.kind() == std::io::ErrorKind::NotFound {
        AppError::NotFound(vec![format!("File not found: {}", file_id)])
      } else {
        AppError::ChatFileError(format!("Failed to delete file: {}", e))
      }
    })
  }

  async fn get_url(&self, file_id: &str) -> Result<String, AppError> {
    // For local storage, just return the URL with prefix
    Ok(format!("{}/{}", self.url_prefix, file_id))
  }

  async fn exists(&self, file_id: &str) -> Result<bool, AppError> {
    // Extract hash and extension from file_id
    let parts: Vec<&str> = file_id.split('.').collect();
    if parts.len() != 2 {
      return Ok(false);
    }

    let hash = parts[0];
    let extension = parts[1];
    let file_path = self.hash_to_path(hash, extension);

    Ok(file_path.exists())
  }
}
