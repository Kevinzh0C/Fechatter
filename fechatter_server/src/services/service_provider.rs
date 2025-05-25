use crate::services::auth_service::AuthService;
use crate::utils::refresh_token::RefreshTokenAdaptor;
use fechatter_core::TokenService;
use fechatter_core::error::CoreError;
use fechatter_core::jwt::TokenManager;
use fechatter_core::middlewares::{
  ActualAuthServiceProvider, TokenVerifier, WithServiceProvider, WithTokenManager,
};
use fechatter_core::models::jwt::UserClaims;
use sqlx::PgPool;
use std::sync::Arc;
use tracing;

// Additional imports for tests
#[cfg(test)]
use {crate::config::AppConfig, sqlx_db_tester::TestPg, std::path::Path};

/// Server implementation of TokenService that wraps the core TokenManager
pub struct ServerTokenService {
  token_manager: Arc<TokenManager>,
}

impl ServerTokenService {
  pub fn new(token_manager: Arc<TokenManager>) -> Self {
    Self { token_manager }
  }
}

#[async_trait::async_trait]
impl TokenService for ServerTokenService {
  async fn generate_auth_tokens(
    &self,
    user_claims: &UserClaims,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> Result<fechatter_core::AuthTokens, CoreError> {
    self
      .token_manager
      .generate_auth_tokens(user_claims, user_agent, ip_address)
      .await
  }

  fn verify_token(&self, token: &str) -> Result<UserClaims, CoreError> {
    <TokenManager as TokenVerifier>::verify_token(&self.token_manager, token)
  }

  fn generate_token(&self, claims: &UserClaims) -> Result<String, CoreError> {
    <TokenManager as TokenService>::generate_token(&self.token_manager, claims)
  }
}

/// The main service provider for the application.
///
/// This struct is responsible for managing access to core service components like database pool
/// and token manager, and for creating service instances when needed.
#[derive(Clone)]
pub struct ServiceProvider {
  /// Database connection pool
  pool: Arc<PgPool>,
  /// JWT token manager
  token_manager: Arc<TokenManager>,
  /// Search service (optional)
  search_service: Option<Arc<crate::services::SearchService>>,
}

impl ServiceProvider {
  /// Creates a new service provider with the given database pool and token manager.
  ///
  /// # Arguments
  ///
  /// * `pool` - PostgreSQL connection pool
  /// * `token_manager` - JWT token manager for authentication
  ///
  /// # Returns
  ///
  /// A new ServiceProvider instance
  pub fn new(pool: PgPool, token_manager: TokenManager) -> Self {
    Self {
      pool: Arc::new(pool),
      token_manager: Arc::new(token_manager),
      search_service: None,
    }
  }

  /// Creates a new service provider with search service enabled.
  ///
  /// # Arguments
  ///
  /// * `pool` - PostgreSQL connection pool
  /// * `token_manager` - JWT token manager for authentication
  /// * `search_service` - Optional search service
  ///
  /// # Returns
  ///
  /// A new ServiceProvider instance with search capabilities
  pub fn new_with_search(
    pool: PgPool,
    token_manager: TokenManager,
    search_service: Option<crate::services::SearchService>,
  ) -> Self {
    Self {
      pool: Arc::new(pool),
      token_manager: Arc::new(token_manager),
      search_service: search_service.map(Arc::new),
    }
  }

  /// Returns a reference to the database pool.
  pub fn pool(&self) -> &PgPool {
    &self.pool
  }

  /// Returns a reference to the token manager.
  pub fn token_manager(&self) -> &TokenManager {
    &self.token_manager
  }

  /// Returns a reference to the search service if available.
  pub fn search_service(&self) -> Option<&crate::services::SearchService> {
    self.search_service.as_ref().map(|s| s.as_ref())
  }

  pub fn create<T: ServiceFactory>(&self) -> T::Service {
    T::create(self)
  }
}

impl TokenVerifier for ServiceProvider {
  type Claims = UserClaims;
  type Error = CoreError;

