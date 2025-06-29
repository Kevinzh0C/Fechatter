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

use axum::extract::multipart::MultipartError;
use validator::ValidationErrors;

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

  #[error("chat membership error: {message}")]
  ChatMembershipError {
    message: String,
    chat_id: i64,
    user_id: i64,
    membership_status: String,
  },

  #[error("chat access denied: {reason}")]
  ChatAccessDenied {
    reason: String,
    chat_id: i64,
    user_id: i64,
  },

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
  ExternalServiceError(String),

  #[error("internal error: {0}")]
  Internal(String),

  #[error("transport error: {0}")]
  TransportError(EventTransportError),

  #[error("redis error: {0}")]
  RedisError(String),

  #[error("configuration error: {0}")]
  Configuration(String),

  #[error("service unavailable: {0}")]
  ServiceUnavailable(String),

  #[error("Bad request: {0}")]
  BadRequest(String),

  #[error("Serialization error: {0}")]
  SerializationError(String),

  #[error("Event publish error: {0}")]
  EventPublishError(String),

  #[error("Configuration error: {0}")]
  ConfigError(String),

  #[error("Authentication error: {0}")]
  AuthenticationError(String),

  #[error("Operation timed out: {0}")]
  Timeout(String),

  #[error("Rate limit exceeded: {0}")]
  RateLimitExceeded(String),

  #[error("Too many requests: {0}")]
  TooManyRequests(String),

  #[error("Security threat detected: {0}")]
  SecurityThreatDetected(String),

  #[error("Not implemented: {0}")]
  NotImplemented(String),

  #[error("Multipart request error: {0}")]
  MultipartError(String),

  #[error("File upload error: {0}")]
  FileUploadError(String),
}

/// Error types for event transport operations - Centralized Error Management
#[derive(Debug, Error)]
pub enum EventTransportError {
  #[error("Connection error: {0}")]
  Connection(String),

  #[error("Timeout")]
  Timeout,

  #[error("Internal error: {0}")]
  Internal(String),

  #[error("Disconnected")]
  Disconnected,

  #[error("Publish error: {0}")]
  Publish(String),

  #[error("Invalid header: {0}")]
  InvalidHeader(String),

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
        | EventTransportError::Timeout
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
    CoreError::ValidationError(msg) => AppError::ValidationError(msg),
    CoreError::ChatValidation(e) => match e {
      ChatValidationError::InvalidName(msg) => AppError::ChatValidationError(msg),
      ChatValidationError::InvalidMembers(msg) => AppError::ChatValidationError(msg),
      ChatValidationError::PermissionDenied(msg) => AppError::ChatPermissionError(msg),
      ChatValidationError::MemberNotFound(msg) => AppError::NotFound(vec![msg]),
      ChatValidationError::ChatNotFound(msg) => AppError::NotFound(vec![msg]),
    },
    // Specific resource not found errors
    CoreError::UserNotFound(msg) => AppError::NotFound(vec![msg]),
    CoreError::ChatNotFound(msg) => AppError::NotFound(vec![msg]),
    // Constraint violation errors
    CoreError::ForeignKeyViolation(msg) => AppError::BadRequest(msg),
    CoreError::UniqueViolation(msg) => AppError::Conflict(msg),
    // Auth and permission errors
    CoreError::Unauthorized(msg) => AppError::ChatPermissionError(msg),
    CoreError::Authentication(msg) => AppError::Unauthorized(msg),
    CoreError::InvalidToken(_) => AppError::Unauthorized("Invalid token".to_string()),
    // Conflict and duplicate errors
    CoreError::UserAlreadyExists(msg) => AppError::UserAlreadyExists(msg),
    CoreError::Conflict(msg) => AppError::Conflict(msg),
    // Service errors
    CoreError::VectorDbError(e) => AppError::InvalidInput(format!("Vector database error: {}", e)),
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
    tracing::info!("🔍 [HTTP_RESPONSE] ========== Converting AppError to HTTP Response ==========");
    tracing::debug!("🔍 [HTTP_RESPONSE] Input AppError: {:?}", self);

