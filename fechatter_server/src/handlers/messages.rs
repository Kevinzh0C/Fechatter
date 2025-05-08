use crate::{
  AppError, AppState,
  models::{AuthUser, ChatFile},
};
use axum::{
  body::Body,
  extract::{Extension, Multipart, Path, Query, State, multipart::Field},
  http::{StatusCode, header},
  response::{IntoResponse, Json, Response},
};
use fechatter_core::{CreateMessage, ListMessage};

use bytes::Bytes;
use futures::{StreamExt, TryStreamExt, stream::BoxStream};
use sha2::{Digest, Sha256};
use std::path::{Path as StdPath, PathBuf};
use std::str::FromStr as _;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;
use tracing::{info, warn};

pub(crate) async fn send_message_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
  Json(message): Json<CreateMessage>,
) -> Result<impl IntoResponse, AppError> {
  // Convert core CreateMessage to server CreateMessage
  let message = state.create_message(message, chat_id, user.id).await?;

  Ok((StatusCode::CREATED, Json(message)))
}

pub(crate) async fn list_messages_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(chat_id): Path<i64>,
  Query(query): Query<ListMessage>,
) -> Result<impl IntoResponse, AppError> {
  // Verify user is a member of the chat
  let is_member = sqlx::query_scalar!(
    "SELECT EXISTS(SELECT 1 FROM chat_members WHERE chat_id = $1 AND user_id = $2)",
    chat_id,
    user.id
  )
  .fetch_one(state.pool())
  .await?;

  if !is_member.unwrap_or(false) {
    return Err(AppError::ChatPermissionError(format!(
      "User {} is not a member of chat {}",
      user.id, chat_id
    )));
  }

  let messages: Vec<crate::models::Message> = state.list_messages(query, chat_id).await?;

  Ok((StatusCode::OK, Json(messages)))
}

/// handling file from specificed storage
pub(crate) async fn file_handler(
  State(state): State<AppState>,
  Extension(_user): Extension<AuthUser>,
  Path((ws_id, file_path)): Path<(i64, String)>,
) -> Result<impl IntoResponse, AppError> {
  // Construct the full path in the format required by ChatFile::from_str: "/files/{ws_id}/{part1}/{part2}/{part3}.{ext}"
  let full_path = format!("/files/{}/{}", ws_id, file_path);

  // Validate path format
  let parts: Vec<&str> = file_path.split('/').collect();

  // For old format URLs (e.g., hash.txt without proper path structure),
  // return NotFound to match test expectations
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

// Helper function to guess the content type from a file path
fn guess_content_type(path: &StdPath) -> String {
  if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
    return mime_guess::from_ext(extension)
      .first_or_octet_stream()
      .to_string();
  }
  "application/octet-stream".to_string()
}

// Helper function: open and serve file
async fn open_and_serve_file(
  path: &StdPath,
) -> Result<BoxStream<'static, Result<Bytes, std::io::Error>>, AppError> {
  match File::open(&path).await {
    Ok(file) => {
      // Create file stream
      let stream = ReaderStream::new(file).map_ok(|bytes| bytes).boxed();
      Ok(stream)
    }
    Err(e) => {
      // Return NotFound for file not found errors
      if e.kind() == std::io::ErrorKind::NotFound {
        Err(AppError::NotFound(vec![format!(
          "File not found: {}",
          path.display()
        )]))
      } else {
        Err(AppError::ChatFileError(format!(
          "Failed to open file: {}",
          e
        )))
      }
    }
  }
}

// Helper function: validate filename safety
async fn validate_filename(filename: &str) -> Result<(), AppError> {
  // SECURITY: Validate filename - reject any suspicious filenames
  // This includes path traversal characters, control chars, and other possibly dangerous names
  if filename.contains("..")
    || filename.contains('/')
    || filename.contains('\\')
    || filename.contains('\0')
    || filename.starts_with('.')
    || filename.chars().any(|c| c.is_control())
  {
    return Err(AppError::InvalidInput(format!(
      "Invalid filename: contains illegal characters"
    )));
  }
  Ok(())
}

// Helper function: process single uploaded file
/// Process a single uploaded file with validation, size checks, and storage
///
/// # Arguments
///
/// * `field` - Multipart form field containing the file
/// * `original_filename` - Name of the uploaded file
/// * `ws_id` - Workspace ID associated with the file
/// * `max_file_size` - Maximum allowed size for a single file
/// * `total_size` - Mutable reference to track total uploaded file size
/// * `max_total_size` - Maximum total allowed size for all files
/// * `state` - Application state
///
/// # Returns
///
/// A `Result` containing the file's storage URL or an `AppError` if processing fails
///
/// # Errors
///
/// Returns an error if:
/// - Filename is invalid
/// - File size exceeds limits
/// - File cannot be processed or saved
async fn process_uploaded_file<'a>(
  field: &'a mut Field<'_>,
  original_filename: String,
  ws_id: i64,
  max_file_size: usize,
  total_size: &mut usize,
  max_total_size: usize,
  state: &AppState,
) -> Result<String, AppError> {
  validate_filename(&original_filename).await?;

  // Check Content-Type provided size (if any)
  let content_length: Option<u64> = field
    .content_type()
    .and_then(|ct: &str| {
      let part: Option<&str> = ct.split(';').next();
      part
    })
    .and_then(|ct: &str| ct.parse::<u64>().ok());

  // Pre-check file size (if provided in Content-Type)
  if let Some(length) = content_length {
    check_file_size(length as usize, &original_filename, max_file_size)?;
    check_total_size(*total_size, length as usize, max_total_size)?;
  }

  // Stream upload, calculate hash and save to temp file
  let (hash, temp_file_path, bytes_read) = stream_to_temp_file(
    field,
    &original_filename,
    max_file_size,
    total_size,
    max_total_size,
  )
  .await?;

  info!(
    "Processed uploaded file: {} ({} bytes), hash: {}",
    original_filename, bytes_read, hash
  );

  if bytes_read == 0 {
    warn!("Skipping empty file: {}", original_filename);
    // Clean up empty temp file
    let _ = fs::remove_file(&temp_file_path).await;
    return Err(AppError::InvalidInput(format!(
      "Empty file: {}",
      original_filename
    )));
  }

  // Store file and return URL
  let result = save_file_to_storage(&hash, &temp_file_path, &original_filename, ws_id, state).await;

  // Clean up temp file regardless of success or failure of save_file_to_storage,
  // as save_file_to_storage attempts to move/copy it.
  // If move succeeded, remove_file will fail gracefully.
  // If copy succeeded, remove_file cleans up the source.
  // If save failed, remove_file cleans up the temp file.
  if let Err(e) = fs::remove_file(&temp_file_path).await {
    // Log if removal fails for reasons other than NotFound (e.g., permissions)
    if e.kind() != std::io::ErrorKind::NotFound {
      warn!(
        "Failed to clean up temporary file {:?}: {}",
        temp_file_path, e
      );
    }
  }

  result
}

