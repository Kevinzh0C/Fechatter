use axum::http::StatusCode;
use axum::response::Json;
use axum::{
  body::Body,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorOutput {
  pub error: String,
}

#[derive(Error, Debug)]
pub enum AppError {
  #[error("user already exists: {0}")]
  UserAlreadyExists(String),

  #[error("workspace already exists: {0}")]
  WorkspaceAlreadyExists(String),

  #[error("Not Found: {}", .0.join(", "))]
  NotFound(Vec<String>),

  #[error("chat already exists: {0}")]
  ChatAlreadyExists(String),

  #[error("invalid input: {0}")]
  InvalidInput(String),

  #[error("sqlx error: {0}")]
  SqlxError(#[from] sqlx::Error),

  #[error("io error: {0}")]
  IOError(#[from] std::io::Error),

  #[error("password hash error: {0}")]
  PasswordHashError(#[from] argon2::password_hash::Error),

  #[error("jwt error: {0}")]
  JwtError(#[from] jsonwebtoken::errors::Error),

  #[error("general error: {0}")]
  AnyError(#[from] anyhow::Error),

  #[error("http header error: {0}")]
  HttpHeaderError(#[from] axum::http::header::InvalidHeaderValue),

  #[error("chat validation error: {0}")]
  ChatValidationError(String),

  #[error("chat permission error: {0}")]
  ChatPermissionError(String),

  #[error("chat file error: {0}")]
  ChatFileError(String),
}

impl ErrorOutput {
  pub fn new(error: impl Into<String>) -> Self {
    Self {
      error: error.into(),
    }
  }
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response<Body> {
    let status_code = match &self {
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
      AppError::IOError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::ChatFileError(_) => StatusCode::NOT_FOUND,
    };

    (status_code, Json(ErrorOutput::new(self.to_string()))).into_response()
  }
}
