// This is a comprehensive test suite to ensure we never call the core placeholders
#[cfg(test)]
mod tests {
  use fechatter_core::{
    ActualAuthServiceProvider, CreateUser, LogoutService, RefreshTokenService, SigninService,
    SigninUser, SignupService, UserClaims, UserStatus,
  };
  use sqlx::PgPool;

  use crate::config::AppConfig;
  use crate::domains::auth::RefreshTokenRepositoryImpl;
  use crate::services::ServiceProvider;
  use crate::services::service_provider::ServerTokenService;
  use crate::{AppState, AuthService};
  use std::fs;
  use std::sync::Arc;

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

  // Helper to create a test auth service directly
  fn create_test_auth_service(
    pool: Arc<PgPool>,
    token_manager: Arc<fechatter_core::jwt::TokenManager>,
  ) -> AuthService {
    let user_repository = Box::new(crate::models::user::FechatterUserRepository::new(
      pool.clone(),
    ));

    let token_service: Box<dyn fechatter_core::TokenService + Send + Sync + 'static> =
      Box::new(ServerTokenService::new(token_manager));

    let refresh_token_repository = Box::new(RefreshTokenRepositoryImpl::new(pool));

    // Create a simple event publisher for testing
    let event_publisher = Arc::new(
      crate::services::infrastructure::flows::events::SimplifiedEventPublisher::new(Arc::new(
        crate::services::application::CacheStrategyService::new_simple(),
      )),
    );

    AuthService::new(
      user_repository,
      token_service,
      refresh_token_repository,
      event_publisher,
    )
  }

  // Helper to create minimal test data
  fn create_test_data() -> (CreateUser, SigninUser, UserClaims) {
    let create_user = CreateUser {
      email: "test_user@example.com".to_string(),
      fullname: "Test User".to_string(),
      password: "password123".to_string(),
      workspace: "TestWorkspace".to_string(),
    };

    let signin_user = SigninUser {
      email: create_user.email.clone(),
      password: create_user.password.clone(),
    };

    let user_claims = UserClaims {
      id: fechatter_core::UserId(1),
      workspace_id: fechatter_core::WorkspaceId(1),
      fullname: "Test User".to_string(),
      email: "test@example.com".to_string(),
      status: UserStatus::Active,
      created_at: chrono::Utc::now(),
    };

    (create_user, signin_user, user_claims)
  }

  #[tokio::test]
  async fn test_direct_auth_service_implementation() {
    // Load test config
    let config = AppConfig::load().expect("Failed to load config");
    let pool = PgPool::connect_lazy(&config.server.db_url).expect("Failed to create pool");
    let pool_arc = Arc::new(pool);

    // Get real PEM keys using the find_key_files function
    let (encoding_path, decoding_path) = find_key_files();

    // Read key files
    let encoding_key = fs::read_to_string(&encoding_path).expect("Failed to read encoding key");
    let decoding_key: String =
      fs::read_to_string(&decoding_path).expect("Failed to read decoding key");

    // Create a real JWT token config with actual files
    struct TestTokenConfig {
      encoding_key: String,
      decoding_key: String,
    }

    impl fechatter_core::jwt::TokenConfigProvider for TestTokenConfig {
      fn get_encoding_key_pem(&self) -> &str {
        &self.encoding_key
      }
      fn get_decoding_key_pem(&self) -> &str {
        &self.decoding_key
      }
    }

    let test_config = TestTokenConfig {
      encoding_key,
      decoding_key,
    };

    // Create a test token manager
    let refresh_token_repo = Arc::new(RefreshTokenRepositoryImpl::new(pool_arc.clone()));
    let token_manager = Arc::new(
      fechatter_core::jwt::TokenManager::from_config(&test_config, refresh_token_repo)
        .expect("Failed to create token manager"),
    );

    // Create the auth service directly
    let auth_service = create_test_auth_service(pool_arc.clone(), token_manager.clone());

    // We expect operations might fail on a real DB, but they should NEVER call the core placeholders
    // which would panic. We just care that the correct implementation is being used.
    let (create_user, signin_user, _) = create_test_data();

    // Try each method - even if they fail, they shouldn't panic with the placeholder message
    let _ = <AuthService as SignupService>::signup(&auth_service, &create_user, None).await;
    let _ = <AuthService as SigninService>::signin(&auth_service, &signin_user, None).await;
    let _ =
      <AuthService as RefreshTokenService>::refresh_token(&auth_service, "test_token", None).await;
    let _ = <AuthService as LogoutService>::logout(&auth_service, "test_token").await;
    let _ =
      <AuthService as LogoutService>::logout_all(&auth_service, fechatter_core::UserId(1)).await;

    // If we got here, we didn't hit the core placeholders
    assert!(true);
  }