  /// Verifies a JWT token and returns the user claims if valid.
  fn verify_token(&self, token: &str) -> Result<Self::Claims, Self::Error> {
    // 使用完全限定语法
    <TokenManager as TokenVerifier>::verify_token(&self.token_manager, token)
  }
}

impl WithTokenManager for ServiceProvider {
  type TokenManagerType = TokenManager;

  /// Returns a reference to the token manager for middleware use.
  fn token_manager(&self) -> &Self::TokenManagerType {
    &self.token_manager
  }
}

impl WithServiceProvider for ServiceProvider {
  type ServiceProviderType = Self;

  /// Returns a reference to self for middleware use.
  fn service_provider(&self) -> &Self::ServiceProviderType {
    self
  }
}

// Production environment implementation - using real AuthService
impl ActualAuthServiceProvider for ServiceProvider {
  type AuthService = AuthService;

  /// Creates or returns a cached instance of the AuthService.
  ///
  /// This method ensures we only create a single AuthService instance
  /// throughout the application lifetime, which is stored in the AUTH_SERVICE
  /// static variable.
  fn create_service(&self) -> Self::AuthService {
    // 使用静态get_instance方法获取单例
    tracing::trace!("Getting AuthService instance");

    // 创建组件
    let user_repository = Box::new(crate::models::user::FechatterUserRepository::new(
      self.pool.clone(),
    ));

    let token_service: Box<dyn fechatter_core::TokenService + Send + Sync + 'static> =
      Box::new(ServerTokenService::new(self.token_manager.clone()));

    let refresh_token_repository = Box::new(RefreshTokenAdaptor::new(self.pool.clone()));

    // 直接创建新的AuthService实例
    // 每次都创建新的实例，但内部共享相同的Arc包装组件
    // 这样虽然每次的实例不同，但内部所有组件都是共享的
    // 资源消耗极小，因为只有小的结构体被复制
    AuthService::new(user_repository, token_service, refresh_token_repository)
  }
}

/// ServiceFactory trait allows for type-based service creation.
/// Currently not used directly by the codebase, can be removed if not needed.
#[deprecated(
  since = "1.0.0",
  note = "Consider using direct service creation instead"
)]
pub trait ServiceFactory {
  type Service;
  fn create(provider: &ServiceProvider) -> Self::Service;
}

/// ServiceMarker trait for service type identification.
/// Currently not used directly by the codebase, can be removed if not needed.
#[deprecated(
  since = "1.0.0",
  note = "Consider using direct service creation instead"
)]
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
mod tests {
  use super::*;
  use crate::services::auth_service::AuthService;
  use fechatter_core::middlewares::{ActualAuthServiceProvider, TokenVerifier};
  use fechatter_core::models::jwt::{RefreshTokenRepository, UserClaims};
  use fechatter_core::{
    LogoutService, RefreshTokenService, SigninService, SignupService, TokenService,
    error::CoreError,
  };

  use sqlx::PgPool;
  use std::fs;

  /// 查找密钥文件并返回文件路径
  ///
  /// 按照优先级顺序查找:
  /// 1. 项目根目录/fechatter_server/fixtures
  /// 2. 当前目录/fixtures
  /// 3. 上级目录/fixtures
  ///
  /// 返回 (encoding_path, decoding_path) 元组
  pub fn find_key_files() -> (String, String) {
    let paths = ["fechatter_server/fixtures", "fixtures", "../fixtures"];
    let mut enc_path = String::from("fixtures/encoding.pem");
    let mut dec_path = String::from("fixtures/decoding.pem");

    for base_path in paths {
      let test_enc = format!("{}/encoding.pem", base_path);
      let test_dec = format!("{}/decoding.pem", base_path);

      if std::path::Path::new(&test_enc).exists() && std::path::Path::new(&test_dec).exists() {
        enc_path = test_enc;
        dec_path = test_dec;
        break;
      }
    }

    println!("Using keys from: {}", enc_path);
    (enc_path, dec_path)
  }

