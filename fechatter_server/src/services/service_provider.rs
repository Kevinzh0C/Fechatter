use crate::domains::auth::RefreshTokenAdaptor;
use crate::services::application::auth_app_service::AuthService;
use fechatter_core::error::CoreError;
use fechatter_core::{
  middlewares::{ActualAuthServiceProvider, TokenVerifier, WithServiceProvider, WithTokenManager},
  models::jwt::{TokenManager, TokenService},
};
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
    user_claims: &fechatter_core::models::jwt::UserClaims,
    user_agent: Option<String>,
    ip_address: Option<String>,
  ) -> Result<fechatter_core::AuthTokens, CoreError> {
    self
      .token_manager
      .generate_auth_tokens(user_claims, user_agent, ip_address)
      .await
  }

  fn verify_token(
    &self,
    token: &str,
  ) -> Result<fechatter_core::models::jwt::UserClaims, CoreError> {
    <TokenManager as TokenVerifier>::verify_token(&self.token_manager, token)
  }

  fn generate_token(
    &self,
    claims: &fechatter_core::models::jwt::UserClaims,
  ) -> Result<String, CoreError> {
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
  search_service: Option<Arc<crate::services::infrastructure::search::SearchService>>,
}

impl ServiceProvider {
  /// Creates a new service provider.
  ///
  /// # Arguments
  ///
  /// * `pool` - PostgreSQL connection pool
  /// * `token_manager` - JWT token manager
  ///
  /// # Examples
  ///
  /// ```
  /// use crate::services::ServiceProvider;
  /// // let provider = ServiceProvider::new(pool, token_manager);
  /// ```
  pub fn new(pool: PgPool, token_manager: TokenManager) -> Self {
    Self {
      pool: Arc::new(pool),
      token_manager: Arc::new(token_manager),
      search_service: None,
    }
  }

  /// Creates a new service provider with search service.
  ///
  /// # Arguments
  ///
  /// * `pool` - PostgreSQL connection pool
  /// * `token_manager` - JWT token manager
  /// * `search_service` - Optional search service
  pub fn new_with_search(
    pool: PgPool,
    token_manager: TokenManager,
    search_service: Option<crate::services::infrastructure::search::SearchService>,
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
  pub fn search_service(&self) -> Option<&crate::services::infrastructure::search::SearchService> {
    self.search_service.as_ref().map(|s| s.as_ref())
  }
}

impl TokenVerifier for ServiceProvider {
  type Claims = fechatter_core::models::jwt::UserClaims;
  type Error = CoreError;

  /// Verifies a JWT token and returns the user claims.
  fn verify_token(&self, token: &str) -> Result<Self::Claims, Self::Error> {
    <TokenManager as TokenVerifier>::verify_token(&self.token_manager, token)
  }
}

impl WithTokenManager for ServiceProvider {
  type TokenManagerType = TokenManager;

  /// Returns a reference to the token manager.
  fn token_manager(&self) -> &Self::TokenManagerType {
    &self.token_manager
  }
}

impl WithServiceProvider for ServiceProvider {
  type ServiceProviderType = Self;

  /// Returns a reference to itself as the service provider.
  fn service_provider(&self) -> &Self::ServiceProviderType {
    self
  }
}

impl ActualAuthServiceProvider for ServiceProvider {
  type AuthService = AuthService;

