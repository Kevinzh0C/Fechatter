use super::utils::{guess_content_type, open_and_serve_file, process_uploaded_file};
use crate::{
  AppError, AppState,
  error::ErrorOutput,
  models::{AuthUser, ChatFile},
};
use axum::{
  body::Body,
  extract::{Extension, Multipart, Path, State},
  http::{StatusCode, header},
  response::{IntoResponse, Json, Response},
};
use std::str::FromStr as _;
use tokio::{
  fs::{self, File},
  io::AsyncWriteExt as _,
};
use tracing::{info, warn};

/// 获取文件
#[utoipa::path(
    get,
    path = "/files/{ws_id}/{file_path}",
    params(
        ("ws_id" = i64, Path, description = "Workspace ID"),
        ("file_path" = String, Path, description = "File path")
    ),
    security(
        ("access_token" = [])
    ),
    responses(
        (status = 200, description = "File retrieved successfully"),
        (status = 401, description = "Unauthorized", body = ErrorOutput),
        (status = 404, description = "File not found", body = ErrorOutput)
    ),
    tag = "files"
)]
pub async fn file_handler(
  State(state): State<AppState>,
  Extension(_user): Extension<AuthUser>,
  Path((ws_id, file_path)): Path<(i64, String)>,
) -> Result<impl IntoResponse, AppError> {
  // Construct the full path in the format required by ChatFile::from_str
  let full_path = format!("/files/{}/{}", ws_id, file_path);

  // Validate path format
  let parts: Vec<&str> = file_path.split('/').collect();

  // For old format URLs, return NotFound to match test expectations
  if parts.len() < 3 || !file_path.contains('/') {
    return Err(AppError::NotFound(vec![format!(
      "File not found: {}",
      file_path
    )]));
  }

  // Parse path into ChatFile
  let chat_file = match ChatFile::from_str(&full_path) {
    Ok(cf) => cf,
    Err(_e) => {
      return Err(AppError::NotFound(vec![format!(
        "File not found: {}",
        file_path
      )]));
    }
  };

  // Calculate physical file path
  let path = chat_file.from_path(&state.config.server.base_dir);

  // Try to open file and return content
  match open_and_serve_file(&path).await {
    Ok(file_stream) => {
      // Determine Content-Type
      let content_type = guess_content_type(&path);

      // Convert stream to axum body
      let body = Body::from_stream(file_stream);

      // Build response with headers
      let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .body(body)
        .unwrap();

      Ok(response)
    }
    Err(e) => Err(e),
  }
}

/// 上传文件
#[utoipa::path(
    post,
    path = "/api/upload",
    security(
        ("access_token" = [])
    ),
    responses(
        (status = 200, description = "Files uploaded successfully", body = Vec<String>),
        (status = 400, description = "Invalid input", body = ErrorOutput),
        (status = 401, description = "Unauthorized", body = ErrorOutput),
        (status = 413, description = "File too large", body = ErrorOutput)
    ),
    tag = "files"
)]
pub async fn upload_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
  let ws_id: i64 = user.workspace_id.into();
  let _base_dir = state.config.server.base_dir.join(ws_id.to_string());
  info!("User {} uploading files to workspace {}", user.id, ws_id);

  let mut file_urls = vec![];

  // SECURITY: Define size limits to prevent DoS attacks
  const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10 MB per file
  const MAX_TOTAL_SIZE: usize = 50 * 1024 * 1024; // 50 MB total upload
  let mut total_size = 0;

  while let Some(mut field) = match multipart.next_field().await {
    Ok(Some(f)) => Some(f),
    Ok(None) => None,
    Err(e) => {
      return Err(AppError::ChatFileError(format!("Multipart error: {}", e)));
    }
  } {
    let original_filename = match field.file_name() {
      Some(name) if !name.is_empty() => name.to_string(),
      _ => {
        warn!("Skipping field without filename");
        continue;
      }
    };

    // Process uploaded file
    match process_uploaded_file(
      &mut field,
      original_filename,
      ws_id,
      MAX_FILE_SIZE,
      &mut total_size,
      MAX_TOTAL_SIZE,
      &state,
    )
    .await
    {
      Ok(url) => file_urls.push(url),
      Err(e) => {
        // If it's an empty file error, just skip this file
        if e.to_string().contains("Empty file") {
          continue;
        }
        // Other errors directly return
        return Err(e);
      }
    }
  }

  if file_urls.is_empty() {
    warn!("No valid files were uploaded by user {}", user.id);
    return Err(AppError::NotFound(vec![
      "No valid files uploaded".to_string(),
    ]));
  }

  Ok(Json(file_urls))
}

