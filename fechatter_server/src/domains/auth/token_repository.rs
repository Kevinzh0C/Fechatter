use crate::AppError;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;

use fechatter_core::{error::CoreError, UserId};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

pub const REFRESH_TOKEN_EXPIRATION: usize = 14 * 24 * 60 * 60; // 14 days
pub const REFRESH_TOKEN_MAX_LIFETIME: usize = 30 * 24 * 60 * 60; // 30 days

// Domain-specific refresh token entity (not dependent on fechatter_core)
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

// Domain-specific payloads
#[derive(Debug)]
pub struct ReplaceTokenPayload {
    pub old_token_id: i64,
    pub new_raw_token: String,
    pub new_expires_at: DateTime<Utc>,
    pub new_absolute_expires_at: DateTime<Utc>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Debug)]
pub struct StoreTokenPayload {
    pub user_id: UserId,
    pub raw_token: String,
    pub expires_at: DateTime<Utc>,
    pub absolute_expires_at: DateTime<Utc>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

// Domain-specific repository trait
#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
    async fn find_by_token(&self, raw_token: &str)
        -> Result<Option<RefreshTokenEntity>, CoreError>;
    async fn replace(&self, payload: ReplaceTokenPayload) -> Result<RefreshTokenEntity, CoreError>;
    async fn revoke(&self, token_id: i64) -> Result<(), CoreError>;
    async fn revoke_all_for_user(&self, user_id: UserId) -> Result<(), CoreError>;
    async fn create(&self, payload: StoreTokenPayload) -> Result<RefreshTokenEntity, CoreError>;
}

#[allow(unused)]
pub fn generate_refresh_token() -> String {
    use rand::{thread_rng, Rng};

    let mut rng_instance = thread_rng();
    let random_bytes: [u8; 32] = rng_instance.r#gen::<[u8; 32]>();
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

// Database entity, specifically for database interactions
// This was already defined above, so removing the duplicate

// Database operations, separated from the entity
pub struct RefreshTokenStorage;

impl RefreshTokenStorage {
    // Database operations
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

        // Use transaction and row-level locking to ensure concurrency safety
        let mut tx = pool.begin().await?;

        // First check if the token has already been revoked or replaced
        // Use FOR UPDATE to acquire a row lock, ensuring no other session can modify this row during the transaction
        let query = r#"
      SELECT revoked, replaced_by 
      FROM refresh_tokens 
      WHERE id = $1
      FOR UPDATE
    "#;

        let token_status = sqlx::query(query)
            .bind(old_token_id)
            .fetch_optional(&mut *tx)
            .await?;

        // If the token doesn't exist or has already been revoked/replaced, return an error
        match token_status {
            None => {
                tx.rollback().await?;
                return Err(AppError::NotFound(vec![format!(
                    "Refresh token with id {} not found",
                    old_token_id
                )]));
            }
            Some(status) => {
                let revoked: bool = status
                    .try_get("revoked")
                    .map_err(|_| AppError::SqlxError(sqlx::Error::RowNotFound))?;
                let replaced_by: Option<String> = status
                    .try_get("replaced_by")
                    .map_err(|_| AppError::SqlxError(sqlx::Error::RowNotFound))?;

                if revoked || replaced_by.is_some() {
                    tx.rollback().await?;
                    return Err(AppError::JwtError(jsonwebtoken::errors::Error::from(
                        jsonwebtoken::errors::ErrorKind::InvalidToken,
                    )));
                }
            }
        }

        // Replace the old token with the new one
        let query = r#"
      UPDATE refresh_tokens
      SET revoked = TRUE, replaced_by = $1
      WHERE id = $2
    "#;

        sqlx::query(query)
            .bind(&new_token_hash)
            .bind(old_token_id)
            .execute(&mut *tx)
            .await?;

        // Create new refresh token
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

        // Commit the transaction
        tx.commit().await?;

        Ok(refresh_token)
    }

    /// Update user password hash - Domain layer responsibility
    pub async fn update_user_password(
        user_id: i64,
        new_password_hash: &str,
        pool: &PgPool,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
            .bind(new_password_hash)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

// Implementation of the domain repository using database storage
pub struct RefreshTokenRepositoryImpl {
    pool: Arc<PgPool>,
}

impl RefreshTokenRepositoryImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

// Compatibility adapter for fechatter_core traits
pub struct CoreRefreshTokenRepositoryAdapter {
    inner: RefreshTokenRepositoryImpl,
}

impl CoreRefreshTokenRepositoryAdapter {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            inner: RefreshTokenRepositoryImpl::new(pool),
        }
    }
}