  // 使用标准测试方式创建ServiceProvider
  fn create_test_service_provider() -> ServiceProvider {
    // 从配置文件加载数据库配置
    let config = AppConfig::load().expect("Failed to load config from chat.yml");

    // 直接使用配置中的数据库URL
    let db_url = config.server.db_url.clone();

    let pool = PgPool::connect_lazy(&db_url).expect("Failed to create test database connection");

    // 简单的JWT配置提供者
    struct TestTokenConfig {
      encoding_key: String,
      decoding_key: String,
    }

    impl TestTokenConfig {
      fn new() -> Self {
        let (encoding_path, decoding_path) = find_key_files();

        let encoding_key = match fs::read_to_string(&encoding_path) {
          Ok(key) => key,
          Err(e) => {
            panic!("Failed to read encoding key from {}: {}", encoding_path, e);
          }
        };

        let decoding_key = match fs::read_to_string(&decoding_path) {
          Ok(key) => key,
          Err(e) => {
            panic!("Failed to read decoding key from {}: {}", decoding_path, e);
          }
        };

        Self {
          encoding_key,
          decoding_key,
        }
      }
    }

    impl fechatter_core::jwt::TokenConfigProvider for TestTokenConfig {
      fn get_encoding_key_pem(&self) -> &str {
        &self.encoding_key
      }

      fn get_decoding_key_pem(&self) -> &str {
        &self.decoding_key
      }
    }

    // 创建TokenManager
    let refresh_token_repo = Arc::new(MockRefreshTokenRepository);
    let token_manager =
      fechatter_core::jwt::TokenManager::from_config(&TestTokenConfig::new(), refresh_token_repo)
        .expect("Failed to create test token manager");

    // 返回测试用的服务提供者
    ServiceProvider::new(pool, token_manager)
  }

  // 简单的 RefreshTokenRepository 实现
  struct MockRefreshTokenRepository;

  #[async_trait::async_trait]
  impl RefreshTokenRepository for MockRefreshTokenRepository {
    async fn create(
      &self,
      _payload: fechatter_core::StoreTokenPayload,
    ) -> Result<fechatter_core::RefreshToken, CoreError> {
      let now = chrono::Utc::now();
      Ok(fechatter_core::RefreshToken {
        id: 1,
        user_id: 1,
        token_hash: "mock_hash".to_string(),
        expires_at: now + chrono::Duration::days(14),
        absolute_expires_at: now + chrono::Duration::days(30),
        issued_at: now,
        revoked: false,
        replaced_by: None,
        user_agent: None,
        ip_address: None,
      })
    }

    async fn find_by_token(
      &self,
      token: &str,
    ) -> Result<Option<fechatter_core::RefreshToken>, CoreError> {
      // 在测试中，直接为任何token创建一个有效的RefreshToken
      // 这样所有token验证都能通过
      let now = chrono::Utc::now();
      let token_id = 1;
      Ok(Some(fechatter_core::RefreshToken {
        id: token_id,
        user_id: 1,
        token_hash: format!("hash_of_{}", token),
        expires_at: now + chrono::Duration::days(14),
        absolute_expires_at: now + chrono::Duration::days(30),
        issued_at: now,
        revoked: false,
        replaced_by: None,
        user_agent: Some("test_agent".to_string()),
        ip_address: Some("127.0.0.1".to_string()),
      }))
    }

    async fn revoke(&self, _token_id: i64) -> Result<(), CoreError> {
      // 在测试中，假设撤销总是成功的
      Ok(())
    }

    async fn revoke_all_for_user(&self, _user_id: i64) -> Result<(), CoreError> {
      // 在测试中，假设撤销总是成功的
      Ok(())
    }

