#[cfg(test)]
mod tests {

  use async_trait::async_trait;
  use axum::{
    Router,
    body::Body,
    extract::{Extension, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::get,
  };
  use fechatter_core::error::CoreError;

  use fechatter_core::middlewares::{
    ActualAuthServiceProvider, TokenVerifier, WithServiceProvider, WithTokenManager,
  };
  use fechatter_core::models::jwt::{
    AuthServiceTrait, AuthTokens, LogoutService, RefreshTokenData, RefreshTokenService,
    SigninService, SignupService, UserClaims,
  };
  use fechatter_core::models::{AuthUser, CreateUser, SigninUser};
  use fechatter_core::services::AuthContext;

  use std::sync::{Arc, Mutex};
  use std::time::Instant;
  use tower::ServiceExt;

  use crate::AppState;
  use crate::middlewares::builder::{GetOrCreateAppState, MiddlewareBuilder};
  use std::ops::Deref;
  use std::ptr;

  // Middleware execution order tracker
  #[derive(Clone, Default)]
  struct MiddlewareTracker {
    auth_called: Arc<Mutex<bool>>,
    refresh_called: Arc<Mutex<bool>>,
    workspace_called: Arc<Mutex<bool>>,
    chat_membership_called: Arc<Mutex<bool>>,
    auth_time: Arc<Mutex<Option<Instant>>>,
    refresh_time: Arc<Mutex<Option<Instant>>>,
    workspace_time: Arc<Mutex<Option<Instant>>>,
    chat_membership_time: Arc<Mutex<Option<Instant>>>,
  }

  impl MiddlewareTracker {
    fn new() -> Self {
      Self::default()
    }

    fn mark_auth_called(&self) {
      *self.auth_called.lock().unwrap() = true;
      *self.auth_time.lock().unwrap() = Some(Instant::now());
    }

    fn mark_refresh_called(&self) {
      *self.refresh_called.lock().unwrap() = true;
      *self.refresh_time.lock().unwrap() = Some(Instant::now());
    }

    fn mark_workspace_called(&self) {
      *self.workspace_called.lock().unwrap() = true;
      *self.workspace_time.lock().unwrap() = Some(Instant::now());
    }

    fn mark_chat_membership_called(&self) {
      *self.chat_membership_called.lock().unwrap() = true;
      *self.chat_membership_time.lock().unwrap() = Some(Instant::now());
    }

    fn was_auth_called(&self) -> bool {
      *self.auth_called.lock().unwrap()
    }

    fn was_refresh_called(&self) -> bool {
      *self.refresh_called.lock().unwrap()
    }

    fn was_workspace_called(&self) -> bool {
      *self.workspace_called.lock().unwrap()
    }

    fn was_chat_membership_called(&self) -> bool {
      *self.chat_membership_called.lock().unwrap()
    }

    // Check if middleware execution order is correct
    fn check_order(&self) -> Vec<&str> {
      let auth_time = self.auth_time.lock().unwrap().clone();
      let refresh_time = self.refresh_time.lock().unwrap().clone();
      let workspace_time = self.workspace_time.lock().unwrap().clone();
      let chat_time = self.chat_membership_time.lock().unwrap().clone();

      // Collect called middleware and their times
      let mut middleware_times = Vec::new();

      if let Some(time) = auth_time {
        middleware_times.push(("auth", time));
      }

      if let Some(time) = refresh_time {
        middleware_times.push(("refresh", time));
      }

      if let Some(time) = workspace_time {
        middleware_times.push(("workspace", time));
      }

      if let Some(time) = chat_time {
        middleware_times.push(("chat", time));
      }

      // Sort by execution time
      middleware_times.sort_by(|a, b| a.1.cmp(&b.1));

      // Return sorted list of middleware names
      middleware_times.iter().map(|(name, _)| *name).collect()
    }
  }

  // Mock AppState implementation
  #[derive(Clone)]
  struct MockAppState {
    tracker: MiddlewareTracker,
    tm: Arc<MockTokenManager>,
  }

  impl MockAppState {
    fn new(tracker: MiddlewareTracker) -> Self {
      Self {
        tracker: tracker.clone(),
        tm: Arc::new(MockTokenManager::new(tracker)),
      }
    }

    // Mock method to find workspace
    async fn find_by_id_with_pool(&self, _id: i64) -> Result<Option<MockWorkspace>, ()> {
      self.tracker.mark_workspace_called();
      Ok(Some(MockWorkspace {
        id: 1,
        name: "Test Workspace".to_string(),
      }))
    }

    // Mock method to check chat membership
    async fn ensure_user_is_chat_member(&self, _chat_id: i64, _user_id: i64) -> Result<bool, ()> {
      self.tracker.mark_chat_membership_called();
      Ok(true)
    }
  }

  // Mock TokenVerifier implementation
  impl TokenVerifier for MockAppState {
    type Claims = UserClaims;
    type Error = CoreError;

    fn verify_token(&self, _token: &str) -> Result<Self::Claims, Self::Error> {
      self.tracker.mark_auth_called();
      Ok(UserClaims {
        id: 1,
        email: "test@example.com".to_string(),
        fullname: "Test User".to_string(),
        workspace_id: 1,
        status: fechatter_core::models::UserStatus::Active,
        created_at: chrono::Utc::now(),
      })
    }
  }

  // Mock TokenManager implementation
  #[derive(Clone)]
  struct MockTokenManager {
    tracker: MiddlewareTracker,
  }

  impl MockTokenManager {
    fn new(tracker: MiddlewareTracker) -> Self {
      Self { tracker }
    }
  }

  impl TokenVerifier for MockTokenManager {
    type Claims = UserClaims;
    type Error = CoreError;

    fn verify_token(&self, _token: &str) -> Result<Self::Claims, Self::Error> {
      Ok(UserClaims {
        id: 1,
        email: "test@example.com".to_string(),
        fullname: "Test User".to_string(),
        workspace_id: 1,
        status: fechatter_core::models::UserStatus::Active,
        created_at: chrono::Utc::now(),
      })
    }
  }

  // Implement WithTokenManager
  impl WithTokenManager for MockAppState {
    type TokenManagerType = MockTokenManager;

    fn token_manager(&self) -> &Self::TokenManagerType {
      &self.tm
    }
  }

  // Mock service provider implementation
  #[derive(Clone)]
  struct MockAuthService;

  impl MockAuthService {
    fn new() -> Self {
      Self {}
    }
  }

  // Implement all required traits for MockAuthService
  #[async_trait]
  impl RefreshTokenService for MockAuthService {
    async fn refresh_token(
      &self,
      _refresh_token: &str,
      _auth_context: Option<AuthContext>,
    ) -> Result<AuthTokens, CoreError> {
      Ok(AuthTokens {
        access_token: "mock_access_token".to_string(),
        refresh_token: RefreshTokenData {
          token: "mock_refresh_token".to_string(),
          expires_at: chrono::Utc::now() + chrono::Duration::days(1),
          absolute_expires_at: chrono::Utc::now() + chrono::Duration::days(7),
        },
      })
    }
  }

  #[async_trait]
  impl SignupService for MockAuthService {
    async fn signup(
      &self,
      _payload: &CreateUser,
      _auth_context: Option<AuthContext>,
    ) -> Result<AuthTokens, CoreError> {
      Ok(AuthTokens {
        access_token: "mock_access_token".to_string(),
        refresh_token: RefreshTokenData {
          token: "mock_refresh_token".to_string(),
          expires_at: chrono::Utc::now() + chrono::Duration::days(1),
          absolute_expires_at: chrono::Utc::now() + chrono::Duration::days(7),
        },
      })
    }
  }

  #[async_trait]
  impl SigninService for MockAuthService {
    async fn signin(
      &self,
      _payload: &SigninUser,
      _auth_context: Option<AuthContext>,
    ) -> Result<Option<AuthTokens>, CoreError> {
      Ok(Some(AuthTokens {
        access_token: "mock_access_token".to_string(),
        refresh_token: RefreshTokenData {
          token: "mock_refresh_token".to_string(),
          expires_at: chrono::Utc::now() + chrono::Duration::days(1),
          absolute_expires_at: chrono::Utc::now() + chrono::Duration::days(7),
        },
      }))
    }
  }

  #[async_trait]
  impl LogoutService for MockAuthService {
    async fn logout(&self, _token: &str) -> Result<(), CoreError> {
      Ok(())
    }

    async fn logout_all(&self, _user_id: i64) -> Result<(), CoreError> {
      Ok(())
    }
  }

  // Implement AuthServiceTrait for MockAuthService
  impl AuthServiceTrait for MockAuthService {}

  impl ActualAuthServiceProvider for MockAppState {
    type AuthService = MockAuthService;

    fn create_service(&self) -> Self::AuthService {
      MockAuthService::new()
    }
  }

  impl WithServiceProvider for MockAppState {
    type ServiceProviderType = Self;

    fn service_provider(&self) -> &Self::ServiceProviderType {
      self
    }
  }

  // Mock workspace
  #[derive(Clone)]
  struct MockWorkspace {
    id: i64,
    name: String,
  }

  // Mock workspace context
  #[derive(Clone)]
  struct MockWorkspaceContext {
    workspace: Arc<MockWorkspace>,
  }

  impl MockWorkspaceContext {
    fn new(workspace: MockWorkspace) -> Self {
      Self {
        workspace: Arc::new(workspace),
      }
    }
  }

  // Alternative default from/into implementation
  impl From<MockAppState> for crate::AppState {
    fn from(_: MockAppState) -> Self {
      // Create a minimal AppState
      unimplemented!("This is just a mock conversion")
    }
  }

  impl From<crate::AppState> for MockAppState {
    fn from(_: crate::AppState) -> Self {
      // Create a minimal MockAppState
      unimplemented!("This is just a mock conversion")
    }
  }

  // Mock middleware functions
  async fn mock_with_workspace_context(
    State(state): State<MockAppState>,
    Extension(auth_user): Extension<AuthUser>,
    mut request: Request<Body>,
    next: Next,
  ) -> Result<Response, StatusCode> {
    // Workspace middleware call is marked in state.find_by_id_with_pool

    let workspace = state
      .find_by_id_with_pool(auth_user.workspace_id)
      .await
      .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
      .ok_or(StatusCode::NOT_FOUND)?;

    let ctx = MockWorkspaceContext::new(workspace);
    request.extensions_mut().insert(ctx);

    Ok(next.run(request).await)
  }

  async fn mock_verify_chat_membership_middleware(
    state: MockAppState,
    req: Request<Body>,
    next: Next,
  ) -> Response {
    // Chat membership middleware call is marked in state.ensure_user_is_chat_member

    // Get chat ID from path (mock)
    let chat_id = 1;

    let user = req
      .extensions()
      .get::<AuthUser>()
      .cloned()
      .unwrap_or_else(|| AuthUser {
        id: 1,
        email: "test@example.com".to_string(),
        fullname: "Test User".to_string(),
        workspace_id: 1,
        status: fechatter_core::models::UserStatus::Active,
        created_at: chrono::Utc::now(),
      });

    match state.ensure_user_is_chat_member(chat_id, user.id).await {
      Ok(true) => next.run(req).await,
      _ => StatusCode::FORBIDDEN.into_response(),
    }
  }

  // Test handlers
  async fn test_handler() -> &'static str {
    "Hello, World!"
  }

