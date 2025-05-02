use crate::AppState;
use axum::{Router, middleware::from_fn_with_state};
use std::marker::PhantomData;

use super::{
  bearer_auth::verify_token_middleware, token_refresh::refresh_token_middleware,
  workspace::with_workspace_context,
};

// 类型状态标记
pub struct WithoutAuth;
pub struct WithAuth;
pub struct WithRefresh;
pub struct WithWorkspace;

/// 中间件构建器，使用类型状态模式确保中间件安装顺序
pub struct MiddlewareBuilder<S, AuthState = WithoutAuth, WorkspaceState = WithoutAuth> {
  router: Router<S>,
  state: AppState,
  _auth_marker: PhantomData<AuthState>,
  _workspace_marker: PhantomData<WorkspaceState>,
}

impl<S> MiddlewareBuilder<S, WithoutAuth, WithoutAuth>
where
  S: Clone + Send + Sync + 'static,
{
  /// 创建新的中间件构建器
  pub fn new(router: Router<S>, state: AppState) -> Self {
    Self {
      router,
      state,
      _auth_marker: PhantomData,
      _workspace_marker: PhantomData,
    }
  }

  /// 应用令牌刷新中间件
  pub fn with_token_refresh(self) -> MiddlewareBuilder<S, WithRefresh, WithoutAuth> {
    let router = self.router.layer(from_fn_with_state(
      self.state.clone(),
      refresh_token_middleware,
    ));

    MiddlewareBuilder {
      router,
      state: self.state,
      _auth_marker: PhantomData,
      _workspace_marker: PhantomData,
    }
  }
}

impl<S> MiddlewareBuilder<S, WithRefresh, WithoutAuth>
where
  S: Clone + Send + Sync + 'static,
{
  /// 应用认证中间件（需要先应用令牌刷新）
  pub fn with_auth(self) -> MiddlewareBuilder<S, WithAuth, WithoutAuth> {
    let router = self.router.layer(from_fn_with_state(
      self.state.clone(),
      verify_token_middleware,
    ));

    MiddlewareBuilder {
      router,
      state: self.state,
      _auth_marker: PhantomData,
      _workspace_marker: PhantomData,
    }
  }

  /// 构建路由器，仅应用令牌刷新
  pub fn build(self) -> Router {
    self.router
  }
}

impl<S> MiddlewareBuilder<S, WithAuth, WithoutAuth>
where
  S: Clone + Send + Sync + 'static,
{
  /// 应用工作区上下文中间件（需要先应用认证）
  pub fn with_workspace(self) -> MiddlewareBuilder<S, WithAuth, WithWorkspace> {
    let router = self.router.layer(from_fn_with_state(
      self.state.clone(),
      with_workspace_context,
    ));

    MiddlewareBuilder {
      router,
      state: self.state,
      _auth_marker: PhantomData,
      _workspace_marker: PhantomData,
    }
  }

  /// 构建路由器，应用了认证和刷新
  pub fn build(self) -> Router {
    self.router
  }
}

impl<S> MiddlewareBuilder<S, WithAuth, WithWorkspace>
where
  S: Clone + Send + Sync + 'static,
{
  /// 构建路由器，应用了认证、刷新和工作区上下文
  pub fn build(self) -> Router {
    self.router
  }
}

/// 路由器扩展特征，简化中间件应用
pub trait RouterExt<S> {
  /// 使用中间件构建器添加中间件
  fn with_middlewares(self, state: AppState) -> MiddlewareBuilder<S, WithoutAuth, WithoutAuth>;
}

impl<S> RouterExt<S> for Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn with_middlewares(self, state: AppState) -> MiddlewareBuilder<S, WithoutAuth, WithoutAuth> {
    MiddlewareBuilder::new(self, state)
  }
}
