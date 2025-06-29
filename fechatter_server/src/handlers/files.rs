//! # File Handlers - Handles file uploads and downloads
//!
//! **Responsibility**: Manages all HTTP requests related to file operations.
//! **Principle**: Production-ready, secure file handling.

use crate::{
    error::{AppError, ErrorOutput},
    AppState,
    dtos::core::ApiResponse,
    dtos::models::responses::UploadResponse,
    services::infrastructure::storage::{LocalStorage, StorageService},
};
use axum::{
    extract::{multipart::Multipart, Extension, Path as ExtractPath},
    response::{Response, IntoResponse, Json},
    http::{StatusCode, header},
    body::Body,
};
use chrono;
use fechatter_core::models::AuthUser;
use mime_guess;
use serde::Serialize;
use std::path::Path;
use utoipa::ToSchema;
use tokio::fs;
use tracing::{debug, warn, error, info};

/// **Production-grade File Upload Handler**
///
/// Handles single file uploads securely and efficiently.
/// - Uses multipart/form-data.
/// - Guesses MIME type from filename.
/// - Returns a structured JSON response.
#[utoipa::path(
    post,
    path = "/api/files/single",
    summary = "Upload a single file",
    request_body(content_type = "multipart/form-data", content = inline(String)),
    responses(
        (status = 200, description = "File uploaded successfully", body = UploadResponse),
        (status = 400, description = "Bad request", body = ErrorOutput),
        (status = 500, description = "Internal server error", body = ErrorOutput)
    ),
    security(("bearer_auth" = []))
)]
pub async fn upload_single_file_handler(
    Extension(app_state): Extension<AppState>,
    Extension(_user): Extension<AuthUser>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<UploadResponse>>, AppError> {
    debug!("📤 [FILE_UPLOAD] Starting file upload process");

    if let Some(field) = multipart.next_field().await? {
        let filename = field.file_name().unwrap_or("unknown").to_string();
        let data = field.bytes().await?;
        let file_size = data.len() as u64;

        debug!("📤 [FILE_UPLOAD] Received file: {} ({} bytes)", filename, file_size);

        // Validate file size
        if file_size == 0 {
            warn!("❌ [FILE_UPLOAD] Empty file rejected: {}", filename);
            return Err(AppError::BadRequest("File is empty".to_string()));
        }

        if file_size > app_state.config.server.max_upload_size {
            warn!("❌ [FILE_UPLOAD] File too large: {} ({} bytes > {} bytes)", 
                  filename, file_size, app_state.config.server.max_upload_size);
            return Err(AppError::BadRequest(format!(
                "File too large: {} bytes (max: {} bytes)", 
                file_size, 
                app_state.config.server.max_upload_size
            )));
        }

        // Access storage config and create LocalStorage instance
        let storage_config = &app_state.config.storage;
        debug!("📤 [FILE_UPLOAD] Using storage config - path: {}, prefix: {}", 
               storage_config.path, storage_config.url_prefix);

        let storage = LocalStorage::new(&storage_config.path, &storage_config.url_prefix)
            .map_err(|e| {
                error!("❌ [FILE_UPLOAD] Failed to create storage instance: {}", e);
                e
            })?;

        // Upload file using storage service
        let file_url = storage.upload(filename.clone(), data.to_vec()).await
            .map_err(|e| {
                error!("❌ [FILE_UPLOAD] Storage upload failed for {}: {}", filename, e);
                e
            })?;

        info!("✅ [FILE_UPLOAD] File uploaded successfully: {} -> {}", filename, file_url);

        // Extract hash.ext from file_url for symlink creation
        if let Some(file_id) = file_url.strip_prefix(&format!("{}/", storage_config.url_prefix)) {
            debug!("📤 [FILE_UPLOAD] Creating symlink for file_id: {}", file_id);
            if let Err(e) = create_symlink_for_file(&storage_config.path, file_id).await {
                warn!("⚠️ [FILE_UPLOAD] Symlink creation failed (non-critical): {}", e);
                // Don't fail the upload for symlink issues
            }
        } else {
            warn!("⚠️ [FILE_UPLOAD] Could not extract file_id from URL: {}", file_url);
        }

        // Guess MIME type from filename extension
        let mime_type = mime_guess::from_path(&filename)
            .first_or_octet_stream()
            .to_string();

        debug!("📤 [FILE_UPLOAD] Detected MIME type: {} for file: {}", mime_type, filename);

        let resp = UploadResponse {
            id: file_url.clone(),
            filename: filename.clone(),
            url: file_url.clone(),
            mime_type,
            size: file_size,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        info!("✅ [FILE_UPLOAD] Upload completed successfully for {}", filename);
        return Ok(Json(ApiResponse::success(resp, "File uploaded successfully".to_string())));
    }

    warn!("❌ [FILE_UPLOAD] No file found in multipart request");
    Err(AppError::BadRequest("No file found in multipart request".to_string()))
}

/// Create symlink in root directory for ServeDir compatibility
async fn create_symlink_for_file(storage_path: &str, file_id: &str) -> Result<(), AppError> {
    use std::os::unix::fs::symlink;
    
    // Parse file_id to extract hash and extension
    let parts: Vec<&str> = file_id.split('.').collect();
    if parts.len() != 2 {
        return Ok(()); // Skip if invalid format
    }
    
    let hash = parts[0];
    let extension = parts[1];
    
    // Generate hash directory path
    let (part1, part2) = hash.split_at(3);
    let (part2, part3) = part2.split_at(3);
    let hash_path = format!("{}/{}/{}/{}.{}", storage_path, part1, part2, part3, extension);
    let symlink_path = format!("{}/{}", storage_path, file_id);
    
    // Create relative symlink path
    let relative_target = format!("./{}/{}/{}.{}", part1, part2, part3, extension);
    
    // Remove existing symlink if it exists
    if std::path::Path::new(&symlink_path).exists() {
        let _ = tokio::fs::remove_file(&symlink_path).await;
    }
    
    // Create symlink using blocking task
    let symlink_path_clone = symlink_path.clone();
    let relative_str_clone = relative_target.clone();
    let filename_clone = file_id.to_string();
    
    match tokio::task::spawn_blocking(move || {
        symlink(&relative_str_clone, &symlink_path_clone)
    }).await {
        Ok(Ok(_)) => {
            debug!("🔗 [SYMLINK] Created: {} -> {}", filename_clone, relative_target);
            Ok(())
        }
        Ok(Err(e)) => {
            warn!("⚠️ [SYMLINK] Failed to create symlink for {}: {}", filename_clone, e);
            Ok(()) // Don't fail upload for symlink issues
        }
        Err(e) => {
            warn!("⚠️ [SYMLINK] Task error for {}: {}", filename_clone, e);
            Ok(()) // Don't fail upload for symlink issues
        }
    }
}

/// **Production-grade File Download Handler**
///
/// Handles file downloads with proper error handling and logging.
/// - Validates file existence
/// - Sets appropriate MIME types
/// - Provides detailed error responses
#[utoipa::path(
    get,
    path = "/api/files/download/{file_id}",
    summary = "Download a file",
    responses(
        (status = 200, description = "File downloaded successfully"),
        (status = 404, description = "File not found", body = ErrorOutput),
        (status = 500, description = "Internal server error", body = ErrorOutput)
    )
)]
pub async fn download_file_handler(
    Extension(app_state): Extension<AppState>,
    ExtractPath(file_id): ExtractPath<String>,
) -> Result<Response<Body>, AppError> {
    debug!("📥 [FILE_DOWNLOAD] Starting download for file: {}", file_id);
    
    // Validate file_id format
    if file_id.is_empty() {
        warn!("❌ [FILE_DOWNLOAD] Empty file_id provided");
        return Err(AppError::BadRequest("File ID cannot be empty".to_string()));
    }

    // Check file_id format (should be hash.extension)
    let parts: Vec<&str> = file_id.split('.').collect();
    if parts.len() != 2 {
        warn!("❌ [FILE_DOWNLOAD] Invalid file_id format: {} (expected: hash.extension)", file_id);
        return Err(AppError::BadRequest(format!("Invalid file ID format: {}", file_id)));
    }

    let hash = parts[0];
    let extension = parts[1];
    
    if hash.len() < 6 {
        warn!("❌ [FILE_DOWNLOAD] Hash too short in file_id: {}", file_id);
        return Err(AppError::BadRequest("Invalid file hash".to_string()));
    }

    debug!("📥 [FILE_DOWNLOAD] Parsed file_id - hash: {}, extension: {}", hash, extension);

    let storage_config = &app_state.config.storage;
    debug!("📥 [FILE_DOWNLOAD] Using storage config - path: {}, prefix: {}", 
           storage_config.path, storage_config.url_prefix);

    let storage = LocalStorage::new(&storage_config.path, &storage_config.url_prefix)
        .map_err(|e| {
            error!("❌ [FILE_DOWNLOAD] Failed to create storage instance: {}", e);
            AppError::ChatFileError(format!("Storage initialization failed: {}", e))
        })?;
    
    // Check if file exists first
    match storage.exists(&file_id).await {
        Ok(true) => {
            debug!("📥 [FILE_DOWNLOAD] File exists: {}", file_id);
        }
        Ok(false) => {
            warn!("❌ [FILE_DOWNLOAD] File not found: {}", file_id);
            return Err(AppError::NotFound(vec![format!("File not found: {}", file_id)]));
        }
        Err(e) => {
            error!("❌ [FILE_DOWNLOAD] Error checking file existence: {}", e);
            return Err(AppError::ChatFileError(format!("File existence check failed: {}", e)));
        }
    }
    
    // Download file data
    match storage.download(&file_id).await {
        Ok(file_data) => {
            let file_size = file_data.len();
            info!("✅ [FILE_DOWNLOAD] File read successfully: {} ({} bytes)", file_id, file_size);
            
            // Guess MIME type from file extension
            let mime_type = mime_guess::from_path(&file_id)
                .first_or_octet_stream();
            
            debug!("📥 [FILE_DOWNLOAD] Detected MIME type: {} for file: {}", mime_type, file_id);
            
            // Build response with appropriate headers
            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime_type.as_ref())
                .header(header::CONTENT_LENGTH, file_size)
                .header(header::CACHE_CONTROL, "public, max-age=31536000") // Cache for 1 year
                .header(header::CONTENT_DISPOSITION, format!("inline; filename=\"{}\"", file_id))
                .body(Body::from(file_data))
                .map_err(|e| {
                    error!("❌ [FILE_DOWNLOAD] Failed to build HTTP response: {}", e);
                    AppError::ChatFileError(format!("Failed to build response: {}", e))
                })?;
            
            debug!("✅ [FILE_DOWNLOAD] Response built successfully for: {}", file_id);
            Ok(response)
        }
        Err(e) => {
            error!("❌ [FILE_DOWNLOAD] Failed to read file {}: {:?}", file_id, e);
            match e {
                AppError::NotFound(_) => {
                    Err(AppError::NotFound(vec![format!("File not found: {}", file_id)]))
                }
                _ => {
                    Err(AppError::ChatFileError(format!("File read failed: {}", e)))
                }
            }
        }
    }
}

