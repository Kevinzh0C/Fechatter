use axum::http::StatusCode;
use axum::response::Json;
use axum::{
  body::Body,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use fechatter_core::error::{ChatValidationError, CoreError, ErrorMapper};
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
}

impl ErrorMapper for AppError {
  type Error = AppError;

  fn map_error(error: CoreError) -> Self::Error {
    match error {
      CoreError::Database(e) => AppError::SqlxError(sqlx::Error::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        e,
      ))),
      CoreError::UserAlreadyExists(e) => AppError::UserAlreadyExists(e),
      CoreError::Validation(e) => AppError::ChatValidationError(e),
      CoreError::ChatValidation(e) => match e {
        ChatValidationError::InvalidName(msg) => AppError::ChatValidationError(msg),
        ChatValidationError::InvalidMembers(msg) => AppError::ChatValidationError(msg),
        ChatValidationError::PermissionDenied(msg) => AppError::ChatPermissionError(msg),
        ChatValidationError::MemberNotFound(msg) => AppError::NotFound(vec![msg]),
        ChatValidationError::ChatNotFound(msg) => AppError::NotFound(vec![msg]),
      },
      CoreError::NotFound(e) => AppError::NotFound(vec![e]),
      CoreError::Conflict(e) => AppError::Conflict(e),
      CoreError::Authentication(_e) => AppError::JwtError(jsonwebtoken::errors::Error::from(
        jsonwebtoken::errors::ErrorKind::InvalidToken,
      )),
      CoreError::InvalidToken(_msg) => AppError::JwtError(jsonwebtoken::errors::Error::from(
        jsonwebtoken::errors::ErrorKind::InvalidToken,
      )),
      CoreError::Unauthorized(e) => AppError::Unauthorized(e),
      CoreError::Internal(e) => AppError::AnyError(anyhow::anyhow!(e)),
    }
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
