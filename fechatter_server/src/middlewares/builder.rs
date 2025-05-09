use axum::{Router, middleware::from_fn};
use std::marker::PhantomData;
use std::sync::Arc;

// 本地定义类型状态标记，而不是导入
// 认证状态标记
pub struct WithoutAuth;
pub struct WithAuth;

// 令牌刷新状态标记
pub struct WithoutRefresh;
pub struct WithRefresh;

// 服务器特定的工作区和聊天成员资格状态标记
pub struct WithoutWorkspace;
pub struct WithWorkspace;
pub struct WithoutChatMembership;
pub struct WithChatMembership;

// 导入所需的中间件函数
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

/// Middleware builder for the server, handling Auth, Refresh, Workspace, and ChatMembership states.
pub struct MiddlewareBuilder<
  S,
  T,
  AuthState = WithoutAuth,                     // Uses core's auth states
  RefreshState = WithoutRefresh,               // Uses core's refresh states
  WorkspaceState = WithoutWorkspace,           // Server's own workspace states
  ChatMembershipState = WithoutChatMembership, // Server's own chat membership states
> {
  router: Router<S>,
  state: T,
  #[cfg(test)]
  pub app_state: Option<Arc<AppState>>, // 测试模式下公开字段
  #[cfg(not(test))]
  app_state: Option<Arc<AppState>>, // 正常模式下保持私有
  _auth_marker: PhantomData<AuthState>,
  _refresh_marker: PhantomData<RefreshState>,
  _workspace_marker: PhantomData<WorkspaceState>,
  _chat_membership_marker: PhantomData<ChatMembershipState>,
}

// 辅助特性，用于获取Arc<AppState>
pub trait GetOrCreateAppState<T: Into<AppState>> {
  fn get_or_create_app_state(&self, state: &T) -> Arc<AppState>;
}

// 为所有MiddlewareBuilder实现统一的获取AppState方法
impl<S, T, A, R, W, C> GetOrCreateAppState<T> for MiddlewareBuilder<S, T, A, R, W, C>
where
  T: Into<AppState> + Clone,
{
  fn get_or_create_app_state(&self, state: &T) -> Arc<AppState> {
    if let Some(app_state) = &self.app_state {
      // 如果已有app_state，直接返回克隆（只增加引用计数）
      app_state.clone()
    } else {
      // 首次调用时创建
      Arc::new(state.clone().into())
    }
  }
}

// Initial state: No middleware applied
impl<S, T>
  MiddlewareBuilder<S, T, WithoutAuth, WithoutRefresh, WithoutWorkspace, WithoutChatMembership>
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
  T: Into<AppState>,
{
  pub fn new(router: Router<S>, state: T) -> Self {
    Self {
      router,
      state,
      app_state: None, // 初始化为None，延迟加载
      _auth_marker: PhantomData,
      _refresh_marker: PhantomData,
      _workspace_marker: PhantomData,
      _chat_membership_marker: PhantomData,
    }
  }

  pub fn with_auth(
    self,
  ) -> MiddlewareBuilder<S, T, WithAuth, WithoutRefresh, WithoutWorkspace, WithoutChatMembership>
  {
    // Use the core library's function to add auth middleware
    let router = add_auth_middleware(self.router, self.state.clone());

    MiddlewareBuilder {
      router,
      state: self.state,
      app_state: self.app_state, // 保持app_state不变
      _auth_marker: PhantomData,
      _refresh_marker: PhantomData,
      _workspace_marker: PhantomData,
      _chat_membership_marker: PhantomData,
    }
  }

  // NOTE: with_token_refresh is intentionally not provided here
  // to ensure that authentication is applied before token refresh,
  // as the core refresh middleware might depend on AuthUser extensions

  pub fn build(self) -> Router<S> {
    self.router
  }
}

