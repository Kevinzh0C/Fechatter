use fechatter_core::error::CoreError;
use fechatter_core::jwt::TokenManager;
use fechatter_core::middlewares::{
  ActualAuthServiceProvider, TokenVerifier, WithServiceProvider, WithTokenManager,
};

use fechatter_core::models::jwt::UserClaims;
use once_cell::sync::OnceCell;
use sqlx::PgPool;
use std::sync::Arc;

// Import the real AuthService
use crate::services::auth_service::AuthService;
use crate::utils::refresh_token::RefreshTokenAdaptor;

// Static cache for AuthService
static AUTH_SERVICE: OnceCell<Arc<AuthService>> = OnceCell::new();

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
  type AuthService = AuthService;

  fn create_service(&self) -> Self::AuthService {
    // Get or initialize the cached AuthService
    let auth_service = AUTH_SERVICE.get_or_init(|| {
      // 创建用户存储库
      let user_repository = Box::new(crate::models::user::FechatterUserRepository::new(
        self.pool.clone(),
      ));

      // 使用TokenManager作为TokenService，需要解引用再装箱
      let token_service: Box<dyn fechatter_core::TokenService + Send + Sync + 'static> =
        Box::new(self.token_manager.as_ref().clone());

      // 创建刷新令牌存储库
      let refresh_token_repository = Box::new(RefreshTokenAdaptor::new(self.pool.clone()));

      // 创建 AuthService 实例并包装在Arc中
      Arc::new(AuthService::new(
        user_repository,
        token_service,
        refresh_token_repository,
      ))
    });

    // 由于AuthService现在实现了Clone特性，我们可以返回一个克隆
    (**auth_service).clone()
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