  /// Creates or returns a cached instance of the AuthService.
  ///
  /// This method ensures we only create a single AuthService instance
  /// throughout the application lifetime, which is stored in the AUTH_SERVICE
  /// static variable.
  fn create_service(&self) -> Self::AuthService {
    // Use static get_instance method to get singleton
    tracing::trace!("Getting AuthService instance");

    // Create components
    let user_repository = Arc::new(crate::domains::user::repository::UserRepositoryImpl::new(
      self.pool.clone(),
    ));

    let token_service: Box<dyn fechatter_core::TokenService + Send + Sync + 'static> =
      Box::new(ServerTokenService::new(self.token_manager.clone()));

    let refresh_token_repository: Box<
      dyn fechatter_core::models::jwt::RefreshTokenRepository + Send + Sync + 'static,
    > = Box::new(RefreshTokenAdaptor::new(self.pool.clone()));

    // Create ApplicationEventPublisher instance
    let event_publisher = Arc::new(
      crate::services::application::application_event_publisher::ApplicationEventPublisher::new(),
    );

    // Create new AuthService instance directly
    // Creates new instance each time but shares same Arc-wrapped components internally
    // Very low resource usage since only small structs are copied
    // All internal components are shared
    AuthService::new(
      user_repository,
      token_service,
      refresh_token_repository,
      event_publisher,
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use fechatter_core::middlewares::{ActualAuthServiceProvider, TokenVerifier};
  use fechatter_core::{AuthTokens, CreateUser, SigninUser, UserId, UserStatus, WorkspaceId};
  use std::fs;

  /// Searches for key files intelligently
  ///
  /// Search order:
  /// 1. Project root/fechatter_server/fixtures
  /// 2. Current directory/fixtures
  /// 3. Parent directory/fixtures
  ///
  /// Returns (encoding_path, decoding_path) tuple
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

  fn create_test_service_provider() -> ServiceProvider {
    let config = AppConfig::load().expect("Failed to load config");
    let pool = PgPool::connect_lazy(&config.server.db_url).expect("Failed to create pool");

    let test_config = TestTokenConfig::new();
    let mock_repo = Arc::new(MockRefreshTokenRepository);

    let token_manager =
      TokenManager::from_config(&test_config, mock_repo).expect("Failed to create token manager");

    ServiceProvider::new(pool, token_manager)
  }

  struct MockRefreshTokenRepository;

  #[async_trait::async_trait]
  impl fechatter_core::models::jwt::RefreshTokenRepository for MockRefreshTokenRepository {
    async fn create(
      &self,
      _payload: fechatter_core::models::jwt::StoreTokenPayload,
    ) -> Result<fechatter_core::models::jwt::RefreshToken, CoreError> {
      let now = chrono::Utc::now();
      Ok(fechatter_core::models::jwt::RefreshToken {
        id: 1, // Use i64 directly
        user_id: UserId(1),
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
    ) -> Result<Option<fechatter_core::models::jwt::RefreshToken>, CoreError> {
      let now = chrono::Utc::now();
      let token_id = 1; // Example token_id
      Ok(Some(fechatter_core::models::jwt::RefreshToken {
        id: token_id, // This is an i64
        user_id: UserId(1),
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

    async fn revoke_all_for_user(&self, _user_id: UserId) -> Result<(), CoreError> {
      Ok(())
    }

    async fn replace(
      &self,
      payload: fechatter_core::models::jwt::ReplaceTokenPayload,
    ) -> Result<fechatter_core::models::jwt::RefreshToken, CoreError> {
      let now = chrono::Utc::now();
      Ok(fechatter_core::models::jwt::RefreshToken {
        id: 2, // Example token_id
        user_id: UserId(1),
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
    // This test verifies that ServiceProvider can be created with proper dependencies
    // Real integration tests would require actual database connections

    let provider = create_test_service_provider();

    // Test that the service provider can access its components
    assert!(!provider.pool().is_closed());
    assert!(provider.search_service().is_none());
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
    let _token_manager = provider.token_manager();
    // Just verify that we can access the token manager without panics
  }

  #[tokio::test]
  async fn test_service_provider_service_provider_trait() {
    let provider = create_test_service_provider();
    let service_provider = provider.service_provider();
    assert!(!service_provider.pool().is_closed());
  }

  #[tokio::test]
  async fn test_token_verifier_invalid_token() {
    let provider = create_test_service_provider();
    let result = provider.verify_token("invalid_token");
    assert!(result.is_err());
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
    let result = provider.verify_token("Bearer.fake.token");
    assert!(result.is_err());
  }

  #[tokio::test]
  async fn test_with_token_manager_trait() {
    let provider = create_test_service_provider();
    let _token_manager = provider.token_manager();
    // Just verify that we can access the token manager without panics
  }

  #[tokio::test]
  async fn test_service_provider_verify_token_success() {
    let provider = create_test_service_provider();

    // Create test user claims
    let test_claims = fechatter_core::models::jwt::UserClaims {
      id: UserId(1),
      workspace_id: WorkspaceId(1),
      fullname: "Test User".to_string(),
      email: "test@example.com".to_string(),
      status: UserStatus::Active,
      created_at: chrono::Utc::now(),
    };

    // Generate a token using the token manager
    let token = provider
      .token_manager()
      .generate_token(&test_claims)
      .unwrap();

    // Verify the token
    let verified_claims = provider.verify_token(&token).unwrap();

    // Assert that the claims match
    assert_eq!(verified_claims.id, test_claims.id);
    assert_eq!(verified_claims.email, test_claims.email);
  }

  #[allow(dead_code)]
  async fn create_test_database_and_provider() -> (TestPg, ServiceProvider) {
    let tdb = TestPg::new(
      "postgresql://localhost".to_string(),
      Path::new("../fechatter_core/migrations"),
    );

    let pool = tdb.get_pool().await;
    let test_config = TestTokenConfig::new();
    let mock_repo = Arc::new(MockRefreshTokenRepository);

    let token_manager =
      TokenManager::from_config(&test_config, mock_repo).expect("Failed to create token manager");

    let provider = ServiceProvider::new(pool, token_manager);

    (tdb, provider)
  }

  #[tokio::test]
  async fn test_integration_auth_flow_through_service_provider() {
    let provider = create_test_service_provider();

    // Test user claims
    let test_claims = fechatter_core::models::jwt::UserClaims {
      id: UserId(1),
      workspace_id: WorkspaceId(1),
      fullname: "Integration Test User".to_string(),
      email: "integration@example.com".to_string(),
      status: UserStatus::Active,
      created_at: chrono::Utc::now(),
    };

    // Test token generation
    let token = provider
      .token_manager()
      .generate_token(&test_claims)
      .expect("Token generation should succeed");

    // Test token verification
    let verified_claims = provider
      .verify_token(&token)
      .expect("Token verification should succeed");

    assert_eq!(verified_claims.id, test_claims.id);
    assert_eq!(verified_claims.email, test_claims.email);
  }

  fn create_mock_service_provider() -> ServiceProvider {
    let config = AppConfig::load().expect("Failed to load config");
    let pool = PgPool::connect_lazy(&config.server.db_url).expect("Failed to create pool");

    let test_config = TestTokenConfig::new();
    let mock_repo = Arc::new(MockRefreshTokenRepository);

    let token_manager =
      TokenManager::from_config(&test_config, mock_repo).expect("Failed to create token manager");

    ServiceProvider::new(pool, token_manager)
  }

  #[tokio::test]
  async fn test_auth_service_never_calls_core_placeholders() {
    let provider = create_mock_service_provider();

    // This test ensures that when we call `create_service()`, we get a real
    // implementation and not the core layer placeholder (which would panic)
    let auth_service = provider.create_service();

    // If this succeeds without panic, we know we're not hitting the core placeholder
    let test_user = CreateUser {
      fullname: "Test User".to_string(),
      email: "test@example.com".to_string(),
      password: "test_password".to_string(),
      workspace: "default".to_string(),
    };

    // These may fail due to database or other issues, but they should NOT panic
    // with the "placeholder implementation" message
    // Use fully qualified syntax to call trait methods
    let _signup_result = <AuthService as fechatter_core::models::jwt::SignupService>::signup(
      &auth_service,
      &test_user,
      None,
    )
    .await;

    let signin_user = SigninUser {
      email: "test@example.com".to_string(),
      password: "test_password".to_string(),
    };
    let _signin_result = <AuthService as fechatter_core::models::jwt::SigninService>::signin(
      &auth_service,
      &signin_user,
      None,
    )
    .await;

    // If we reach here without panicking, the test passes
    assert!(true);
  }

  #[tokio::test]
  async fn test_appstate_auth_service_methods() {
    // This test ensures AppState-provided AuthService works correctly
    let provider = create_mock_service_provider();

    // Create auth service through the provider
    let auth_service = provider.create_service();

    // Test data
    let test_user = CreateUser {
      fullname: "AppState Test User".to_string(),
      email: "appstate@example.com".to_string(),
      password: "secure_password_123".to_string(),
      workspace: "default".to_string(),
    };

    // Test signup - might fail but shouldn't panic
    // Use fully qualified syntax to call trait methods
    let signup_result = <AuthService as fechatter_core::models::jwt::SignupService>::signup(
      &auth_service,
      &test_user,
      None,
    )
    .await;

    // We don't assert success because this is a mock environment
    // We just want to ensure no panics occur (i.e., we're not hitting core placeholders)
    match signup_result {
      Ok(tokens) => {
        // Great! The service worked
        assert!(!tokens.access_token.is_empty());
      }
      Err(e) => {
        // This is expected in a mock environment, as long as it's not a placeholder panic
        assert!(!e.to_string().contains("placeholder implementation"));
      }
    }
  }
}
