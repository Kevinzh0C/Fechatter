use crate::AppError;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};

use fechatter_core::error::CoreError;
use fechatter_core::jwt::{
  RefreshToken as CoreRefreshToken, RefreshTokenRepository, ReplaceTokenPayload, StoreTokenPayload,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

pub const REFRESH_TOKEN_EXPIRATION: usize = 14 * 24 * 60 * 60; // 14 days
pub const REFRESH_TOKEN_MAX_LIFETIME: usize = 30 * 24 * 60 * 60; // 30 days

#[allow(unused)]
pub fn generate_refresh_token() -> String {
  use rand::{Rng, rng};

  let mut rng_instance = rng();
  let random_bytes: [u8; 32] = rng_instance.random::<[u8; 32]>();
  hex::encode(random_bytes)
}

pub fn sha256_hash(token: &str) -> String {
  use sha2::{Digest, Sha256};

  let mut hasher = Sha256::new();
  hasher.update(token.as_bytes());
  let result = hasher.finalize();
  let hash = hex::encode(result);
  hash
}

// 数据库实体，专门用于与数据库交互
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RefreshTokenEntity {
  pub id: i64,
  pub user_id: i64,
  pub token_hash: String,
  pub expires_at: DateTime<Utc>,
  pub issued_at: DateTime<Utc>,
  pub revoked: bool,
  pub replaced_by: Option<String>,
  pub user_agent: Option<String>,
  pub ip_address: Option<String>,
  pub absolute_expires_at: DateTime<Utc>,
}

// 只负责转换的扩展trait
impl RefreshTokenEntity {
  pub fn to_dto(&self) -> CoreRefreshToken {
    CoreRefreshToken {
      id: self.id,
      user_id: self.user_id,
      token_hash: self.token_hash.clone(),
      expires_at: self.expires_at,
      issued_at: self.issued_at,
      revoked: self.revoked,
      replaced_by: self.replaced_by.clone(),
      user_agent: self.user_agent.clone(),
      ip_address: self.ip_address.clone(),
      absolute_expires_at: self.absolute_expires_at,
    }
  }
}

// 数据库操作，与实体分离
pub struct RefreshTokenStorage;

impl RefreshTokenStorage {
  // 数据库操作
  pub async fn create(
    user_id: i64,
    token: &str,
    user_agent: Option<String>,
    ip_address: Option<String>,
    pool: &PgPool,
  ) -> Result<RefreshTokenEntity, AppError> {
    let now = Utc::now();
    let expires_at = now + Duration::seconds(REFRESH_TOKEN_EXPIRATION as i64);
    let absolute_expires_at = now + Duration::seconds(REFRESH_TOKEN_MAX_LIFETIME as i64);
    let token_hash = sha256_hash(token);

    let refresh_token = sqlx::query_as::<_, RefreshTokenEntity>(
      r#"
      INSERT INTO refresh_tokens (user_id, token_hash, expires_at, user_agent, ip_address, absolute_expires_at)
      VALUES ($1, $2, $3, $4, $5, $6)
      RETURNING id, user_id, token_hash, expires_at, issued_at, revoked, replaced_by, user_agent, ip_address, absolute_expires_at
      "#,
    )
    .bind(user_id)
    .bind(&token_hash)
    .bind(expires_at)
    .bind(user_agent)
    .bind(ip_address)
    .bind(absolute_expires_at)
    .fetch_one(pool)
    .await?;

    Ok(refresh_token)
  }

  pub async fn find_by_token(
    token: &str,
    pool: &PgPool,
  ) -> Result<Option<RefreshTokenEntity>, AppError> {
    let token_hash = sha256_hash(token);

    let refresh_token = sqlx::query_as::<_, RefreshTokenEntity>(
      r#"
      SELECT id, user_id, token_hash, expires_at, issued_at, revoked, replaced_by, user_agent, ip_address, absolute_expires_at
      FROM refresh_tokens
      WHERE token_hash = $1 AND revoked = FALSE AND expires_at > NOW()
      "#,
    )
    .bind(&token_hash)
    .fetch_optional(pool)
    .await?;

    Ok(refresh_token)
  }

  pub async fn revoke(id: i64, pool: &PgPool) -> Result<(), AppError> {
    sqlx::query(
      r#"
      UPDATE refresh_tokens
      SET revoked = TRUE
      WHERE id = $1
      "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
  }

  /// Revokes all refresh tokens for a user. Used in security scenarios like:
  /// - Password changes requiring re-login on all devices
  /// - Account being disabled/banned by admin
  /// - Responding to suspicious activity
  /// - Logout from all devices
  #[allow(dead_code)]
  pub async fn revoke_all_for_user(user_id: i64, pool: &PgPool) -> Result<(), AppError> {
    sqlx::query(
      r#"
      UPDATE refresh_tokens
      SET revoked = TRUE
      WHERE user_id = $1
      "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
  }

  pub async fn replace(
    old_token_id: i64,
    user_id: i64,
    new_token: &str,
    user_agent: Option<String>,
    ip_address: Option<String>,
    absolute_expires_at: DateTime<Utc>,
    pool: &PgPool,
  ) -> Result<RefreshTokenEntity, AppError> {
    let now = chrono::Utc::now();
    let new_expires_at = now + Duration::seconds(REFRESH_TOKEN_EXPIRATION as i64);
    let new_token_hash = sha256_hash(new_token);

    // 使用事务和行级锁确保并发安全
    let mut tx = pool.begin().await?;

    // 首先检查令牌是否已经被撤销或替换
    // 使用 FOR UPDATE 获取行锁，确保在事务期间其他会话无法修改该行
    let token_status = sqlx::query!(
      r#"
      SELECT revoked, replaced_by 
      FROM refresh_tokens 
      WHERE id = $1
      FOR UPDATE
      "#,
      old_token_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    // 如果令牌不存在或已经被撤销/替换，返回错误
    match token_status {
      None => {
        tx.rollback().await?;
        return Err(AppError::NotFound(vec![format!(
          "Refresh token with id {} not found",
          old_token_id
        )]));
      }
      Some(status) if status.revoked || status.replaced_by.is_some() => {
        tx.rollback().await?;
        return Err(AppError::JwtError(jsonwebtoken::errors::Error::from(
          jsonwebtoken::errors::ErrorKind::InvalidToken,
        )));
      }
      _ => {} // 令牌存在且未被撤销
    }

    // 使用新令牌替换旧令牌
    sqlx::query!(
      r#"
      UPDATE refresh_tokens
      SET revoked = TRUE, replaced_by = $1
      WHERE id = $2
      "#,
      &new_token_hash,
      old_token_id
    )
    .execute(&mut *tx)
    .await?;

    // 创建新的刷新令牌
    let refresh_token = sqlx::query_as::<_, RefreshTokenEntity>(
      r#"
      INSERT INTO refresh_tokens (user_id, token_hash, expires_at, user_agent, ip_address, absolute_expires_at)
      VALUES ($1, $2, $3, $4, $5, $6)
      RETURNING id, user_id, token_hash, expires_at, issued_at, revoked, replaced_by, user_agent, ip_address, absolute_expires_at
      "#,
    )
    .bind(user_id)
    .bind(&new_token_hash)
    .bind(if new_expires_at < absolute_expires_at { new_expires_at } else { absolute_expires_at })
    .bind(user_agent)
    .bind(ip_address)
    .bind(absolute_expires_at)
    .fetch_one(&mut *tx)
    .await?;

    // 提交事务
    tx.commit().await?;

    Ok(refresh_token)
  }
}

// 适配器实现 - 将RefreshTokenStorage适配到RefreshTokenRepository trait
pub struct RefreshTokenAdaptor {
  pool: Arc<PgPool>,
}

impl RefreshTokenAdaptor {
  pub fn new(pool: Arc<PgPool>) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl RefreshTokenRepository for RefreshTokenAdaptor {
  async fn find_by_token(&self, raw_token: &str) -> Result<Option<CoreRefreshToken>, CoreError> {
    let pool = self.pool.clone();
    let token = raw_token.to_string();
    let found = RefreshTokenStorage::find_by_token(&token, &pool)
      .await
      .map_err(|e| CoreError::Internal(e.to_string()))?;

    Ok(found.map(|token| token.to_dto()))
  }

  async fn replace(&self, payload: ReplaceTokenPayload) -> Result<CoreRefreshToken, CoreError> {
    let pool = self.pool.clone();

    // 从payload中找出related_token记录获取user_id
    let old_token_id = payload.old_token_id;

    // 首先从旧token ID查询用户ID
    let user_id = sqlx::query_scalar::<_, i64>("SELECT user_id FROM refresh_tokens WHERE id = $1")
      .bind(old_token_id)
      .fetch_one(&*pool)
      .await
      .map_err(|e| CoreError::Internal(e.to_string()))?;

    // 调用RefreshTokenStorage::replace来执行token替换
    let result = RefreshTokenStorage::replace(
      old_token_id,
      user_id,
      &payload.new_raw_token,
      payload.user_agent.clone(),
      payload.ip_address.clone(),
      payload.new_absolute_expires_at,
      &pool,
    )
    .await
    .map_err(|e| CoreError::Internal(e.to_string()))?;

    Ok(result.to_dto())
  }

  async fn revoke(&self, token_id: i64) -> Result<(), CoreError> {
    let pool = self.pool.clone();
    RefreshTokenStorage::revoke(token_id, &pool)
      .await
      .map_err(|e| CoreError::Internal(e.to_string()))
  }

  async fn revoke_all_for_user(&self, user_id: i64) -> Result<(), CoreError> {
    let pool = self.pool.clone();
    RefreshTokenStorage::revoke_all_for_user(user_id, &pool)
      .await
      .map_err(|e| CoreError::Internal(e.to_string()))
  }

  async fn create(&self, payload: StoreTokenPayload) -> Result<CoreRefreshToken, CoreError> {
    let pool = self.pool.clone();
    let result = RefreshTokenStorage::create(
      payload.user_id,
      &payload.raw_token,
      payload.user_agent.clone(),
      payload.ip_address.clone(),
      &pool,
    )
    .await
    .map_err(|e| CoreError::Internal(e.to_string()))?;

    Ok(result.to_dto())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{AppConfig, models::UserStatus, setup_test_users};
  use anyhow::Result;
  use fechatter_core::TokenService;
  use fechatter_core::{User, UserClaims, jwt::TokenManager};

  // Mock repository implementation for test environment
  #[derive(Clone)]
  struct MockRefreshTokenRepo;

  #[async_trait::async_trait]
  impl RefreshTokenRepository for MockRefreshTokenRepo {
    async fn find_by_token(&self, _raw_token: &str) -> Result<Option<CoreRefreshToken>, CoreError> {
      Ok(None)
    }

    async fn replace(&self, _payload: ReplaceTokenPayload) -> Result<CoreRefreshToken, CoreError> {
      Err(CoreError::Internal("Not implemented".to_string()))
    }

    async fn revoke(&self, _token_id: i64) -> Result<(), CoreError> {
      Ok(())
    }

    async fn revoke_all_for_user(&self, _user_id: i64) -> Result<(), CoreError> {
      Ok(())
    }

    async fn create(&self, _payload: StoreTokenPayload) -> Result<CoreRefreshToken, CoreError> {
      // 返回一个假的RefreshToken用于测试
      let now = Utc::now();
      Ok(CoreRefreshToken {
        id: 1,
        user_id: _payload.user_id,
        token_hash: "fake_hash".to_string(),
        expires_at: _payload.expires_at,
        issued_at: now,
        revoked: false,
        replaced_by: None,
        user_agent: _payload.user_agent,
        ip_address: _payload.ip_address,
        absolute_expires_at: _payload.absolute_expires_at,
      })
    }
  }

  #[test]
  fn jwt_token_authentication_should_work() -> Result<()> {
    let config = AppConfig::load()?;
    let token_manager = TokenManager::from_config(&config.auth, Arc::new(MockRefreshTokenRepo))?;

    let user = User {
      id: 1,
      fullname: "John Doe".to_string(),
      email: "john.doe@example.com".to_string(),
      password_hash: Default::default(),
      status: UserStatus::Active,
      created_at: chrono::Utc::now(),
      workspace_id: 1,
    };
    let user_claims = UserClaims {
      id: user.id,
      workspace_id: user.workspace_id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
    };

    let token = token_manager.generate_token_for_user(&user)?;

    let user_claims2 = token_manager.verify_token(&token)?;

    // Compare individual fields instead of the whole struct
    assert_eq!(user_claims.id, user_claims2.id);
    assert_eq!(user_claims.email, user_claims2.email);
    assert_eq!(user_claims.fullname, user_claims2.fullname);
    assert_eq!(user_claims.workspace_id, user_claims2.workspace_id);

    Ok(())
  }

  #[tokio::test]
  async fn refresh_token_create_and_find_works() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    let token_str = generate_refresh_token();

    // 使用RefreshTokenStorage创建token
    let _token = RefreshTokenStorage::create(
      user.id,
      &token_str,
      Some("test-agent".to_string()),
      Some("127.0.0.1".to_string()),
      state.pool(),
    )
    .await?;

    let found_token = RefreshTokenStorage::find_by_token(&token_str, state.pool()).await?;

    assert!(found_token.is_some());
    let found_token = found_token.unwrap();
    assert_eq!(found_token.user_id, user.id);
    assert_eq!(found_token.user_agent, Some("test-agent".to_string()));
    assert_eq!(found_token.ip_address, Some("127.0.0.1".to_string()));

    Ok(())
  }

  #[tokio::test]
  async fn refresh_token_revoke_works() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    let token_str = generate_refresh_token();

    let token = RefreshTokenStorage::create(user.id, &token_str, None, None, state.pool()).await?;

    RefreshTokenStorage::revoke(token.id, state.pool()).await?;

    let found_token = RefreshTokenStorage::find_by_token(&token_str, state.pool()).await?;

    assert!(found_token.is_none());

    Ok(())
  }

  #[tokio::test]
  async fn refresh_token_replace_works() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    let token_str = generate_refresh_token();

    let token = RefreshTokenStorage::create(user.id, &token_str, None, None, state.pool()).await?;

    let new_token_str = generate_refresh_token();
    let _new_token = RefreshTokenStorage::replace(
      token.id,
      token.user_id,
      &new_token_str,
      None,
      None,
      token.absolute_expires_at,
      state.pool(),
    )
    .await?;

    let old_token = RefreshTokenStorage::find_by_token(&token_str, state.pool()).await?;
    assert!(old_token.is_none());

    let found_new_token = RefreshTokenStorage::find_by_token(&new_token_str, state.pool()).await?;
    assert!(found_new_token.is_some());

    Ok(())
  }

  #[tokio::test]
  async fn refresh_token_revoke_all_for_user_works() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    let token_str1 = generate_refresh_token();
    let token_str2 = generate_refresh_token();

    RefreshTokenStorage::create(user.id, &token_str1, None, None, state.pool()).await?;

    RefreshTokenStorage::create(user.id, &token_str2, None, None, state.pool()).await?;

    RefreshTokenStorage::revoke_all_for_user(user.id, state.pool()).await?;

    let found_token1 = RefreshTokenStorage::find_by_token(&token_str1, state.pool()).await?;
    let found_token2 = RefreshTokenStorage::find_by_token(&token_str2, state.pool()).await?;

    assert!(found_token1.is_none());
    assert!(found_token2.is_none());

    Ok(())
  }

  // 添加一个适配器Repository的测试
  #[tokio::test]
  async fn refresh_token_repository_adapter_works() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];
    let pool_arc = Arc::new(state.pool().clone());
    let repo = RefreshTokenAdaptor::new(pool_arc);

    let token_str = generate_refresh_token();
    let now = Utc::now();
    let expires_at = now + Duration::seconds(REFRESH_TOKEN_EXPIRATION as i64);
    let absolute_expires_at = now + Duration::seconds(REFRESH_TOKEN_MAX_LIFETIME as i64);

    // 使用适配器创建token
    let payload = StoreTokenPayload {
      user_id: user.id,
      raw_token: token_str.clone(),
      expires_at,
      absolute_expires_at,
      user_agent: Some("test-agent".to_string()),
      ip_address: Some("127.0.0.1".to_string()),
    };

    let token = repo.create(payload).await?;
    assert_eq!(token.user_id, user.id);

    // 使用适配器查找token
    let found = repo.find_by_token(&token_str).await?;
    assert!(found.is_some());

    // 使用适配器撤销token
    repo.revoke(token.id).await?;

    // 确认token已撤销
    let found_after_revoke = repo.find_by_token(&token_str).await?;
    assert!(found_after_revoke.is_none());

    Ok(())
  }
}