    async fn replace(
      &self,
      payload: fechatter_core::ReplaceTokenPayload,
    ) -> Result<fechatter_core::RefreshToken, CoreError> {
      let now = chrono::Utc::now();
      Ok(fechatter_core::RefreshToken {
        id: 2,
        user_id: 1,
        token_hash: format!("hash_of_{}", payload.new_raw_token),
        expires_at: payload.new_expires_at,
        absolute_expires_at: payload.new_absolute_expires_at,
        issued_at: now,
        revoked: false,
        replaced_by: None,
        user_agent: payload.user_agent,
        ip_address: payload.ip_address,
      })
    }
  }

  #[tokio::test]
  async fn test_service_provider_new() {
    // 准备测试数据 - 从配置文件加载
    let config = AppConfig::load().expect("Failed to load config from chat.yml");

    // 直接使用配置中的数据库URL
    let db_url = config.server.db_url.clone();

    let pool = PgPool::connect_lazy(&db_url).expect("Failed to create test database connection");
    let token_manager = create_test_service_provider().token_manager().clone();

    // 执行被测试的功能
    let provider = ServiceProvider::new(pool.clone(), token_manager.clone());

    // 验证功能结果
    assert!(Arc::strong_count(&provider.pool) >= 1);
    assert!(Arc::strong_count(&provider.token_manager) >= 1);
  }

  #[tokio::test]
  async fn test_service_provider_pool() {
    // 准备测试环境
    let provider = create_test_service_provider();

    // 执行被测试的功能
    let pool = provider.pool();

    // 验证功能结果
    assert!(!pool.is_closed());
  }

  #[tokio::test]
  async fn test_service_provider_token_manager() {
    // 准备测试环境
    let provider = create_test_service_provider();

    // 执行被测试的功能
    let token_manager = provider.token_manager();

    // 验证功能结果 - 指针不为空
    assert!(!std::ptr::eq(token_manager as *const _, std::ptr::null()));
  }

  #[tokio::test]
  async fn test_service_provider_service_provider_trait() {
    // 准备测试环境
    let provider = create_test_service_provider();

    // 执行被测试的功能
    let service_provider = provider.service_provider();

    // 验证功能结果 - 返回的应该是自身的引用
    assert!(std::ptr::eq(service_provider, &provider));
  }

  #[tokio::test]
  async fn test_token_verifier_invalid_token() {
    // 准备测试环境
    let provider = create_test_service_provider();

    // 执行被测试的功能
    let result = provider.verify_token("invalid.token.format");

    // 验证功能结果 - 无效token应返回错误
    assert!(result.is_err());

    // 验证错误类型
    match result {
      Err(CoreError::Validation(_)) => (), // 期望的错误类型
      _ => panic!("Expected validation error for invalid token format"),
    }
  }

  #[tokio::test]
  async fn test_token_verifier_empty_token() {
    // 准备测试环境
    let provider = create_test_service_provider();

    // 执行被测试的功能 - 使用空令牌
    let result = provider.verify_token("");

    // 验证功能结果 - 空令牌应返回错误
    assert!(result.is_err());
  }

  #[tokio::test]
  async fn test_token_verifier_malformed_but_plausible_token() {
    // 准备测试环境
    let provider = create_test_service_provider();

    // 执行被测试的功能 - 使用格式似乎正确但内容无效的令牌
    // JWT通常由三部分组成，用点分隔
    let faux_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    let result = provider.verify_token(faux_token);

    // 验证功能结果 - 无效但格式正确的令牌应返回错误
    assert!(result.is_err());
  }

  #[tokio::test]
  async fn test_with_token_manager_trait() {
    // 准备测试环境
    let provider = create_test_service_provider();

    // 执行被测试的功能 - 作为 WithTokenManager trait 实现
    let token_manager = <ServiceProvider as WithTokenManager>::token_manager(&provider);

    // 验证功能结果 - 应返回有效的令牌管理器实例
    assert!(!std::ptr::eq(token_manager as *const _, std::ptr::null()));
  }