// State: Auth applied, No Refresh, No Workspace, No ChatMembership
impl<S, T>
  MiddlewareBuilder<S, T, WithAuth, WithoutRefresh, WithoutWorkspace, WithoutChatMembership>
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
  T: Into<AppState>,
{
  pub fn with_token_refresh(
    self,
  ) -> MiddlewareBuilder<S, T, WithAuth, WithRefresh, WithoutWorkspace, WithoutChatMembership> {
    // Use the core library's function to add refresh middleware
    let router = add_refresh_middleware(self.router, self.state.clone());

    MiddlewareBuilder {
      router,
      state: self.state,
      app_state: self.app_state, // 保持app_state不变
      _auth_marker: PhantomData,
      _refresh_marker: PhantomData,
      _workspace_marker: PhantomData,
      _chat_membership_marker: PhantomData,
    }
  }

  pub fn with_workspace(
    self,
  ) -> MiddlewareBuilder<S, T, WithAuth, WithoutRefresh, WithWorkspace, WithoutChatMembership> {
    // 获取或创建Arc<AppState>
    let app_state_arc = self.get_or_create_app_state(&self.state);

    // 克隆Arc内部的AppState并传给中间件函数
    let app_state_clone = (*app_state_arc).clone();
    let router = add_workspace_middleware(self.router, app_state_clone);

    MiddlewareBuilder {
      router,
      state: self.state,
      app_state: Some(app_state_arc), // 保存Arc引用
      _auth_marker: PhantomData,
      _refresh_marker: PhantomData,
      _workspace_marker: PhantomData,
      _chat_membership_marker: PhantomData,
    }
  }

  pub fn build(self) -> Router<S> {
    self.router
  }
}

// NOTE: The WithoutAuth+WithRefresh state is intentionally NOT implemented
// because we now require Auth before Refresh to ensure type safety

// State: Auth, Refresh applied, No Workspace, No ChatMembership
impl<S, T> MiddlewareBuilder<S, T, WithAuth, WithRefresh, WithoutWorkspace, WithoutChatMembership>
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
  T: Into<AppState>,
{
  pub fn with_workspace(
    self,
  ) -> MiddlewareBuilder<S, T, WithAuth, WithRefresh, WithWorkspace, WithoutChatMembership> {
    // 获取或创建Arc<AppState>
    let app_state_arc = self.get_or_create_app_state(&self.state);

    // 克隆Arc内部的AppState并传给中间件函数
    let app_state_clone = (*app_state_arc).clone();
    let router = add_workspace_middleware(self.router, app_state_clone);

    MiddlewareBuilder {
      router,
      state: self.state,
      app_state: Some(app_state_arc), // 保存Arc引用
      _auth_marker: PhantomData,
      _refresh_marker: PhantomData,
      _workspace_marker: PhantomData,
      _chat_membership_marker: PhantomData,
    }
  }

  pub fn build(self) -> Router<S> {
    self.router
  }
}

// State: Auth, No Refresh, Workspace applied, No ChatMembership
impl<S, T> MiddlewareBuilder<S, T, WithAuth, WithoutRefresh, WithWorkspace, WithoutChatMembership>
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
  T: Into<AppState>,
{
  pub fn with_token_refresh(
    self,
  ) -> MiddlewareBuilder<S, T, WithAuth, WithRefresh, WithWorkspace, WithoutChatMembership> {
    // Use the core library's function to add refresh middleware
    let router = add_refresh_middleware(self.router, self.state.clone());

    MiddlewareBuilder {
      router,
      state: self.state,
      app_state: None,
      _auth_marker: PhantomData,
      _refresh_marker: PhantomData,
      _workspace_marker: PhantomData,
      _chat_membership_marker: PhantomData,
    }
  }

  pub fn with_chat_membership(
    self,
  ) -> MiddlewareBuilder<S, T, WithAuth, WithoutRefresh, WithWorkspace, WithChatMembership> {
    // 获取或创建Arc<AppState>
    let app_state_arc = self.get_or_create_app_state(&self.state);

    // 克隆Arc内部的AppState并传给中间件函数
    let app_state_clone = (*app_state_arc).clone();
    let router = add_chat_membership_middleware(self.router, app_state_clone);

    MiddlewareBuilder {
      router,
      state: self.state,
      app_state: Some(app_state_arc), // 保存Arc引用
      _auth_marker: PhantomData,
      _refresh_marker: PhantomData,
      _workspace_marker: PhantomData,
      _chat_membership_marker: PhantomData,
    }
  }

  pub fn build(self) -> Router<S> {
    self.router
  }
}