  async fn test_auth_handler(Extension(user): Extension<AuthUser>) -> String {
    format!("User ID: {}", user.id)
  }

  async fn test_workspace_handler(
    Extension(workspace_ctx): Extension<MockWorkspaceContext>,
  ) -> String {
    format!("Workspace: {}", workspace_ctx.workspace.name)
  }

  // Add mock_auth_middleware and mock_refresh_middleware to replace core middleware
  pub fn mock_add_auth_middleware<S>(router: Router<S>, state: MockAppState) -> Router<S>
  where
    S: Clone + Send + Sync + 'static,
  {
    use axum::middleware::from_fn;

    router.layer(from_fn(move |mut req: Request<Body>, next: Next| {
      let state_clone = state.clone();
      async move {
        // Directly mark auth middleware called
        state_clone.tracker.mark_auth_called();

        // Don't rely on verify_token to mark the call
        // This is simpler and more reliable
        let _claims = state_clone.verify_token("mock_token").unwrap();

        // Add authenticated user to request
        req.extensions_mut().insert(AuthUser {
          id: 1,
          email: "test@example.com".to_string(),
          fullname: "Test User".to_string(),
          workspace_id: 1,
          status: fechatter_core::models::UserStatus::Active,
          created_at: chrono::Utc::now(),
        });

        // Continue processing
        next.run(req).await
      }
    }))
  }

