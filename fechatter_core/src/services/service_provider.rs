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

impl ActualAuthServiceProvider for ServiceProvider {
  type AuthService = DummyAuthService;

  fn create_service(&self) -> Self::AuthService {
    DummyAuthService
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

#[derive(Clone)]
pub struct DummyAuthService;

// Implement each trait with placeholder logic returning an error. These implementations are
// only meant to satisfy the compiler during the ongoing refactor.

use futures::Future;
use std::pin::Pin;

impl RefreshTokenService for DummyAuthService {
  fn refresh_token(
    &self,
    _refresh_token: &str,
    _auth_context: Option<crate::services::AuthContext>,
  ) -> Pin<Box<dyn Future<Output = Result<AuthTokens, CoreError>> + Send>> {
    Box::pin(async move { Err(CoreError::Internal(anyhow!("Not implemented"))) })
  }
}

impl SignupService for DummyAuthService {
  fn signup(
    &self,
    _payload: &crate::models::CreateUser,
    _auth_context: Option<crate::services::AuthContext>,
  ) -> Pin<Box<dyn Future<Output = Result<AuthTokens, CoreError>> + Send>> {
    Box::pin(async move { Err(CoreError::Internal(anyhow!("Not implemented"))) })
  }
}

impl SigninService for DummyAuthService {
  fn signin(
    &self,
    _payload: &crate::models::SigninUser,
    _auth_context: Option<crate::services::AuthContext>,
  ) -> Pin<Box<dyn Future<Output = Result<Option<AuthTokens>, CoreError>> + Send>> {
    Box::pin(async move { Err(CoreError::Internal(anyhow!("Not implemented"))) })
  }
}

impl LogoutService for DummyAuthService {
  fn logout(
    &self,
    _refresh_token: &str,
  ) -> Pin<Box<dyn Future<Output = Result<(), CoreError>> + Send>> {
    Box::pin(async move { Err(CoreError::Internal(anyhow!("Not implemented"))) })
  }

  fn logout_all(
    &self,
    _user_id: i64,
  ) -> Pin<Box<dyn Future<Output = Result<(), CoreError>> + Send>> {
    Box::pin(async move { Err(CoreError::Internal(anyhow!("Not implemented"))) })
  }
}

impl AuthServiceTrait for DummyAuthService {}
