use crate::{
  AppError,
  models::{CreateUser, SigninUser, User},
  utils::jwt::{AuthTokens, RefreshToken, RefreshTokenData},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod auth_service;
<<<<<<< HEAD
mod service_provider;
mod workspace_service;

=======
pub mod service_provider;
>>>>>>> 19b2301 (refactor: middleware refresh_token & auth cleanup (#20))
pub use service_provider::ServiceProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
  pub user_agent: Option<String>,
  pub ip_address: Option<String>,
}

#[async_trait]
pub trait AuthServiceTrait: Send + Sync {
  async fn signup(
    &self,
    user_data: &CreateUser,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> Result<AuthTokens, AppError>;

  async fn signin(
    &self,
    credentials: &SigninUser,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> Result<Option<AuthTokens>, AppError>;

  async fn refresh_token(
    &self,
    refresh_token: &str,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> Result<AuthTokens, AppError>;

  async fn logout(&self, token: &str) -> Result<(), AppError>;

  async fn logout_all(&self, user_id: i64) -> Result<(), AppError>;

  async fn find_user_by_token(&self, token: &RefreshToken) -> Result<User, AppError>;

  async fn generate_auth_tokens(
    &self,
    user: &User,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> Result<AuthTokens, AppError>;

  async fn create_user(&self, payload: &CreateUser) -> Result<User, AppError>;

  async fn authenticate(&self, payload: &SigninUser) -> Result<Option<User>, AppError>;

  async fn validate_refresh_token(&self, token_str: &str) -> Result<RefreshToken, AppError>;

  async fn rotate_tokens(
    &self,
    old_token: &RefreshToken,
    user: &User,
    current_user_agent: Option<String>,
    current_ip_address: Option<String>,
  ) -> Result<(String, RefreshTokenData), AppError>;
}
