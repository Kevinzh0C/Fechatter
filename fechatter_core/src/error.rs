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

/// Core error types that can be mapped to app-specific errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum CoreError {
  /// Database errors from sqlx
  #[error("database error: {0}")]
  Database(String),

  /// Validation errors for business rules
  #[error("validation error: {0}")]
  Validation(String),

  /// Chat validation specific errors
  #[error("chat validation error: {0}")]
  ChatValidation(ChatValidationError),

  /// Specific validation error for user already exists case
  #[error("user already exists: {0}")]
  UserAlreadyExists(String),

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

  /// Internal/unexpected errors
  #[error("internal error: {0}")]
  Internal(String),
}

// Implement conversions from common error types to CoreError
impl From<sqlx::Error> for CoreError {
  fn from(err: sqlx::Error) -> Self {
    CoreError::Database(err.to_string())
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
