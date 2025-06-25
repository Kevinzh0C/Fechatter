use axum::{middleware::from_fn, Router};
use fechatter_core::SetLayer as _;
use std::sync::Arc;
use tower::ServiceBuilder;

// Locally define type state markers instead of importing
// Authentication state markers
pub struct WithoutAuth;
pub struct WithAuth;

// Token refresh state markers
pub struct WithoutRefresh;
pub struct WithRefresh;

// Server-specific workspace and chat membership state markers
pub struct WithoutWorkspace;
pub struct WithWorkspace;
pub struct WithoutChatMembership;
pub struct WithChatMembership;

// Import required middleware functions
use super::{chat::verify_chat_membership_middleware, workspace::with_workspace_context};
use crate::AppState;
use axum::body::Body;
use axum::extract::{Extension, Path, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use fechatter_core::jwt::TokenManager;
use fechatter_core::middlewares::custom_builder::{add_auth_middleware, add_refresh_middleware};
use fechatter_core::middlewares::server_time::ServerTimeLayer as _; // Import server time middleware
use fechatter_core::middlewares::{
  ActualAuthServiceProvider, TokenVerifier, WithServiceProvider, WithTokenManager,
};
use fechatter_core::models::jwt::UserClaims;
use fechatter_core::models::AuthUser;

/// Helper function to add workspace middleware to a router following the same pattern as core middleware functions
pub fn add_workspace_middleware<S>(router: Router<S>, state: AppState) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  // 🎯 EXTENSION-BASED: Use Extension pattern for AuthUser, but use the provided AppState parameter
  router.layer(from_fn(move |mut req: Request<Body>, next: Next| {
    let state_clone = state.clone();

    async move {
      // Use the provided AppState parameter instead of trying to get it from Extension
      let app_state = state_clone;

      // Get AuthUser from Extension (added by auth middleware)
      let auth_user = match req.extensions().get::<AuthUser>().cloned() {
        Some(user) => user,
        None => {
          tracing::error!("🚫 [WORKSPACE_MIDDLEWARE] AuthUser extension missing!");
          return StatusCode::UNAUTHORIZED.into_response();
        }
      };

      // Call the original workspace context function with the parameters
      with_workspace_context(State(app_state), Extension(auth_user), req, next).await
    }
  }))
}

/// Helper function to add chat membership middleware to a router following the same pattern as core middleware functions
pub fn add_chat_membership_middleware<S>(router: Router<S>, state: AppState) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  // 🎯 EXTENSION-BASED: Use Extension pattern for AuthUser, but use the provided AppState parameter
  router.layer(from_fn(move |req: Request<Body>, next: Next| {
    let state_clone = state.clone();

    async move {
      // Use the provided AppState parameter instead of trying to get it from Extension
      let app_state = state_clone;

      // Get AuthUser from Extension (added by auth middleware)
      let auth_user = match req.extensions().get::<AuthUser>().cloned() {
        Some(user) => user,
        None => {
          tracing::error!("🚫 [CHAT_MIDDLEWARE] AuthUser extension missing!");
          return StatusCode::UNAUTHORIZED.into_response();
        }
      };

      // Call the original chat membership middleware with the provided AppState
      verify_chat_membership_middleware(app_state, req, next).await
    }
  }))
}

// Use bit flags to represent applied middleware types
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MiddlewareFlags(u8);