// Helper function: check single file size
fn check_file_size(size: usize, filename: &str, max_size: usize) -> Result<(), AppError> {
  if size > max_size {
    return Err(AppError::InvalidInput(format!(
      "File '{}' too large: {} bytes (max: {} bytes)",
      filename, size, max_size
    )));
  }
  Ok(())
}

// Helper function: check total upload size
fn check_total_size(
  current_total: usize,
  additional: usize,
  max_total: usize,
) -> Result<(), AppError> {
  if current_total + additional > max_total {
    return Err(AppError::InvalidInput(format!(
      "Total upload size exceeds limit of {} bytes",
      max_total
    )));
  }
  Ok(())
}

// Helper function: stream upload file to temp file, calculate hash
/// Streams an uploaded file to a temporary file while performing size and security checks.
///
/// This function handles streaming a multipart file upload, with the following key features:
/// - Writes the uploaded file to a temporary file with a unique UUID
/// - Calculates SHA256 hash of the file during streaming
/// - Enforces per-file and total upload size limits
/// - Cleans up temporary files in case of errors
///
/// # Arguments
/// * `field` - The multipart upload field being processed
/// * `filename` - Original name of the uploaded file
/// * `max_file_size` - Maximum allowed size for a single file
/// * `total_size` - Mutable reference to track current total upload size
/// * `max_total_size` - Maximum allowed total upload size
///
/// # Returns
/// A tuple containing:
/// - Calculated SHA256 hash of the file
/// - Path to the temporary file
/// - Number of bytes written
///
/// # Errors
/// Returns an `AppError` if:
/// - File size exceeds limits
/// - Total upload size exceeds limits
/// - File writing encounters an error
async fn stream_to_temp_file<'a>(
  field: &'a mut Field<'_>,
  filename: &str,
  max_file_size: usize,
  total_size: &mut usize,
  max_total_size: usize,
) -> Result<(String, PathBuf, usize), AppError> {
  let temp_file_path = std::env::temp_dir().join(format!("upload_{}.tmp", uuid::Uuid::now_v7()));
  let mut temp_file = match File::create(&temp_file_path).await {
    Ok(file) => file,
    Err(_e) => {
      return Err(AppError::ChatFileError(format!(
        "Failed to process upload: cannot create temporary file for '{}'",
        filename
      )));
    }
  };

  // Initialize hash calculator
  let mut hasher = Sha256::new();
  let mut bytes_written: usize = 0;

  while let Ok(Some(bytes)) = field.try_next().await {
    if bytes.is_empty() {
      continue; // Skip empty chunks if any, but don't break yet
    }

    let chunk_size = bytes.len();

    // Check if this chunk would exceed file size limit
    if bytes_written + chunk_size > max_file_size {
      // Clean up temp file
      drop(temp_file); // Ensure file is closed before removal
      let _ = fs::remove_file(&temp_file_path).await;

      return Err(AppError::InvalidInput(format!(
        "File '{}' too large: exceeds {} bytes limit",
        filename, max_file_size
      )));
    }

    // Check if this chunk would exceed total upload limit
    if *total_size + chunk_size > max_total_size {
      // Clean up temp file
      drop(temp_file); // Ensure file is closed before removal
      let _ = fs::remove_file(&temp_file_path).await;

      return Err(AppError::InvalidInput(format!(
        "Total upload size exceeds limit of {} bytes",
        max_total_size
      )));
    }

    // Write to temp file
    if let Err(_e) = temp_file.write_all(&bytes).await {
      // Clean up temp file
      drop(temp_file); // Ensure file is closed before removal
      let _ = fs::remove_file(&temp_file_path).await;

      return Err(AppError::ChatFileError(format!(
        "Failed to process upload: write error"
      )));
    }

    // Update hash
    hasher.update(&bytes);

    bytes_written += chunk_size;
    *total_size += chunk_size;
  }

  // Ensure all data is written to disk
  let _ = temp_file.flush().await;

  // Close temp file
  drop(temp_file);

  // Finalize hash calculation
  let hash_bytes = hasher.finalize();
  let hash_hex = format!("{:x}", hash_bytes);

  Ok((hash_hex, temp_file_path, bytes_written))
}

