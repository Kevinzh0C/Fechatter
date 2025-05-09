use axum::{Router, middleware::from_fn};
use fechatter_core::SetLayer as _;
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
use fechatter_core::middlewares::server_time::ServerTimeLayer; // 导入服务器时间中间件
use fechatter_core::middlewares::{
  ActualAuthServiceProvider,
  TokenVerifier,
  WithServiceProvider,
  WithTokenManager,
  request_id_middleware, // 导入请求ID中间件
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

// 使用位标志来表示已应用的中间件类型
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

/// 一个更简洁、灵活的中间件构建器
/// 使用位标志而不是类型参数来跟踪中间件状态
pub struct MiddlewareBuilder<S, T> {
  router: Router<S>,
  state: T,
  app_state: Option<Arc<AppState>>,
  applied: MiddlewareFlags, // 跟踪已应用的中间件
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
  /// 创建一个新的中间件构建器
  pub fn new(router: Router<S>, state: T) -> Self {
    Self {
      router,
      state,
      app_state: None,
      applied: MiddlewareFlags::NONE,
    }
  }

  /// 获取或创建AppState
  fn get_or_create_app_state(&mut self) -> Arc<AppState> {
    if let Some(app_state) = &self.app_state {
      app_state.clone()
    } else {
      let app_state = Arc::new(self.state.clone().into());
      self.app_state = Some(app_state.clone());
      app_state
    }
  }

  /// 添加认证中间件
  pub fn with_auth(mut self) -> Self {
    if !self.applied.contains(MiddlewareFlags::AUTH) {
      self.router = add_auth_middleware(self.router, self.state.clone());
      self.applied = self.applied.add(MiddlewareFlags::AUTH);
    }
    self
  }

  /// 添加令牌刷新中间件
  pub fn with_refresh(mut self) -> Self {
    if !self.applied.contains(MiddlewareFlags::REFRESH) {
      self.router = add_refresh_middleware(self.router, self.state.clone());
      self.applied = self.applied.add(MiddlewareFlags::REFRESH);
    }
    self
  }

  /// 添加工作区中间件
  pub fn with_workspace(mut self) -> Self {
    if !self.applied.contains(MiddlewareFlags::WORKSPACE) {
      let app_state = self.get_or_create_app_state();
      self.router = add_workspace_middleware(self.router, (*app_state).clone());
      self.applied = self.applied.add(MiddlewareFlags::WORKSPACE);
    }
    self
  }

  /// 添加聊天成员资格中间件
  pub fn with_chat_membership(mut self) -> Self {
    if !self.applied.contains(MiddlewareFlags::CHAT_MEMBERSHIP) {
      let app_state = self.get_or_create_app_state();
      self.router = add_chat_membership_middleware(self.router, (*app_state).clone());
      self.applied = self.applied.add(MiddlewareFlags::CHAT_MEMBERSHIP);
    }
    self
  }

  /// 添加所有业务中间件（Auth, Refresh, Workspace, Chat Membership）
  /// 顺序将自动设置为：Auth -> Refresh -> Workspace -> ChatMembership
  pub fn with_all_middlewares(self) -> Self {
    self
      .with_chat_membership()
      .with_workspace()
      .with_refresh()
      .with_auth()
  }

  /// 应用Auth和Refresh中间件
  pub fn with_auth_refresh(self) -> Self {
    self.with_refresh().with_auth()
  }

  /// 应用Auth, Refresh和Workspace中间件
  pub fn with_auth_refresh_workspace(self) -> Self {
    self.with_workspace().with_refresh().with_auth()
  }

  /// 构建最终的路由，应用基础设施中间件并返回
  pub fn build(self) -> Router<S> {
    // 应用基础设施中间件（ServerTime, RequestId, Compression, Trace）
    self.router.set_layer()
  }
}

// 提供RouterExt扩展特性以便于链式调用
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

// 为了保持与旧API的兼容性，保留这些方法
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
  /// 类似于finalize，但不添加任何中间件
  /// 只添加基础设施中间件（ServerTime, RequestId, Compression, Trace）
  pub fn finalize_base(self) -> Router<S> {
    self.build()
  }

  /// 添加认证中间件和基础设施中间件
  pub fn finalize_auth_only(self) -> Router<S> {
    self.with_auth().build()
  }

  /// 添加认证、刷新中间件和基础设施中间件
  pub fn finalize_auth_refresh(self) -> Router<S> {
    self.with_auth_refresh().build()
  }

  /// 添加认证、刷新、工作区中间件和基础设施中间件
  pub fn finalize_auth_refresh_workspace(self) -> Router<S> {
    self.with_auth_refresh_workspace().build()
  }

  /// 添加所有中间件：认证、刷新、工作区、聊天成员资格和基础设施中间件
  pub fn finalize(self) -> Router<S> {
    self.with_all_middlewares().build()
  }
}
