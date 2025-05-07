use sqlx::postgres::PgRow;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
  #[error("database error: {0}")]
  SqlxError(#[from] sqlx::Error),

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

#[derive(Error, Debug)]
pub enum CoreError {
  #[error("database error: {0}")]
  Database(#[from] sqlx::Error),

  #[error("validation error: {0}")]
  Validation(String),

  #[error("not found: {0}")]
  NotFound(String),

  #[error("conflict: {0}")]
  Conflict(String),

  #[error("authentication error: {0}")]
  Authentication(#[from] jsonwebtoken::errors::Error),

  #[error("unauthorized: {0}")]
  Unauthorized(String),

  #[error("internal error: {0}")]
  Internal(#[from] anyhow::Error),
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
