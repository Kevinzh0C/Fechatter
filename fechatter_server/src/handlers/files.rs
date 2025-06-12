//! # File Handlers - Clean Architecture Implementation
//!
//! **Architecture Principles**: Clean Architecture + DDD  
//! **Handler Responsibilities**: HTTP coordination + Service delegation + Response construction (≤20 lines/function)
//! **Dependency Direction**: Handler → Application Service → Domain Service → Infrastructure
//!
//! ## Correct Modern Architecture Implementation
//! - Handlers only handle HTTP coordination, no business logic
//! - Using existing StorageService (services/infrastructure/storage)
//! - All business logic delegated to Service layer
//! - Technical details (file IO, path checks) handled by Infrastructure
//! - Follow proper dependency chain

use crate::services::infrastructure::storage::{LocalStorage, StorageService};
use crate::{AppError, AppState};
use anyhow;
use axum::{
  body::Body,
  extract::{Extension, Multipart, Path},
  http::StatusCode,
  response::{Json, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

// =============================================================================
// RESPONSE DTOs - Simple Response Types
// =============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct UploadResponse {
  pub file_url: String,
  pub file_size: u64,
  pub file_name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FileStorageStatus {
  pub workspace_id: i64,
  pub status: String,
  pub message: String,
}

// =============================================================================
// SERVICE UTILITIES - Infrastructure Access
// =============================================================================

/// Get storage service helper function - Infrastructure Layer
async fn get_storage_service(state: &AppState) -> Result<LocalStorage, AppError> {
  // Use LocalStorage as default - could be configurable in future
  LocalStorage::new(&state.config.server.base_dir, "/files")
    .map_err(|e| AppError::AnyError(anyhow::anyhow!("Failed to initialize storage: {}", e)))
}

// =============================================================================
// HANDLERS - HTTP Coordination Layer (Using Modern Architecture)
// =============================================================================

/// File Upload Handler (Multiple files support)
///
/// **Modern Architecture**: Handler → Application Service → Infrastructure Service
/// **Frontend Compatibility**: Returns array of file URLs as expected by frontend
#[utoipa::path(
    post,
    path = "/api/upload",
    request_body(content = String, description = "File data", content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Files uploaded successfully", body = Vec<String>),
        (status = 400, description = "Invalid file data"),
        (status = 500, description = "Upload failed")
    ),
    tag = "files"
)]
pub async fn upload_handler(
  Extension(state): Extension<AppState>,
  mut multipart: Multipart,
) -> Result<Json<Vec<String>>, AppError> {
  let storage = get_storage_service(&state).await?;
  let mut uploaded_files = Vec::new();

  while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|e| AppError::AnyError(anyhow::anyhow!("Multipart error: {}", e)))?
  {
    // Support both 'file' and 'files' field names
    if field.name() == Some("file") || field.name() == Some("files") {
      let filename = field.file_name().unwrap_or("unknown").to_string();
      let data = field
        .bytes()
        .await
        .map_err(|e| AppError::AnyError(anyhow::anyhow!("Failed to read file data: {}", e)))?
        .to_vec();

      let file_url = storage.upload(filename, data).await?;
      uploaded_files.push(file_url);
    }
  }

  if uploaded_files.is_empty() {
    return Err(AppError::InvalidInput(
      "No files found in request".to_string(),
    ));
  }

  // Return array of file URLs as expected by frontend
  Ok(Json(uploaded_files))
}

/// Single file upload handler (backward compatibility)
///
/// **Legacy Support**: For clients that expect single file response
pub async fn upload_single_file_handler(
  Extension(state): Extension<AppState>,
  mut multipart: Multipart,
) -> Result<Json<UploadResponse>, AppError> {
  let storage = get_storage_service(&state).await?;

  while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|e| AppError::AnyError(anyhow::anyhow!("Multipart error: {}", e)))?
  {
    if field.name() == Some("file") {
      let filename = field.file_name().unwrap_or("unknown").to_string();
      let data = field
        .bytes()
        .await
        .map_err(|e| AppError::AnyError(anyhow::anyhow!("Failed to read file data: {}", e)))?
        .to_vec();

      // Calculate all needed values before moving data
      let file_size = data.len() as u64;
      let filename_for_response = filename.clone();

      let file_url = storage.upload(filename, data).await?;

      return Ok(Json(UploadResponse {
        file_url,
        file_size,
        file_name: filename_for_response,
      }));
    }
  }

  Err(AppError::InvalidInput(
    "No file found in request".to_string(),
  ))
}

