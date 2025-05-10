use std::sync::Arc;

use async_trait::async_trait;
use fechatter_core::models::jwt::{RefreshToken, RefreshTokenRepository, ReplaceTokenPayload};
use sqlx::PgPool;

pub struct RefreshTokenAdaptor {
  pool: Arc<PgPool>,
}

impl RefreshTokenAdaptor {
  pub fn new(pool: Arc<PgPool>) -> Self {
    Self { pool }
  }
}

// 添加Mock实现用于创建默认TokenManager
pub struct MockRefreshTokenRepo;

impl MockRefreshTokenRepo {
  pub fn new() -> Self {
    Self {}
  }
}

#[async_trait]
impl RefreshTokenRepository for MockRefreshTokenRepo {
  async fn find_by_token(
    &self,
    _token: &str,
  ) -> Result<Option<RefreshToken>, fechatter_core::CoreError> {
    // 简单返回None
    Ok(None)
  }

  async fn create(
    &self,
    _user_id: i64,
    _token: &str,
    _expires_at: chrono::DateTime<chrono::Utc>,
    _absolute_expires_at: chrono::DateTime<chrono::Utc>,
    _user_agent: Option<&str>,
    _ip_address: Option<&str>,
  ) -> Result<RefreshToken, fechatter_core::CoreError> {
    // 返回错误
    Err(fechatter_core::CoreError::Internal(anyhow::anyhow!(
      "Mock: Not implemented"
    )))
  }

  async fn revoke(&self, _token: &str) -> Result<bool, fechatter_core::CoreError> {
    // 成功的空操作
    Ok(true)
  }

  async fn revoke_all_for_user(&self, _user_id: i64) -> Result<bool, fechatter_core::CoreError> {
    // 成功的空操作
    Ok(true)
  }

  async fn replace(
    &self,
    _payload: &ReplaceTokenPayload,
  ) -> Result<RefreshToken, fechatter_core::CoreError> {
    // 返回错误
    Err(fechatter_core::CoreError::Internal(anyhow::anyhow!(
      "Mock: Not implemented"
    )))
  }
}