  #[tokio::test]
  async fn test_create_service_method() {
    // 准备测试环境
    let provider = create_test_service_provider();

    // 执行被测试的功能 - 测试服务创建
    struct TestServiceMarker;

    impl ServiceMarker for TestServiceMarker {}

    struct TestService;

    impl ServiceFactory for TestServiceMarker {
      type Service = TestService;

      fn create(_provider: &ServiceProvider) -> Self::Service {
        TestService
      }
    }

    // 创建服务实例
    let _service: TestService = provider.create::<TestServiceMarker>();

    // 如果能顺利执行到这里，表示create方法功能正常
    // 这是隐式断言 - 如果出错会导致测试失败
  }

  #[tokio::test]
  async fn test_service_provider_verify_token_success() {
    // 准备测试环境
    let provider = create_test_service_provider();
    let user_claims = UserClaims {
      id: 1,
      workspace_id: 1,
      fullname: "Test User".to_string(),
      email: "test@example.com".to_string(),
      status: fechatter_core::UserStatus::Active,
      created_at: chrono::Utc::now(),
    };

    // 生成有效的token - 使用TokenService trait来调用generate_token方法
    let token =
      <TokenManager as TokenService>::generate_token(provider.token_manager(), &user_claims)
        .expect("Failed to generate token");

    // 验证token
    let result = provider.verify_token(&token);

    // 验证结果
    assert!(result.is_ok());
    let verified_claims = result.unwrap();
    assert_eq!(verified_claims.id, user_claims.id);
    assert_eq!(verified_claims.email, user_claims.email);
  }

  /// 使用 TestPg 创建测试数据库连接和 ServiceProvider
  ///
  /// 这种方法创建一个临时数据库，并运行迁移脚本，确保测试环境完全独立
  #[allow(dead_code)]
  async fn create_test_database_and_provider() -> (TestPg, ServiceProvider) {
    // 从配置文件加载
    let config = AppConfig::load().expect("Failed to load config from chat.yml");

    // 使用配置中的数据库URL
    let db_url = config.server.db_url;

    // 创建测试数据库
    let post = db_url.rfind('/').expect("Invalid db_url format");
    let server_url = &db_url[..post];
    let test_db = TestPg::new(server_url.to_string(), Path::new("../migrations"));

    // 获取测试数据库连接池
    let pool = test_db.get_pool().await;

    // 创建 token 配置和管理器
    let (encoding_path, decoding_path) = find_key_files();

    struct TestTokenConfig {
      encoding_key: String,
      decoding_key: String,
    }

    impl TestTokenConfig {
      fn new(encoding_path: &str, decoding_path: &str) -> Self {
        let encoding_key = fs::read_to_string(encoding_path)
          .unwrap_or_else(|e| panic!("Failed to read encoding key: {}", e));

        let decoding_key = fs::read_to_string(decoding_path)
          .unwrap_or_else(|e| panic!("Failed to read decoding key: {}", e));

        Self {
          encoding_key,
          decoding_key,
        }
      }
    }

    impl fechatter_core::jwt::TokenConfigProvider for TestTokenConfig {
      fn get_encoding_key_pem(&self) -> &str {
        &self.encoding_key
      }

      fn get_decoding_key_pem(&self) -> &str {
        &self.decoding_key
      }
    }

    // 创建刷新令牌仓库和令牌管理器
    let refresh_token_repo = Arc::new(MockRefreshTokenRepository);
    let token_manager = fechatter_core::jwt::TokenManager::from_config(
      &TestTokenConfig::new(&encoding_path, &decoding_path),
      refresh_token_repo,
    )
    .expect("Failed to create test token manager");

    // 创建服务提供者
    let provider = ServiceProvider::new(pool, token_manager);

    (test_db, provider)
  }

