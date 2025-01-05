use axum::http::StatusCode;
use axum::response::Json;
use axum::{
  body::Body,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use fechatter_core::error::{CoreError, ErrorMapper};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
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
  SqlxError(#[from] sqlx::Error),

  #[error("invalid input: {0}")]
  InvalidInput(String),

  #[error("Not Found: {}", .0.join(", "))]
  NotFound(Vec<String>),

  #[error("conflict: {0}")]
  Conflict(String),

  #[error("io error: {0}")]
  IOError(#[from] std::io::Error),

  #[error("password hash error: {0}")]
  PasswordHashError(#[from] argon2::password_hash::Error),

  #[error("database error: {0}")]
  JwtError(#[from] jsonwebtoken::errors::Error),

  #[error("unauthorized: {0}")]
  Unauthorized(String),

  #[error("internal error: {0}")]
  AnyError(#[from] anyhow::Error),

  #[error("http header error: {0}")]
  HttpHeaderError(#[from] axum::http::header::InvalidHeaderValue),

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
}

impl ErrorMapper for AppError {
  type Error = Self;

  fn map_error(error: CoreError) -> Self::Error {
    match error {
      CoreError::Database(e) => AppError::SqlxError(e),
      CoreError::Validation(msg) => {
        if msg.contains("chat") || msg.contains("Chat") {
          AppError::ChatValidationError(msg)
        } else {
          AppError::InvalidInput(msg)
        }
      }
      CoreError::NotFound(msg) => AppError::NotFound(vec![msg]),
      CoreError::Conflict(msg) => {
        if msg.contains("User") || msg.contains("email") {
          AppError::UserAlreadyExists(msg)
        } else if msg.contains("Workspace") {
          AppError::WorkspaceAlreadyExists(msg)
        } else {
          AppError::ChatAlreadyExists(msg)
        }
      }
      CoreError::Authentication(e) => AppError::JwtError(e),
      CoreError::Unauthorized(msg) => {
        if msg.contains("chat") || msg.contains("Chat") {
          AppError::ChatPermissionError(msg)
        } else {
          AppError::Unauthorized(msg)
        }
      }
      CoreError::Internal(e) => AppError::AnyError(e),
    }
  }
}

impl From<CoreError> for AppError {
  fn from(error: CoreError) -> Self {
    Self::map_error(error)
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
    };

    let code = status.as_u16();
    let body = Json(ErrorOutput {
      code,
      error: self.to_string(),
    });
    (status, body).into_response()
  }
}