impl MiddlewareFlags {
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

/// 🎯 EXTENSION-BASED: Enhanced MiddlewareBuilder that works with Extension pattern
///
/// This builder now internally uses Extension pattern while maintaining the same API
pub struct MiddlewareBuilder<S, T> {
  router: Router<S>,
  state: T,
  app_state: Option<Arc<AppState>>,
  applied: MiddlewareFlags, // Track applied middlewares
  use_extension: bool,      // Flag to indicate Extension-based mode
}

impl<S, T> MiddlewareBuilder<S, T>
where
  S: Clone + Send + Sync + 'static,
  T: TokenVerifier<Claims = UserClaims>
    + WithTokenManager<TokenManagerType = TokenManager>
    + WithServiceProvider
    + Into<AppState>
    + Clone
    + Send
    + Sync
    + 'static,
  <T as TokenVerifier>::Error: Send + 'static,
  <T as WithServiceProvider>::ServiceProviderType: ActualAuthServiceProvider,
  AuthUser: From<UserClaims>,
{
  /// Create a new middleware builder
  pub fn new(router: Router<S>, state: T) -> Self {
    Self {
      router,
      state,
      app_state: None,
      applied: MiddlewareFlags::NONE,
      use_extension: false,
    }
  }

  /// 🎯 Create a new middleware builder in Extension mode
  pub fn new_extension_based(router: Router<S>, state: T) -> Self {
    Self {
      router,
      state,
      app_state: None,
      applied: MiddlewareFlags::NONE,
      use_extension: true,
    }
  }

  /// Get or create AppState
  fn get_or_create_app_state(&mut self) -> Arc<AppState> {
    if let Some(app_state) = &self.app_state {
      app_state.clone()
    } else {
      let app_state = Arc::new(self.state.clone().into());
      self.app_state = Some(app_state.clone());
      app_state
    }
  }

  /// 🎯 Add AppState as Extension middleware (must be called first in Extension mode)
  pub fn with_state_extension(mut self) -> Self {
    if self.use_extension {
      let app_state = self.get_or_create_app_state();
      let state_for_middleware = (*app_state).clone();

      self.router = self
        .router
        .layer(from_fn(move |mut req: Request<Body>, next: Next| {
          let state = state_for_middleware.clone();
          async move {
            req.extensions_mut().insert(state);
            next.run(req).await
          }
        }));
    }
    self
  }

  /// Add authentication middleware
  pub fn with_auth(mut self) -> Self {
    if !self.applied.contains(MiddlewareFlags::AUTH) {
      if self.use_extension {
        // 🎯 Extension-based auth middleware
        let state = self.state.clone();
        self.router = self
          .router
          .layer(from_fn(move |mut req: Request<Body>, next: Next| {
            let state = state.clone();
            async move {
              // Get AppState from extensions (if available)
              let app_state = req.extensions().get::<AppState>().cloned();

              if let Some(auth_header) = req.headers().get("authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                  if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    // Use state's token manager to verify
                    match state.token_manager().verify_token(token) {
                      Ok(claims) => {
                        let auth_user: AuthUser = claims.into();
                        req.extensions_mut().insert(auth_user);
                        return Ok(next.run(req).await);
                      }
                      Err(e) => {
                        tracing::error!("Auth failed: {:?}", e);
                        return Err(StatusCode::UNAUTHORIZED.into_response());
                      }
                    }
                  }
                }
              }
              Err(StatusCode::UNAUTHORIZED.into_response())
            }
          }));
      } else {
        // Original State-based auth middleware
        self.router = add_auth_middleware(self.router, self.state.clone());
      }
      self.applied = self.applied.add(MiddlewareFlags::AUTH);
    }
    self
  }

  /// Add token refresh middleware
  pub fn with_refresh(mut self) -> Self {
    if !self.applied.contains(MiddlewareFlags::REFRESH) {
      if self.use_extension {
        // 🎯 Extension-based refresh middleware
        // For now, use the original refresh middleware since it's complex
        self.router = add_refresh_middleware(self.router, self.state.clone());
      } else {
        self.router = add_refresh_middleware(self.router, self.state.clone());
      }
      self.applied = self.applied.add(MiddlewareFlags::REFRESH);
    }
    self
  }

  /// Add workspace middleware
  pub fn with_workspace(mut self) -> Self {
    if !self.applied.contains(MiddlewareFlags::WORKSPACE) {
      let app_state = self.get_or_create_app_state();
      self.router = add_workspace_middleware(self.router, (*app_state).clone());
      self.applied = self.applied.add(MiddlewareFlags::WORKSPACE);
    }
    self
  }

  /// Add chat membership middleware
  pub fn with_chat_membership(mut self) -> Self {
    if !self.applied.contains(MiddlewareFlags::CHAT_MEMBERSHIP) {
      let app_state = self.get_or_create_app_state();
      self.router = add_chat_membership_middleware(self.router, (*app_state).clone());
      self.applied = self.applied.add(MiddlewareFlags::CHAT_MEMBERSHIP);
    }
    self
  }

  /// Add all business middlewares (Auth, Refresh, Workspace, Chat Membership)
  /// 🔧 CRITICAL FIX: Reverse order so auth middleware executes first and adds AuthUser extension
  /// Application order: chat_membership -> workspace -> refresh -> auth (auth applied last, executes first)
  pub fn with_all_middlewares(self) -> Self {
    tracing::debug!("🔧 [MIDDLEWARE_BUILDER] Applying middleware chain: chat -> workspace -> refresh -> auth");
    
    self
      .with_chat_membership()
      .with_workspace() 
      .with_refresh()
      .with_auth()
  }

  /// Apply Auth and Refresh middlewares
  pub fn with_auth_refresh(self) -> Self {
    self.with_refresh().with_auth()
  }

  /// Apply Auth, Refresh and Workspace middlewares
  pub fn with_auth_refresh_workspace(self) -> Self {
    self.with_workspace().with_refresh().with_auth()
  }

  /// Build the final router, apply infrastructure middlewares and return
  pub fn build(self) -> Router<S> {
    // Apply infrastructure middlewares (ServerTime, RequestId, Compression, Trace)
    // Note: set_layer() changes Router<S> to Router<()>, but we need to return Router<S>
    // Since we use Extension pattern, the state type doesn't matter for functionality
    use fechatter_core::middlewares::SetLayer;
    self.router.set_layer()
  }
}