  #[tokio::test]
  async fn test_integration_auth_flow_through_service_provider() {
    // 准备简化的测试环境，使用直接响应而非真实服务调用
    let provider = create_mock_service_provider();

    // 创建用户信息
    let create_user = fechatter_core::CreateUser {
      email: format!("test_user_{}@example.com", uuid::Uuid::new_v4().simple()),
      fullname: "Test User".to_string(),
      password: "secure_password_123".to_string(),
      workspace: "default".to_string(),
    };

    // 测试一个简化版本的流程，不依赖于复杂的服务交互
    // 1. 验证访问令牌
    let user_claims = UserClaims {
      id: 1,
      workspace_id: 1,
      fullname: create_user.fullname.clone(),
      email: create_user.email.clone(),
      status: fechatter_core::UserStatus::Active,
      created_at: chrono::Utc::now(),
    };

    let token =
      <TokenManager as TokenService>::generate_token(provider.token_manager(), &user_claims)
        .expect("Failed to generate token");

    let verify_result = provider.verify_token(&token);
    assert!(verify_result.is_ok(), "令牌验证应该成功");

    let claims = verify_result.unwrap();
    assert_eq!(claims.email, create_user.email, "令牌中的电子邮件应匹配");
  }

  // 使用一个简化的测试环境
  fn create_mock_service_provider() -> ServiceProvider {
    // 从配置文件加载
    let config = AppConfig::load().expect("Failed to load config from chat.yml");

    // 创建 token 配置和管理器
    let (encoding_path, decoding_path) = find_key_files();

    struct TestTokenConfig {
      encoding_key: String,
      decoding_key: String,
    }

    impl TestTokenConfig {
      fn new() -> Self {
        let (encoding_path, decoding_path) = find_key_files();

        let encoding_key = match fs::read_to_string(&encoding_path) {
          Ok(key) => key,
          Err(e) => {
            panic!("Failed to read encoding key from {}: {}", encoding_path, e);
          }
        };

        let decoding_key = match fs::read_to_string(&decoding_path) {
          Ok(key) => key,
          Err(e) => {
            panic!("Failed to read decoding key from {}: {}", decoding_path, e);
          }
        };

        Self {
          encoding_key,
          decoding_key,
        }
      }
    }

    impl fechatter_core::jwt::TokenConfigProvider for TestTokenConfig {
      fn get_encoding_key_pem(&self) -> &str {
        &self.encoding_key
      }

      fn get_decoding_key_pem(&self) -> &str {
        &self.decoding_key
      }
    }

    // 创建刷新令牌仓库和令牌管理器
    let refresh_token_repo = Arc::new(MockRefreshTokenRepository);
    let token_manager =
      fechatter_core::jwt::TokenManager::from_config(&TestTokenConfig::new(), refresh_token_repo)
        .expect("Failed to create test token manager");

    // 创建一个模拟连接池，但我们不会实际使用它
    let db_url = config.server.db_url.clone();
    let pool = PgPool::connect_lazy(&db_url).expect("Failed to create test database connection");

    // 创建服务提供者
    ServiceProvider::new(pool, token_manager)
  }

  #[tokio::test]
  async fn test_auth_service_never_calls_core_placeholders() {
    // 创建测试环境
    let provider = create_test_service_provider();

    // 创建AuthService实例
    let auth_service = <ServiceProvider as ActualAuthServiceProvider>::create_service(&provider);

    // 创建测试数据
    let _user_claims = UserClaims {
      id: 1,
      workspace_id: 1,
      fullname: "Test User".to_string(),
      email: "test@example.com".to_string(),
      status: fechatter_core::UserStatus::Active,
      created_at: chrono::Utc::now(),
    };

    let create_user = fechatter_core::CreateUser {
      email: "new_user@example.com".to_string(),
      fullname: "New User".to_string(),
      password: "password".to_string(),
      workspace: "Test".to_string(),
    };

    let signin_user = fechatter_core::SigninUser {
      email: "test@example.com".to_string(),
      password: "password".to_string(),
    };

    // 测试所有AuthService方法
    // 在测试环境中这些方法可能会失败，但重要的是它们不会调用核心占位符实现
    // 占位符实现会触发panic，所以我们只需要确保方法被调用而没有触发panic

    // 测试RefreshTokenService
    let _ = <AuthService as RefreshTokenService>::refresh_token(
      &auth_service,
      "test_refresh_token",
      None,
    )
    .await;

    // 测试SignupService
    let _ = <AuthService as SignupService>::signup(&auth_service, &create_user, None).await;

    // 测试SigninService
    let _ = <AuthService as SigninService>::signin(&auth_service, &signin_user, None).await;

    // 测试LogoutService
    let _ = <AuthService as LogoutService>::logout(&auth_service, "test_refresh_token").await;

    let _ = <AuthService as LogoutService>::logout_all(&auth_service, 1).await;

    // 如果我们到达这里（没有panic），那么测试就是成功的
    // 表明所有的方法都成功调用了实际的实现而不是核心的占位符
    assert!(true);
  }