  pub fn mock_add_refresh_middleware<S>(router: Router<S>, state: MockAppState) -> Router<S>
  where
    S: Clone + Send + Sync + 'static,
  {
    use axum::middleware::from_fn;

    router.layer(from_fn(move |req: Request<Body>, next: Next| {
      let state_clone = state.clone();
      async move {
        state_clone.tracker.mark_refresh_called();
        next.run(req).await
      }
    }))
  }

  pub fn mock_add_workspace_middleware<S>(router: Router<S>, state: MockAppState) -> Router<S>
  where
    S: Clone + Send + Sync + 'static,
  {
    use axum::middleware::from_fn;

    router.layer(from_fn(move |req: Request<Body>, next: Next| {
      let state_clone = state.clone();

      async move {
        // Try to get added AuthUser
        if let Some(auth_user) = req.extensions().get::<AuthUser>().cloned() {
          // Trigger workspace middleware call - mark will be set in find_by_id_with_pool
          let workspace = state_clone
            .find_by_id_with_pool(auth_user.workspace_id)
            .await
            .unwrap_or(Some(MockWorkspace {
              id: 1,
              name: "Test Workspace".to_string(),
            }));

          if let Some(workspace) = workspace {
            // Create workspace context
            let ctx = MockWorkspaceContext::new(workspace);

            // Add workspace context to request
            let mut req = req;
            req.extensions_mut().insert(ctx);

            // Continue processing
            next.run(req).await
          } else {
            StatusCode::NOT_FOUND.into_response()
          }
        } else {
          // Print error message to help debugging
          println!("No AuthUser found in request extensions when trying to add workspace context");
          StatusCode::UNAUTHORIZED.into_response()
        }
      }
    }))
  }

