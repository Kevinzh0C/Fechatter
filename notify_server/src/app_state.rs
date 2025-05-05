use dashmap::DashMap;
use fechatter_core::{
  TokenVerifier,
  error::{CoreError, ErrorMapper},
  state::{WithDbPool, WithTokenManager},
  utils::jwt::{TokenManager, UserClaims},
};
use sqlx::PgPool;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotifyError {
  #[error("database error: {0}")]
  Database(#[from] sqlx::Error),

  #[error("unauthorized: {0}")]
  Unauthorized(String),

  #[error("internal error: {0}")]
  Internal(#[from] anyhow::Error),
}

impl ErrorMapper for NotifyError {
  type Error = Self;

  fn map_error(error: CoreError) -> Self::Error {
    match error {
      CoreError::Database(e) => NotifyError::Database(e),
      CoreError::Authentication(_) => {
        NotifyError::Unauthorized("Authentication failed".to_string())
      }
      CoreError::Unauthorized(msg) => NotifyError::Unauthorized(msg),
      _ => NotifyError::Internal(anyhow::anyhow!("{}", error)),
    }
  }
}

#[derive(Clone)]
pub struct NotifyState {
  inner: Arc<NotifyStateInner>,
}

struct NotifyStateInner {
  pool: PgPool,
  token_manager: TokenManager,
  event_subscribers: DashMap<i64, Vec<String>>, // user_id -> list of subscription IDs
}

impl WithDbPool for NotifyState {
  fn db_pool(&self) -> &PgPool {
    &self.inner.pool
  }
}

impl WithTokenManager for NotifyState {
  fn token_manager(&self) -> &TokenManager {
    &self.inner.token_manager
  }
}

impl TokenVerifier for NotifyState {
  type Error = NotifyError;
  type Claims = UserClaims;

  fn verify_token(&self, token: &str) -> Result<UserClaims, Self::Error> {
    self
      .inner
      .token_manager
      .verify_token(token)
      .map_err(|_| NotifyError::Unauthorized("Invalid token".to_string()))
  }
}

impl NotifyState {
  pub async fn new(
    db_url: &str,
    auth_config: &fechatter_core::utils::jwt::AuthConfig,
  ) -> Result<Self, NotifyError> {
    let pool = sqlx::postgres::PgPoolOptions::new()
      .max_connections(10)
      .connect(db_url)
      .await?;

    let token_manager =
      TokenManager::from_config(auth_config).map_err(|e| NotifyError::Internal(e.into()))?;

    Ok(Self {
      inner: Arc::new(NotifyStateInner {
        pool,
        token_manager,
        event_subscribers: DashMap::new(),
      }),
    })
  }

  pub fn add_subscriber(&self, user_id: i64, subscription_id: String) {
    self
      .inner
      .event_subscribers
      .entry(user_id)
      .or_insert_with(Vec::new)
      .push(subscription_id);
  }

  pub fn remove_subscriber(&self, user_id: i64, subscription_id: &str) {
    if let Some(mut subscribers) = self.inner.event_subscribers.get_mut(&user_id) {
      subscribers.retain(|id| id != subscription_id);
    }
  }

  pub fn get_subscribers(&self, user_id: i64) -> Vec<String> {
    self
      .inner
      .event_subscribers
      .get(&user_id)
      .map(|subscribers| subscribers.clone())
      .unwrap_or_default()
  }
}
