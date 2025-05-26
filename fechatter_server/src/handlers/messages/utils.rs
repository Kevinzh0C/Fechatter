use crate::{AppError, AppState, models::ChatFile};
use axum::extract::multipart::Field;
use bytes::Bytes;
use futures::{StreamExt, TryStreamExt, stream::BoxStream};
use sha2::{Digest, Sha256};
use std::path::{Path as StdPath, PathBuf};
use tokio::{
  fs::{self, File},
  io::AsyncWriteExt as _,
};
use tokio_util::io::ReaderStream;
use tracing::{info, warn};

// Helper function to guess the content type from a file path
pub fn guess_content_type(path: &StdPath) -> String {
  if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
    return mime_guess::from_ext(extension)
      .first_or_octet_stream()
      .to_string();
  }
  "application/octet-stream".to_string()
}

// Helper function: open and serve file
pub async fn open_and_serve_file(
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
pub async fn validate_filename(filename: &str) -> Result<(), AppError> {
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
pub async fn process_uploaded_file<'a>(
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

  // Clean up temp file regardless of success or failure of save_file_to_storage
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
pub fn check_file_size(size: usize, filename: &str, max_size: usize) -> Result<(), AppError> {
  if size > max_size {
    return Err(AppError::InvalidInput(format!(
      "File '{}' too large: {} bytes (max: {} bytes)",
      filename, size, max_size
    )));
  }
  Ok(())
}

// Helper function: check total upload size
pub fn check_total_size(
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
pub async fn stream_to_temp_file<'a>(
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
pub async fn save_file_to_storage(
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
pub async fn handle_existing_path(path: &std::path::Path) -> Result<bool, AppError> {
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