  #[tokio::test]
  async fn test_server_provider_implementation() {
    // Load test config
    let config = AppConfig::load().expect("Failed to load config");
    let pool = PgPool::connect_lazy(&config.server.db_url).expect("Failed to create pool");

    // Get real PEM keys using the find_key_files function
    let (encoding_path, decoding_path) = find_key_files();

    // Read key files
    let encoding_key = fs::read_to_string(&encoding_path).expect("Failed to read encoding key");
    let decoding_key = fs::read_to_string(&decoding_path).expect("Failed to read decoding key");

    // Create a real JWT token config with actual files
    struct TestTokenConfig {
      encoding_key: String,
      decoding_key: String,
    }

    impl fechatter_core::jwt::TokenConfigProvider for TestTokenConfig {
      fn get_encoding_key_pem(&self) -> &str {
        &self.encoding_key
      }
      fn get_decoding_key_pem(&self) -> &str {
        &self.decoding_key
      }
    }

    let test_config = TestTokenConfig {
      encoding_key,
      decoding_key,
    };

    // Create a test token manager
    let refresh_token_repo = Arc::new(RefreshTokenRepositoryImpl::new(Arc::new(pool.clone())));
    let token_manager =
      fechatter_core::jwt::TokenManager::from_config(&test_config, refresh_token_repo)
        .expect("Failed to create token manager");

    // Create the ServiceProvider
    let provider = ServiceProvider::new(pool, token_manager);

    // Get the auth service through the provider
    let auth_service = <ServiceProvider as ActualAuthServiceProvider>::create_service(&provider);

    // We expect operations might fail on a real DB, but they should NEVER call the core placeholders
    let (create_user, signin_user, _) = create_test_data();

    // Try each method - even if they fail, they shouldn't panic with the placeholder message
    let _ = <AuthService as SignupService>::signup(&auth_service, &create_user, None).await;
    let _ = <AuthService as SigninService>::signin(&auth_service, &signin_user, None).await;
    let _ =
      <AuthService as RefreshTokenService>::refresh_token(&auth_service, "test_token", None).await;
    let _ = <AuthService as LogoutService>::logout(&auth_service, "test_token").await;
    let _ =
      <AuthService as LogoutService>::logout_all(&auth_service, fechatter_core::UserId(1)).await;

    // If we got here, we didn't hit the core placeholders
    assert!(true);
  }

  #[tokio::test]
  async fn test_app_state_implementation() {
    // Load test config
    let _config = AppConfig::load().expect("Failed to load config");

    // Create a test AppState
    let (_, app_state) = AppState::test_new()
      .await
      .expect("Failed to create test AppState");

    // Test methods on AppState
    let (create_user, signin_user, _) = create_test_data();

    // Try each method - even if they fail, they shouldn't panic with the placeholder message
    let _ = app_state.signup(&create_user, None).await;
    let _ = app_state.signin(&signin_user, None).await;
    let _ = app_state.refresh_token("test_token", None).await;
    let _ = app_state.logout("test_token").await;
    let _ = app_state.logout_all(fechatter_core::UserId(1)).await;

    // If we got here, we didn't hit the core placeholders
    assert!(
      true,
      "Should not panic with core placeholder implementations"
    );
  }

  // 测试全面的调用链路 - 模拟前端API调用
  #[tokio::test]
  async fn test_app_handler_to_service_chain() {
    // Create test app state
    let (_, app_state) = AppState::test_new()
      .await
      .expect("Failed to create test AppState");
    let state = app_state.clone();

    // Simulate handler call to signup
    let create_user = CreateUser {
      email: format!("test_user_{}@example.com", uuid::Uuid::new_v4()),
      fullname: "Test User".to_string(),
      password: "password123".to_string(),
      workspace: "TestWorkspace".to_string(),
    };

    // This exact chain of calls would happen in a real API request
    let result = state.signup(&create_user, None).await;

    // We don't care if it succeeds or fails as long as it doesn't panic
    if let Err(e) = &result {
      println!("Expected error in test (not a problem): {:?}", e);
    }

    // Test the signin flow
    let signin_user = SigninUser {
      email: create_user.email.clone(),
      password: create_user.password.clone(),
    };

    let result = state.signin(&signin_user, None).await;
    if let Err(e) = &result {
      println!("Expected error in test (not a problem): {:?}", e);
    }

    // If we got here without panicking, the server implementation is being used correctly
    assert!(true);
  }
}