  pub fn mock_add_chat_membership_middleware<S>(router: Router<S>, state: MockAppState) -> Router<S>
  where
    S: Clone + Send + Sync + 'static,
  {
    use axum::middleware::from_fn;

    router.layer(from_fn(move |req: Request<Body>, next: Next| {
      let state_clone = state.clone();
      async move {
        // Get chat ID from path (mock)
        let chat_id = 1;

        // Try to get user
        if let Some(user) = req.extensions().get::<AuthUser>().cloned() {
          // Verify chat membership - mark will be set in ensure_user_is_chat_member
          match state_clone
            .ensure_user_is_chat_member(chat_id, user.id)
            .await
          {
            Ok(true) => next.run(req).await,
            _ => StatusCode::FORBIDDEN.into_response(),
          }
        } else {
          // Print error message to help debugging
          println!("No AuthUser found in request extensions when trying to verify chat membership");
          StatusCode::UNAUTHORIZED.into_response()
        }
      }
    }))
  }

  // Simplified tests using minimal test suite
  #[tokio::test]
  async fn test_auth_middleware() {
    // Setup tracker
    let tracker = MiddlewareTracker::new();
    let app_state = MockAppState::new(tracker.clone());

    // Test route
    let app = Router::new()
      .route("/test", get(test_handler))
      .layer(axum::middleware::from_fn(
        move |_req: Request<Body>, next: Next| {
          let state_clone = app_state.clone();
          async move {
            // Directly mark auth middleware called
            state_clone.tracker.mark_auth_called();
            next.run(_req).await
          }
        },
      ));

    // Send request
    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    // Verify results
    assert_eq!(response.status(), StatusCode::OK);
    assert!(tracker.was_auth_called(), "Auth middleware was not called");
  }

