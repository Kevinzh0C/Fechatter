use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid;

use super::{CreateUser, SigninUser, User, UserRepository, UserStatus};
use crate::error::CoreError;
use crate::middlewares::TokenVerifier as MwTokenVerifier;
use crate::services::AuthContext;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
  pub id: i64,
  pub workspace_id: i64,
  pub fullname: String,
  pub email: String,
  pub status: UserStatus,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RefreshToken {
  pub id: i64,
  pub user_id: i64,
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
  refresh_token_repo: std::sync::Arc<dyn RefreshTokenRepository + Send + Sync>,
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

pub trait TokenService: Send + Sync {
  fn generate_token(&self, user_claims: &UserClaims) -> Result<String, CoreError>;
  fn verify_token(&self, token: &str) -> Result<UserClaims, CoreError>;
  fn generate_auth_tokens(
    &self,
    user_claims: &UserClaims,
    auth_context: Option<AuthContext>,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<AuthTokens, CoreError>> + Send>>;
}

// Renamed from ReplaceRefreshTokenArgs
#[derive(Debug)]
pub struct ReplaceTokenPayload<'a> {
  pub old_token_id: i64,
  pub new_raw_token: &'a str,
  pub new_expires_at: DateTime<Utc>,
  pub new_absolute_expires_at: DateTime<Utc>,
  pub user_agent: Option<String>,
  pub ip_address: Option<String>,
}

// Renamed from StoreNewTokenArgs
#[derive(Debug)]
pub struct StoreTokenPayload {
  pub user_id: i64,
  pub raw_token: String,
  pub expires_at: DateTime<Utc>,
  pub absolute_expires_at: DateTime<Utc>,
  pub user_agent: Option<String>,
  pub ip_address: Option<String>,
}

pub trait RefreshTokenRepository: Send + Sync {
  fn find_by_token(
    &self,
    raw_token: &str,
  ) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<Option<RefreshToken>, CoreError>> + Send>,
  >;
  fn replace(
    &self,
    payload: ReplaceTokenPayload,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<RefreshToken, CoreError>> + Send>>;
  fn revoke(
    &self,
    token_id: i64,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), CoreError>> + Send>>;
  fn revoke_all_for_user(
    &self,
    user_id: i64,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), CoreError>> + Send>>;

  fn store_new_token(
    &self,
    payload: StoreTokenPayload,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<RefreshToken, CoreError>> + Send>>;
}

pub trait RefreshTokenService: Send + Sync {
  fn refresh_token(
    &self,
    refresh_token: &str,
    auth_context: Option<AuthContext>,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<AuthTokens, CoreError>> + Send>>;
}

pub trait SignupService: Send + Sync {
  fn signup(
    &self,
    payload: &CreateUser,
    auth_context: Option<AuthContext>,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<AuthTokens, CoreError>> + Send>>;
}

pub trait SigninService: Send + Sync {
  fn signin(
    &self,
    payload: &SigninUser,
    auth_context: Option<AuthContext>,
  ) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<Option<AuthTokens>, CoreError>> + Send>,
  >;
}

pub trait LogoutService: Send + Sync {
  fn logout(
    &self,
    refresh_token: &str,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), CoreError>> + Send>>;

  fn logout_all(
    &self,
    user_id: i64,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), CoreError>> + Send>>;
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
  pub fn from_config<C: TokenConfigProvider>(
    config: &C,
    refresh_token_repo: std::sync::Arc<dyn RefreshTokenRepository + Send + Sync>,
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
        .map_err(|e| CoreError::Internal(e.into()))?,
      decoding_key: DecodingKey::from_ed_pem(pk_pem.as_bytes())
        .map_err(|e| CoreError::Internal(e.into()))?,
      validation,
      refresh_token_repo,
    })
  }

  pub fn generate_token(&self, user: &User) -> Result<String, CoreError> {
    let claims = Claims::new(user);
    let header = Header::new(Algorithm::EdDSA);
    encode(&header, &claims, &self.encoding_key).map_err(|e| CoreError::Validation(e.to_string()))
  }

  pub async fn generate_auth_tokens(
    &self,
    user: &User,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> Result<AuthTokens, CoreError> {
    let access_token = self.generate_token(user)?;
    let raw_refresh_token = uuid::Uuid::new_v4().to_string();

    let now = Utc::now();
    let refresh_expires_at = now + chrono::Duration::seconds(REFRESH_TOKEN_EXPIRATION as i64);
    let refresh_absolute_expires_at =
      now + chrono::Duration::seconds(REFRESH_TOKEN_MAX_LIFETIME as i64);

    let store_payload = StoreTokenPayload {
      user_id: user.id,
      raw_token: raw_refresh_token.clone(),
      expires_at: refresh_expires_at,
      absolute_expires_at: refresh_absolute_expires_at,
      user_agent,
      ip_address,
    };
    let token_record: RefreshToken = self
      .refresh_token_repo
      .store_new_token(store_payload)
      .await?;

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

  pub fn verify_token(&self, token: &str) -> Result<UserClaims, CoreError> {
    let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
      .map_err(|e| CoreError::Validation(e.to_string()))?;
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

impl MwTokenVerifier for TokenManager {
  type Error = CoreError;
  type Claims = UserClaims;

  fn verify_token(&self, token: &str) -> Result<Self::Claims, Self::Error> {
    crate::models::jwt::TokenManager::verify_token(self, token)
  }
}