// Implement fechatter_core's RefreshTokenRepository trait
#[async_trait]
impl fechatter_core::models::jwt::RefreshTokenRepository for CoreRefreshTokenRepositoryAdapter {
    async fn find_by_token(
        &self,
        raw_token: &str,
    ) -> Result<Option<fechatter_core::models::jwt::RefreshToken>, CoreError> {
        let domain_token = self.inner.find_by_token(raw_token).await?;

        match domain_token {
            Some(token) => {
                let core_token = fechatter_core::models::jwt::RefreshToken {
                    id: token.id,
                    user_id: fechatter_core::UserId(token.user_id),
                    token_hash: token.token_hash,
                    expires_at: token.expires_at,
                    issued_at: token.issued_at,
                    revoked: token.revoked,
                    replaced_by: token.replaced_by,
                    user_agent: token.user_agent,
                    ip_address: token.ip_address,
                    absolute_expires_at: token.absolute_expires_at,
                };
                Ok(Some(core_token))
            }
            None => Ok(None),
        }
    }

    async fn replace(
        &self,
        payload: fechatter_core::models::jwt::ReplaceTokenPayload,
    ) -> Result<fechatter_core::models::jwt::RefreshToken, CoreError> {
        let domain_payload = ReplaceTokenPayload {
            old_token_id: payload.old_token_id,
            new_raw_token: payload.new_raw_token,
            new_expires_at: payload.new_expires_at,
            new_absolute_expires_at: payload.new_absolute_expires_at,
            user_agent: payload.user_agent,
            ip_address: payload.ip_address,
        };

        let domain_token = self.inner.replace(domain_payload).await?;

        let core_token = fechatter_core::models::jwt::RefreshToken {
            id: domain_token.id,
            user_id: fechatter_core::UserId(domain_token.user_id),
            token_hash: domain_token.token_hash,
            expires_at: domain_token.expires_at,
            issued_at: domain_token.issued_at,
            revoked: domain_token.revoked,
            replaced_by: domain_token.replaced_by,
            user_agent: domain_token.user_agent,
            ip_address: domain_token.ip_address,
            absolute_expires_at: domain_token.absolute_expires_at,
        };
        Ok(core_token)
    }

    async fn revoke(&self, token_id: i64) -> Result<(), CoreError> {
        self.inner.revoke(token_id).await
    }

    async fn revoke_all_for_user(&self, user_id: fechatter_core::UserId) -> Result<(), CoreError> {
        self.inner.revoke_all_for_user(user_id).await
    }

    async fn create(
        &self,
        payload: fechatter_core::models::jwt::StoreTokenPayload,
    ) -> Result<fechatter_core::models::jwt::RefreshToken, CoreError> {
        let domain_payload = StoreTokenPayload {
            user_id: payload.user_id,
            raw_token: payload.raw_token,
            expires_at: payload.expires_at,
            absolute_expires_at: payload.absolute_expires_at,
            user_agent: payload.user_agent,
            ip_address: payload.ip_address,
        };

        let domain_token = self.inner.create(domain_payload).await?;

        let core_token = fechatter_core::models::jwt::RefreshToken {
            id: domain_token.id,
            user_id: fechatter_core::UserId(domain_token.user_id),
            token_hash: domain_token.token_hash,
            expires_at: domain_token.expires_at,
            issued_at: domain_token.issued_at,
            revoked: domain_token.revoked,
            replaced_by: domain_token.replaced_by,
            user_agent: domain_token.user_agent,
            ip_address: domain_token.ip_address,
            absolute_expires_at: domain_token.absolute_expires_at,
        };
        Ok(core_token)
    }
}

#[async_trait]
impl RefreshTokenRepository for RefreshTokenRepositoryImpl {
    async fn find_by_token(
        &self,
        raw_token: &str,
    ) -> Result<Option<RefreshTokenEntity>, CoreError> {
        let pool = self.pool.clone();
        let token = raw_token.to_string();
        let result = RefreshTokenStorage::find_by_token(&token, &pool)
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        Ok(result)
    }

