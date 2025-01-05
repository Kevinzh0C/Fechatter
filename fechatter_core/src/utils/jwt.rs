use crate::{
  UserStatus,
  error::CoreError,
  models::{AuthUser, User},
};
use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::{Rng, rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::future::Future;

#[cfg(test)]
use mockall::automock;

#[cfg(test)]
pub use mockall::{mock, predicate};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthConfig {
  pub sk: String,
  pub pk: String,
}

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

#[derive(Clone)]
pub struct TokenManager {
  encoding_key: EncodingKey,
  decoding_key: DecodingKey,
  validation: Validation,
}

impl std::fmt::Debug for TokenManager {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("TokenManager")
      .field("validation", &self.validation)
      .finish_non_exhaustive()
  }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenInfo {
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
  let mut rng = rng();
  let mut random_bytes = [0u8; 32];
  rng.fill(&mut random_bytes);
  hex::encode(random_bytes)
}

pub fn sha256_hash(token: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(token.as_bytes());
  let result = hasher.finalize();
  hex::encode(result)
}

pub trait RefreshTokenRepository: Send + Sync {
  fn create(
    &self,
    user_id: i64,
    token: &str,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> impl Future<Output = Result<RefreshTokenInfo, CoreError>> + Send;

  fn find_by_token(
    &self,
    token: &str,
  ) -> impl Future<Output = Result<Option<RefreshTokenInfo>, CoreError>> + Send;

  fn revoke(&self, token_id: i64) -> impl Future<Output = Result<(), CoreError>> + Send;

  fn revoke_all_for_user(&self, user_id: i64)
  -> impl Future<Output = Result<(), CoreError>> + Send;

  fn replace(
    &self,
    token_id: i64,
    new_token: &str,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> impl Future<Output = Result<RefreshTokenInfo, CoreError>> + Send;
}

impl Claims {
  pub fn new(user: &User) -> Self {
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

  pub fn from_auth_user(user: &AuthUser) -> Self {
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
  pub fn from_config(auth: &AuthConfig) -> Result<Self, CoreError> {
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

  pub fn generate_token_from_user(&self, user: &User) -> Result<String, CoreError> {
    let claims = Claims::new(user);
    self.generate_token_from_claims(&claims)
  }

  pub fn generate_token_from_auth_user(&self, user: &AuthUser) -> Result<String, CoreError> {
    let claims = Claims::from_auth_user(user);
    self.generate_token_from_claims(&claims)
  }

  fn generate_token_from_claims(&self, claims: &Claims) -> Result<String, CoreError> {
    let header = Header::new(Algorithm::EdDSA);
    let token = encode(&header, claims, &self.encoding_key)?;
    Ok(token)
  }

  pub fn verify_token(&self, token: &str) -> Result<UserClaims, CoreError> {
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

/// TokenManager trait that can be mocked for testing
#[cfg_attr(test, automock)]
pub trait TokenManagerTrait {
  fn generate_token_from_user(&self, user: &User) -> Result<String, CoreError>;
  fn generate_token_from_auth_user(&self, user: &AuthUser) -> Result<String, CoreError>;
  fn verify_token(&self, token: &str) -> Result<UserClaims, CoreError>;
}

impl TokenManagerTrait for TokenManager {
  fn generate_token_from_user(&self, user: &User) -> Result<String, CoreError> {
    self.generate_token_from_user(user)
  }

  fn generate_token_from_auth_user(&self, user: &AuthUser) -> Result<String, CoreError> {
    self.generate_token_from_auth_user(user)
  }

  fn verify_token(&self, token: &str) -> Result<UserClaims, CoreError> {
    self.verify_token(token)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::utils::test_helpers::create_test_user;
  use anyhow::Result;

  #[test]
  fn jwt_token_authentication_should_work() -> Result<()> {
    let mut token_manager = MockTokenManagerTrait::new();
    token_manager
      .expect_generate_token_from_user()
      .returning(|user| Ok(format!("test_token_for_user_{}", user.id)));

    token_manager.expect_verify_token().returning(|token| {
      let parts: Vec<&str> = token.rsplitn(2, '_').collect();
      let id = parts[0].parse::<i64>().unwrap();
      Ok(UserClaims {
        id,
        workspace_id: 1,
        fullname: format!("Test User {}", id),
        email: format!("user{}@test.com", id),
        status: UserStatus::Active,
        created_at: Utc::now(),
      })
    });

    let user = create_test_user(1, 1);

    let token = token_manager.generate_token_from_user(&user)?;
    assert_eq!(token, "test_token_for_user_1");

    let user_claims = token_manager.verify_token(&token)?;
    assert_eq!(user_claims.id, 1);
    assert_eq!(user_claims.email, "user1@test.com");

    Ok(())
  }

  #[test]
  fn refresh_token_generation_should_work() -> Result<()> {
    let token = generate_refresh_token();
    assert_eq!(token.len(), 64); // 32 bytes => 64 hex chars

    // Generate another to make sure they're different
    let token2 = generate_refresh_token();
    assert_ne!(token, token2);

    Ok(())
  }

  #[test]
  fn sha256_hash_should_work() -> Result<()> {
    let hash = sha256_hash("test-token");

    // SHA-256 produces a 32-byte hash, which is 64 hex characters
    assert_eq!(hash.len(), 64);

    // Same input should produce same hash
    let hash2 = sha256_hash("test-token");
    assert_eq!(hash, hash2);

    // Different input should produce different hash
    let hash3 = sha256_hash("different-token");
    assert_ne!(hash, hash3);

    Ok(())
  }
}