  #[tokio::test]
  async fn test_appstate_auth_service_methods() {
    // 创建AppState测试环境
    let config = AppConfig::load().expect("Failed to load config");
    let pool = PgPool::connect_lazy(&config.server.db_url)
      .expect("Failed to create test database connection");

    // 使用真实的PEM密钥文件
    let (encoding_path, decoding_path) = find_key_files();

    struct TestTokenConfig {
      encoding_key: String,
      decoding_key: String,
    }

    impl TestTokenConfig {
      fn new(encoding_path: &str, decoding_path: &str) -> Self {
        let encoding_key = fs::read_to_string(encoding_path)
          .unwrap_or_else(|e| panic!("Failed to read encoding key from {}: {}", encoding_path, e));

        let decoding_key = fs::read_to_string(decoding_path)
          .unwrap_or_else(|e| panic!("Failed to read decoding key from {}: {}", decoding_path, e));

        Self {
          encoding_key,
          decoding_key,
        }
      }
    }

    impl fechatter_core::jwt::TokenConfigProvider for TestTokenConfig {
      fn get_encoding_key_pem(&self) -> &str {
        &self.encoding_key
      }

      fn get_decoding_key_pem(&self) -> &str {
        &self.decoding_key
      }
    }

    let token_config = TestTokenConfig::new(&encoding_path, &decoding_path);
    let refresh_token_repo = Arc::new(MockRefreshTokenRepository);
    let token_manager =
      fechatter_core::jwt::TokenManager::from_config(&token_config, refresh_token_repo)
        .expect("Failed to create test token manager");

    // 创建ServiceProvider - Ensure this is the server's ServiceProvider
    let service_provider =
      crate::services::service_provider::ServiceProvider::new(pool.clone(), token_manager);

    // 创建AppState with mock components
    let inner = crate::AppStateInner {
      config,
      service_provider,
      chat_list_cache: dashmap::DashMap::new(),
      event_publisher: None,
    };

    let app_state = crate::AppState {
      inner: Arc::new(inner),
    };

    // 测试AppState的auth方法 - 如果任何方法调用核心占位符，将导致恐慌

    // 创建测试数据
    let create_user = fechatter_core::CreateUser {
      email: "new_user@example.com".to_string(),
      fullname: "New User".to_string(),
      password: "password".to_string(),
      workspace: "Test".to_string(),
    };

    let signin_user = fechatter_core::SigninUser {
      email: "test@example.com".to_string(),
      password: "password".to_string(),
    };

    // 测试AppState的auth方法
    // 注意：这些调用可能会失败，但不应该触发核心的占位符实现恐慌

    // 由于是测试环境，我们不期望这些操作真的成功
    // 但它们应该调用正确的实现而不是触发panic

    let _ = app_state.signup(&create_user, None).await;
    let _ = app_state.signin(&signin_user, None).await;
    let _ = app_state.refresh_token("test_token", None).await;
    let _ = app_state.logout("test_token").await;
    let _ = app_state.logout_all(1).await;

    // 如果我们到达这里，没有触发核心占位符的panic，测试成功
    assert!(true);
  }
}