/// File Download Handler
///
/// **Modern Architecture**: Handler → Application Service → Infrastructure Service
#[utoipa::path(
    get,
    path = "/api/files/{workspace_id}/{file_id}",
    params(
        ("workspace_id" = i64, Path, description = "Workspace ID"),
        ("file_id" = String, Path, description = "File identifier (hash.ext format)")
    ),
    responses(
        (status = 200, description = "File content", content_type = "application/octet-stream"),
        (status = 404, description = "File not found"),
        (status = 403, description = "Access denied")
    ),
    tag = "files"
)]
pub async fn file_handler(
  Extension(state): Extension<AppState>,
  Path((_workspace_id, file_id)): Path<(i64, String)>,
) -> Result<Response<Body>, AppError> {
  // 1. Get storage service
  let storage = get_storage_service(&state).await?;

  // 2. Load file data via storage service
  let file_data = storage.download(&file_id).await?;

  // 3. Determine MIME type from file_id (hash.ext format)
  let mime_type = if let Some(extension) = file_id.split('.').nth(1) {
    mime_guess::from_ext(extension)
      .first_or_octet_stream()
      .to_string()
  } else {
    "application/octet-stream".to_string()
  };

  // 4. Build HTTP response
  Response::builder()
    .status(StatusCode::OK)
    .header("content-type", mime_type)
    .header("content-length", file_data.len())
    .body(Body::from(file_data))
    .map_err(|e| AppError::AnyError(anyhow::anyhow!("Failed to build response: {}", e)))
}

/// Fix File Storage Handler
///
/// **Modern Architecture**: Handler → Application Service → Infrastructure Service
#[utoipa::path(
    post,
    path = "/api/workspaces/{workspace_id}/files/fix",
    params(("workspace_id" = i64, Path, description = "Workspace ID")),
    responses(
        (status = 200, description = "Storage fix completed", body = FileStorageStatus),
        (status = 500, description = "Fix operation failed")
    ),
    tag = "files"
)]
pub async fn fix_file_storage_handler(
  Extension(state): Extension<AppState>,
  Path(workspace_id): Path<i64>,
) -> Result<Json<FileStorageStatus>, AppError> {
  // 1. Get Storage Service (correct architecture)
  let storage = get_storage_service(&state).await?;

  // 2. Delegate storage health check to Infrastructure Service
  // For now, just verify storage is accessible
  let test_file = format!("health_check_{}.tmp", chrono::Utc::now().timestamp());
  let test_data = b"health check".to_vec();

  // 3. Test storage operations
  match storage.upload(test_file.clone(), test_data).await {
    Ok(file_url) => {
      // Cleanup test file
      let _ = storage.delete(&file_url).await;

      // 4. Build success response
      Ok(Json(FileStorageStatus {
        workspace_id,
        status: "healthy".to_string(),
        message: "File storage is working correctly".to_string(),
      }))
    }
    Err(_) => {
      // 4. Build error response
      Ok(Json(FileStorageStatus {
        workspace_id,
        status: "error".to_string(),
        message: "File storage has issues".to_string(),
      }))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::setup_test_users;
  use anyhow::Result;

  #[tokio::test]
  async fn test_storage_service_creation() -> Result<()> {
    let (_tdb, state, _users) = setup_test_users!(0).await;

    // Test storage service creation
    let storage = get_storage_service(&state).await?;

    // Test basic upload/download cycle
    let test_data = b"test file content".to_vec();
    let file_url = storage
      .upload("test.txt".to_string(), test_data.clone())
      .await?;

    let downloaded = storage.download(&file_url).await?;
    assert_eq!(downloaded, test_data);

    // Cleanup
    storage.delete(&file_url).await?;

    Ok(())
  }
}
