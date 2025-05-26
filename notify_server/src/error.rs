use anyhow;
use fechatter_core::{CoreError, ErrorMapper};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotifyError {
  #[error("IO error: {0}")]
  IOError(#[from] std::io::Error),

  #[error("JWT error: {0}")]
  JwtError(#[from] jsonwebtoken::errors::Error),

  #[error("sqlx error: {0}")]
  SqlxError(#[from] sqlx::Error),

  #[error("SSE connection error: {0}")]
  SSEConnectionError(#[from] axum::extract::rejection::JsonSyntaxError),

  #[error("HTTP header parsing error: {0}")]
  HttpHeaderParsingError(#[from] axum::http::header::InvalidHeaderValue),

  #[error("Authentication error: {0}")]
  AuthenticationError(String),

  #[error("Not found error: {0}")]
  NotFoundError(String),

  #[error("Conflict error: {0}")]
  ConflictError(String),

  #[error("Unauthorized error: {0}")]
  UnauthorizedError(String),

  #[error("Validation error: {0}")]
  ValidationError(String),

  #[error("any error: {0}")]
  AnyError(#[from] anyhow::Error),
}

impl ErrorMapper for NotifyError {
  type Error = NotifyError;

  fn map_error(error: CoreError) -> Self::Error {
    match error {
      CoreError::Database(e) => NotifyError::SqlxError(sqlx::Error::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        e,
      ))),
      CoreError::Validation(msg) => NotifyError::ValidationError(msg),
      CoreError::Authentication(e) => NotifyError::AuthenticationError(e.to_string()),
      CoreError::NotFound(msg) => NotifyError::NotFoundError(msg),
      CoreError::Conflict(msg) => NotifyError::ConflictError(msg),
      CoreError::Unauthorized(msg) => NotifyError::UnauthorizedError(msg),
      CoreError::Internal(e) => NotifyError::AnyError(anyhow::anyhow!(e)),
      _ => NotifyError::AnyError(anyhow::anyhow!("Unhandled error type")),
    }
  }
}
