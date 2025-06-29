pub mod auth_service;
pub mod mock;
pub mod retry;
pub mod service_provider;
pub mod workspace_service;

use crate::models::user::AuthUser;
use crate::{
  contracts::AuthContext,
  error::CoreError,
  models::jwt::{AuthTokens, UserClaims},
  models::{CreateUser, SigninUser, UserId},
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
    user_id: UserId,
  ) -> impl std::future::Future<Output = Result<(), CoreError>> + Send;

  fn verify_token(&self, token: &str) -> Result<UserClaims, CoreError>;

  fn user_from_claims(&self, claims: UserClaims) -> AuthUser;
}

// Re-export for backward compatibility
pub use auth_service::*;
pub use service_provider::*;
pub use workspace_service::*;