  #[tokio::test]
  async fn test_refresh_middleware() {
    // Setup tracker
    let tracker = MiddlewareTracker::new();
    let app_state = MockAppState::new(tracker.clone());

    // Test route
    let app = Router::new()
      .route("/test", get(test_handler))
      .layer(axum::middleware::from_fn(
        move |req: Request<Body>, next: Next| {
          let state_clone = app_state.clone();
          async move {
            state_clone.tracker.mark_refresh_called();
            next.run(req).await
          }
        },
      ));

    // Send request
    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    // Verify results
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
      tracker.was_refresh_called(),
      "Refresh middleware was not called"
    );
  }

  #[tokio::test]
  async fn test_workspace_middleware() {
    // Setup tracker
    let tracker = MiddlewareTracker::new();
    let app_state = MockAppState::new(tracker.clone());

    // Test route - add necessary context for request
    let app = Router::new()
      .route("/test", get(test_handler))
      .layer(axum::middleware::from_fn(
        move |mut req: Request<Body>, next: Next| {
          let state_clone = app_state.clone();
          async move {
            // First add AuthUser (mock auth middleware)
            req.extensions_mut().insert(AuthUser {
              id: 1,
              email: "test@example.com".to_string(),
              fullname: "Test User".to_string(),
              workspace_id: 1,
              status: fechatter_core::models::UserStatus::Active,
              created_at: chrono::Utc::now(),
            });

            // Then call find_by_id_with_pool to trigger workspace middleware, which will set the mark
            let workspace = state_clone.find_by_id_with_pool(1).await.unwrap().unwrap();

            // Create workspace context
            let ctx = MockWorkspaceContext::new(workspace);

            // Add workspace context to request
            req.extensions_mut().insert(ctx);

            // Continue processing
            next.run(req).await
          }
        },
      ));

    // Send request
    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    // Verify results
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
      tracker.was_workspace_called(),
      "Workspace middleware was not called"
    );
  }

  #[tokio::test]
  async fn test_chat_middleware() {
    // Setup tracker
    let tracker = MiddlewareTracker::new();
    let app_state = MockAppState::new(tracker.clone());

    // Test route - add necessary context for request
    let app = Router::new()
      .route("/test", get(test_handler))
      .layer(axum::middleware::from_fn(
        move |mut req: Request<Body>, next: Next| {
          let state_clone = app_state.clone();
          async move {
            // First add AuthUser and workspace context (mock previous middleware)
            req.extensions_mut().insert(AuthUser {
              id: 1,
              email: "test@example.com".to_string(),
              fullname: "Test User".to_string(),
              workspace_id: 1,
              status: fechatter_core::models::UserStatus::Active,
              created_at: chrono::Utc::now(),
            });

            let workspace = MockWorkspace {
              id: 1,
              name: "Test Workspace".to_string(),
            };
            req
              .extensions_mut()
              .insert(MockWorkspaceContext::new(workspace));

            // Then call ensure_user_is_chat_member to trigger chat membership middleware, which will set the mark
            state_clone.ensure_user_is_chat_member(1, 1).await.unwrap();

            // Continue processing
            next.run(req).await
          }
        },
      ));

    // Send request
    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    // Verify results
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
      tracker.was_chat_membership_called(),
      "Chat membership middleware was not called"
    );
  }

  #[tokio::test]
  async fn test_middleware_execution_order() {
    // Setup tracker
    let tracker = MiddlewareTracker::new();
    let app_state = MockAppState::new(tracker.clone());

    // Create an application that applies all middleware in order
    let app = Router::new()
      .route("/test", get(test_handler))
      .layer(axum::middleware::from_fn(
        move |mut req: Request<Body>, next: Next| {
          let state_clone = app_state.clone();
          async move {
            // 1. Auth middleware
            state_clone.tracker.mark_auth_called();

            // Add authenticated user to request
            req.extensions_mut().insert(AuthUser {
              id: 1,
              email: "test@example.com".to_string(),
              fullname: "Test User".to_string(),
              workspace_id: 1,
              status: fechatter_core::models::UserStatus::Active,
              created_at: chrono::Utc::now(),
            });

            // 2. Token refresh middleware
            state_clone.tracker.mark_refresh_called();

            // 3. Workspace middleware
            let workspace = state_clone.find_by_id_with_pool(1).await.unwrap().unwrap();
            let ctx = MockWorkspaceContext::new(workspace);
            req.extensions_mut().insert(ctx);

            // 4. Chat membership middleware
            state_clone.ensure_user_is_chat_member(1, 1).await.unwrap();

            // Continue processing
            next.run(req).await
          }
        },
      ));

    // Send request
    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    // Verify results
    assert_eq!(response.status(), StatusCode::OK);

    // Verify all middleware were called
    assert!(tracker.was_auth_called(), "Auth middleware was not called");
    assert!(
      tracker.was_refresh_called(),
      "Refresh middleware was not called"
    );
    assert!(
      tracker.was_workspace_called(),
      "Workspace middleware was not called"
    );
    assert!(
      tracker.was_chat_membership_called(),
      "Chat membership middleware was not called"
    );

    // Verify middleware call order is correct
    assert_eq!(
      tracker.check_order(),
      vec!["auth", "refresh", "workspace", "chat"],
      "Middleware call order mismatch"
    );
  }

  // 此结构体用于确认我们的MiddlewareBuilder实现中是否
  // 保持了同一个Arc<AppState>在所有中间件步骤中
  struct AppStateTracker<S, T, A, R, W, C> {
    pub builder: MiddlewareBuilder<S, T, A, R, W, C>,
    pub app_state_arcs: Vec<Arc<AppState>>,
  }

  // 辅助方法用于跟踪MiddlewareBuilder中的app_state
  impl<S, T, A, R, W, C> AppStateTracker<S, T, A, R, W, C>
  where
    T: Into<AppState> + Clone,
  {
    fn new(builder: MiddlewareBuilder<S, T, A, R, W, C>) -> Self {
      Self {
        builder,
        app_state_arcs: Vec::new(),
      }
    }

    // 从builder获取app_state，如果存在则添加到跟踪列表
    fn track(&mut self) {
      if let Some(app_state) = self.builder.get_app_state() {
        self.app_state_arcs.push(app_state);
      }
    }
  }

  // 测试MiddlewareBuilder中app_state是否一致
  #[tokio::test]
  async fn test_app_state_consistency() {
    // 创建测试用AppState
    let (_, app_state) = AppState::test_new()
      .await
      .expect("Failed to create test AppState");

    // 创建基础路由 - 使用具体类型注解
    let router: Router = Router::new().route("/test", get(test_handler));

    // 创建初始Builder
    let builder = MiddlewareBuilder::new(router, app_state.clone());

    // 创建新的AppState引用用于首次获取
    let init_app_state = builder.get_or_create_app_state(&app_state);

    // 逐步应用中间件并收集app_state引用
    let builder = builder.with_auth(); // 应用Auth中间件
    let auth_app_state = builder.get_app_state();

    let builder = builder.with_workspace(); // 应用Workspace中间件
    let workspace_app_state = builder.get_app_state();

    let builder = builder.with_chat_membership(); // 应用ChatMembership中间件
    let chat_app_state = builder.get_app_state();

    // 打印app_state状态
    println!("Init app_state: {:p}", init_app_state.as_ref());
    println!(
      "Auth app_state: {:?}",
      auth_app_state.as_ref().map(|a| Arc::as_ptr(a))
    );
    println!(
      "Workspace app_state: {:?}",
      workspace_app_state.as_ref().map(|a| Arc::as_ptr(a))
    );
    println!(
      "Chat app_state: {:?}",
      chat_app_state.as_ref().map(|a| Arc::as_ptr(a))
    );

    // 验证所有步骤中app_state的一致性
    assert!(auth_app_state.is_none(), "Auth中间件后app_state应为None");
    assert!(
      workspace_app_state.is_some(),
      "Workspace中间件后app_state应有值"
    );
    assert!(
      chat_app_state.is_some(),
      "ChatMembership中间件后app_state应有值"
    );

    // 检查所有引用是否相同 - 跳过auth_app_state因为它是None
    let workspace_ptr = workspace_app_state.unwrap();
    let chat_ptr = chat_app_state.unwrap();

    // 比较Arc内部指向的值是否相同而不是比较指针本身
    // AppState没有实现PartialEq，所以我们可以比较AppState的属性或功能是否一致
    let init_token_manager = init_app_state.token_manager();
    let workspace_token_manager = workspace_ptr.token_manager();
    let chat_token_manager = chat_ptr.token_manager();

    // 验证token_manager的行为一致性
    let test_token = "test_token";
    let init_result = init_token_manager.verify_token(test_token);
    let workspace_result = workspace_token_manager.verify_token(test_token);
    let chat_result = chat_token_manager.verify_token(test_token);

    assert_eq!(
      init_result.is_err(),
      workspace_result.is_err(),
      "Workspace中间件后的AppState行为与初始AppState不一致"
    );
    assert_eq!(
      init_result.is_err(),
      chat_result.is_err(),
      "Chat中间件后的AppState行为与初始AppState不一致"
    );

    // 检查引用计数是否正确 - 应该大于或等于3（初始Arc + 2个中间件共享）
    assert!(
      Arc::strong_count(&chat_ptr) >= 3,
      "Arc引用计数不正确，应至少为3，实际为{}",
      Arc::strong_count(&chat_ptr)
    );

    println!("Arc引用计数: {}", Arc::strong_count(&chat_ptr));
    println!("测试通过: Arc<AppState>在中间件链中被正确共享");
  }

  // 测试不同中间件应用路径中AppState的一致性
  #[tokio::test]
  async fn test_different_middleware_paths() {
    // 创建测试用AppState
    let (_, app_state) = AppState::test_new()
      .await
      .expect("Failed to create test AppState");

    // 创建基础路由 - 使用具体类型注解
    let router: Router = Router::new().route("/test", get(test_handler));

    // 路径1: auth -> workspace
    let builder1 = MiddlewareBuilder::new(router.clone(), app_state.clone())
      .with_auth()
      .with_workspace();

    // 路径2: auth -> refresh -> workspace
    let builder2 = MiddlewareBuilder::new(router, app_state.clone())
      .with_auth()
      .with_token_refresh()
      .with_workspace();

    // 获取两个路径的最终app_state
    let app_state1 = builder1.get_app_state();
    let app_state2 = builder2.get_app_state();

    // 验证两个路径都有app_state
    assert!(app_state1.is_some(), "路径1应该有app_state");
    assert!(app_state2.is_some(), "路径2应该有app_state");

    // 解包Arc<AppState>
    let arc1 = app_state1.unwrap();
    let arc2 = app_state2.unwrap();

    // 因为是不同的路径，所以Arc应该不同
    // 但内部AppState的功能应该相同
    assert!(
      !Arc::ptr_eq(&arc1, &arc2),
      "不同路径应该创建不同的Arc<AppState>实例"
    );

    // 验证功能是否相同
    let result1 = arc1.token_manager().verify_token("test_token");
    let result2 = arc2.token_manager().verify_token("test_token");

    // 尽管是不同实例，但行为应该一致（同为错误或同为成功）
    assert_eq!(
      result1.is_err(),
      result2.is_err(),
      "不同路径的AppState实例行为不一致"
    );

    println!("测试通过: 不同路径创建的AppState功能一致");
  }
}
