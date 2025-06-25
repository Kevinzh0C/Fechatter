use anyhow;
use axum::{
  Json,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use fechatter_core::{CoreError, ErrorMapper};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotifyError {
  #[error("IO error: {0}")]
  IOError(#[from] std::io::Error),

  #[error("JWT error: {0}")]
  JwtError(#[from] jsonwebtoken::errors::Error),

  #[error("sqlx error: {0}")]
  SqlxError(#[from] sqlx::Error),

  #[error("Database error: {0}")]
  Database(String),

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

  #[error("Authentication failed: {0}")]
  AuthenticationFailed(String),

  #[error("Authorization failed: {0}")]
  Unauthorized(String),

  #[error("Serialization error: {0}")]
  Serialization(#[from] serde_json::Error),

  #[error("Internal server error: {0}")]
  Internal(String),

  #[error("Invalid JSON: {0}")]
  InvalidJson(String),

  #[error("Configuration error: {0}")]
  Config(String),

  #[error("NATS error: {0}")]
  Nats(String),
}

impl IntoResponse for NotifyError {
  fn into_response(self) -> Response {
    let (status, error_message) = match self {
      NotifyError::AuthenticationFailed(msg) => (StatusCode::UNAUTHORIZED, msg),
      NotifyError::Unauthorized(msg) => (StatusCode::FORBIDDEN, msg),
      NotifyError::Database(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
      NotifyError::Serialization(err) => (StatusCode::BAD_REQUEST, err.to_string()),
      NotifyError::JwtError(err) => (StatusCode::UNAUTHORIZED, err.to_string()),
      NotifyError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
      NotifyError::InvalidJson(err) => (StatusCode::BAD_REQUEST, err),
      NotifyError::Config(err) => (StatusCode::INTERNAL_SERVER_ERROR, err),
      NotifyError::Nats(err) => (StatusCode::SERVICE_UNAVAILABLE, err),
      _ => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Unhandled error type".to_string(),
      ),
    };

    let body = Json(json!({
        "error": error_message,
        "status": status.as_u16()
    }));

    (status, body).into_response()
  }
}

impl ErrorMapper for NotifyError {
  type Error = NotifyError;

  fn map_error(error: CoreError) -> Self::Error {
    match error {
      CoreError::Database(e) => NotifyError::Database(e.to_string()),
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