// Provide RouterExt extension trait for chained calls
pub trait RouterExt<S>: Sized {
  fn with_middlewares<T>(self, state: T) -> MiddlewareBuilder<S, T>
  where
    S: Clone + Send + Sync + 'static,
    T: TokenVerifier<Claims = UserClaims>
      + WithTokenManager<TokenManagerType = TokenManager>
      + WithServiceProvider
      + Into<AppState>
      + Clone
      + Send
      + Sync
      + 'static,
    <T as TokenVerifier>::Error: Send + 'static,
    <T as WithServiceProvider>::ServiceProviderType: ActualAuthServiceProvider,
    AuthUser: From<UserClaims>;
}

impl<S> RouterExt<S> for Router<S> {
  fn with_middlewares<T>(self, state: T) -> MiddlewareBuilder<S, T>
  where
    S: Clone + Send + Sync + 'static,
    T: TokenVerifier<Claims = UserClaims>
      + WithTokenManager<TokenManagerType = TokenManager>
      + WithServiceProvider
      + Into<AppState>
      + Clone
      + Send
      + Sync
      + 'static,
    <T as TokenVerifier>::Error: Send + 'static,
    <T as WithServiceProvider>::ServiceProviderType: ActualAuthServiceProvider,
    AuthUser: From<UserClaims>,
  {
    MiddlewareBuilder::new(self, state)
  }
}

// 🎯 Extension trait for Extension-based builder - Works with stateless routers
pub trait RouterExtExtension: Sized {
  fn with_extension_middlewares<T>(self, state: T) -> StatelessMiddlewareBuilder<T>
  where
    T: TokenVerifier<Claims = UserClaims>
      + WithTokenManager<TokenManagerType = TokenManager>
      + WithServiceProvider
      + Into<AppState>
      + Clone
      + Send
      + Sync
      + 'static,
    <T as TokenVerifier>::Error: Send + 'static,
    <T as WithServiceProvider>::ServiceProviderType: ActualAuthServiceProvider,
    AuthUser: From<UserClaims>;
}

impl RouterExtExtension for Router {
  fn with_extension_middlewares<T>(self, state: T) -> StatelessMiddlewareBuilder<T>
  where
    T: TokenVerifier<Claims = UserClaims>
      + WithTokenManager<TokenManagerType = TokenManager>
      + WithServiceProvider
      + Into<AppState>
      + Clone
      + Send
      + Sync
      + 'static,
    <T as TokenVerifier>::Error: Send + 'static,
    <T as WithServiceProvider>::ServiceProviderType: ActualAuthServiceProvider,
    AuthUser: From<UserClaims>,
  {
    StatelessMiddlewareBuilder::new(self, state)
  }
}

/// Stateless middleware builder that always works with Router<()>
pub struct StatelessMiddlewareBuilder<T> {
  router: Router,
  state: T,
  app_state: Option<Arc<AppState>>,
  applied: MiddlewareFlags,
}

