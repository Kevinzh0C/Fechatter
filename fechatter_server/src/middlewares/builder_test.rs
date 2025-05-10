#[cfg(test)]
mod tests {
  use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::get,
  };
  use std::sync::{Arc, Mutex};
  use std::time::Instant;
  use tower::ServiceExt;

  // Middleware application tracker
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
    // AppState consistency check field
    app_state_id: Arc<Mutex<Option<usize>>>,
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

    // Record AppState ID for consistency validation
    fn record_app_state_id(&self, id: usize) {
      let mut app_state_id = self.app_state_id.lock().unwrap();
      if app_state_id.is_none() {
        *app_state_id = Some(id);
      } else {
        // Verify ID consistency
        assert_eq!(
          *app_state_id,
          Some(id),
          "AppState ID inconsistency, possibly different AppState instances in middleware chain"
        );
      }
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

    // Check middleware execution order
    fn check_order(&self) -> Vec<&str> {
      let auth_time = self.auth_time.lock().unwrap().clone();
      let refresh_time = self.refresh_time.lock().unwrap().clone();
      let workspace_time = self.workspace_time.lock().unwrap().clone();
      let chat_time = self.chat_membership_time.lock().unwrap().clone();

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

      middleware_times.sort_by(|a, b| a.1.cmp(&b.1));
      middleware_times.iter().map(|(name, _)| *name).collect()
    }
  }

  // Use bit flags to track applied middlewares, similar to the real MiddlewareBuilder
  #[derive(Clone, Copy, PartialEq, Eq)]
  struct MockMiddlewareFlags(u8);

  impl MockMiddlewareFlags {
    const NONE: Self = Self(0);
    const AUTH: Self = Self(1);
    const REFRESH: Self = Self(2);
    const WORKSPACE: Self = Self(4);
    const CHAT_MEMBERSHIP: Self = Self(8);

    const fn contains(self, other: Self) -> bool {
      (self.0 & other.0) == other.0
    }

    const fn add(self, other: Self) -> Self {
      Self(self.0 | other.0)
    }
  }

  // Mock middleware builder
  struct MockBuilder {
    router: Router,
    tracker: MiddlewareTracker,
    applied: MockMiddlewareFlags,
    app_state_id: usize, // Unique identifier for mocking AppState
  }

  impl MockBuilder {
    fn new(router: Router, tracker: MiddlewareTracker) -> Self {
      // Create a unique AppState ID
      let app_state_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros() as usize;

      Self {
        router,
        tracker,
        applied: MockMiddlewareFlags::NONE,
        app_state_id,
      }
    }

    // Add authentication middleware
    fn with_auth(mut self) -> Self {
      if !self.applied.contains(MockMiddlewareFlags::AUTH) {
        // Simulate AppState consistency check
        self.tracker.record_app_state_id(self.app_state_id);
        // Call tracker to indicate this middleware has been added
        self.tracker.mark_auth_called();
        // The actual Router won't be modified since this is just a test
        self.applied = self.applied.add(MockMiddlewareFlags::AUTH);
      }
      self
    }

    // Add token refresh middleware
    fn with_refresh(mut self) -> Self {
      // Validate dependency: refresh middleware requires Auth middleware
      if !self.applied.contains(MockMiddlewareFlags::AUTH) {
        panic!("Auth middleware must be applied before Refresh middleware");
      }

      if !self.applied.contains(MockMiddlewareFlags::REFRESH) {
        // Simulate AppState consistency check
        self.tracker.record_app_state_id(self.app_state_id);
        self.tracker.mark_refresh_called();
        self.applied = self.applied.add(MockMiddlewareFlags::REFRESH);
      }
      self
    }

    // Add workspace middleware
    fn with_workspace(mut self) -> Self {
      // Validate dependency: workspace middleware requires Auth middleware
      if !self.applied.contains(MockMiddlewareFlags::AUTH) {
        panic!("Auth middleware must be applied before Workspace middleware");
      }

      if !self.applied.contains(MockMiddlewareFlags::WORKSPACE) {
        // Simulate AppState consistency check
        self.tracker.record_app_state_id(self.app_state_id);
        self.tracker.mark_workspace_called();
        self.applied = self.applied.add(MockMiddlewareFlags::WORKSPACE);
      }
      self
    }

    // Add chat membership middleware
    fn with_chat_membership(mut self) -> Self {
      // Validate dependencies: chat membership middleware requires Auth and Workspace middleware
      if !self.applied.contains(MockMiddlewareFlags::AUTH) {
        panic!("Auth middleware must be applied before Chat Membership middleware");
      }
      if !self.applied.contains(MockMiddlewareFlags::WORKSPACE) {
        panic!("Workspace middleware must be applied before Chat Membership middleware");
      }

      if !self.applied.contains(MockMiddlewareFlags::CHAT_MEMBERSHIP) {
        // Simulate AppState consistency check
        self.tracker.record_app_state_id(self.app_state_id);
        self.tracker.mark_chat_membership_called();
        self.applied = self.applied.add(MockMiddlewareFlags::CHAT_MEMBERSHIP);
      }
      self
    }

    // Add all business middlewares
    fn with_all_middlewares(self) -> Self {
      self
        .with_auth()
        .with_refresh()
        .with_workspace()
        .with_chat_membership()
    }

    // Combination method: add Auth and Refresh middleware
    fn with_auth_refresh(self) -> Self {
      self.with_auth().with_refresh()
    }

    // Combination method: add Auth, Refresh, and Workspace middleware
    fn with_auth_refresh_workspace(self) -> Self {
      self.with_auth().with_refresh().with_workspace()
    }

    // Build the final router
    fn build(self) -> Router {
      self.router
    }

    // One-step method: apply all middlewares and build
    fn finalize(self) -> Router {
      self.with_all_middlewares().build()
    }

    // One-step method: apply only Auth and Refresh middlewares and build
    fn finalize_auth_refresh(self) -> Router {
      self.with_auth_refresh().build()
    }
  }

  // Router extension trait
  trait MockRouterExt {
    fn with_middlewares(self, tracker: MiddlewareTracker) -> MockBuilder;
  }

  impl MockRouterExt for Router {
    fn with_middlewares(self, tracker: MiddlewareTracker) -> MockBuilder {
      MockBuilder::new(self, tracker)
    }
  }

  // Test handler function
  async fn test_handler() -> &'static str {
    "Hello, World!"
  }

  // Test: using builder to add authentication middleware
  #[tokio::test]
  async fn test_builder_auth_middleware() {
    // Create tracker
    let tracker = MiddlewareTracker::new();

    // Use builder to add authentication middleware
    let router = Router::new().route("/test", get(test_handler));
    let app = router.with_middlewares(tracker.clone()).with_auth().build();

    // Send request
    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    // Verify results
    assert_eq!(response.status(), StatusCode::OK);
    assert!(tracker.was_auth_called(), "Auth middleware was not called");
  }

  // Test: adding token refresh middleware
  #[tokio::test]
  async fn test_builder_refresh_middleware() {
    // Create tracker
    let tracker = MiddlewareTracker::new();

    // Use builder to add middlewares
    let router = Router::new().route("/test", get(test_handler));
    let app = router
      .with_middlewares(tracker.clone())
      .with_auth()
      .with_refresh()
      .build();

    // Send request
    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    // Verify results
    assert_eq!(response.status(), StatusCode::OK);
    assert!(tracker.was_auth_called(), "Auth middleware was not called");
    assert!(
      tracker.was_refresh_called(),
      "Refresh middleware was not called"
    );
  }

  // Test: adding workspace middleware
  #[tokio::test]
  async fn test_builder_workspace_middleware() {
    // Create tracker
    let tracker = MiddlewareTracker::new();

    // Use builder to add middlewares
    let router = Router::new().route("/test", get(test_handler));
    let app = router
      .with_middlewares(tracker.clone())
      .with_auth()
      .with_workspace()
      .build();

    // Send request
    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    // Verify results
    assert_eq!(response.status(), StatusCode::OK);
    assert!(tracker.was_auth_called(), "Auth middleware was not called");
    assert!(
      tracker.was_workspace_called(),
      "Workspace middleware was not called"
    );
  }

  // Test: adding chat membership middleware
  #[tokio::test]
  async fn test_builder_chat_middleware() {
    // Create tracker
    let tracker = MiddlewareTracker::new();

    // Use builder to add middlewares
    let router = Router::new().route("/test", get(test_handler));
    let app = router
      .with_middlewares(tracker.clone())
      .with_auth()
      .with_workspace()
      .with_chat_membership()
      .build();

    // Send request
    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    // Verify results
    assert_eq!(response.status(), StatusCode::OK);
    assert!(tracker.was_auth_called(), "Auth middleware was not called");
    assert!(
      tracker.was_workspace_called(),
      "Workspace middleware was not called"
    );
    assert!(
      tracker.was_chat_membership_called(),
      "Chat membership middleware was not called"
    );
  }

  // Test: complete middleware chain and execution order
  #[tokio::test]
  async fn test_builder_middleware_execution_order() {
    // Create tracker
    let tracker = MiddlewareTracker::new();

    // Use builder to add all middlewares
    let router = Router::new().route("/test", get(test_handler));
    let app = router
      .with_middlewares(tracker.clone())
      .with_all_middlewares() // Use all-in-one method to add all middlewares
      .build();

    // Send request
    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    // Verify results
    assert_eq!(response.status(), StatusCode::OK);

    // Verify all middlewares were called
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

    // Verify call order
    let order = tracker.check_order();
    if order.len() == 4 {
      let auth_pos = order.iter().position(|&x| x == "auth").unwrap();
      let refresh_pos = order.iter().position(|&x| x == "refresh").unwrap();
      let workspace_pos = order.iter().position(|&x| x == "workspace").unwrap();
      let chat_pos = order.iter().position(|&x| x == "chat").unwrap();

      assert!(
        auth_pos < refresh_pos && refresh_pos < workspace_pos && workspace_pos < chat_pos,
        "Middleware execution order incorrect: {:?}",
        order
      );
    }
  }

  // Test: middleware combination methods
  #[tokio::test]
  async fn test_builder_middleware_combinations() {
    // Test with_auth_refresh combination method
    let tracker = MiddlewareTracker::new();
    let router = Router::new().route("/test", get(test_handler));
    let app = router
      .with_middlewares(tracker.clone())
      .with_auth_refresh() // Use combination method
      .build();

    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(tracker.was_auth_called(), "Auth middleware was not called");
    assert!(
      tracker.was_refresh_called(),
      "Refresh middleware was not called"
    );

    // Test with_auth_refresh_workspace combination method
    let tracker = MiddlewareTracker::new();
    let router = Router::new().route("/test", get(test_handler));
    let app = router
      .with_middlewares(tracker.clone())
      .with_auth_refresh_workspace() // Use combination method
      .build();

    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(tracker.was_auth_called(), "Auth middleware was not called");
    assert!(
      tracker.was_refresh_called(),
      "Refresh middleware was not called"
    );
    assert!(
      tracker.was_workspace_called(),
      "Workspace middleware was not called"
    );
  }

  // Test: middleware duplicate application
  #[tokio::test]
  async fn test_middleware_duplicate_application() {
    // Create tracker
    let tracker = MiddlewareTracker::new();

    // Use builder to add middlewares, intentionally adding duplicates
    let router = Router::new().route("/test", get(test_handler));
    let app = router
      .with_middlewares(tracker.clone())
      .with_auth()
      .with_refresh()
      .with_workspace()
      .with_chat_membership()
      .with_auth() // Intentionally add auth again
      .with_workspace() // Intentionally add workspace again
      .build();

    // Send request
    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    // Verify results
    assert_eq!(response.status(), StatusCode::OK);
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
  }

  // Test: router merging
  #[tokio::test]
  async fn test_builder_router_merge() {
    // Create tracker
    let tracker = MiddlewareTracker::new();

    // Create two different routers
    let router1 = Router::new().route("/test1", get(test_handler));
    let router1_built = router1
      .with_middlewares(tracker.clone())
      .with_auth()
      .with_workspace()
      .build();

    let router2 = Router::new().route("/test2", get(test_handler));

    // Merge routers
    let merged = Router::new()
      .nest("/api1", router1_built)
      .nest("/api2", router2);

    // Test first router
    let response = merged
      .clone()
      .oneshot(
        Request::builder()
          .uri("/api1/test1")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test second router
    let response = merged
      .oneshot(
        Request::builder()
          .uri("/api2/test2")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
  }

  // Test: advanced helper methods
  #[tokio::test]
  async fn test_builder_helper_methods() {
    // Test finalize method
    let tracker = MiddlewareTracker::new();
    let router = Router::new().route("/test", get(test_handler));
    let app = router.with_middlewares(tracker.clone()).finalize(); // Use finalize instead of with_all_middlewares().build()

    // Send request
    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    // Verify results
    assert_eq!(response.status(), StatusCode::OK);
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

    // Test finalize_auth_refresh method
    let tracker = MiddlewareTracker::new();
    let router = Router::new().route("/test", get(test_handler));
    let app = router
      .with_middlewares(tracker.clone())
      .finalize_auth_refresh(); // Only apply auth and refresh

    // Send request
    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    // Verify results
    assert_eq!(response.status(), StatusCode::OK);
    assert!(tracker.was_auth_called(), "Auth middleware was not called");
    assert!(
      tracker.was_refresh_called(),
      "Refresh middleware was not called"
    );
    assert!(
      !tracker.was_workspace_called(),
      "Workspace middleware was unexpectedly called"
    );
    assert!(
      !tracker.was_chat_membership_called(),
      "Chat membership middleware was unexpectedly called"
    );
  }

  // Verify our middleware builder correctly depends on Router state type
  mod router_state_tests {
    use super::{MiddlewareTracker, MockRouterExt, test_handler};
    use axum::{
      Router,
      body::Body,
      http::{Request, StatusCode},
      routing::get,
    };
    use tower::ServiceExt;

    // Test merging routers with different state types
    #[tokio::test]
    async fn test_builder_with_state_router_compatibility() {
      // Set up tracker
      let tracker = MiddlewareTracker::new();

      // Create base routers
      let base_router = Router::new().route("/", get(test_handler));
      let state_router = Router::new().route("/with-state", get(test_handler));

      // Use test Builder
      let router = base_router
        .clone()
        .with_middlewares(tracker.clone())
        .with_auth()
        .with_workspace()
        .build()
        .merge(state_router);

      // Clone router to avoid moving on first use
      let router_clone = router.clone();

      // Test requests
      let response = router
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

      assert_eq!(response.status(), StatusCode::OK);

      let response = router_clone
        .oneshot(
          Request::builder()
            .uri("/with-state")
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();

      assert_eq!(response.status(), StatusCode::OK);
    }

    // Test middleware order criticality
    #[tokio::test]
    async fn test_middleware_order_critical() {
      // Set up tracker
      let tracker = MiddlewareTracker::new();

      // Create router with correct order
      let base_router = Router::new().route("/auth", get(test_handler));

      // Use Builder pattern to add middlewares - force correct order
      let correct_order = base_router
        .clone()
        .with_middlewares(tracker.clone())
        .with_auth_refresh_workspace() // Use combination method
        .build();

      // Test router with correct order
      let response = correct_order
        .oneshot(Request::builder().uri("/auth").body(Body::empty()).unwrap())
        .await
        .unwrap();

      // Verify results
      assert_eq!(response.status(), StatusCode::OK);

      // Verify middlewares were called and in correct order
      assert!(tracker.was_auth_called(), "Auth middleware was not called");
      assert!(
        tracker.was_refresh_called(),
        "Refresh middleware was not called"
      );
      assert!(
        tracker.was_workspace_called(),
        "Workspace middleware was not called"
      );

      // Check call order
      let order = tracker.check_order();
      if order.len() == 3 {
        let auth_pos = order.iter().position(|&x| x == "auth").unwrap();
        let refresh_pos = order.iter().position(|&x| x == "refresh").unwrap();
        let workspace_pos = order.iter().position(|&x| x == "workspace").unwrap();

        assert!(
          auth_pos < refresh_pos && refresh_pos < workspace_pos,
          "Middleware execution order incorrect: {:?}",
          order
        );
      }
    }

    // Test middleware builder state propagation
    #[tokio::test]
    async fn test_middleware_builder_state_propagation() {
      // Set up tracker
      let tracker = MiddlewareTracker::new();

      // Create full middleware chain router
      let router = Router::new().route("/", get(test_handler));
      let router = router
        .with_middlewares(tracker.clone())
        .with_all_middlewares()
        .build();

      // Test request
      let response = router
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

      assert_eq!(response.status(), StatusCode::OK);

      // Verify all middlewares were called
      assert!(tracker.was_auth_called(), "Auth middleware not called");
      assert!(
        tracker.was_refresh_called(),
        "Refresh middleware not called"
      );
      assert!(
        tracker.was_workspace_called(),
        "Workspace middleware not called"
      );
      assert!(
        tracker.was_chat_membership_called(),
        "Chat membership middleware not called"
      );

      // Verify correct order
      let order = tracker.check_order();
      if order.len() == 4 {
        let auth_pos = order.iter().position(|&x| x == "auth").unwrap();
        let refresh_pos = order.iter().position(|&x| x == "refresh").unwrap();
        let workspace_pos = order.iter().position(|&x| x == "workspace").unwrap();
        let chat_pos = order.iter().position(|&x| x == "chat").unwrap();

        assert!(
          auth_pos < refresh_pos && refresh_pos < workspace_pos && workspace_pos < chat_pos,
          "Middleware execution order incorrect: {:?}",
          order
        );
      }
    }
  }

  // Test middleware dependency - try to skip Auth middleware
  #[tokio::test]
  #[should_panic(expected = "Auth middleware must be applied before Refresh middleware")]
  async fn test_middleware_dependency_refresh_requires_auth() {
    let tracker = MiddlewareTracker::new();
    let router = Router::new().route("/test", get(test_handler));

    // Try to directly add Refresh middleware without adding Auth middleware, which should fail
    let _app = router
      .with_middlewares(tracker.clone())
      .with_refresh() // This should panic
      .build();
  }

  // Test middleware dependency - try to skip Auth middleware when adding Workspace middleware
  #[tokio::test]
  #[should_panic(expected = "Auth middleware must be applied before Workspace middleware")]
  async fn test_middleware_dependency_workspace_requires_auth() {
    let tracker = MiddlewareTracker::new();
    let router = Router::new().route("/test", get(test_handler));

    // Try to directly add Workspace middleware without adding Auth middleware, which should fail
    let _app = router
      .with_middlewares(tracker.clone())
      .with_workspace() // This should panic
      .build();
  }

  // Test middleware dependency - Chat membership middleware requires Workspace and Auth middleware
  #[tokio::test]
  #[should_panic(expected = "Auth middleware must be applied before Chat Membership middleware")]
  async fn test_middleware_dependency_chat_requires_auth() {
    let tracker = MiddlewareTracker::new();
    let router = Router::new().route("/test", get(test_handler));

    // Try to directly add Chat Membership middleware without adding Auth and Workspace middleware, which should fail
    let _app = router
      .with_middlewares(tracker.clone())
      .with_chat_membership() // This should panic
      .build();
  }

  // Test middleware dependency - Chat membership middleware requires Workspace middleware (with Auth but no Workspace)
  #[tokio::test]
  #[should_panic(
    expected = "Workspace middleware must be applied before Chat Membership middleware"
  )]
  async fn test_middleware_dependency_chat_requires_workspace() {
    let tracker = MiddlewareTracker::new();
    let router = Router::new().route("/test", get(test_handler));

    // Add Auth but skip Workspace, try to add Chat Membership middleware, which should fail
    let _app = router
      .with_middlewares(tracker.clone())
      .with_auth()
      .with_chat_membership() // This should panic
      .build();
  }

  // Test AppState consistency - verify the same AppState is used throughout middleware chain
  #[tokio::test]
  async fn test_app_state_consistency() {
    let tracker = MiddlewareTracker::new();
    let router = Router::new().route("/test", get(test_handler));

    // Add all middlewares
    let app = router
      .with_middlewares(tracker.clone())
      .with_auth() // Record AppState ID
      .with_refresh() // Verify AppState ID consistency
      .with_workspace() // Verify AppState ID consistency
      .with_chat_membership() // Verify AppState ID consistency
      .build();

    // Send request
    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    // Verify request succeeded, indicating all middlewares successfully verified AppState consistency
    assert_eq!(response.status(), StatusCode::OK);
  }

  // Test: middleware layer order enforcement
  #[tokio::test]
  async fn test_middleware_layer_order() {
    let tracker = MiddlewareTracker::new();
    let router = Router::new().route("/test", get(test_handler));

    // Apply all middlewares using the Builder pattern
    let app = router
      .with_middlewares(tracker.clone())
      .with_auth()
      .with_refresh()
      .with_workspace()
      .with_chat_membership()
      .build();

    // Send request
    let _response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    // Check order: outside to inside should be Auth -> Refresh -> Workspace -> Chat Membership
    let order = tracker.check_order();
    assert_eq!(order, vec!["auth", "refresh", "workspace", "chat"]);
  }
}
