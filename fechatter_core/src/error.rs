use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
  #[error("database error: {0}")]
  SqlxError(sqlx::Error),

  #[error("validation error: {0}")]
  ChatValidationError(String),

  #[error("not found: {0:?}")]
  NotFound(Vec<String>),

  #[error("chat already exists: {0}")]
  ChatAlreadyExists(String),

  #[error("chat permission error: {0}")]
  ChatPermissionError(String),

  #[error("chat file error: {0}")]
  ChatFileError(String),

  #[error("invalid input: {0}")]
  InvalidInput(String),

  #[error("publish error: {0}")]
  PublishError(PublishError),
}

/// Event publishing error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum PublishError {
  /// Serialization errors
  #[error("serialization error: {0}")]
  Serialize(String),

  /// Network connection errors
  #[error("network error: {0}")]
  Network(String),

  /// Timeout errors
  #[error("timeout error: {0}")]
  Timeout(String),

  /// Configuration errors
  #[error("configuration error: {0}")]
  Configuration(String),

  /// Authentication errors for event publishing
  #[error("authentication error: {0}")]
  Authentication(String),

  /// Rate limiting errors
  #[error("rate limit exceeded: {0}")]
  RateLimit(String),
}

impl PublishError {
  /// Check if this error type is retryable
  pub fn is_retryable(&self) -> bool {
    match self {
      PublishError::Network(_) | PublishError::Timeout(_) | PublishError::RateLimit(_) => true,
      PublishError::Serialize(_)
      | PublishError::Configuration(_)
      | PublishError::Authentication(_) => false,
    }
  }
}

/// Vector database specific errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum VectorDbError {
  #[error("Validation error: {0}")]
  Validation(String),

  #[error("Not found: {0}")]
  NotFound(String),

  #[error("Permanent error: {0}")]
  Permanent(String),

  #[error("Authentication error: {0}")]
  Authentication(String),

  #[error("Rate limit error: {0}")]
  RateLimit(String),

  #[error("Transient error: {0}")]
  Transient(String),
}

/// Specific token validation error types to provide more granular control
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum TokenValidationError {
  /// Token not found in the database
  #[error("Invalid or expired refresh token")]
  NotFound,

  /// Token is expired
  #[error("Invalid or expired refresh token")]
  Expired,

  /// Token has been revoked
  #[error("Invalid or revoked refresh token")]
  Revoked,

  /// User agent validation failed
  #[error("Security validation failed for token")]
  SecurityMismatch,
}

/// Specific chat validation error types to provide more granular control
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ChatValidationError {
  /// Invalid chat name
  #[error("Invalid chat name: {0}")]
  InvalidName(String),

  /// Invalid chat members configuration
  #[error("Invalid chat members: {0}")]
  InvalidMembers(String),

  /// Chat permissions error
  #[error("Chat permission denied: {0}")]
  PermissionDenied(String),

  /// Member not found
  #[error("Chat member not found: {0}")]
  MemberNotFound(String),

  /// Chat not found
  #[error("Chat not found: {0}")]
  ChatNotFound(String),
}

