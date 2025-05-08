use crate::error::CoreError;
use crate::jwt::TokenManager;
use crate::middlewares::{
  ActualAuthServiceProvider, TokenVerifier, WithServiceProvider, WithTokenManager,
};
use crate::models::jwt::{
  AuthServiceTrait, AuthTokens, LogoutService, RefreshTokenService, SigninService, SignupService,
  UserClaims,
};
use anyhow::anyhow;
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct ServiceProvider {
  pool: Arc<PgPool>,
  token_manager: Arc<TokenManager>,
}

impl ServiceProvider {
  /// Create a new service provider
  pub fn new(pool: PgPool, token_manager: TokenManager) -> Self {
    Self {
      pool: Arc::new(pool),
      token_manager: Arc::new(token_manager),
    }
  }

  pub fn pool(&self) -> &PgPool {
    &self.pool
  }

  pub fn token_manager(&self) -> &TokenManager {
    &self.token_manager
  }

  pub fn create<T: ServiceFactory>(&self) -> T::Service {
    T::create(self)
  }
}

impl TokenVerifier for ServiceProvider {
  type Claims = UserClaims;
  type Error = CoreError;

  fn verify_token(&self, token: &str) -> Result<Self::Claims, Self::Error> {
    self.token_manager.verify_token(token)
  }
}

impl WithTokenManager for ServiceProvider {
  type TokenManagerType = TokenManager;

  fn token_manager(&self) -> &Self::TokenManagerType {
    &self.token_manager
  }
}

impl WithServiceProvider for ServiceProvider {
  type ServiceProviderType = Self;

  fn service_provider(&self) -> &Self::ServiceProviderType {
    self
  }
}

#[cfg(test)]
impl ActualAuthServiceProvider for ServiceProvider {
  type AuthService = DummyAuthService;

  fn create_service(&self) -> Self::AuthService {
    DummyAuthService
  }
}

#[cfg(not(test))]
impl ActualAuthServiceProvider for ServiceProvider {
  type AuthService = PanicService;

  fn create_service(&self) -> Self::AuthService {
    PanicService
  }
}

#[cfg(not(test))]
#[derive(Clone)]
pub struct PanicService;

#[cfg(not(test))]
impl AuthServiceTrait for PanicService {}

#[cfg(not(test))]
#[async_trait]
impl RefreshTokenService for PanicService {
  async fn refresh_token(
    &self,
    _refresh_token: &str,
    _auth_context: Option<crate::services::AuthContext>,
  ) -> Result<AuthTokens, CoreError> {
    panic!("PanicService is not meant to be used in production code")
  }
}

#[cfg(not(test))]
#[async_trait]
impl SignupService for PanicService {
  async fn signup(
    &self,
    _payload: &crate::models::CreateUser,
    _auth_context: Option<crate::services::AuthContext>,
  ) -> Result<AuthTokens, CoreError> {
    panic!("PanicService is not meant to be used in production code")
  }
}

#[cfg(not(test))]
#[async_trait]
impl SigninService for PanicService {
  async fn signin(
    &self,
    _payload: &crate::models::SigninUser,
    _auth_context: Option<crate::services::AuthContext>,
  ) -> Result<Option<AuthTokens>, CoreError> {
    panic!("PanicService is not meant to be used in production code")
  }
}

#[cfg(not(test))]
#[async_trait]
impl LogoutService for PanicService {
  async fn logout(&self, _refresh_token: &str) -> Result<(), CoreError> {
    panic!("PanicService is not meant to be used in production code")
  }

  async fn logout_all(&self, _user_id: i64) -> Result<(), CoreError> {
    panic!("PanicService is not meant to be used in production code")
  }
}

pub trait ServiceFactory {
  type Service;

  fn create(provider: &ServiceProvider) -> Self::Service;
}

pub trait ServiceMarker {}

#[macro_export]
macro_rules! define_service {
    (
        $service_name:ident,
        $marker:ident,
        $($field:ident: $ty:ty),*
    ) => {
        pub struct $marker;
        impl $crate::services::service_provider::ServiceMarker for $marker {}

        pub struct $service_name {
            $(pub(crate) $field: $ty,)*
            _marker: std::marker::PhantomData<$marker>,
        }


        // Implement service factory
        impl $crate::services::service_provider::ServiceFactory for $marker {
            type Service = $service_name;

            fn create(provider: &$crate::services::service_provider::ServiceProvider) -> Self::Service {
                $service_name {
                    $(
                        $field: compile_error!("Please specify how to create each field"),
                    )*
                    _marker: std::marker::PhantomData,
                }
            }
        }
    };
}

#[cfg(test)]
#[derive(Clone)]
pub struct DummyAuthService;

// Implement each trait with placeholder logic returning an error. These implementations are
// only meant to satisfy the compiler during the ongoing refactor.

#[cfg(test)]
#[async_trait]
impl RefreshTokenService for DummyAuthService {
  async fn refresh_token(
    &self,
    _refresh_token: &str,
    _auth_context: Option<crate::services::AuthContext>,
  ) -> Result<AuthTokens, CoreError> {
    Err(CoreError::Internal(anyhow!("Not implemented")))
  }
}

#[cfg(test)]
#[async_trait]
impl SignupService for DummyAuthService {
  async fn signup(
    &self,
    _payload: &crate::models::CreateUser,
    _auth_context: Option<crate::services::AuthContext>,
  ) -> Result<AuthTokens, CoreError> {
    Err(CoreError::Internal(anyhow!("Not implemented")))
  }
}

#[cfg(test)]
#[async_trait]
impl SigninService for DummyAuthService {
  async fn signin(
    &self,
    _payload: &crate::models::SigninUser,
    _auth_context: Option<crate::services::AuthContext>,
  ) -> Result<Option<AuthTokens>, CoreError> {
    Err(CoreError::Internal(anyhow!("Not implemented")))
  }
}

#[cfg(test)]
#[async_trait]
impl LogoutService for DummyAuthService {
  async fn logout(&self, _refresh_token: &str) -> Result<(), CoreError> {
    Err(CoreError::Internal(anyhow!("Not implemented")))
  }

  async fn logout_all(&self, _user_id: i64) -> Result<(), CoreError> {
    Err(CoreError::Internal(anyhow!("Not implemented")))
  }
}

#[cfg(test)]
impl AuthServiceTrait for DummyAuthService {}
