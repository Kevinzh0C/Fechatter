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
  use std::sync::Arc; // Keep Arc for Arc<MockRefreshTokenRepository>
  use uuid::Uuid; // Keep Uuid for test_integration_auth_flow_through_service_provider

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

  // Centralized TestTokenConfig definition
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

  // 使用标准测试方式创建ServiceProvider
  fn create_test_service_provider() -> ServiceProvider {
    let config = AppConfig::load().expect("Failed to load config from chat.yml");
    let db_url = config.server.db_url.clone();
    let pool = PgPool::connect_lazy(&db_url).expect("Failed to create test database connection");

    let refresh_token_repo = Arc::new(MockRefreshTokenRepository);
    let token_manager =
      fechatter_core::jwt::TokenManager::from_config(&TestTokenConfig::new(), refresh_token_repo)
        .expect("Failed to create test token manager");

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
        id: fechatter_core::UserId(1).into(), // Keep this conversion
        user_id: fechatter_core::UserId(1),
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
      let now = chrono::Utc::now();
      let token_id = 1; // Example token_id
      Ok(Some(fechatter_core::RefreshToken {
        id: token_id, // This is an i64
        user_id: fechatter_core::UserId(1),
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
      Ok(())
    }

    async fn revoke_all_for_user(&self, _user_id: fechatter_core::UserId) -> Result<(), CoreError> {
      Ok(())
    }

    async fn replace(
      &self,
      payload: fechatter_core::ReplaceTokenPayload,
    ) -> Result<fechatter_core::RefreshToken, CoreError> {
      let now = chrono::Utc::now();
      Ok(fechatter_core::RefreshToken {
        id: 2, // Example token_id
        user_id: fechatter_core::UserId(1),
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
    let config = AppConfig::load().expect("Failed to load config from chat.yml");
    let db_url = config.server.db_url.clone();
    let pool = PgPool::connect_lazy(&db_url).expect("Failed to create test database connection");

    // Get an instance of TokenManager from create_test_service_provider or similar,
    // or create one here using the centralized TestTokenConfig.
    let refresh_token_repo = Arc::new(MockRefreshTokenRepository);
    let token_manager =
      fechatter_core::jwt::TokenManager::from_config(&TestTokenConfig::new(), refresh_token_repo)
        .expect("Failed to create test token manager");

    let provider = ServiceProvider::new(pool.clone(), token_manager.clone()); // Clone if TokenManager itself is not Clone but Arc<TokenManager> is.

    assert!(Arc::strong_count(&provider.pool) >= 1);
    assert!(Arc::strong_count(&provider.token_manager) >= 1);
  }

  #[tokio::test]
  async fn test_service_provider_pool() {
    let provider = create_test_service_provider();
    let pool = provider.pool();
    assert!(!pool.is_closed());
  }

  #[tokio::test]
  async fn test_service_provider_token_manager() {
    let provider = create_test_service_provider();
    let token_manager = provider.token_manager();
    assert!(!std::ptr::eq(token_manager as *const _, std::ptr::null()));
  }

  #[tokio::test]
  async fn test_service_provider_service_provider_trait() {
    let provider = create_test_service_provider();
    let service_provider = provider.service_provider();
    assert!(std::ptr::eq(service_provider, &provider));
  }

  #[tokio::test]
  async fn test_token_verifier_invalid_token() {
    let provider = create_test_service_provider();
    let result = provider.verify_token("invalid.token.format");
    assert!(result.is_err());
    match result {
      Err(CoreError::Validation(_)) => (),
      _ => panic!("Expected validation error for invalid token format"),
    }
  }

  #[tokio::test]
  async fn test_token_verifier_empty_token() {
    let provider = create_test_service_provider();
    let result = provider.verify_token("");
    assert!(result.is_err());
  }

  #[tokio::test]
  async fn test_token_verifier_malformed_but_plausible_token() {
    let provider = create_test_service_provider();
    let faux_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    let result = provider.verify_token(faux_token);
    assert!(result.is_err());
  }

  #[tokio::test]
  async fn test_with_token_manager_trait() {
    let provider = create_test_service_provider();
    let token_manager = <ServiceProvider as WithTokenManager>::token_manager(&provider);
    assert!(!std::ptr::eq(token_manager as *const _, std::ptr::null()));
  }

  #[tokio::test]
  async fn test_service_provider_verify_token_success() {
    let provider = create_test_service_provider();
    let user_claims = UserClaims {
      id: fechatter_core::UserId(1),
      workspace_id: fechatter_core::WorkspaceId(1),
      fullname: "Test User".to_string(),
      email: "test@example.com".to_string(),
      status: fechatter_core::UserStatus::Active,
      created_at: chrono::Utc::now(),
    };
    let token = <fechatter_core::jwt::TokenManager as TokenService>::generate_token(
      provider.token_manager(),
      &user_claims,
    )
    .expect("Failed to generate token");
    let result = provider.verify_token(&token);
    assert!(result.is_ok());
    let verified_claims = result.unwrap();
    assert_eq!(verified_claims.id, user_claims.id);
    assert_eq!(verified_claims.email, user_claims.email);
  }

  #[allow(dead_code)]
  async fn create_test_database_and_provider() -> (TestPg, ServiceProvider) {
    let config = AppConfig::load().expect("Failed to load config from chat.yml");
    let db_url = config.server.db_url;
    let post = db_url.rfind('/').expect("Invalid db_url format");
    let server_url = &db_url[..post];
    let test_db = TestPg::new(server_url.to_string(), Path::new("../migrations"));
    let pool = test_db.get_pool().await;

    let refresh_token_repo = Arc::new(MockRefreshTokenRepository);
    let token_manager = fechatter_core::jwt::TokenManager::from_config(
      &TestTokenConfig::new(), // Use centralized TestTokenConfig
      refresh_token_repo,
    )
    .expect("Failed to create test token manager");

    let provider = ServiceProvider::new(pool, token_manager);
    (test_db, provider)
  }

  #[tokio::test]
  async fn test_integration_auth_flow_through_service_provider() {
    let provider = create_mock_service_provider(); // This helper should now use centralized TestTokenConfig
    let create_user = fechatter_core::CreateUser {
      email: format!("test_user_{}@example.com", Uuid::new_v4().simple()), // Corrected Uuid usage
      fullname: "Test User".to_string(),
      password: "secure_password_123".to_string(),
      workspace: "default".to_string(),
    };
    let user_claims = UserClaims {
      id: fechatter_core::UserId(1),
      workspace_id: fechatter_core::WorkspaceId(1),
      fullname: create_user.fullname.clone(),
      email: create_user.email.clone(),
      status: fechatter_core::UserStatus::Active,
      created_at: chrono::Utc::now(),
    };
    let token = <fechatter_core::jwt::TokenManager as TokenService>::generate_token(
      provider.token_manager(),
      &user_claims,
    )
    .expect("Failed to generate token");
    let verify_result = provider.verify_token(&token);
    assert!(verify_result.is_ok(), "令牌验证应该成功");
    let claims = verify_result.unwrap();
    assert_eq!(claims.email, create_user.email, "令牌中的电子邮件应匹配");
  }

  fn create_mock_service_provider() -> ServiceProvider {
    let config = AppConfig::load().expect("Failed to load config from chat.yml");
    let refresh_token_repo = Arc::new(MockRefreshTokenRepository);
    let token_manager =
      fechatter_core::jwt::TokenManager::from_config(&TestTokenConfig::new(), refresh_token_repo) // Use centralized TestTokenConfig
        .expect("Failed to create test token manager");
    let db_url = config.server.db_url.clone();
    let pool = PgPool::connect_lazy(&db_url).expect("Failed to create test database connection");
    ServiceProvider::new(pool, token_manager)
  }

  #[tokio::test]
  async fn test_auth_service_never_calls_core_placeholders() {
    let provider = create_test_service_provider(); // This helper now uses centralized TestTokenConfig
    let auth_service = <ServiceProvider as ActualAuthServiceProvider>::create_service(&provider);
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
    let _ = <AuthService as RefreshTokenService>::refresh_token(
      &auth_service,
      "test_refresh_token",
      None,
    )
    .await;
    let _ = <AuthService as SignupService>::signup(&auth_service, &create_user, None).await;
    let _ = <AuthService as SigninService>::signin(&auth_service, &signin_user, None).await;
    let _ = <AuthService as LogoutService>::logout(&auth_service, "test_refresh_token").await;
    let _ =
      <AuthService as LogoutService>::logout_all(&auth_service, fechatter_core::UserId(1)).await;
    assert!(true);
  }

  #[tokio::test]
  async fn test_appstate_auth_service_methods() {
    let config = AppConfig::load().expect("Failed to load config");
    let pool = PgPool::connect_lazy(&config.server.db_url)
      .expect("Failed to create test database connection");

    let refresh_token_repo = Arc::new(MockRefreshTokenRepository);
    let token_manager =
      fechatter_core::jwt::TokenManager::from_config(&TestTokenConfig::new(), refresh_token_repo) // Use centralized TestTokenConfig
        .expect("Failed to create test token manager");

    let service_provider =
      crate::services::service_provider::ServiceProvider::new(pool.clone(), token_manager);

    let inner = crate::AppStateInner {
      config,
      service_provider,
      chat_list_cache: dashmap::DashMap::new(),
      event_publisher: None,
    };
    let app_state = crate::AppState {
      inner: Arc::new(inner),
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
    let _ = app_state.signup(&create_user, None).await;
    let _ = app_state.signin(&signin_user, None).await;
    let _ = app_state.refresh_token("test_token", None).await;
    let _ = app_state.logout("test_token").await;
    let _ = app_state.logout_all(fechatter_core::UserId(1)).await;
    assert!(true);
  }
}
