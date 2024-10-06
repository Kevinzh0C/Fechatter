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

  #[error("sqlx error: {0}")]
  SqlxError(#[from] sqlx::Error),

  #[error("password hash error: {0}")]
  PasswordHashError(#[from] argon2::password_hash::Error),

  #[error("jwt error: {0}")]
  JwtError(#[from] jsonwebtoken::errors::Error),

  #[error("general error: {0}")]
  AnyError(#[from] anyhow::Error),

  #[error("http header error: {0}")]
  HttpHeaderError(#[from] axum::http::header::InvalidHeaderValue),
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
      AppError::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::PasswordHashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
      AppError::JwtError(_) => StatusCode::FORBIDDEN,
      AppError::AnyError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::HttpHeaderError(_) => StatusCode::UNPROCESSABLE_ENTITY,
    };

    (status_code, Json(ErrorOutput::new(self.to_string()))).into_response()
  }
}
