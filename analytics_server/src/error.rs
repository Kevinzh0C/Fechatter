use axum::http::StatusCode;
use axum::response::Json;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::warn;
use utoipa::ToSchema;

#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct ErrorOutput {
  pub error: String,
}

#[derive(Error, Debug)]
pub enum AppError {
  #[error("clickhouse error: {0}")]
  ClickhouseError(#[from] clickhouse::error::Error),

  #[error("missing event context")]
  MissingEventContext,

  #[error("missing event data")]
  MissingEventData,

  #[error("missing system info")]
  MissingSystemInfo,

  #[error("general error: {0}")]
  AnyError(#[from] anyhow::Error),
}

impl ErrorOutput {
  pub fn new(error: impl Into<String>) -> Self {
    Self {
      error: error.into(),
    }
  }
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response<axum::body::Body> {
    let status = match &self {
      Self::MissingEventContext => StatusCode::BAD_REQUEST,
      Self::MissingEventData => StatusCode::BAD_REQUEST,
      Self::MissingSystemInfo => StatusCode::BAD_REQUEST,
      Self::ClickhouseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      Self::AnyError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let msg = self.to_string();
    warn!("Status: {}, error: {}", status, msg);

    (status, Json(ErrorOutput::new(msg))).into_response()
  }
}
