use serde::{Deserialize, Serialize};

pub mod auth_service;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
  pub user_agent: Option<String>,
  pub ip_address: Option<String>,
}

use crate::{
  error::CoreError,
  models::{AuthUser, CreateUser, SigninUser},
  utils::jwt::{AuthTokens, UserClaims},
};

pub trait AuthService: Send + Sync {
  fn signup(
    &self,
    payload: &CreateUser,
    auth_context: Option<AuthContext>,
  ) -> impl std::future::Future<Output = Result<AuthTokens, CoreError>> + Send;

  fn signin(
    &self,
    payload: &SigninUser,
    auth_context: Option<AuthContext>,
  ) -> impl std::future::Future<Output = Result<Option<AuthTokens>, CoreError>> + Send;

  fn refresh_token(
    &self,
    refresh_token: &str,
    auth_context: Option<AuthContext>,
  ) -> impl std::future::Future<Output = Result<AuthTokens, CoreError>> + Send;

  fn logout(
    &self,
    refresh_token: &str,
  ) -> impl std::future::Future<Output = Result<(), CoreError>> + Send;

  fn logout_all(
    &self,
    user_id: i64,
  ) -> impl std::future::Future<Output = Result<(), CoreError>> + Send;

  fn verify_token(&self, token: &str) -> Result<UserClaims, CoreError>;

  fn user_from_claims(&self, claims: UserClaims) -> AuthUser;
}