    let status = match &self {
      AppError::UserAlreadyExists(_) => StatusCode::CONFLICT,
      AppError::NotFound(_) => {
        tracing::info!("🔍 [HTTP_RESPONSE] 🔍 NotFound error -> HTTP 404");
        StatusCode::NOT_FOUND
      }
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
      AppError::ChatMembershipError { .. } => {
        tracing::info!("🔍 [HTTP_RESPONSE] 🔍 ChatMembershipError -> HTTP 403");
        StatusCode::FORBIDDEN
      }
      AppError::ChatAccessDenied { .. } => {
        tracing::info!("🔍 [HTTP_RESPONSE] 🔍 ChatAccessDenied -> HTTP 403");
        StatusCode::FORBIDDEN
      }
      AppError::NatsError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::EventPublishingError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::SearchError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::PermissionDenied(_) => StatusCode::FORBIDDEN,
      AppError::Forbidden(_) => StatusCode::FORBIDDEN,
      AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
      AppError::ServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::ExternalServiceError(_) => StatusCode::BAD_GATEWAY,
      AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::TransportError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::RedisError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
      AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
      AppError::SerializationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::EventPublishError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::ConfigError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::AuthenticationError(_) => StatusCode::UNAUTHORIZED,
      AppError::Timeout(_) => StatusCode::REQUEST_TIMEOUT,
      AppError::RateLimitExceeded(_) => StatusCode::TOO_MANY_REQUESTS,
      AppError::TooManyRequests(_) => StatusCode::TOO_MANY_REQUESTS,
      AppError::SecurityThreatDetected(_) => StatusCode::FORBIDDEN,
      AppError::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,
      AppError::MultipartError(_) => StatusCode::BAD_REQUEST,
      AppError::FileUploadError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    let code = status.as_u16();
    tracing::error!(
      "🔍 [HTTP_RESPONSE] ✅ Final HTTP Status: {} ({})",
      code,
      status.canonical_reason().unwrap_or("Unknown")
    );
    tracing::debug!("🔍 [HTTP_RESPONSE] Error message: {}", self.to_string());

    let body = Json(ErrorOutput {
      code,
      error: self.to_string(),
    });

    tracing::info!("🔍 [HTTP_RESPONSE] ========== HTTP Response Generated ==========");
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

impl From<std::time::SystemTimeError> for AppError {
  fn from(error: std::time::SystemTimeError) -> Self {
    AppError::Internal(format!("System time error: {}", error))
  }
}

// ============================================================================
// Enhanced Error Types - Advanced error handling with context
// ============================================================================

use std::collections::HashMap;
use uuid::Uuid;

/// Enhanced error context with correlation ID and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
  pub correlation_id: String,
  pub user_id: Option<i64>,
  pub workspace_id: Option<i64>,
  pub request_path: Option<String>,
  pub metadata: HashMap<String, String>,
  pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for ErrorContext {
  fn default() -> Self {
    Self {
      correlation_id: Uuid::new_v4().to_string(),
      user_id: None,
      workspace_id: None,
      request_path: None,
      metadata: HashMap::new(),
      timestamp: chrono::Utc::now(),
    }
  }
}

/// Enhanced error types with detailed categorization
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum EnhancedError {
  #[error("Authentication failed: {message}")]
  Authentication {
    message: String,
    context: ErrorContext,
    error_code: String,
  },

  #[error("Validation failed: {message}")]
  Validation {
    message: String,
    context: ErrorContext,
    field: Option<String>,
    value: Option<String>,
  },

  #[error("Database error: {message}")]
  Database {
    message: String,
    context: ErrorContext,
    operation: String,
    table: Option<String>,
  },

  #[error("Resource not found: {message}")]
  NotFound {
    message: String,
    context: ErrorContext,
    resource_type: String,
    resource_id: Option<String>,
  },

  #[error("Internal server error: {message}")]
  Internal {
    message: String,
    context: ErrorContext,
    source_error: Option<String>,
  },
}

impl EnhancedError {
  /// Get the HTTP status code for this error
  pub fn status_code(&self) -> u16 {
    match self {
      EnhancedError::Authentication { .. } => 401,
      EnhancedError::Validation { .. } => 400,
      EnhancedError::NotFound { .. } => 404,
      EnhancedError::Database { .. } => 500,
      EnhancedError::Internal { .. } => 500,
    }
  }

  /// Get the error context
  pub fn context(&self) -> &ErrorContext {
    match self {
      EnhancedError::Authentication { context, .. } => context,
      EnhancedError::Validation { context, .. } => context,
      EnhancedError::Database { context, .. } => context,
      EnhancedError::NotFound { context, .. } => context,
      EnhancedError::Internal { context, .. } => context,
    }
  }

  /// Log the error with appropriate level
  pub fn log_error(&self) {
    use tracing::{error, warn};

    match self {
      EnhancedError::Internal { .. } | EnhancedError::Database { .. } => {
        error!(
            correlation_id = %self.context().correlation_id,
            error_message = %self,
            "Error occurred"
        );
      }
      _ => {
        warn!(
            correlation_id = %self.context().correlation_id,
            error_message = %self,
            "Error occurred"
        );
      }
    }
  }
}

/// Error builders for common error types
pub struct ErrorBuilder;

impl ErrorBuilder {
  pub fn authentication(message: impl Into<String>) -> EnhancedError {
    EnhancedError::Authentication {
      message: message.into(),
      context: ErrorContext::default(),
      error_code: "AUTH_FAILED".to_string(),
    }
  }

  pub fn validation(message: impl Into<String>) -> EnhancedError {
    EnhancedError::Validation {
      message: message.into(),
      context: ErrorContext::default(),
      field: None,
      value: None,
    }
  }

  pub fn not_found(resource_type: impl Into<String>) -> EnhancedError {
    let resource_type = resource_type.into();
    EnhancedError::NotFound {
      message: format!("{} not found", resource_type),
      context: ErrorContext::default(),
      resource_type,
      resource_id: None,
    }
  }

  pub fn database(operation: impl Into<String>, error: impl std::error::Error) -> EnhancedError {
    EnhancedError::Database {
      message: error.to_string(),
      context: ErrorContext::default(),
      operation: operation.into(),
      table: None,
    }
  }

  pub fn internal(message: impl Into<String>) -> EnhancedError {
    EnhancedError::Internal {
      message: message.into(),
      context: ErrorContext::default(),
      source_error: None,
    }
  }
}

/// Convert ChatMembershipStatus to AppError with precise error classification
pub fn membership_status_to_app_error(
  status: crate::domains::chat::ChatMembershipStatus,
) -> AppError {
  use crate::domains::chat::ChatMembershipStatus;

  tracing::info!(
    "🔍 [ERROR_CONVERTER] ========== Converting membership status to AppError =========="
  );
  tracing::debug!("🔍 [ERROR_CONVERTER] Input status: {:?}", status);

  let app_error = match status {
    ChatMembershipStatus::ActiveMember { .. } => {
      // This should not happen in error scenarios
      tracing::warn!(
        "🔍 [ERROR_CONVERTER] ⚠️ Active member status in error context - this is unexpected"
      );
      AppError::Internal("Active member status in error context".to_string())
    }
    ChatMembershipStatus::ChatNotFound { chat_id } => {
      tracing::info!(
        "🔍 [ERROR_CONVERTER] 🔍 Converting ChatNotFound to AppError::NotFound for chat_id={}",
        chat_id
      );
      AppError::NotFound(vec![format!("Chat {} not found", chat_id)])
    }
    ChatMembershipStatus::NotMember { chat_id, user_id } => {
      tracing::info!("🔍 [ERROR_CONVERTER] 🔍 Converting NotMember to AppError::ChatAccessDenied for user={} in chat={}", user_id, chat_id);
      AppError::ChatAccessDenied {
        reason: "User is not a member of this chat".to_string(),
        chat_id,
        user_id,
      }
    }
    ChatMembershipStatus::UserLeftChat {
      chat_id,
      user_id,
      left_at,
    } => {
      tracing::info!("🔍 [ERROR_CONVERTER] 🔍 Converting UserLeftChat to AppError::ChatAccessDenied for user={} in chat={}, left_at={}", user_id, chat_id, left_at);
      AppError::ChatAccessDenied {
        reason: format!(
          "User left this chat on {}",
          left_at.format("%Y-%m-%d %H:%M:%S")
        ),
        chat_id,
        user_id,
      }
    }
    ChatMembershipStatus::DataInconsistency {
      chat_id,
      user_id,
      issue,
    } => {
      tracing::error!("🔍 [ERROR_CONVERTER] ❌ Converting DataInconsistency to AppError::ChatMembershipError for user={} in chat={}, issue={}", user_id, chat_id, issue);
      AppError::ChatMembershipError {
        message: format!("Data inconsistency detected: {}", issue),
        chat_id,
        user_id,
        membership_status: "inconsistent".to_string(),
      }
    }
  };

  tracing::info!(
    "🔍 [ERROR_CONVERTER] ✅ Generated AppError: {:?}",
    app_error
  );
  tracing::debug!(
    "🔍 [ERROR_CONVERTER] Error will generate HTTP status: {}",
    match &app_error {
      AppError::NotFound(_) => 404,
      AppError::ChatAccessDenied { .. } => 403,
      AppError::ChatMembershipError { .. } => 403,
      AppError::Internal(_) => 500,
      _ => 500,
    }
  );

  app_error
}

impl From<MultipartError> for AppError {
  fn from(err: MultipartError) -> Self {
    AppError::MultipartError(err.to_string())
  }
}