// State: Auth, Refresh, Workspace applied, No ChatMembership
impl<S, T> MiddlewareBuilder<S, T, WithAuth, WithRefresh, WithWorkspace, WithoutChatMembership>
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
  T: Into<AppState>,
{
  pub fn with_chat_membership(
    self,
  ) -> MiddlewareBuilder<S, T, WithAuth, WithRefresh, WithWorkspace, WithChatMembership> {
    // 获取或创建Arc<AppState>
    let app_state_arc = self.get_or_create_app_state(&self.state);

    // 克隆Arc内部的AppState并传给中间件函数
    let app_state_clone = (*app_state_arc).clone();
    let router = add_chat_membership_middleware(self.router, app_state_clone);

    MiddlewareBuilder {
      router,
      state: self.state,
      app_state: Some(app_state_arc), // 保存Arc引用
      _auth_marker: PhantomData,
      _refresh_marker: PhantomData,
      _workspace_marker: PhantomData,
      _chat_membership_marker: PhantomData,
    }
  }

  pub fn build(self) -> Router<S> {
    self.router
  }
}

// State: Auth, No Refresh, Workspace, ChatMembership applied (Final state for this path)
impl<S, T> MiddlewareBuilder<S, T, WithAuth, WithoutRefresh, WithWorkspace, WithChatMembership>
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
  T: Into<AppState>,
{
  pub fn with_token_refresh(
    self,
  ) -> MiddlewareBuilder<S, T, WithAuth, WithRefresh, WithWorkspace, WithChatMembership> {
    // Use the core library's function to add refresh middleware
    let router = add_refresh_middleware(self.router, self.state.clone());

    MiddlewareBuilder {
      router,
      state: self.state,
      app_state: None,
      _auth_marker: PhantomData,
      _refresh_marker: PhantomData,
      _workspace_marker: PhantomData,
      _chat_membership_marker: PhantomData,
    }
  }

  pub fn build(self) -> Router<S> {
    self.router
  }
}

// Final builder state with all possible middleware applied
impl<S, T> MiddlewareBuilder<S, T, WithAuth, WithRefresh, WithWorkspace, WithChatMembership>
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
  T: Into<AppState>,
{
  pub fn build(self) -> Router<S> {
    self.router
  }
}

// RouterExt trait for the server's MiddlewareBuilder
pub trait RouterExt<S, T> {
  fn with_middlewares(
    self,
    state: T,
  ) -> MiddlewareBuilder<S, T, WithoutAuth, WithoutRefresh, WithoutWorkspace, WithoutChatMembership>;
}

impl<S, T> RouterExt<S, T> for Router<S>
where
  // Base router constraints
  S: Clone + Send + Sync + 'static,
  // Token and service provider constraints
  T: TokenVerifier<Claims = UserClaims>
    + WithTokenManager<TokenManagerType = TokenManager>
    + WithServiceProvider
    + Into<AppState>
    + Clone
    + Send
    + Sync
    + 'static,
  // Associated type constraints
  <T as TokenVerifier>::Error: Send + 'static,
  <T as WithServiceProvider>::ServiceProviderType: ActualAuthServiceProvider,
  // User claims mapping
  AuthUser: From<UserClaims>,
{
  fn with_middlewares(
    self,
    state: T,
  ) -> MiddlewareBuilder<S, T, WithoutAuth, WithoutRefresh, WithoutWorkspace, WithoutChatMembership>
  {
    MiddlewareBuilder::new(self, state)
  }
}

// 为所有MiddlewareBuilder实现获取app_state的方法
impl<S, T, A, R, W, C> MiddlewareBuilder<S, T, A, R, W, C>
where
  T: Into<AppState> + Clone,
{
  /// 获取当前已缓存的AppState，或者创建一个新的并返回
  pub fn get_app_state(&self) -> Option<Arc<AppState>> {
    self.app_state.clone()
  }
}