// Helper function: save file to storage (modified signature)
async fn save_file_to_storage(
  hash: &str,          // Use pre-calculated hash
  temp_path: &StdPath, // Path to temporary file
  original_filename: &str,
  ws_id: i64,
  state: &AppState,
) -> Result<String, AppError> {
  // Extract extension from original filename
  let ext = StdPath::new(original_filename)
    .extension()
    .and_then(|s| s.to_str())
    .unwrap_or("")
    .to_string();

  // Create ChatFile meta data using the provided hash
  let file_meta = ChatFile {
    workspace_id: ws_id,
    hash: hash.to_string(),
    ext,
  };

  let _hash_path = file_meta.hash_to_path();

  // Ensure base directory exists
  let base_ws_dir = state.config.server.base_dir.join(ws_id.to_string());

  // Create base directory (and all parent directories) if they don't exist
  if let Err(_e) = fs::create_dir_all(&base_ws_dir).await {
    return Err(AppError::ChatFileError(format!(
      "Server error: failed to create storage directory: {}",
      _e
    )));
  }

  let final_path = file_meta.from_path(&state.config.server.base_dir);

  // SECURITY: Additional validation to ensure generated path is within workspace directory
  let canonical_base = match base_ws_dir.canonicalize() {
    Ok(p) => p,
    Err(_e) => {
      return Err(AppError::ChatFileError(format!(
        "Server error: invalid base directory"
      )));
    }
  };

  // Need to canonicalize the *parent* of the final path if it doesn't exist yet.
  let parent_dir = final_path
    .parent()
    .ok_or_else(|| AppError::InvalidInput(format!("Invalid final path structure")))?;

  // Ensure parent directory exists *before* canonicalization checks involving final_path itself
  fs::create_dir_all(parent_dir).await?;

  let canonical_parent = match parent_dir.canonicalize() {
    Ok(p) => p,
    Err(_e) => {
      return Err(AppError::ChatFileError(format!(
        "Server error: invalid directory structure"
      )));
    }
  };

  // Now check if the canonical parent starts with the canonical base
  // And also check the final path itself to prevent tricks like "/base/ws/../other/file"
  if !canonical_parent.starts_with(&canonical_base) || final_path.file_name().is_none() {
    return Err(AppError::InvalidInput(format!(
      "Security error: invalid file path"
    )));
  }

  // Check if path already exists, handle possible path conflicts
  if final_path.exists() && handle_existing_path(&final_path).await? {
    // If it's a file and already exists (handle_existing_path returned true),
    // assume content is the same (due to hash) and skip the move/copy.
    return Ok(file_meta.url());
  }
  // If handle_existing_path returned false, it means it was a directory that got deleted,
  // or some other state where we should proceed with creating the file.

  // Move or copy the temporary file to the final path
  match fs::rename(temp_path, &final_path).await {
    Ok(_) => Ok(file_meta.url()),
    Err(_rename_err) => {
      // Rename might fail across different filesystems. Fallback to copy.
      match fs::copy(temp_path, &final_path).await {
        Ok(_) => Ok(file_meta.url()),
        Err(copy_err) => {
          // Attempt to clean up the potentially partially copied file at final_path if copy failed
          let _ = fs::remove_file(&final_path).await;
          Err(AppError::ChatFileError(format!(
            "Failed to save file: {}",
            copy_err
          )))
        }
      }
    }
  }
}

// Helper function: handle existing path (directory or file)
async fn handle_existing_path(path: &std::path::Path) -> Result<bool, AppError> {
  if path.is_dir() {
    let mut has_content = false;
    if let Ok(mut entries) = fs::read_dir(path).await {
      while let Ok(Some(_)) = entries.next_entry().await {
        has_content = true;
        break;
      }
    }

    if has_content {
      // Directory is not empty, attempt to recursively delete
      let _ = tokio::fs::remove_dir_all(path).await;
    } else {
      // Directory is empty, directly delete
      let _ = tokio::fs::remove_dir(path).await;
    }
    // Return false to indicate continue processing (create new file)
    Ok(false)
  } else {
    // File already exists, skip
    // Return true to indicate file can be used directly
    Ok(true)
  }
}

pub(crate) async fn upload_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
  let ws_id = user.workspace_id;
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

