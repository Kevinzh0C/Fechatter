use crate::{AppError, User, config::AuthConfig, models::UserStatus};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

const JWT_ISSUER: &str = "fechatter-server";
const JWT_AUDIENCE: &str = "fechatter-web";
const JWT_LEEWAY: u64 = 60;
pub const ACCESS_TOKEN_EXPIRATION: usize = 30 * 60; // 30 minutes
pub const REFRESH_TOKEN_EXPIRATION: usize = 14 * 24 * 60 * 60; // 14 days
pub const REFRESH_TOKEN_MAX_LIFETIME: usize = 30 * 24 * 60 * 60; // 30 days

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  sub: String, // User ID
  exp: usize,  // Expiration time (as UTC timestamp)
  iat: usize,  // Issued at (as UTC timestamp)
  aud: String, // Audience
  iss: String, // Issuer
  user: UserClaims,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserClaims {
  pub id: i64,
  pub workspace_id: i64,
  pub fullname: String,
  pub email: String,
  pub status: UserStatus,
  pub created_at: DateTime<Utc>,
}

pub struct TokenManager {
  encoding_key: EncodingKey,
  decoding_key: DecodingKey,
  validation: Validation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenData {
  pub token: String,
  pub expires_at: DateTime<Utc>,
  pub absolute_expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthTokens {
  pub access_token: String,
  pub refresh_token: RefreshTokenData,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RefreshToken {
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

pub fn generate_refresh_token() -> String {
  use rand::{Rng, thread_rng};

  let mut rng = thread_rng();
  let random_bytes: [u8; 32] = rng.r#gen::<[u8; 32]>();
  hex::encode(random_bytes)
}

pub fn sha256_hash(token: &str) -> String {
  use sha2::{Digest, Sha256};

  let mut hasher = Sha256::new();
  hasher.update(token.as_bytes());
  let result = hasher.finalize();
  hex::encode(result)
}

impl RefreshToken {
  pub async fn create(
    user_id: i64,
    token: &str,
    user_agent: Option<String>,
    ip_address: Option<String>,
    pool: &PgPool,
  ) -> Result<Self, AppError> {
    let now = Utc::now();
    let expires_at = now + Duration::seconds(REFRESH_TOKEN_EXPIRATION as i64);
    let absolute_expires_at = now + Duration::seconds(REFRESH_TOKEN_MAX_LIFETIME as i64);
    let token_hash = sha256_hash(token);

    let refresh_token = sqlx::query_as::<_, RefreshToken>(
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

  pub async fn find_by_token(token: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
    let token_hash = sha256_hash(token);

    let refresh_token = sqlx::query_as::<_, RefreshToken>(
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

  pub async fn revoke(&self, pool: &PgPool) -> Result<(), AppError> {
    sqlx::query(
      r#"
      UPDATE refresh_tokens
      SET revoked = TRUE
      WHERE id = $1
      "#,
    )
    .bind(self.id)
    .execute(pool)
    .await?;

    Ok(())
  }

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
    &self,
    new_token: &str,
    user_agent: Option<String>,
    ip_address: Option<String>,
    pool: &PgPool,
  ) -> Result<Self, AppError> {
    let now = Utc::now();
    let new_expires_at = now + Duration::seconds(REFRESH_TOKEN_EXPIRATION as i64);
    let absolute_expires_at = self.absolute_expires_at; // Keep the same absolute expiry
    let new_token_hash = sha256_hash(new_token);

    let mut tx = pool.begin().await?;

    sqlx::query(
      r#"
      UPDATE refresh_tokens
      SET revoked = TRUE, replaced_by = $1
      WHERE id = $2
      "#,
    )
    .bind(&new_token_hash)
    .bind(self.id)
    .execute(&mut *tx)
    .await?;

    let refresh_token = sqlx::query_as::<_, RefreshToken>(
      r#"
      INSERT INTO refresh_tokens (user_id, token_hash, expires_at, user_agent, ip_address, absolute_expires_at)
      VALUES ($1, $2, $3, $4, $5, $6)
      RETURNING id, user_id, token_hash, expires_at, issued_at, revoked, replaced_by, user_agent, ip_address, absolute_expires_at
      "#,
    )
    .bind(self.user_id)
    .bind(&new_token_hash)
    .bind(if new_expires_at < absolute_expires_at { new_expires_at } else { absolute_expires_at })
    .bind(user_agent)
    .bind(ip_address)
    .bind(absolute_expires_at)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(refresh_token)
  }
}
impl Claims {
  fn new(user: &User) -> Self {
    let now = chrono::Utc::now().timestamp() as usize;
    Self {
      sub: user.id.to_string(),
      exp: now + ACCESS_TOKEN_EXPIRATION,
      iat: now,
      aud: JWT_AUDIENCE.to_string(),
      iss: JWT_ISSUER.to_string(),
      user: UserClaims {
        id: user.id,
        workspace_id: user.workspace_id,
        email: user.email.clone(),
        status: user.status,
        fullname: user.fullname.clone(),
        created_at: user.created_at,
      },
    }
  }
}

impl TokenManager {
  pub fn from_config(auth: &AuthConfig) -> Result<Self, AppError> {
    let mut validation = Validation::new(Algorithm::EdDSA);
    validation.leeway = JWT_LEEWAY;
    validation.reject_tokens_expiring_in_less_than = 300;

    validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);

    validation.validate_aud = true;
    validation.set_audience(&[JWT_AUDIENCE]);
    validation.set_issuer(&[JWT_ISSUER]);

    let sk = auth.sk.replace("\\n", "\n");
    let pk = auth.pk.replace("\\n", "\n");

    Ok(Self {
      encoding_key: EncodingKey::from_ed_pem(sk.as_bytes())?,
      decoding_key: DecodingKey::from_ed_pem(pk.as_bytes())?,
      validation,
    })
  }

  pub fn generate_token(&self, user: &User) -> Result<String, AppError> {
    let claims = Claims::new(user);
    let header = Header::new(Algorithm::EdDSA);
    let token = encode(&header, &claims, &self.encoding_key)?;

    Ok(token)
  }

  pub async fn generate_auth_tokens(
    &self,
    user: &User,
    user_agent: Option<String>,
    ip_address: Option<String>,
    pool: &PgPool,
  ) -> Result<AuthTokens, AppError> {
    let access_token = self.generate_token(user)?;
    let refresh_token = generate_refresh_token();

    let token_record =
      RefreshToken::create(user.id, &refresh_token, user_agent, ip_address, pool).await?;

    let refresh_token_data = RefreshTokenData {
      token: refresh_token,
      expires_at: token_record.expires_at,
      absolute_expires_at: token_record.absolute_expires_at,
    };

    Ok(AuthTokens {
      access_token,
      refresh_token: refresh_token_data,
    })
  }

  pub fn verify_token(&self, token: &str) -> Result<UserClaims, AppError> {
    let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)?;
    let user_claims = UserClaims {
      id: token_data.claims.user.id,
      workspace_id: token_data.claims.user.workspace_id,
      fullname: token_data.claims.user.fullname,
      email: token_data.claims.user.email,
      status: token_data.claims.user.status,
      created_at: token_data.claims.user.created_at,
    };

    Ok(user_claims)
  }
}

#[cfg(test)]
mod tests {

  use crate::{AppConfig, models::UserStatus, setup_test_users};

  use super::*;
  use anyhow::Result;

  #[test]
  fn jwt_token_authentication_should_work() -> Result<()> {
    let config = AppConfig::load()?;
    let token_manager = TokenManager::from_config(&config.auth)?;

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

    let token = token_manager.generate_token(&user)?;

    let user_claims2 = token_manager.verify_token(&token)?;
    assert_eq!(user_claims, user_claims2);

    Ok(())
  }
  
  #[tokio::test]
  async fn refresh_token_create_and_find_works() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];
    
    let token_str = generate_refresh_token();
    
    let _token = RefreshToken::create(
      user.id,
      &token_str,
      Some("test-agent".to_string()),
      Some("127.0.0.1".to_string()),
      &state.pool
    ).await?;
    
    let found_token = RefreshToken::find_by_token(&token_str, &state.pool).await?;
    
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
    
    let token = RefreshToken::create(
      user.id,
      &token_str,
      None,
      None,
      &state.pool
    ).await?;
    
    token.revoke(&state.pool).await?;
    
    let found_token = RefreshToken::find_by_token(&token_str, &state.pool).await?;
    
    assert!(found_token.is_none());
    
    Ok(())
  }
  
  #[tokio::test]
  async fn refresh_token_replace_works() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];
    
    let token_str = generate_refresh_token();
    
    let token = RefreshToken::create(
      user.id,
      &token_str,
      None,
      None,
      &state.pool
    ).await?;
    
    let new_token_str = generate_refresh_token();
    let _new_token = token.replace(
      &new_token_str,
      None,
      None,
      &state.pool
    ).await?;
    
    let old_token = RefreshToken::find_by_token(&token_str, &state.pool).await?;
    assert!(old_token.is_none());
    
    let found_new_token = RefreshToken::find_by_token(&new_token_str, &state.pool).await?;
    assert!(found_new_token.is_some());
    
    Ok(())
  }
  
  #[tokio::test]
  async fn refresh_token_revoke_all_for_user_works() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];
    
    let token_str1 = generate_refresh_token();
    let token_str2 = generate_refresh_token();
    
    RefreshToken::create(
      user.id,
      &token_str1,
      None,
      None,
      &state.pool
    ).await?;
    
    RefreshToken::create(
      user.id,
      &token_str2,
      None,
      None,
      &state.pool
    ).await?;
    
    RefreshToken::revoke_all_for_user(user.id, &state.pool).await?;
    
    let found_token1 = RefreshToken::find_by_token(&token_str1, &state.pool).await?;
    let found_token2 = RefreshToken::find_by_token(&token_str2, &state.pool).await?;
    
    assert!(found_token1.is_none());
    assert!(found_token2.is_none());
    
    Ok(())
  }
}
