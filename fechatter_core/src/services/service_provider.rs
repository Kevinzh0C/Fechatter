use crate::error::CoreError;
use crate::jwt::TokenManager;
use crate::middlewares::{
  ActualAuthServiceProvider, TokenVerifier, WithServiceProvider, WithTokenManager,
};
use crate::models::UserId;
use crate::models::jwt::{
  AuthServiceTrait, AuthTokens, LogoutService, RefreshTokenService, SigninService, SignupService,
  UserClaims,
};
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

// 注意：这里故意不完整，需要在fechatter_server中实现这个特性
// 我们设置一个占位符类型，应该在server中被正确替换
#[cfg(not(test))]
impl ActualAuthServiceProvider for ServiceProvider {
  // 定义一个占位符类型，这个类型会在服务器层被实际实现
  type AuthService = RealAuthServicePlaceholder;

  fn create_service(&self) -> Self::AuthService {
    // 这个函数永远不应该被直接调用，因为实际的实现应该在服务器层
    // 如果这个函数被调用了，那就是配置错误
    panic!(
      "This is a placeholder implementation. The actual implementation should be in the server layer"
    )
  }
}

// 定义一个占位符类型，这个类型在服务器层会被正确替换
#[cfg(not(test))]
#[derive(Clone)]
pub struct RealAuthServicePlaceholder;

// 为占位符实现必要的特性，这样编译器才不会报错
#[cfg(not(test))]
impl AuthServiceTrait for RealAuthServicePlaceholder {}

#[cfg(not(test))]
#[async_trait]
impl RefreshTokenService for RealAuthServicePlaceholder {
  async fn refresh_token(
    &self,
    _refresh_token: &str,
    __auth_context: Option<crate::services::AuthContext>,
  ) -> Result<AuthTokens, CoreError> {
    panic!("This is a placeholder implementation")
  }
}

#[cfg(not(test))]
#[async_trait]
impl SignupService for RealAuthServicePlaceholder {
  async fn signup(
    &self,
    __payload: &crate::models::CreateUser,
    __auth_context: Option<crate::services::AuthContext>,
  ) -> Result<AuthTokens, CoreError> {
    panic!("This is a placeholder implementation")
  }
}

#[cfg(not(test))]
#[async_trait]
impl SigninService for RealAuthServicePlaceholder {
  async fn signin(
    &self,
    _payload: &crate::models::SigninUser,
    __auth_context: Option<crate::services::AuthContext>,
  ) -> Result<Option<AuthTokens>, CoreError> {
    panic!("This is a placeholder implementation")
  }
}

#[cfg(not(test))]
#[async_trait]
impl LogoutService for RealAuthServicePlaceholder {
  async fn logout(&self, _refresh_token: &str) -> Result<(), CoreError> {
    Err(CoreError::Internal(
      "Real auth service not implemented".to_string(),
    ))
  }

  async fn logout_all(&self, _user_id: UserId) -> Result<(), CoreError> {
    Err(CoreError::Internal(
      "Real auth service not implemented".to_string(),
    ))
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
    let now = chrono::Utc::now();
    let expires_at = now + chrono::Duration::days(7);
    let absolute_expires_at = now + chrono::Duration::days(30);

    // 创建基本的模拟令牌
    Ok(AuthTokens {
      access_token: "mock-access-token-for-test".to_string(),
      refresh_token: crate::models::jwt::RefreshTokenData {
        token: "mock-refresh-token-for-test".to_string(),
        expires_at,
        absolute_expires_at,
      },
    })
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
    // 简单地创建一个基本的模拟令牌
    let now = chrono::Utc::now();
    let expires_at = now + chrono::Duration::days(7);
    let absolute_expires_at = now + chrono::Duration::days(30);

    Ok(AuthTokens {
      access_token: "mock-access-token-for-test".to_string(),
      refresh_token: crate::models::jwt::RefreshTokenData {
        token: "mock-refresh-token-for-test".to_string(),
        expires_at,
        absolute_expires_at,
      },
    })
  }
}

#[cfg(test)]
#[async_trait]
impl SigninService for DummyAuthService {
  async fn signin(
    &self,
    payload: &crate::models::SigninUser,
    _auth_context: Option<crate::services::AuthContext>,
  ) -> Result<Option<AuthTokens>, CoreError> {
    // 模拟成功登录
    let now = chrono::Utc::now();
    let expires_at = now + chrono::Duration::days(7);
    let absolute_expires_at = now + chrono::Duration::days(30);

    // 如果是测试用户名"nonexistent@acme.test"，返回None
    if payload.email == "nonexistent@acme.test" {
      return Ok(None);
    }

    Ok(Some(AuthTokens {
      access_token: "mock-access-token-for-test".to_string(),
      refresh_token: crate::models::jwt::RefreshTokenData {
        token: "mock-refresh-token-for-test".to_string(),
        expires_at,
        absolute_expires_at,
      },
    }))
  }
}

#[cfg(test)]
#[async_trait]
impl LogoutService for DummyAuthService {
  async fn logout(&self, _refresh_token: &str) -> Result<(), CoreError> {
    // 简单地返回成功
    Ok(())
  }

  async fn logout_all(&self, _user_id: UserId) -> Result<(), CoreError> {
    // 简单地返回成功
    Ok(())
  }
}

#[cfg(test)]
impl AuthServiceTrait for DummyAuthService {}