    async fn replace(&self, payload: ReplaceTokenPayload) -> Result<RefreshTokenEntity, CoreError> {
        let pool = self.pool.clone();

        // Get user_id from old_token record in payload
        let old_token_id = payload.old_token_id;

        // First query the user ID from the old token ID
        let user_id =
            sqlx::query_scalar::<_, i64>("SELECT user_id FROM refresh_tokens WHERE id = $1")
                .bind(old_token_id)
                .fetch_one(&*pool)
                .await
                .map_err(|e| CoreError::Internal(e.to_string()))?;

        // Call RefreshTokenStorage::replace to perform token replacement
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

        Ok(result)
    }

    async fn revoke(&self, token_id: i64) -> Result<(), CoreError> {
        let pool = self.pool.clone();
        RefreshTokenStorage::revoke(token_id, &pool)
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))
    }

    async fn revoke_all_for_user(&self, user_id: fechatter_core::UserId) -> Result<(), CoreError> {
        let pool = self.pool.clone();
        RefreshTokenStorage::revoke_all_for_user(user_id.into(), &pool)
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))
    }

    async fn create(&self, payload: StoreTokenPayload) -> Result<RefreshTokenEntity, CoreError> {
        let pool = self.pool.clone();
        let result = RefreshTokenStorage::create(
            i64::from(payload.user_id),
            &payload.raw_token,
            payload.user_agent.clone(),
            payload.ip_address.clone(),
            &pool,
        )
        .await
        .map_err(|e| CoreError::Internal(e.to_string()))?;

        Ok(result)
    }
}