/// 修复文件存储结构
#[utoipa::path(
    post,
    path = "/api/workspaces/{ws_id}/fix-storage",
    params(
        ("ws_id" = i64, Path, description = "Workspace ID")
    ),
    security(
        ("access_token" = [])
    ),
    responses(
        (status = 200, description = "Storage fix completed"),
        (status = 401, description = "Unauthorized", body = ErrorOutput),
        (status = 403, description = "Permission denied", body = ErrorOutput)
    ),
    tag = "files"
)]
pub async fn fix_file_storage_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(ws_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
  // Ensure user has access to the workspace
  if user.workspace_id != fechatter_core::WorkspaceId(ws_id) {
    return Err(AppError::ChatPermissionError(format!(
      "User {} does not have access to workspace {}",
      user.id, ws_id
    )));
  }

  // Workspace directory
  let ws_dir = state.config.server.base_dir.join(ws_id.to_string());

  if !ws_dir.exists() {
    return Ok(Json(serde_json::json!({
        "status": "success",
        "message": "No workspace directory found, nothing to fix",
        "stats": {
            "scanned": 0,
            "fixed": 0,
            "errors": 0
        }
    })));
  }

  // Statistics
  let mut scanned_count = 0;
  let mut fixed_count = 0;
  let mut error_count = 0;

  // Recursively scan directories
  let mut dirs_to_scan = vec![ws_dir.clone()];

  while let Some(dir) = dirs_to_scan.pop() {
    if let Ok(mut entries) = fs::read_dir(&dir).await {
      while let Ok(Some(entry)) = entries.next_entry().await {
        scanned_count += 1;
        let path = entry.path();

        if path.is_dir() {
          // Check if directory looks like a file path (has .ext extension)
          if let Some(file_name) = path.file_name() {
            let name = file_name.to_string_lossy();
            if name.contains('.') {
              // It might be a file path, now check if it contains only one file
              let mut inner_files = Vec::new();

              if let Ok(mut inner_entries) = fs::read_dir(&path).await {
                while let Ok(Some(inner_entry)) = inner_entries.next_entry().await {
                  let inner_path = inner_entry.path();
                  if inner_path.is_file() {
                    inner_files.push(inner_path);
                  }
                }
              }

              // If there is only one file, fix it
              if inner_files.len() == 1 {
                let source_file = &inner_files[0];

                // Read file content
                if let Ok(content) = fs::read(&source_file).await {
                  // Create or overwrite target file
                  match fs::remove_dir_all(&path).await {
                    Ok(_) => {
                      // After successfully deleting the directory, create the file
                      if let Ok(mut file) = File::create(&path).await {
                        match file.write_all(&content).await {
                          Ok(_) => {
                            fixed_count += 1;
                          }
                          Err(_) => {
                            error_count += 1;
                          }
                        }
                      } else {
                        error_count += 1;
                      }
                    }
                    Err(_) => {
                      error_count += 1;
                    }
                  }
                } else {
                  error_count += 1;
                }
              } else {
                // Add to scan queue to continue deep search
                dirs_to_scan.push(path);
              }
            } else {
              // Regular directory, add to scan queue
              dirs_to_scan.push(path);
            }
          }
        }
      }
    }
  }

  Ok(Json(serde_json::json!({
      "status": "success",
      "message": "File storage structure fix completed",
      "stats": {
          "scanned": scanned_count,
          "fixed": fixed_count,
          "errors": error_count
      }
  })))
}