impl<T> StatelessMiddlewareBuilder<T>
where
  T: TokenVerifier<Claims = UserClaims>
    + WithTokenManager<TokenManagerType = TokenManager>
    + WithServiceProvider
    + Into<AppState>
    + Clone
    + Send
    + Sync
    + 'static,
  <T as TokenVerifier>::Error: Send + 'static,
  <T as WithServiceProvider>::ServiceProviderType: ActualAuthServiceProvider,
  AuthUser: From<UserClaims>,
{
  /// Create a new stateless middleware builder
  pub fn new(router: Router, state: T) -> Self {
    Self {
      router,
      state,
      app_state: None,
      applied: MiddlewareFlags::NONE,
    }
  }

  /// Get or create AppState
  fn get_or_create_app_state(&mut self) -> Arc<AppState> {
    if let Some(app_state) = &self.app_state {
      app_state.clone()
    } else {
      let app_state = Arc::new(self.state.clone().into());
      self.app_state = Some(app_state.clone());
      app_state
    }
  }

  /// Add AppState as Extension middleware
  pub fn with_state_extension(mut self) -> Self {
    let app_state = self.get_or_create_app_state();
    let state_for_middleware = (*app_state).clone();

    self.router = self
      .router
      .layer(from_fn(move |mut req: Request<Body>, next: Next| {
        let state = state_for_middleware.clone();
        async move {
          req.extensions_mut().insert(state);
          next.run(req).await
        }
      }));

    self
  }

  /// Add authentication middleware
  pub fn with_auth(mut self) -> Self {
    if !self.applied.contains(MiddlewareFlags::AUTH) {
      let state = self.state.clone();
      self.router = self
        .router
        .layer(from_fn(move |mut req: Request<Body>, next: Next| {
          let state = state.clone();
          async move {
            if let Some(auth_header) = req.headers().get("authorization") {
              if let Ok(auth_str) = auth_header.to_str() {
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                  match state.token_manager().verify_token(token) {
                    Ok(claims) => {
                      let auth_user: AuthUser = claims.into();
                      req.extensions_mut().insert(auth_user);
                      return next.run(req).await;
                    }
                    Err(e) => {
                      tracing::error!("Auth failed: {:?}", e);
                      return StatusCode::UNAUTHORIZED.into_response();
                    }
                  }
                }
              }
            }
            StatusCode::UNAUTHORIZED.into_response()
          }
        }));
      self.applied = self.applied.add(MiddlewareFlags::AUTH);
    }
    self
  }

  /// Add workspace middleware
  pub fn with_workspace(mut self) -> Self {
    if !self.applied.contains(MiddlewareFlags::WORKSPACE) {
      let app_state = self.get_or_create_app_state();
      self.router = add_workspace_middleware(self.router, (*app_state).clone());
      self.applied = self.applied.add(MiddlewareFlags::WORKSPACE);
    }
    self
  }

  /// Add chat membership middleware
  pub fn with_chat_membership(mut self) -> Self {
    if !self.applied.contains(MiddlewareFlags::CHAT_MEMBERSHIP) {
      let app_state = self.get_or_create_app_state();
      self.router = add_chat_membership_middleware(self.router, (*app_state).clone());
      self.applied = self.applied.add(MiddlewareFlags::CHAT_MEMBERSHIP);
    }
    self
  }

  /// Add all middlewares
  pub fn with_all_middlewares(self) -> Self {
    self.with_chat_membership().with_workspace().with_auth()
  }

  /// Finalize the stateless router
  pub fn finalize_extension_based(self) -> Router {
    use axum::middleware::from_fn;
    use fechatter_core::middlewares::{request_id_middleware, ServerTimeLayer};
    use tower::ServiceBuilder;
    use tower_http::{
      compression::CompressionLayer,
      trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
      LatencyUnit,
    };
    use tracing::Level;

    // The router is already Router (stateless), so we can directly apply layers
    self.router.layer(
      ServiceBuilder::new()
        .layer(
          TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(
              DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(LatencyUnit::Micros),
            ),
        )
        .layer(CompressionLayer::new().gzip(true))
        .layer(from_fn(request_id_middleware))
        .layer(ServerTimeLayer),
    )
  }
}

/// Factory function to create a pure stateless router with routes
/// This ensures Router<()> type from the beginning
pub fn create_stateless_router_with_routes<F>(route_builder: F) -> Router
where
  F: FnOnce(Router) -> Router,
{
  let base_router: Router = Router::new();
  route_builder(base_router)
}

/// Create Extension-based middleware builder with guaranteed stateless router
pub fn create_extension_middleware_builder<T>(
  router: Router,
  state: T,
) -> StatelessMiddlewareBuilder<T>
where
  T: TokenVerifier<Claims = UserClaims>
    + WithTokenManager<TokenManagerType = TokenManager>
    + WithServiceProvider
    + Into<AppState>
    + Clone
    + Send
    + Sync
    + 'static,
  <T as TokenVerifier>::Error: Send + 'static,
  <T as WithServiceProvider>::ServiceProviderType: ActualAuthServiceProvider,
  AuthUser: From<UserClaims>,
{
  StatelessMiddlewareBuilder::new(router, state)
}
