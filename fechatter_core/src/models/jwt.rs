use chrono::{DateTime, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{CreateUser, SigninUser, UserRepository, UserStatus};
use crate::error::CoreError;
use crate::services::AuthContext;

pub const ACCESS_TOKEN_EXPIRATION: usize = 900; // 15 minutes in seconds
pub const JWT_ISSUER: &str = "fechatter-server";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub secret: String,
    pub audience: String,
    pub issuer: String,
    pub expiration: usize,
}

pub fn generate_refresh_token() -> String {
    Uuid::new_v4().to_string()
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

pub trait TokenVerifier {
  type Error;
  type Claims;

  fn verify_token(&self, token: &str) -> Result<Self::Claims, Self::Error>;
}

// Define interfaces for the dependencies
pub trait TokenService: Send + Sync {
  fn generate_token(&self, user_claims: &UserClaims) -> Result<String, CoreError>;
  fn verify_token(&self, token: &str) -> Result<UserClaims, CoreError>;
  fn generate_auth_tokens(
    &self,
    user_claims: &UserClaims,
    auth_context: Option<AuthContext>,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<AuthTokens, CoreError>> + Send>>;
}

pub trait RefreshTokenRepository: Send + Sync {
  fn find_by_token(&self, token: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<RefreshToken>, CoreError>> + Send>>;
  fn replace(
    &self,
    id: i64,
    new_token: &str,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<RefreshToken, CoreError>> + Send>>;
  fn revoke(&self, id: i64) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), CoreError>> + Send>>;
  fn revoke_all_for_user(&self, user_id: i64) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), CoreError>> + Send>>;
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