/// Core domain error types
/// These errors represent business domain issues, not technical implementation details
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum CoreError {
  /// Database errors from sqlx
  #[error("database error: {0}")]
  Database(String),

  /// Validation errors for business rules
  #[error("validation error: {0}")]
  Validation(String),

  /// Validation errors for general validation cases
  #[error("validation error: {0}")]
  ValidationError(String),

  /// Chat validation specific errors
  #[error("chat validation error: {0}")]
  ChatValidation(ChatValidationError),

  /// Specific validation error for user already exists case
  #[error("user already exists: {0}")]
  UserAlreadyExists(String),

  /// User not found errors
  #[error("user not found: {0}")]
  UserNotFound(String),

  /// Chat not found errors  
  #[error("chat not found: {0}")]
  ChatNotFound(String),

  /// Foreign key constraint violation errors
  #[error("referenced resource not found: {0}")]
  ForeignKeyViolation(String),

  /// Unique constraint violation errors
  #[error("resource already exists: {0}")]
  UniqueViolation(String),

  /// Not found errors
  #[error("not found: {0}")]
  NotFound(String),

  /// Conflict errors
  #[error("conflict: {0}")]
  Conflict(String),

  /// JWT authentication errors
  #[error("authentication error: {0}")]
  Authentication(String),

  /// Unauthorized access errors
  #[error("unauthorized: {0}")]
  Unauthorized(String),

  /// Specific token validation errors
  #[error("invalid token: {0}")]
  InvalidToken(TokenValidationError),

  /// Event publishing errors
  #[error("publish error: {0}")]
  PublishError(PublishError),

  /// Vector database errors
  #[error("vector database error: {0}")]
  VectorDbError(#[from] VectorDbError),

  /// Unimplemented functionality (for development)
  #[error("unimplemented: {0}")]
  Unimplemented(String),

  /// Internal/unexpected errors
  #[error("internal error: {0}")]
  Internal(String),
}

/// Database error mapping utility
impl CoreError {
  /// Map sqlx database errors to specific CoreError variants
  pub fn from_database_error(error: sqlx::Error) -> Self {
    if let Some(db_err) = error.as_database_error() {
      // Handle constraint violations
      if db_err.is_foreign_key_violation() {
        return Self::map_foreign_key_error(&error.to_string());
      }

      if db_err.is_unique_violation() {
        return Self::map_unique_constraint_error(&error.to_string());
      }

      // Handle other constraint types
      if db_err.is_check_violation() {
        return CoreError::Validation(format!("Data validation failed: {}", db_err.message()));
      }
    }

    // Default to generic database error
    CoreError::Database(error.to_string())
  }

  /// Map foreign key constraint errors to specific business errors
  fn map_foreign_key_error(error_msg: &str) -> Self {
    if error_msg.contains("chat_members_user_id_fkey") {
      CoreError::UserNotFound("Cannot add user to chat: user does not exist".to_string())
    } else if error_msg.contains("chat_members_chat_id_fkey") {
      CoreError::ChatNotFound("Cannot add user to chat: chat does not exist".to_string())
    } else if error_msg.contains("messages_chat_id_fkey") {
      CoreError::ChatNotFound("Cannot send message: chat does not exist".to_string())
    } else if error_msg.contains("messages_sender_id_fkey") {
      CoreError::UserNotFound("Cannot send message: sender does not exist".to_string())
    } else if error_msg.contains("refresh_tokens_user_id_fkey") {
      CoreError::UserNotFound("Cannot create refresh token: user does not exist".to_string())
    } else if error_msg.contains("users_workspace_id_fkey") {
      CoreError::NotFound("Cannot create user: workspace does not exist".to_string())
    } else {
      // Generic foreign key error
      CoreError::ForeignKeyViolation(format!("Referenced resource not found: {}", error_msg))
    }
  }

  /// Map unique constraint errors to specific business errors  
  fn map_unique_constraint_error(error_msg: &str) -> Self {
    if error_msg.contains("users_email_key") {
      CoreError::UserAlreadyExists("A user with this email already exists".to_string())
    } else if error_msg.contains("workspaces_name_key") {
      CoreError::Conflict("A workspace with this name already exists".to_string())
    } else if error_msg.contains("refresh_tokens_token_hash_key") {
      CoreError::Conflict("Refresh token already exists".to_string())
    } else {
      // Generic unique constraint error
      CoreError::UniqueViolation(format!("Resource already exists: {}", error_msg))
    }
  }
}

// Implement conversions from common error types to CoreError
impl From<sqlx::Error> for CoreError {
  fn from(err: sqlx::Error) -> Self {
    CoreError::from_database_error(err)
  }
}

impl From<jsonwebtoken::errors::Error> for CoreError {
  fn from(err: jsonwebtoken::errors::Error) -> Self {
    CoreError::Authentication(err.to_string())
  }
}

impl From<anyhow::Error> for CoreError {
  fn from(err: anyhow::Error) -> Self {
    CoreError::Internal(err.to_string())
  }
}

impl From<PublishError> for CoreError {
  fn from(err: PublishError) -> Self {
    CoreError::PublishError(err)
  }
}

impl From<serde_json::Error> for PublishError {
  fn from(err: serde_json::Error) -> Self {
    PublishError::Serialize(err.to_string())
  }
}

impl From<std::io::Error> for PublishError {
  fn from(err: std::io::Error) -> Self {
    match err.kind() {
      std::io::ErrorKind::TimedOut => PublishError::Timeout(err.to_string()),
      _ => PublishError::Network(err.to_string()),
    }
  }
}

pub trait ErrorMapper {
  type Error;

  fn map_error(error: CoreError) -> Self::Error;
}

pub trait TryFromRow<T>: Sized {
  fn try_from_row(row: PgRow) -> Result<Self, CoreError>;
}

impl<T, E> TryFromRow<PgRow> for T
where
  T: TryFrom<PgRow, Error = E>,
  E: Into<CoreError>,
{
  fn try_from_row(row: PgRow) -> Result<Self, CoreError> {
    T::try_from(row).map_err(Into::into)
  }
}
