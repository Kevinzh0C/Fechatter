use axum::http::StatusCode;
use axum::response::Json;
use axum::{
  body::Body,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use fechatter_core::error::{ChatValidationError, CoreError, ErrorMapper};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorOutput {
  pub code: u16,
  pub error: String,
}

impl ErrorOutput {
  pub fn new(error: impl Into<String>) -> Self {
    Self {
      code: StatusCode::BAD_REQUEST.as_u16(),
      error: error.into(),
    }
  }
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AppError {
  #[error("sqlx error: {0}")]
  SqlxError(sqlx::Error),

  #[error("invalid input: {0}")]
  InvalidInput(String),

  #[error("Not Found: {}", .0.join(", "))]
  NotFound(Vec<String>),

  #[error("conflict: {0}")]
  Conflict(String),

  #[error("io error: {0}")]
  IOError(std::io::Error),

  #[error("password hash error: {0}")]
  PasswordHashError(argon2::password_hash::Error),

  #[error("database error: {0}")]
  JwtError(jsonwebtoken::errors::Error),

  #[error("unauthorized: {0}")]
  Unauthorized(String),

  #[error("internal error: {0}")]
  AnyError(anyhow::Error),

  #[error("http header error: {0}")]
  HttpHeaderError(axum::http::header::InvalidHeaderValue),

  #[error("user already exists: {0}")]
  UserAlreadyExists(String),

  #[error("workspace already exists: {0}")]
  WorkspaceAlreadyExists(String),

  #[error("chat already exists: {0}")]
  ChatAlreadyExists(String),

  #[error("chat validation error: {0}")]
  ChatValidationError(String),

  #[error("chat permission error: {0}")]
  ChatPermissionError(String),

  #[error("chat file error: {0}")]
  ChatFileError(String),

  #[error("NATS error: {0}")]
  NatsError(String),

  #[error("event publishing error: {0}")]
  EventPublishingError(String),

  #[error("search error: {0}")]
  SearchError(String),

  #[error("permission denied: {0}")]
  PermissionDenied(String),

  #[error("forbidden: {0}")]
  Forbidden(String),

  #[error("validation error: {0}")]
  ValidationError(String),

  #[error("server error: {0}")]
  ServerError(String),

  #[error("external service error: {0}")]
  ExternalService(String),

  #[error("internal error: {0}")]
  Internal(String),

  #[error("transport error: {0}")]
  TransportError(EventTransportError),

  #[error("redis error: {0}")]
  RedisError(String),

  #[error("configuration error: {0}")]
  Configuration(String),
}

/// Error types for event transport operations - Centralized Error Management
#[derive(Debug, Error)]
pub enum EventTransportError {
  #[error("Connection error: {0}")]
  Connection(String),

  #[error("Publish error: {0}")]
  Publish(String),

  #[error("Invalid header: {0}")]
  InvalidHeader(String),

  #[error("Timeout: {0}")]
  Timeout(String),

  #[error("IO error: {0}")]
  Io(String),

  #[error("Not implemented: {0}")]
  NotImplemented(String),

  #[error("Unknown error: {0}")]
  Other(String),
}

impl EventTransportError {
  /// Check if this error is retryable
  pub fn is_retryable(&self) -> bool {
    matches!(
      self,
      EventTransportError::Connection(_)
        | EventTransportError::Timeout(_)
        | EventTransportError::Io(_)
    )
  }
}

/// Convert async_nats errors to EventTransportError - Centralized Error Management
impl From<async_nats::Error> for EventTransportError {
  fn from(error: async_nats::Error) -> Self {
    // Simplified error handling for compatibility
    EventTransportError::Other(error.to_string())
  }
}

/// Pure error type conversion - No business logic
/// Maps CoreError to AppError for application layer use
pub fn map_core_error_to_app_error(core_error: CoreError) -> AppError {
  match core_error {
    CoreError::Database(msg) => AppError::Internal(format!("Database error: {}", msg)),
    CoreError::Internal(msg) => AppError::Internal(msg),
    CoreError::NotFound(msg) => AppError::NotFound(vec![msg]),
    CoreError::Validation(msg) => AppError::InvalidInput(msg),
    CoreError::ChatValidation(e) => match e {
      ChatValidationError::InvalidName(msg) => AppError::ChatValidationError(msg),
      ChatValidationError::InvalidMembers(msg) => AppError::ChatValidationError(msg),
      ChatValidationError::PermissionDenied(msg) => AppError::ChatPermissionError(msg),
      ChatValidationError::MemberNotFound(msg) => AppError::NotFound(vec![msg]),
      ChatValidationError::ChatNotFound(msg) => AppError::NotFound(vec![msg]),
    },
    CoreError::Unauthorized(msg) => AppError::ChatPermissionError(msg),
    CoreError::UserAlreadyExists(msg) => AppError::InvalidInput(msg),
    CoreError::Conflict(msg) => AppError::Conflict(msg),
    CoreError::Authentication(msg) => AppError::Unauthorized(msg),
    CoreError::InvalidToken(_) => AppError::Unauthorized("Invalid token".to_string()),
    CoreError::VectorDbError(e) => AppError::InvalidInput(format!("Vector database error: {}", e)),
    CoreError::ValidationError(e) => AppError::ValidationError(e),
    CoreError::PublishError(e) => AppError::EventPublishingError(e.to_string()),
    CoreError::Unimplemented(e) => AppError::AnyError(anyhow::anyhow!(e)),
  }
}

impl ErrorMapper for AppError {
  type Error = AppError;
  fn map_error(error: CoreError) -> Self::Error {
    map_core_error_to_app_error(error)
  }
}

/// Convert EventTransportError to AppError
impl From<EventTransportError> for AppError {
  fn from(error: EventTransportError) -> Self {
    AppError::TransportError(error)
  }
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response<Body> {
    let status = match &self {
      AppError::UserAlreadyExists(_) => StatusCode::CONFLICT,
      AppError::NotFound(_) => StatusCode::NOT_FOUND,
      AppError::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::PasswordHashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
      AppError::JwtError(_) => StatusCode::FORBIDDEN,
      AppError::AnyError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::HttpHeaderError(_) => StatusCode::UNPROCESSABLE_ENTITY,
      AppError::ChatValidationError(_) => StatusCode::BAD_REQUEST,
      AppError::ChatAlreadyExists(_) => StatusCode::CONFLICT,
      AppError::ChatPermissionError(_) => StatusCode::FORBIDDEN,
      AppError::WorkspaceAlreadyExists(_) => StatusCode::CONFLICT,
      AppError::InvalidInput(_) => StatusCode::BAD_REQUEST,
      AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
      AppError::Conflict(_) => StatusCode::CONFLICT,
      AppError::IOError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::ChatFileError(_) => StatusCode::NOT_FOUND,
      AppError::NatsError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::EventPublishingError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::SearchError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::PermissionDenied(_) => StatusCode::FORBIDDEN,
      AppError::Forbidden(_) => StatusCode::FORBIDDEN,
      AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
      AppError::ServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::ExternalService(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::TransportError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::RedisError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    let code = status.as_u16();
    let body = Json(ErrorOutput {
      code,
      error: self.to_string(),
    });
    (status, body).into_response()
  }
}

impl From<sqlx::Error> for AppError {
  fn from(error: sqlx::Error) -> Self {
    Self::SqlxError(error)
  }
}

impl From<std::io::Error> for AppError {
  fn from(error: std::io::Error) -> Self {
    Self::IOError(error)
  }
}

impl From<argon2::password_hash::Error> for AppError {
  fn from(error: argon2::password_hash::Error) -> Self {
    Self::PasswordHashError(error)
  }
}

impl From<jsonwebtoken::errors::Error> for AppError {
  fn from(error: jsonwebtoken::errors::Error) -> Self {
    Self::JwtError(error)
  }
}

impl From<anyhow::Error> for AppError {
  fn from(error: anyhow::Error) -> Self {
    Self::AnyError(error)
  }
}

impl From<axum::http::header::InvalidHeaderValue> for AppError {
  fn from(error: axum::http::header::InvalidHeaderValue) -> Self {
    Self::HttpHeaderError(error)
  }
}

impl From<CoreError> for AppError {
  fn from(error: CoreError) -> Self {
    Self::map_error(error)
  }
}