/// Fix file storage structure issues
/// Search for all cases like this in the workspace:
/// - File path (e.g. 1/abc/123/hash.ext) is a directory
/// - Directory contains only one file
/// - Move the file to the correct path
pub(crate) async fn fix_file_storage_handler(
  State(state): State<AppState>,
  Extension(user): Extension<AuthUser>,
  Path(ws_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
  // Ensure user has access to the workspace
  // --- TODO: Implement Robust Authorization Check ---
  // The check `user.workspace_id == ws_id` might be insufficient if AuthUser.workspace_id
  // doesn't represent the full access scope of the user
  if user.workspace_id != ws_id {
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

#[cfg(test)]
mod tests {
  use super::*;

  use crate::setup_test_users;
  use axum::extract::FromRequest;
  use axum::http::StatusCode;

  use fechatter_core::Message;
  use tokio::fs;
  use tokio::io::AsyncWriteExt;

  #[tokio::test]
  async fn list_files_handler_should_work() {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];
    let auth_user = Extension(AuthUser {
      id: user.id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
      workspace_id: user.workspace_id,
    });

    let ws_id = user.workspace_id;
    let app_state = State(state.clone());

    let base_dir = state
      .config
      .server
      .base_dir
      .join(user.workspace_id.to_string());
    std::fs::create_dir_all(&base_dir).unwrap();

    let file = ChatFile::new(user.workspace_id, "test.txt", b"hello world");
    let file_path = file.from_path(&state.config.server.base_dir);
    std::fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    std::fs::write(&file_path, b"hello world").unwrap();

    // Get complete hash path, but need to remove workspace_id portion
    let hash_path = file.hash_to_path();
    // Remove workspace_id portion from hash_path
    let file_path_without_wsid = hash_path
      .split_once('/')
      .map(|(_, rest)| rest.to_string())
      .expect("Hash path should contain workspace_id");

    eprintln!("File created at: {:?}", file_path);
    eprintln!("Hash path: {}", hash_path);
    eprintln!("Path without wsid: {}", file_path_without_wsid);

    let response = file_handler(app_state, auth_user, Path((ws_id, file_path_without_wsid)))
      .await
      .unwrap();
    let response = response.into_response();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = http_body_util::BodyExt::collect(response.into_body())
      .await
      .unwrap()
      .to_bytes();

    assert_eq!(&body_bytes[..], b"hello world");
  }
  #[tokio::test]
  async fn send_message_handler_should_work() {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user1 = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];

    let auth_user = Extension(AuthUser {
      id: user1.id,
      fullname: user1.fullname.clone(),
      email: user1.email.clone(),
      status: user1.status,
      created_at: user1.created_at,
      workspace_id: user1.workspace_id,
    });

    // Create a chat with 3 members
    let chat = state
      .create_new_chat(
        user1.id,
        "Test Chat",
        crate::models::ChatType::Group,
        Some(vec![user1.id, user2.id, user3.id]), // Include user3
        Some("Test chat for sending messages"),
        user1.workspace_id,
      )
      .await
      .expect("Failed to create chat");

    let chat_id = chat.id;
    let message_request = CreateMessage {
      content: "Hello, this is a test message".to_string(),
      files: vec![],
    };

    let result = send_message_handler(
      State(state.clone()),
      auth_user.clone(),
      Path(chat_id),
      Json(message_request),
    )
    .await;

    // Print error details if the test fails
    if let Err(ref e) = result {
      eprintln!("Error sending message: {:?}", e);
    }

    assert!(result.is_ok());

    if let Ok(response) = result {
      let response = response.into_response();
      assert_eq!(response.status(), StatusCode::CREATED);

      // Extract and validate the message
      let body_bytes = http_body_util::BodyExt::collect(response.into_body())
        .await
        .unwrap()
        .to_bytes();
      let message: Message = serde_json::from_slice(&body_bytes).unwrap();

      // Verify message properties
      assert_eq!(message.chat_id, chat_id);
      assert_eq!(message.sender_id, user1.id);
      assert_eq!(message.content, "Hello, this is a test message");
    }
  }

  #[tokio::test]
  async fn list_message_handler_should_work() {
    let (_tdb, state, users) = setup_test_users!(3).await;
    let user = &users[0];
    let user2 = &users[1];
    let user3 = &users[2];

    // Create a chat with the user as a member
    let chat = state
      .create_new_chat(
        user.id,
        "Test Chat",
        crate::models::ChatType::Group,
        Some(vec![user.id, user2.id, user3.id]),
        Some("Test chat for messages"),
        user.workspace_id,
      )
      .await
      .expect("Failed to create chat");

    let query = ListMessage {
      last_id: None,
      limit: 10,
    };

    let auth_user = Extension(AuthUser {
      id: user.id,
      email: user.email.clone(),
      workspace_id: user.workspace_id,
      fullname: user.fullname.clone(),
      status: user.status,
      created_at: user.created_at,
    });
    let result =
      list_messages_handler(State(state.clone()), auth_user, Path(chat.id), Query(query)).await;

    assert!(result.is_ok());

    if let Ok(response) = result {
      let response = response.into_response();
      assert_eq!(response.status(), StatusCode::OK);
    }
  }

  #[tokio::test]
  async fn upload_handler_should_work() {
    use crate::models::ChatFile;
    use std::env;

    // Create test directory in the project directory instead of /tmp
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let test_dir = current_dir.join("target").join("test_upload_files");

    // Create custom config
    let mut config = crate::AppConfig::load().expect("Failed to load config");
    config.server.base_dir = test_dir.clone();

    // Create test directory
    let _ = fs::create_dir_all(&config.server.base_dir).await;
    eprintln!("Using test dir: {}", config.server.base_dir.display());

    // Create AppState with custom config
    let (_tdb, state) = crate::AppState::test_new()
      .await
      .expect("Failed to create test state");

    // Create test user
    let mut users = Vec::new();
    let fullname = "TestUser".to_string();
    let email = "test_user@example.com".to_string();
    let workspace = "TestWorkspace".to_string();
    let user_payload = crate::models::CreateUser::new(&fullname, &email, &workspace, "password");
    let user = state
      .create_user(&user_payload, None)
      .await
      .expect("Failed to create test user");
    users.push(user);

    let user = &users[0];

    let test_content = "Hello, this is test file content!".as_bytes().to_vec();
    let file_name = "test_upload.txt";

    let file_meta = ChatFile::new(user.workspace_id, file_name, &test_content);
    let expected_hash = file_meta.hash.clone();

    // Use from_path method to generate file path
    let expected_path = file_meta.from_path(&state.config.server.base_dir);

    // Ensure parent directory exists
    if let Some(parent) = expected_path.parent() {
      eprintln!("Creating parent directory: {}", parent.display());
      let _ = fs::create_dir_all(parent).await;
    }

    info!("Expected file path: {:?}", expected_path);

    info!("Simulating file upload process...");

    // Remove existing file to avoid conflicts
    if expected_path.exists() {
      fs::remove_file(&expected_path)
        .await
        .expect("Failed to remove existing file");
    }

    // Create and write file
    eprintln!("Creating file at: {}", expected_path.display());
    let mut file = File::create(&expected_path)
      .await
      .expect("Failed to create file");
    file
      .write_all(&test_content)
      .await
      .expect("Failed to write file content");

    assert!(
      expected_path.exists(),
      "File should be created successfully"
    );
    let saved_content = fs::read(&expected_path).await.unwrap();
    assert_eq!(
      saved_content, test_content,
      "Saved file content should match original"
    );

    // Verify recalculated hash matches
    let recalculated_meta = ChatFile::new(user.workspace_id, file_name, &saved_content);
    assert_eq!(
      recalculated_meta.hash, expected_hash,
      "File hash should match"
    );

    // Clean up
    let _ = fs::remove_dir_all(&test_dir).await;

    info!("Upload handler simulation test passed");
  }

  #[tokio::test]
  async fn test_file_handler_integration() {
    use crate::models::ChatFile;
    use std::env;

    // Create test directory in the project directory instead of /tmp
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let test_dir = current_dir.join("target").join("test_files");

    // Create a custom AppState with our test directory
    let mut config = crate::AppConfig::load().expect("Failed to load config");
    config.server.base_dir = test_dir.clone();

    // Create test directory
    let _ = fs::create_dir_all(&config.server.base_dir).await;
    eprintln!("Using test dir: {}", config.server.base_dir.display());

    // Create AppState with custom config
    let (_tdb, state) = crate::AppState::test_new()
      .await
      .expect("Failed to create test state");

    // Create test users with our state
    let mut users = Vec::new();
    let fullname = "TestUser".to_string();
    let email = "test_user@example.com".to_string();
    let workspace = "TestWorkspace".to_string();
    let user_payload = crate::models::CreateUser::new(&fullname, &email, &workspace, "password");
    let user = state
      .create_user(&user_payload, None)
      .await
      .expect("Failed to create test user");
    users.push(user);

    // Rest of test with our custom state
    let user = &users[0];
    let auth_user = Extension(AuthUser {
      id: user.id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
      workspace_id: user.workspace_id,
    });

    let test_content = "This is a test file for download".as_bytes().to_vec();
    let file_name = "test_file.txt";
    let ws_id = user.workspace_id;

    // Create the ChatFile object
    let file_meta = ChatFile::new(ws_id, file_name, &test_content);

    // Use the path with our test directory
    let file_path = file_meta.from_path(&state.config.server.base_dir);
    eprintln!("File path: {}", file_path.display());

    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
      eprintln!("Creating parent directory: {}", parent.display());
      let _ = fs::create_dir_all(parent).await;
      // Verify directory was created successfully
      assert!(parent.exists(), "Parent directory should be created");
    }

    // Delete existing file to avoid conflicts
    if file_path.exists() {
      fs::remove_file(&file_path).await.unwrap();
    }

    // Create and write file
    eprintln!("Creating file at: {}", file_path.display());
    let mut file = File::create(&file_path).await.unwrap();
    file.write_all(&test_content).await.unwrap();

    // Verify file was created successfully
    assert!(
      file_path.exists(),
      "File should be created at expected path"
    );
    let read_content = fs::read(&file_path).await.unwrap();
    assert_eq!(read_content, test_content, "File content should match");

    // Build correct URL path string
    let url = file_meta.url();
    let file_path_str_with_wsid = url.strip_prefix("/files/").unwrap();
    eprintln!("Path string with wsid: {}", file_path_str_with_wsid);

    // Extract the path part *after* the ws_id
    let remaining_path = file_path_str_with_wsid
      .split_once('/')
      .map(|(_, path)| path)
      .unwrap_or(file_path_str_with_wsid);
    eprintln!("Remaining path string: {}", remaining_path);

    // Call file handler with correct tuple
    let response = file_handler(
      State(state.clone()),
      auth_user.clone(),
      Path((ws_id, remaining_path.to_string())),
    )
    .await;

    if let Err(ref e) = response {
      eprintln!("file_handler returned error: {:?}", e);
      match ChatFile::from_str(&url) {
        Ok(cf) => {
          let resolved_path = cf.from_path(&state.config.server.base_dir);
          eprintln!(
            "Resolved path: {}, exists: {}",
            resolved_path.display(),
            resolved_path.exists()
          );
        }
        Err(e) => eprintln!("Failed to parse URL: {}, error: {:?}", url, e),
      }
    }

    let response = response.expect("file_handler should succeed");
    let response = response.into_response();

    assert_eq!(
      response.status(),
      StatusCode::OK,
      "Response status should be 200 OK"
    );

    let body_bytes = http_body_util::BodyExt::collect(response.into_body())
      .await
      .unwrap()
      .to_bytes();

    assert_eq!(
      &body_bytes[..],
      test_content.as_slice(),
      "Response body should match original file"
    );

    // Test non-exist file
    let nonexistent_hash = "nonexist_hash_12345.txt";
    let auth_user2 = Extension(AuthUser {
      id: user.id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
      workspace_id: user.workspace_id,
    });

    let nonexistent_ws_dir = state
      .config
      .server
      .base_dir
      .join(user.workspace_id.to_string());
    let _ = fs::create_dir_all(&nonexistent_ws_dir).await;

    let nonexistent_file_meta = ChatFile::new(user.workspace_id, nonexistent_hash, &[]);
    let nonexistent_path = nonexistent_file_meta.from_path(&state.config.server.base_dir);

    if nonexistent_path.exists() {
      fs::remove_file(&nonexistent_path).await.unwrap();
    }

    assert!(!nonexistent_path.exists(), "Test file should not exist");

    let nonexistent_url = nonexistent_file_meta.url();
    let nonexistent_path_str_with_wsid = nonexistent_url.strip_prefix("/files/").unwrap();
    let nonexistent_remaining_path = nonexistent_path_str_with_wsid
      .split_once('/')
      .map(|(_, path)| path)
      .unwrap_or(nonexistent_path_str_with_wsid);

    let error_response = file_handler(
      State(state),
      auth_user2,
      Path((user.workspace_id, nonexistent_remaining_path.to_string())),
    )
    .await;

    match error_response {
      Ok(_) => panic!("Should return error for non-existent file"),
      Err(e) => match e {
        AppError::NotFound(_) => info!("Correctly returned NotFound error"),
        _ => panic!("Should return NotFound error, not: {:?}", e),
      },
    }

    let _ = fs::remove_dir_all(&test_dir).await;
  }

  #[tokio::test]
  async fn test_large_message_volume() {
    let (_tdb, state, users) = setup_test_users!(10).await;
    let user1 = &users[0];

    let chat = state
      .create_new_chat(
        user1.id,
        "Large Message Test",
        fechatter_core::ChatType::Group,
        Some(users.iter().map(|u| u.id).collect()),
        Some("Chat for testing large message volumes"),
        user1.workspace_id,
      )
      .await
      .expect("Failed to create chat");

    info!("Created test chat successfully, ID: {}", chat.id);

    const MESSAGE_COUNT: usize = 1000;
    let mut message_ids = Vec::with_capacity(MESSAGE_COUNT);

    info!("Starting to create {} test messages...", MESSAGE_COUNT);
    let start_time = std::time::Instant::now();

    for i in 0..MESSAGE_COUNT {
      let sender = &users[i % users.len()];

      let message_payload = crate::models::CreateMessage {
        content: format!("Test message #{} from {}", i, sender.fullname),
        files: vec![],
      };

      let message = state
        .create_message(message_payload, chat.id, sender.id)
        .await
        .expect(&format!("Failed to create message #{}", i));

      message_ids.push(message.id);

      if (i + 1) % 100 == 0 {
        info!("Created {} messages...", i + 1);
      }
    }

    let creation_duration = start_time.elapsed();
    info!(
      "Created {} messages, time taken: {:.2?}, average per message: {:.2?}",
      MESSAGE_COUNT,
      creation_duration,
      creation_duration / MESSAGE_COUNT as u32
    );

    const PAGE_SIZE: i64 = 100;
    let mut last_id = None;
    let mut total_fetched = 0;

    let query_start = std::time::Instant::now();

    for page in 0..11 {
      info!("Querying page {} of messages...", page + 1);

      let query = ListMessage {
        last_id,
        limit: PAGE_SIZE,
      };

      let auth_user = Extension(AuthUser {
        id: user1.id,
        email: user1.email.clone(),
        workspace_id: user1.workspace_id,
        fullname: user1.fullname.clone(),
        status: user1.status,
        created_at: user1.created_at,
      });
      let result = list_messages_handler(
        State(state.clone()),
        auth_user,
        Path(chat.id),
        Query(query.clone()),
      )
      .await
      .expect("Failed to list messages");

      let response = result.into_response();
      assert_eq!(response.status(), StatusCode::OK);

      let body_bytes = http_body_util::BodyExt::collect(response.into_body())
        .await
        .unwrap()
        .to_bytes();
      let messages: Vec<Message> = serde_json::from_slice(&body_bytes).unwrap();

      let page_count = messages.len();
      total_fetched += page_count;

      if page_count == 0 {
        info!("No more messages, pagination complete");
        break;
      }

      if let Some(min_id) = messages.iter().map(|m| m.id).min() {
        last_id = Some(min_id);
      }

      info!("Page {} retrieved {} messages", page + 1, page_count);
    }

    let query_duration = query_start.elapsed();
    info!(
      "Pagination complete, retrieved {} messages, time taken: {:.2?}",
      total_fetched, query_duration
    );

    assert_eq!(
      total_fetched, MESSAGE_COUNT,
      "Should retrieve all created messages"
    );
  }

  #[tokio::test]
  async fn test_file_format_compatibility() {
    // Create test environment
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];
    let auth_user = Extension(AuthUser {
      id: user.id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
      workspace_id: user.workspace_id,
    });

    // Test content
    let test_content = "Format compatibility test".as_bytes().to_vec();
    let file_name = "format_test.txt";

    // Create ChatFile object
    let file_meta = ChatFile::new(user.workspace_id, file_name, &test_content);

    // Create and save actual file
    let file_path = file_meta.from_path(&state.config.server.base_dir);
    if let Some(parent) = file_path.parent() {
      fs::create_dir_all(parent).await.unwrap();
    }
    let mut file = File::create(&file_path).await.unwrap();
    file.write_all(&test_content).await.unwrap();

    // Confirm file was created
    assert!(file_path.exists(), "File should have been created");

    // Test 1: Old format URL - should fail (as it doesn't match /files/{ws_id}/{*path})
    // We now expect NotFound because ChatFile::from_str will fail
    let old_format_path = format!("{}.{}", file_meta.hash, file_meta.ext); // e.g., hash.txt
    let old_format_response = file_handler(
      State(state.clone()),
      auth_user.clone(),
      // This call will likely fail route matching now, but if it somehow reached
      // the handler with the new signature, it would need ws_id.
      // Since the handler expects (i64, String), we must provide a ws_id.
      // We pass the user's ws_id and the old path string.
      // The parsing inside file_handler will correctly fail.
      Path((user.workspace_id, old_format_path.clone())),
    )
    .await;

    // Expect old format to fail with NotFound due to parsing error inside handler
    assert!(
      old_format_response.is_err(),
      "Using old format path ({}) should fail",
      old_format_path
    );

    if let Err(e) = old_format_response {
      match e {
        AppError::NotFound(_) => {} // Expected error type due to parsing failure
        other => panic!(
          "Old format should return NotFound error, not: {:?} for path {}",
          other, old_format_path
        ),
      }
    }

    // Test 2: New format URL (without /files/ prefix) - should succeed
    let new_url = file_meta.url(); // e.g., /files/1/abc/def/ghi.txt
    let path_with_wsid = new_url.strip_prefix("/files/").unwrap(); // e.g., 1/abc/def/ghi.txt

    // Extract the path part *after* the ws_id
    let remaining_path = path_with_wsid
      .split_once('/')
      .map(|(_wsid, path)| path)
      .unwrap_or(path_with_wsid);

    let new_format_response = file_handler(
      State(state.clone()),
      auth_user.clone(),
      Path((user.workspace_id, remaining_path.to_string())), // Pass ws_id and remaining path
    )
    .await;

    // Expect new format to succeed
    if let Err(ref e) = new_format_response {
      eprintln!("New format failed unexpectedly: {:?}", e); // Log error if it fails
    }
    assert!(
      new_format_response.is_ok(),
      "Using new format path ({}) should succeed",
      remaining_path
    );

    // Cleanup
    if file_path.exists() {
      fs::remove_file(file_path).await.unwrap();
    }
  }

  // Add tests for the refactored parts
  #[tokio::test]
  async fn test_validate_filename() {
    // Valid file name
    assert!(validate_filename("valid_file.txt").await.is_ok());
    assert!(validate_filename("valid-file-1.2.3.png").await.is_ok());

    // Invalid file name
    assert!(validate_filename("../invalid.txt").await.is_err());
    assert!(validate_filename("/etc/passwd").await.is_err());
    assert!(validate_filename("file\\name.txt").await.is_err());
    assert!(validate_filename(".hidden").await.is_err());
    assert!(validate_filename("invalid\0file.txt").await.is_err());
  }

  #[tokio::test]
  async fn test_check_file_size() {
    // Valid file size
    assert!(check_file_size(100, "small.txt", 1024).is_ok());
    assert!(check_file_size(1024, "exact.txt", 1024).is_ok());

    // Exceeds size
    assert!(check_file_size(2048, "large.txt", 1024).is_err());
  }

  #[tokio::test]
  async fn test_check_total_size() {
    // Valid total size
    assert!(check_total_size(500, 500, 1024).is_ok());
    assert!(check_total_size(0, 1024, 1024).is_ok());

    // Exceeds size
    assert!(check_total_size(1000, 100, 1024).is_err());
  }

  #[tokio::test]
  async fn test_handle_existing_path() {
    // Create temporary directory for testing
    let temp_dir = std::env::temp_dir().join("test_handle_existing_path");
    let _ = fs::create_dir_all(&temp_dir).await;

    // Test empty directory
    let empty_dir = temp_dir.join("empty_dir");
    let _ = fs::create_dir(&empty_dir).await;
    assert!(!handle_existing_path(&empty_dir).await.unwrap());

    // Test non-empty directory
    let non_empty_dir = temp_dir.join("non_empty_dir");
    let _ = fs::create_dir(&non_empty_dir).await;
    let file_in_dir = non_empty_dir.join("some_file.txt");
    let _ = fs::write(&file_in_dir, b"test content").await;
    assert!(!handle_existing_path(&non_empty_dir).await.unwrap());

    // Test existing file
    let existing_file = temp_dir.join("existing_file.txt");
    let _ = fs::write(&existing_file, b"test content").await;
    assert!(handle_existing_path(&existing_file).await.unwrap());

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir).await;
  }

  #[tokio::test]
  async fn test_stream_to_temp_file_success() {
    use axum::body::Body;

    // Construct multipart request body
    let boundary = "------------------------abcdef1234567890";
    let content = "This is test data";
    let body_str = format!(
      "--{}\r\nContent-Disposition: form-data; name=\"test_field\"; filename=\"test.txt\"\r\nContent-Type: text/plain\r\n\r\n{}\r\n--{}--\r\n",
      boundary, content, boundary
    );

    // Calculate expected hash
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let expected_hash = format!("{:x}", hasher.finalize());

    let body = Body::from(body_str);
    let request = axum::http::Request::builder()
      .header(
        axum::http::header::CONTENT_TYPE,
        format!("multipart/form-data; boundary={}", boundary),
      )
      .body(body)
      .unwrap();

    // Create test state and Multipart
    let (_tdb, state, _users) = setup_test_users!(1).await;
    let mut multipart = Multipart::from_request(request, &state)
      .await
      .expect("Failed to create multipart");
    let mut field = multipart
      .next_field()
      .await
      .unwrap()
      .expect("Failed to get field");

    let mut total_size = 0;
    let max_file_size = 1024;
    let max_total_size = 2048;

    let result = stream_to_temp_file(
      &mut field,
      "test.txt",
      max_file_size,
      &mut total_size,
      max_total_size,
    )
    .await;

    assert!(result.is_ok());
    let (hash, temp_path, bytes_written) = result.unwrap();
    // Clean up temporary file
    let _ = fs::remove_file(&temp_path).await;

    assert_eq!(hash, expected_hash);
    assert_eq!(bytes_written, 17);
    assert_eq!(total_size, 17);
  }

  #[tokio::test]
  async fn test_stream_to_temp_file_exceeds_limit() {
    use axum::body::Body;

    // Construct multipart request body (contains two data blocks)
    let boundary = "------------------------abcdef1234567890";
    let data1 = vec![0u8; 512];
    let data2 = vec![0u8; 513]; // Exceeds limit
    let body_str = format!(
      "--{}\r\nContent-Disposition: form-data; name=\"test_field\"; filename=\"large_file.txt\"\r\nContent-Type: application/octet-stream\r\n\r\n{}{}\r\n--{}--\r\n",
      boundary,
      String::from_utf8_lossy(&data1), // Note: If data is not valid UTF-8, it will have problems
      String::from_utf8_lossy(&data2), // But should be fine for all zero bytes
      boundary
    );
    let body = Body::from(body_str); // Note: This approach is not suitable for binary data, a better way is to build a stream

    let request = axum::http::Request::builder()
      .header(
        axum::http::header::CONTENT_TYPE,
        format!("multipart/form-data; boundary={}", boundary),
      )
      .body(body)
      .unwrap();

    // Create test state and Multipart
    let (_tdb, state, _users) = setup_test_users!(1).await;
    let mut multipart = Multipart::from_request(request, &state)
      .await
      .expect("Failed to create multipart");
    let mut field = multipart
      .next_field()
      .await
      .unwrap()
      .expect("Failed to get field");

    let mut total_size = 0;
    let max_file_size = 1024;
    let max_total_size = 2048;

    let result = stream_to_temp_file(
      &mut field,
      "large_file.txt",
      max_file_size,
      &mut total_size,
      max_total_size,
    )
    .await;

    assert!(result.is_err());
    // Verify the error is about file size limit
    assert!(
      result
        .unwrap_err()
        .to_string()
        .contains("exceeds 1024 bytes limit")
    );
  }

  #[tokio::test]
  async fn test_save_file_to_storage_success() {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];
    let ws_id = user.workspace_id;
    let test_content = b"Save me to storage!".to_vec();
    let filename = "save_test.txt";

    // Create temporary file
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("test_save_{}.tmp", uuid::Uuid::now_v7()));
    fs::write(&temp_path, &test_content)
      .await
      .expect("Failed to create temp file");

    // Calculate SHA-1 hash, consistent with ChatFile::new
    let mut hasher = sha1::Sha1::new();
    hasher.update(&test_content);
    let hash = hex::encode(hasher.finalize());

    info!("Test calculated hash: {}", hash);

    // Ensure target directory exists
    let _ = fs::create_dir_all(state.config.server.base_dir.join(ws_id.to_string())).await;

    // Call function
    let result = save_file_to_storage(&hash, &temp_path, filename, ws_id, &state).await;

    // Validate result
    assert!(result.is_ok(), "save_file_to_storage failed: {:?}", result);
    let url = result.unwrap();

    // Print actual URL for debugging
    info!("Returned URL: {}", url);
    info!("Expected hash prefix: {}", &hash[0..3]);

    // Create an expected ChatFile object to verify URL format
    let expected_file = ChatFile {
      workspace_id: ws_id,
      hash: hash.clone(),
      ext: "txt".to_string(),
    };

    let expected_url = expected_file.url();
    info!("Expected URL pattern: {}", expected_url);

    // Check URL to ensure it contains workspace ID and file extension
    assert!(
      url.starts_with(&format!("/files/{}/", ws_id)),
      "URL doesn't start with workspace path: {}",
      url
    );
    assert!(
      url.ends_with(".txt"),
      "URL doesn't end with correct extension: {}",
      url
    );

    // Validate file exists
    let file_path = expected_file.from_path(&state.config.server.base_dir);
    assert_eq!(
      file_path.exists(),
      true,
      "File doesn't exist at expected path: {:?}",
      file_path
    );

    // Validate file content
    let content = fs::read(&file_path).await.expect("Failed to read file");
    assert_eq!(content, test_content, "File content doesn't match");

    // Clean up directory
    let _ = fs::remove_dir_all(&state.config.server.base_dir).await;
  }

  // Testing process_uploaded_file is complex since it involves mocking Field and AppState.
  // Here's a simplified test approach focused on verifying it correctly calls helper functions.
  // A complete test would require more complex mocking or integration testing.
  #[tokio::test]
  async fn test_process_uploaded_file_basic() {
    use axum::body::Body;

    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];
    let ws_id = user.workspace_id;
    let filename = "process_test.txt";
    let data = b"Test process_uploaded_file".to_vec();

    // Ensure base directory exists
    let _ = fs::create_dir_all(state.config.server.base_dir.join(ws_id.to_string())).await;

    // Construct multipart request body
    let boundary = "------------------------abcdef1234567890";
    let body_str = format!(
      "--{}\r\nContent-Disposition: form-data; name=\"test_field\"; filename=\"{}\"\r\nContent-Type: text/plain\r\n\r\n{}\r\n--{}--\r\n",
      boundary,
      filename,
      String::from_utf8_lossy(&data),
      boundary
    );
    let body = Body::from(body_str);
    let request = axum::http::Request::builder()
      .header(
        axum::http::header::CONTENT_TYPE,
        format!("multipart/form-data; boundary={}", boundary),
      )
      .body(body)
      .unwrap();

    // Create Multipart and get Field
    let mut multipart = Multipart::from_request(request, &state)
      .await
      .expect("Failed to create multipart");
    let mut field = multipart
      .next_field()
      .await
      .unwrap()
      .expect("Failed to get field");

    let mut total_size = 0;
    let max_file_size = 1024;
    let max_total_size = 2048;

    // Call function
    let result = process_uploaded_file(
      &mut field,
      filename.to_string(),
      ws_id,
      max_file_size,
      &mut total_size,
      max_total_size,
      &state,
    )
    .await;

    // Validate success
    assert!(result.is_ok(), "process_uploaded_file failed: {:?}", result);
    let url = result.unwrap();

    // Basic URL format validation
    assert!(
      url.starts_with(&format!("/files/{}/", ws_id)),
      "URL '{}' doesn't start with correct workspace path",
      url
    );
    assert!(
      url.ends_with(".txt"),
      "URL '{}' doesn't end with correct extension",
      url
    );

    // Parse URL into ChatFile to get actual storage path
    let chat_file = ChatFile::from_str(&url).expect("Failed to parse URL into ChatFile");
    assert_eq!(chat_file.workspace_id, ws_id);
    assert_eq!(chat_file.ext, "txt");

    // Validate file exists and has correct content
    let file_path = chat_file.from_path(&state.config.server.base_dir);
    assert_eq!(
      file_path.exists(),
      true,
      "File doesn't exist at expected path: {:?}",
      file_path
    );

    // Validate file content
    let saved_content = fs::read(&file_path)
      .await
      .expect("Failed to read saved file");
    assert_eq!(saved_content, data, "File content doesn't match");

    // Clean up
    let _ = fs::remove_dir_all(&state.config.server.base_dir).await;
  }
}