pub fn auth_context_matches(
    token_user_agent: Option<&str>,
    token_ip_address: Option<&str>,
    request_user_agent: Option<&str>,
    request_ip_address: Option<&str>,
) -> bool {
    // Print direct println debug info along with tracing
    println!("!! DEBUG auth_context_matches:");
    println!("!! Token User-Agent: {:?}", token_user_agent);
    println!("!! Request User-Agent: {:?}", request_user_agent);
    println!("!! Token IP: {:?}", token_ip_address);
    println!("!! Request IP: {:?}", request_ip_address);

    tracing::debug!(
        target: "auth_context_match",
        stored_ua = ?token_user_agent,
        stored_ip = ?token_ip_address,
        request_ua = ?request_user_agent,
        request_ip = ?request_ip_address,
        "Performing auth context match"
    );

    // If token has no user agent, we don't care about the request's user agent
    let ua_match = token_user_agent.map_or(true, |t_ua| request_user_agent == Some(t_ua));
    // If token has no IP, we don't care about the request's IP
    let ip_match = token_ip_address.map_or(true, |t_ip| request_ip_address == Some(t_ip));

    println!(
        "!! ua_match: {}, ip_match: {}, final: {}",
        ua_match,
        ip_match,
        ua_match && ip_match
    );

    tracing::debug!(
        target: "auth_context_match",
        ua_match,
        ip_match,
        final_match = ua_match && ip_match,
        "Auth context match results"
    );

    ua_match && ip_match
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::UserStatus, setup_test_users, AppConfig};
    use anyhow::Result;
    use fechatter_core::TokenService;
    use fechatter_core::{jwt::TokenManager, User, UserClaims};

    // Mock repository implementation for test environment
    #[derive(Clone)]
    struct MockRefreshTokenRepo;

    #[async_trait::async_trait]
    impl RefreshTokenRepository for MockRefreshTokenRepo {
        async fn find_by_token(
            &self,
            _raw_token: &str,
        ) -> Result<Option<RefreshTokenEntity>, CoreError> {
            Ok(None)
        }

        async fn replace(
            &self,
            _payload: ReplaceTokenPayload,
        ) -> Result<RefreshTokenEntity, CoreError> {
            Err(CoreError::Internal("Not implemented".to_string()))
        }

        async fn revoke(&self, _token_id: i64) -> Result<(), CoreError> {
            Ok(())
        }

        async fn revoke_all_for_user(
            &self,
            _user_id: fechatter_core::UserId,
        ) -> Result<(), CoreError> {
            Ok(())
        }

        async fn create(
            &self,
            _payload: StoreTokenPayload,
        ) -> Result<RefreshTokenEntity, CoreError> {
            // Return a fake RefreshToken for testing
            let now = Utc::now();
            Ok(RefreshTokenEntity {
                id: 1,
                user_id: i64::from(_payload.user_id),
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
        let token_manager =
            TokenManager::from_config(&config.auth, Arc::new(MockRefreshTokenRepo))?;

        let user = User {
            id: fechatter_core::UserId(1).into(),
            fullname: "John Doe".to_string(),
            email: "john.doe@example.com".to_string(),
            password_hash: Default::default(),
            status: UserStatus::Active,
            created_at: chrono::Utc::now(),
            workspace_id: fechatter_core::WorkspaceId(1),
        };
        let user_claims = UserClaims {
            id: user.id.into(),
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

        // Create token using RefreshTokenStorage
        let _token = RefreshTokenStorage::create(
            user.id.into(),
            &token_str,
            Some("test-agent".to_string()),
            Some("127.0.0.1".to_string()),
            state.pool(),
        )
        .await?;

        let found_token = RefreshTokenStorage::find_by_token(&token_str, state.pool()).await?;

        assert!(found_token.is_some());
        let found_token = found_token.unwrap();
        assert_eq!(found_token.user_id, i64::from(user.id));
        assert_eq!(found_token.user_agent, Some("test-agent".to_string()));
        assert_eq!(found_token.ip_address, Some("127.0.0.1".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn refresh_token_revoke_works() -> Result<()> {
        let (_tdb, state, users) = setup_test_users!(1).await;
        let user = &users[0];

        let token_str = generate_refresh_token();

        let token =
            RefreshTokenStorage::create(user.id.into(), &token_str, None, None, state.pool())
                .await?;

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

        let token =
            RefreshTokenStorage::create(user.id.into(), &token_str, None, None, state.pool())
                .await?;

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

        let found_new_token =
            RefreshTokenStorage::find_by_token(&new_token_str, state.pool()).await?;
        assert!(found_new_token.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn refresh_token_revoke_all_for_user_works() -> Result<()> {
        let (_tdb, state, users) = setup_test_users!(1).await;
        let user = &users[0];

        let token_str1 = generate_refresh_token();
        let token_str2 = generate_refresh_token();

        RefreshTokenStorage::create(user.id.into(), &token_str1, None, None, state.pool()).await?;

        RefreshTokenStorage::create(user.id.into(), &token_str2, None, None, state.pool()).await?;

        RefreshTokenStorage::revoke_all_for_user(user.id.into(), state.pool()).await?;

        let found_token1 = RefreshTokenStorage::find_by_token(&token_str1, state.pool()).await?;
        let found_token2 = RefreshTokenStorage::find_by_token(&token_str2, state.pool()).await?;

        assert!(found_token1.is_none());
        assert!(found_token2.is_none());

        Ok(())
    }

    // Add a test for the Repository adapter
    #[tokio::test]
    async fn refresh_token_repository_adapter_works() -> Result<()> {
        let (_tdb, state, users) = setup_test_users!(1).await;
        let user = &users[0];
        let pool_arc = Arc::new(state.pool().clone());
        let repo = RefreshTokenRepositoryImpl::new(pool_arc);

        let token_str = generate_refresh_token();
        let now = Utc::now();
        let expires_at = now + Duration::seconds(REFRESH_TOKEN_EXPIRATION as i64);
        let absolute_expires_at = now + Duration::seconds(REFRESH_TOKEN_MAX_LIFETIME as i64);

        // Create token using the adapter
        let payload = StoreTokenPayload {
            user_id: user.id.into(),
            raw_token: token_str.clone(),
            expires_at,
            absolute_expires_at,
            user_agent: Some("test-agent".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
        };

        let token = repo.create(payload).await?;
        assert_eq!(token.user_id, i64::from(user.id));

        // Find token using the adapter
        let found = repo.find_by_token(&token_str).await?;
        assert!(found.is_some());

        // Revoke token using the adapter
        repo.revoke(token.id).await?;

        // Confirm token is revoked
        let found_after_revoke = repo.find_by_token(&token_str).await?;
        assert!(found_after_revoke.is_none());

        Ok(())
    }
}
