use crate::AppError;
use crate::utils::jwt::TokenManager;
use sqlx::PgPool;
use std::sync::Arc;

/// 服务标记特征，用于静态多态
pub trait ServiceMarker {}

/// 统一服务提供者，减少重复代码
#[derive(Clone)]
pub struct ServiceProvider {
  pool: Arc<PgPool>,
  token_manager: Arc<TokenManager>,
}

impl ServiceProvider {
  /// 创建新的服务提供者
  pub fn new(pool: PgPool, token_manager: TokenManager) -> Self {
    Self {
      pool: Arc::new(pool),
      token_manager: Arc::new(token_manager),
    }
  }

  /// 获取数据库连接池
  pub fn pool(&self) -> &PgPool {
    &self.pool
  }

  /// 获取令牌管理器
  pub fn token_manager(&self) -> &TokenManager {
    &self.token_manager
  }

  /// 创建特定类型的服务
  pub fn create<T: ServiceFactory>(&self) -> T::Service {
    T::create(self)
  }

  /// 创建特定类型的服务使用特征对象
  pub fn create_service<T: 'static + Send + Sync>(
    &self,
  ) -> Box<dyn crate::services::AuthServiceTrait + '_> {
    Box::new(crate::services::auth_service::AuthService::new(self))
  }
}

/// 工厂特征，用于创建服务
pub trait ServiceFactory {
  /// 服务类型
  type Service;

  /// 从服务提供者创建服务
  fn create(provider: &ServiceProvider) -> Self::Service;
}

/// 统一错误处理 Result 类型
pub type ServiceResult<T> = Result<T, AppError>;

/// 服务构建宏，自动生成服务工厂实现
#[macro_export]
macro_rules! define_service {
    (
        $service_name:ident,
        $marker:ident,
        $($field:ident: $ty:ty),*
    ) => {
        // 定义服务标记
        pub struct $marker;
        impl $crate::services::service_provider::ServiceMarker for $marker {}

        // 定义服务
        pub struct $service_name {
            $(pub(crate) $field: $ty,)*
            _marker: std::marker::PhantomData<$marker>,
        }

        // 实现服务工厂
        impl crate::services::service_provider::ServiceFactory for $marker {
            type Service = $service_name;

            fn create(provider: &crate::services::service_provider::ServiceProvider) -> Self::Service {
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
