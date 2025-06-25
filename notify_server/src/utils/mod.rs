use sqlx::PgPool;
/// Utilities module
///
/// Simplified utility functions and adapters
use std::sync::Arc;

/// Database adapter (simplified)
pub struct RefreshTokenAdaptor {
  _pool: Arc<PgPool>,
}

impl RefreshTokenAdaptor {
  pub fn new(pool: Arc<PgPool>) -> Self {
    Self { _pool: pool }
  }
}

/// Mock implementation for testing
#[derive(Debug, Clone)]
pub struct MockRefreshTokenRepo;

impl MockRefreshTokenRepo {
  pub fn new() -> Self {
    Self {}
  }
}

impl Default for MockRefreshTokenRepo {
  fn default() -> Self {
    Self::new()
  }
}
