use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid;

use super::{CreateUser, SigninUser, User, UserId, UserRepository, UserStatus, WorkspaceId};
use crate::contracts::AuthContext;
use crate::error::CoreError;
use crate::middlewares::TokenVerifier as MwTokenVerifier;

const JWT_ISSUER: &str = "fechatter-server";
const JWT_AUDIENCE: &str = "fechatter-web";
const JWT_LEEWAY: u64 = 60;
pub const ACCESS_TOKEN_EXPIRATION: usize = 30 * 60; // 30 minutes
pub const REFRESH_TOKEN_EXPIRATION: usize = 14 * 24 * 60 * 60; // 14 days
pub const REFRESH_TOKEN_MAX_LIFETIME: usize = 30 * 24 * 60 * 60; // 30 days

pub trait TokenConfigProvider {
  fn get_encoding_key_pem(&self) -> &str;
  fn get_decoding_key_pem(&self) -> &str;
  fn get_jwt_leeway(&self) -> u64 {
    JWT_LEEWAY
  } // Default implementation
  fn get_jwt_audience(&self) -> Option<&str> {
    Some(JWT_AUDIENCE)
  } // Default
  fn get_jwt_issuer(&self) -> Option<&str> {
    Some(JWT_ISSUER)
  } // Default
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  sub: String, // User ID
  exp: usize,  // Expiration time (as UTC timestamp)
  iat: usize,  // Issued at (as UTC timestamp)
  aud: String, // Audience
  iss: String, // Issuer
  user: UserClaims,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserClaims {
  pub id: UserId,
  pub workspace_id: WorkspaceId,
  pub fullname: String,
  pub email: String,
  pub status: UserStatus,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct RefreshToken {
  pub id: i64,
  pub user_id: UserId,
  pub token_hash: String, // The repo impl will handle hashing.
  pub expires_at: DateTime<Utc>,
  pub issued_at: DateTime<Utc>,
  pub revoked: bool,
  pub replaced_by: Option<String>,
  pub user_agent: Option<String>,
  pub ip_address: Option<String>,
  pub absolute_expires_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct TokenManager {
  encoding_key: EncodingKey,
  decoding_key: DecodingKey,
  validation: Validation,
  refresh_token_repo: Arc<dyn RefreshTokenRepository + Send + Sync>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RefreshTokenData {
  pub token: String,
  pub expires_at: DateTime<Utc>,
  pub absolute_expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuthTokens {
  pub access_token: String,
  pub refresh_token: RefreshTokenData,
}

#[async_trait]
pub trait TokenService: Send + Sync {
  fn generate_token(&self, user_claims: &UserClaims) -> Result<String, CoreError>;
  fn verify_token(&self, token: &str) -> Result<UserClaims, CoreError>;
  async fn generate_auth_tokens(
    &self,
    user_claims: &UserClaims,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> Result<AuthTokens, CoreError>;
}

// Renamed from ReplaceRefreshTokenArgs
#[derive(Debug)]
pub struct ReplaceTokenPayload {
  pub old_token_id: i64,
  pub new_raw_token: String,
  pub new_expires_at: DateTime<Utc>,
  pub new_absolute_expires_at: DateTime<Utc>,
  pub user_agent: Option<String>,
  pub ip_address: Option<String>,
}

// Renamed from StoreNewTokenArgs
#[derive(Debug)]
pub struct StoreTokenPayload {
  pub user_id: UserId,
  pub raw_token: String,
  pub expires_at: DateTime<Utc>,
  pub absolute_expires_at: DateTime<Utc>,
  pub user_agent: Option<String>,
  pub ip_address: Option<String>,
}

#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
  async fn find_by_token(&self, raw_token: &str) -> Result<Option<RefreshToken>, CoreError>;
  async fn replace(&self, payload: ReplaceTokenPayload) -> Result<RefreshToken, CoreError>;
  async fn revoke(&self, token_id: i64) -> Result<(), CoreError>;
  async fn revoke_all_for_user(&self, user_id: UserId) -> Result<(), CoreError>;

  async fn create(&self, payload: StoreTokenPayload) -> Result<RefreshToken, CoreError>;
}

#[async_trait]
pub trait RefreshTokenService: Send + Sync {
  async fn refresh_token(
    &self,
    refresh_token: &str,
    auth_context: Option<AuthContext>,
  ) -> Result<AuthTokens, CoreError>;
}

#[async_trait]
pub trait SignupService: Send + Sync {
  async fn signup(
    &self,
    payload: &CreateUser,
    auth_context: Option<AuthContext>,
  ) -> Result<AuthTokens, CoreError>;
}

#[async_trait]
pub trait SigninService: Send + Sync {
  async fn signin(
    &self,
    payload: &SigninUser,
    auth_context: Option<AuthContext>,
  ) -> Result<Option<AuthTokens>, CoreError>;
}

#[async_trait]
pub trait LogoutService: Send + Sync {
  async fn logout(&self, refresh_token: &str) -> Result<(), CoreError>;

  async fn logout_all(&self, user_id: UserId) -> Result<(), CoreError>;
}

pub trait AuthServiceFactory {
  fn new<U: UserRepository + Sync, T: TokenService + Sync, R: RefreshTokenRepository + Sync>(
    user_repository: U,
    token_service: T,
    refresh_token_repository: R,
  ) -> Self
  where
    Self: Sized;
}

#[async_trait]
pub trait AuthServiceTrait:
  RefreshTokenService + SignupService + SigninService + LogoutService
{
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
  pub fn new<C: TokenConfigProvider>(config: &C) -> Result<Self, CoreError> {
    let mut validation = Validation::new(Algorithm::EdDSA);
    validation.leeway = config.get_jwt_leeway();
    validation.reject_tokens_expiring_in_less_than = 300;
    validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
    validation.validate_aud = true;
    if let Some(aud) = config.get_jwt_audience() {
      validation.set_audience(&[aud]);
    }
    if let Some(iss) = config.get_jwt_issuer() {
      validation.set_issuer(&[iss]);
    }

    let sk_pem = config.get_encoding_key_pem().replace("\\n", "\n");
    let pk_pem = config.get_decoding_key_pem().replace("\\n", "\n");

    let refresh_token_repo = Arc::new(DummyRefreshTokenRepository);

    Ok(Self {
      encoding_key: EncodingKey::from_ed_pem(sk_pem.as_bytes())
        .map_err(|e| CoreError::Internal(e.to_string()))?,
      decoding_key: DecodingKey::from_ed_pem(pk_pem.as_bytes())
        .map_err(|e| CoreError::Internal(e.to_string()))?,
      validation,
      refresh_token_repo,
    })
  }

  pub fn from_config<C: TokenConfigProvider>(
    config: &C,
    refresh_token_repo: Arc<dyn RefreshTokenRepository + Send + Sync>,
  ) -> Result<Self, CoreError> {
    let mut validation = Validation::new(Algorithm::EdDSA);
    validation.leeway = config.get_jwt_leeway();
    validation.reject_tokens_expiring_in_less_than = 300;
    validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
    validation.validate_aud = true;
    if let Some(aud) = config.get_jwt_audience() {
      validation.set_audience(&[aud]);
    }
    if let Some(iss) = config.get_jwt_issuer() {
      validation.set_issuer(&[iss]);
    }

    let sk_pem = config.get_encoding_key_pem().replace("\\n", "\n");
    let pk_pem = config.get_decoding_key_pem().replace("\\n", "\n");

    Ok(Self {
      encoding_key: EncodingKey::from_ed_pem(sk_pem.as_bytes())
        .map_err(|e| CoreError::Internal(e.to_string()))?,
      decoding_key: DecodingKey::from_ed_pem(pk_pem.as_bytes())
        .map_err(|e| CoreError::Internal(e.to_string()))?,
      validation,
      refresh_token_repo,
    })
  }

  fn create_claims_from_user(&self, user: &User) -> Claims {
    Claims::new(user)
  }

  fn create_claims_from_user_claims(&self, user_claims: &UserClaims) -> Claims {
    Claims {
      sub: user_claims.id.to_string(),
      exp: (Utc::now().timestamp() as usize) + ACCESS_TOKEN_EXPIRATION,
      iat: Utc::now().timestamp() as usize,
      aud: JWT_AUDIENCE.to_string(),
      iss: JWT_ISSUER.to_string(),
      user: user_claims.clone(),
    }
  }

  pub fn generate_token_for_user(&self, user: &User) -> Result<String, CoreError> {
    let claims = self.create_claims_from_user(user);
    let header = Header::new(Algorithm::EdDSA);
    encode(&header, &claims, &self.encoding_key).map_err(|e| CoreError::Validation(e.to_string()))
  }

  pub fn internal_generate_token(&self, user_claims: &UserClaims) -> Result<String, CoreError> {
    let claims = self.create_claims_from_user_claims(user_claims);
    let header = Header::new(Algorithm::EdDSA);
    encode(&header, &claims, &self.encoding_key).map_err(|e| CoreError::Validation(e.to_string()))
  }

  pub async fn internal_generate_auth_tokens(
    &self,
    user_claims: &UserClaims,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> Result<AuthTokens, CoreError> {
    let access_token = self.internal_generate_token(user_claims)?;
    let raw_refresh_token = uuid::Uuid::new_v4().to_string();

    let now = Utc::now();
    let refresh_expires_at = now + chrono::Duration::seconds(REFRESH_TOKEN_EXPIRATION as i64);
    let refresh_absolute_expires_at =
      now + chrono::Duration::seconds(REFRESH_TOKEN_MAX_LIFETIME as i64);

    let store_payload = StoreTokenPayload {
      user_id: user_claims.id,
      raw_token: raw_refresh_token.clone(),
      expires_at: refresh_expires_at,
      absolute_expires_at: refresh_absolute_expires_at,
      user_agent,
      ip_address,
    };
    let token_record: RefreshToken = self.refresh_token_repo.create(store_payload).await?;

    let refresh_token_data = RefreshTokenData {
      token: raw_refresh_token,
      expires_at: token_record.expires_at,
      absolute_expires_at: token_record.absolute_expires_at,
    };

    Ok(AuthTokens {
      access_token,
      refresh_token: refresh_token_data,
    })
  }

  pub fn internal_verify_token(&self, token: &str) -> Result<UserClaims, CoreError> {
    let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
      .map_err(|e| CoreError::Validation(e.to_string()))?;
    Ok(token_data.claims.user)
  }

  pub fn gen_jwt_token(&self, claims: &UserClaims) -> Result<String, CoreError> {
    // Create JWT with given claims
    // Uses RS256 algorithm with private key
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::EdDSA);
    jsonwebtoken::encode(&header, claims, &self.encoding_key)
      .map_err(|e| CoreError::Authentication(e.to_string()))
  }

  pub fn verify_jwt_token(&self, token: &str) -> Result<UserClaims, CoreError> {
    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::EdDSA);
    match jsonwebtoken::decode::<UserClaims>(token, &self.decoding_key, &validation) {
      Ok(token_data) => Ok(token_data.claims),
      Err(_) => Err(CoreError::Authentication(
        "Invalid or expired refresh token".to_string(),
      )),
    }
  }
}

#[async_trait]
impl TokenService for TokenManager {
  fn generate_token(&self, user_claims: &UserClaims) -> Result<String, CoreError> {
    self.internal_generate_token(user_claims)
  }

  fn verify_token(&self, token: &str) -> Result<UserClaims, CoreError> {
    self.internal_verify_token(token)
  }

  async fn generate_auth_tokens(
    &self,
    user_claims: &UserClaims,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> Result<AuthTokens, CoreError> {
    self
      .internal_generate_auth_tokens(user_claims, user_agent, ip_address)
      .await
  }
}

impl MwTokenVerifier for TokenManager {
  type Error = CoreError;
  type Claims = UserClaims;

  fn verify_token(&self, token: &str) -> Result<Self::Claims, Self::Error> {
    self.internal_verify_token(token)
  }
}

struct DummyRefreshTokenRepository;

#[async_trait]
impl RefreshTokenRepository for DummyRefreshTokenRepository {
  async fn find_by_token(&self, _raw_token: &str) -> Result<Option<RefreshToken>, CoreError> {
    Ok(Some(RefreshToken {
      id: 1,
      user_id: UserId::new(1),
      token_hash: "dummy_hash".to_string(),
      expires_at: Utc::now() + chrono::Duration::days(1),
      issued_at: Utc::now(),
      revoked: false,
      replaced_by: None,
      user_agent: None,
      ip_address: None,
      absolute_expires_at: Utc::now() + chrono::Duration::days(30),
    }))
  }

  async fn replace(&self, _payload: ReplaceTokenPayload) -> Result<RefreshToken, CoreError> {
    Ok(RefreshToken {
      id: 2,
      user_id: UserId::new(1),
      token_hash: "new_dummy_hash".to_string(),
      expires_at: Utc::now() + chrono::Duration::days(1),
      issued_at: Utc::now(),
      revoked: false,
      replaced_by: None,
      user_agent: None,
      ip_address: None,
      absolute_expires_at: Utc::now() + chrono::Duration::days(30),
    })
  }

  async fn revoke(&self, _token_id: i64) -> Result<(), CoreError> {
    Ok(())
  }

  async fn revoke_all_for_user(&self, _user_id: UserId) -> Result<(), CoreError> {
    Ok(())
  }

  async fn create(&self, payload: StoreTokenPayload) -> Result<RefreshToken, CoreError> {
    Ok(RefreshToken {
      id: 1,
      user_id: payload.user_id,
      token_hash: payload.raw_token,
      expires_at: payload.expires_at,
      issued_at: Utc::now(),
      revoked: false,
      replaced_by: None,
      user_agent: payload.user_agent,
      ip_address: payload.ip_address,
      absolute_expires_at: payload.absolute_expires_at,
    })
  }
}
