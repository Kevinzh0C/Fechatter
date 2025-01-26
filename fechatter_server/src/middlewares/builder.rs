use axum::{Router, middleware::from_fn};
use fechatter_core::SetLayer as _;
use std::sync::Arc;

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
use axum::extract::{Extension, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use fechatter_core::jwt::TokenManager;
use fechatter_core::middlewares::custom_builder::{add_auth_middleware, add_refresh_middleware};
use fechatter_core::middlewares::{
  ActualAuthServiceProvider, TokenVerifier, WithServiceProvider, WithTokenManager,
};
use fechatter_core::models::AuthUser;
use fechatter_core::models::jwt::UserClaims;

/// Helper function to add workspace middleware to a router following the same pattern as core middleware functions
pub fn add_workspace_middleware<S>(router: Router<S>, state: AppState) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  // The from_fn middleware expects a function that returns Future<Output = Response>
  router.layer(from_fn(move |req: Request<Body>, next: Next| {
    let state_clone = state.clone();

    // We must extract Extension<AuthUser> from the request directly,
    // as it's added by the auth middleware
    async move {
      let extension = req.extensions().get::<AuthUser>().cloned();

      if let Some(auth_user) = extension {
        // Pass auth_user as Extension
        with_workspace_context(State(state_clone), Extension(auth_user), req, next).await
      } else {
        // Auth user extension is missing, return 401
        // Return a Response directly
        StatusCode::UNAUTHORIZED.into_response()
      }
    }
  }))
}

/// Helper function to add chat membership middleware to a router following the same pattern as core middleware functions
pub fn add_chat_membership_middleware<S>(router: Router<S>, state: AppState) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  // The from_fn middleware expects a function that returns Future<Output = Response>
  router.layer(from_fn(move |req: Request<Body>, next: Next| {
    let state_clone = state.clone();

    // Pass the AppState directly, not wrapped in State
    async move {
      // First, verify that user is authenticated (safety check)
      let extension = req.extensions().get::<AuthUser>().cloned();

      if let Some(_auth_user) = extension {
        // AuthUser exists, proceed with chat membership check
        verify_chat_membership_middleware(state_clone, req, next).await
      } else {
        // Auth user extension is missing, return 401
        StatusCode::UNAUTHORIZED.into_response()
      }
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

/// A more concise and flexible middleware builder
/// Uses bit flags instead of type parameters to track middleware state
pub struct MiddlewareBuilder<S, T> {
  router: Router<S>,
  state: T,
  app_state: Option<Arc<AppState>>,
  applied: MiddlewareFlags, // Track applied middlewares
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

  /// Add authentication middleware
  pub fn with_auth(mut self) -> Self {
    if !self.applied.contains(MiddlewareFlags::AUTH) {
      self.router = add_auth_middleware(self.router, self.state.clone());
      self.applied = self.applied.add(MiddlewareFlags::AUTH);
    }
    self
  }

  /// Add token refresh middleware
  pub fn with_refresh(mut self) -> Self {
    if !self.applied.contains(MiddlewareFlags::REFRESH) {
      self.router = add_refresh_middleware(self.router, self.state.clone());
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
  /// Order will automatically be set to: Auth -> Refresh -> Workspace -> ChatMembership
  pub fn with_all_middlewares(self) -> Self {
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

// To maintain compatibility with the old API, keep these methods
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
  /// Similar to finalize, but doesn't add any middlewares
  /// Only adds infrastructure middlewares (ServerTime, RequestId, Compression, Trace)
  pub fn finalize_base(self) -> Router<S> {
    self.build()
  }

  /// Add authentication middleware and infrastructure middlewares
  pub fn finalize_auth_only(self) -> Router<S> {
    self.with_auth().build()
  }

  /// Add authentication, refresh middleware and infrastructure middlewares
  pub fn finalize_auth_refresh(self) -> Router<S> {
    self.with_auth_refresh().build()
  }

  /// Add authentication, refresh, workspace middleware and infrastructure middlewares
  pub fn finalize_auth_refresh_workspace(self) -> Router<S> {
    self.with_auth_refresh_workspace().build()
  }

  /// Add all middlewares: authentication, refresh, workspace, chat membership and infrastructure middlewares
  pub fn finalize(self) -> Router<S> {
    self.with_all_middlewares().build()
  }
}