/// Initialize symlinks for existing files
pub async fn initialize_file_symlinks(storage_path: &str) -> Result<(), AppError> {
    info!("🔗 [INIT_SYMLINKS] Initializing symlinks for existing files in: {}", storage_path);
    
    let storage_dir = std::path::Path::new(storage_path);
    if !storage_dir.exists() {
        warn!("⚠️ [INIT_SYMLINKS] Storage directory does not exist: {}", storage_path);
        return Ok(());
    }
    
    let mut created_count = 0;
    let mut skipped_count = 0;
    
    // Use async recursive function to scan directories
    scan_directory_recursive(storage_dir, storage_dir, &mut created_count, &mut skipped_count).await?;
    
    info!("✅ [INIT_SYMLINKS] Completed: {} created, {} skipped", created_count, skipped_count);
    Ok(())
}

/// Recursively scan directories and create symlinks
async fn scan_directory_recursive(
    current_dir: &std::path::Path,
    storage_root: &std::path::Path,
    created_count: &mut i32,
    skipped_count: &mut i32,
) -> Result<(), AppError> {
    let mut entries = tokio::fs::read_dir(current_dir).await
        .map_err(|e| AppError::ChatFileError(format!("Failed to read directory: {}", e)))?;
    
    while let Some(entry) = entries.next_entry().await
        .map_err(|e| AppError::ChatFileError(format!("Failed to read entry: {}", e)))? {
        
        let path = entry.path();
        
        if path.is_dir() {
            // Skip if this is the root directory to avoid infinite recursion
            if path != storage_root {
                Box::pin(scan_directory_recursive(&path, storage_root, created_count, skipped_count)).await?;
            }
        } else if path.is_file() && !path.is_symlink() {
            // Skip files in root directory (already accessible)
            if path.parent() == Some(storage_root) {
                continue;
            }
            
            // Extract filename
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                let symlink_path = storage_root.join(filename);
                
                // Skip if symlink already exists
                if symlink_path.exists() {
                    *skipped_count += 1;
                    continue;
                }
                
                // Calculate relative path from storage root to file
                if let Ok(relative_path) = path.strip_prefix(storage_root) {
                    let relative_str = format!("./{}", relative_path.display());
                    
                    // Create symlink using blocking task
                    let symlink_path_clone = symlink_path.clone();
                    let relative_str_clone = relative_str.clone();
                    let filename_clone = filename.to_string();
                    
                    match tokio::task::spawn_blocking(move || {
                        std::os::unix::fs::symlink(&relative_str_clone, &symlink_path_clone)
                    }).await {
                        Ok(Ok(_)) => {
                            debug!("🔗 [INIT_SYMLINKS] Created: {} -> {}", filename_clone, relative_str);
                            *created_count += 1;
                        }
                        Ok(Err(e)) => {
                            warn!("⚠️ [INIT_SYMLINKS] Failed to create symlink for {}: {}", filename_clone, e);
                        }
                        Err(e) => {
                            warn!("⚠️ [INIT_SYMLINKS] Task error for {}: {}", filename_clone, e);
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}
