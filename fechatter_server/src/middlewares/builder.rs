use crate::AppState;
use axum::{Router, middleware::from_fn_with_state};
use std::marker::PhantomData;

use super::{
  bearer_auth::verify_token_middleware, token_refresh::refresh_token_middleware,
  workspace::with_workspace_context,
};

// Type state markers
pub struct WithoutAuth;
pub struct WithAuth;
pub struct WithRefresh;
pub struct WithWorkspace;

/// Middleware builder that uses type state pattern to ensure correct middleware installation order
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

  pub fn new(router: Router<S>, state: AppState) -> Self {
    Self {
      router,
      state,
      _auth_marker: PhantomData,
      _workspace_marker: PhantomData,
    }
  }


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


  pub fn build(self) -> Router<S> {
    self.router
  }
}

impl<S> MiddlewareBuilder<S, WithAuth, WithoutAuth>
where
  S: Clone + Send + Sync + 'static,
{

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


  pub fn build(self) -> Router<S> {
    self.router
  }
}

impl<S> MiddlewareBuilder<S, WithAuth, WithWorkspace>
where
  S: Clone + Send + Sync + 'static,
{

  pub fn build(self) -> Router<S> {
    self.router
  }
}


pub trait RouterExt<S> {

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
