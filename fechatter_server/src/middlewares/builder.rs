use crate::AppState;
use axum::{Router, middleware::from_fn_with_state};
use std::marker::PhantomData;

use super::{
  bearer_auth::verify_token_middleware, token_refresh::refresh_token_middleware,
  verify_chat_membership_middleware, workspace::with_workspace_context,
};

// Type state markers
pub struct WithoutAuth;
pub struct WithAuth;
pub struct WithoutRefresh;
pub struct WithRefresh;
pub struct WithoutWorkspace;
pub struct WithWorkspace;
pub struct WithoutChatMembership;
pub struct WithChatMembership;

/// Middleware builder that uses type state pattern to ensure correct middleware installation order
pub struct MiddlewareBuilder<
  S,
  AuthState = WithoutAuth,
  RefreshState = WithoutRefresh,
  WorkspaceState = WithoutWorkspace,
  ChatMembershipState = WithoutChatMembership,
> {
  router: Router<S>,
  state: AppState,
  _auth_marker: PhantomData<AuthState>,
  _refresh_marker: PhantomData<RefreshState>,
  _workspace_marker: PhantomData<WorkspaceState>,
  _chat_membership_marker: PhantomData<ChatMembershipState>,
}

impl<S> MiddlewareBuilder<S, WithoutAuth, WithoutRefresh, WithoutWorkspace, WithoutChatMembership>
where
  S: Clone + Send + Sync + 'static,
{
  pub fn new(router: Router<S>, state: AppState) -> Self {
    Self {
      router,
      state,
      _auth_marker: PhantomData,
      _refresh_marker: PhantomData,
      _workspace_marker: PhantomData,
      _chat_membership_marker: PhantomData,
    }
  }

  // Allow directly adding auth before token refresh
  pub fn with_auth(
    self,
  ) -> MiddlewareBuilder<S, WithAuth, WithoutRefresh, WithoutWorkspace, WithoutChatMembership> {
    let router = self.router.layer(from_fn_with_state(
      self.state.clone(),
      verify_token_middleware,
    ));

    MiddlewareBuilder {
      router,
      state: self.state,
      _auth_marker: PhantomData,
      _refresh_marker: PhantomData,
      _workspace_marker: PhantomData,
      _chat_membership_marker: PhantomData,
    }
  }
}

// For backwards compatibility, allow token refresh first if needed
impl<S> MiddlewareBuilder<S, WithoutAuth, WithoutRefresh, WithoutWorkspace, WithoutChatMembership>
where
  S: Clone + Send + Sync + 'static,
{
  pub fn with_token_refresh(
    self,
  ) -> MiddlewareBuilder<S, WithoutAuth, WithRefresh, WithoutWorkspace, WithoutChatMembership> {
    let router = self.router.layer(from_fn_with_state(
      self.state.clone(),
      refresh_token_middleware,
    ));

    MiddlewareBuilder {
      router,
      state: self.state,
      _auth_marker: PhantomData,
      _refresh_marker: PhantomData,
      _workspace_marker: PhantomData,
      _chat_membership_marker: PhantomData,
    }
  }
}

// Allow adding token refresh after auth
impl<S> MiddlewareBuilder<S, WithAuth, WithoutRefresh, WithoutWorkspace, WithoutChatMembership>
where
  S: Clone + Send + Sync + 'static,
{
  pub fn with_token_refresh(
    self,
  ) -> MiddlewareBuilder<S, WithAuth, WithRefresh, WithoutWorkspace, WithoutChatMembership> {
    let router = self.router.layer(from_fn_with_state(
      self.state.clone(),
      refresh_token_middleware,
    ));

    MiddlewareBuilder {
      router,
      state: self.state,
      _auth_marker: PhantomData,
      _refresh_marker: PhantomData,
      _workspace_marker: PhantomData,
      _chat_membership_marker: PhantomData,
    }
  }

  pub fn with_workspace(
    self,
  ) -> MiddlewareBuilder<S, WithAuth, WithoutRefresh, WithWorkspace, WithoutChatMembership> {
    let router = self.router.layer(from_fn_with_state(
      self.state.clone(),
      with_workspace_context,
    ));

    MiddlewareBuilder {
      router,
      state: self.state,
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

// Token refresh can be added first (for backwards compatibility)
impl<S> MiddlewareBuilder<S, WithoutAuth, WithRefresh, WithoutWorkspace, WithoutChatMembership>
where
  S: Clone + Send + Sync + 'static,
{
  pub fn with_auth(
    self,
  ) -> MiddlewareBuilder<S, WithAuth, WithRefresh, WithoutWorkspace, WithoutChatMembership> {
    let router = self.router.layer(from_fn_with_state(
      self.state.clone(),
      verify_token_middleware,
    ));

    MiddlewareBuilder {
      router,
      state: self.state,
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

// Allow adding workspace context after adding token refresh
impl<S> MiddlewareBuilder<S, WithAuth, WithRefresh, WithoutWorkspace, WithoutChatMembership>
where
  S: Clone + Send + Sync + 'static,
{
  pub fn with_workspace(
    self,
  ) -> MiddlewareBuilder<S, WithAuth, WithRefresh, WithWorkspace, WithoutChatMembership> {
    let router = self.router.layer(from_fn_with_state(
      self.state.clone(),
      with_workspace_context,
    ));

    MiddlewareBuilder {
      router,
      state: self.state,
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

// Add workspace after auth without token refresh
impl<S> MiddlewareBuilder<S, WithAuth, WithoutRefresh, WithWorkspace, WithoutChatMembership>
where
  S: Clone + Send + Sync + 'static,
{
  pub fn with_token_refresh(
    self,
  ) -> MiddlewareBuilder<S, WithAuth, WithRefresh, WithWorkspace, WithoutChatMembership> {
    let router = self.router.layer(from_fn_with_state(
      self.state.clone(),
      refresh_token_middleware,
    ));

    MiddlewareBuilder {
      router,
      state: self.state,
      _auth_marker: PhantomData,
      _refresh_marker: PhantomData,
      _workspace_marker: PhantomData,
      _chat_membership_marker: PhantomData,
    }
  }

  pub fn with_chat_membership(
    self,
  ) -> MiddlewareBuilder<S, WithAuth, WithoutRefresh, WithWorkspace, WithChatMembership> {
    let router = self.router.layer(from_fn_with_state(
      self.state.clone(),
      verify_chat_membership_middleware,
    ));

    MiddlewareBuilder {
      router,
      state: self.state,
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

// Add chat membership after auth and workspace with token refresh
impl<S> MiddlewareBuilder<S, WithAuth, WithRefresh, WithWorkspace, WithoutChatMembership>
where
  S: Clone + Send + Sync + 'static,
{
  pub fn with_chat_membership(
    self,
  ) -> MiddlewareBuilder<S, WithAuth, WithRefresh, WithWorkspace, WithChatMembership> {
    let router = self.router.layer(from_fn_with_state(
      self.state.clone(),
      verify_chat_membership_middleware,
    ));

    MiddlewareBuilder {
      router,
      state: self.state,
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

// Allow adding token refresh after workspace
impl<S> MiddlewareBuilder<S, WithAuth, WithoutRefresh, WithWorkspace, WithChatMembership>
where
  S: Clone + Send + Sync + 'static,
{
  pub fn with_token_refresh(
    self,
  ) -> MiddlewareBuilder<S, WithAuth, WithRefresh, WithWorkspace, WithChatMembership> {
    let router = self.router.layer(from_fn_with_state(
      self.state.clone(),
      refresh_token_middleware,
    ));

    MiddlewareBuilder {
      router,
      state: self.state,
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

// Final builder with all middleware
impl<S> MiddlewareBuilder<S, WithAuth, WithRefresh, WithWorkspace, WithChatMembership>
where
  S: Clone + Send + Sync + 'static,
{
  pub fn build(self) -> Router<S> {
    self.router
  }
}

pub trait RouterExt<S> {
  fn with_middlewares(
    self,
    state: AppState,
  ) -> MiddlewareBuilder<S, WithoutAuth, WithoutRefresh, WithoutWorkspace, WithoutChatMembership>;
}

impl<S> RouterExt<S> for Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn with_middlewares(
    self,
    state: AppState,
  ) -> MiddlewareBuilder<S, WithoutAuth, WithoutRefresh, WithoutWorkspace, WithoutChatMembership>
  {
    MiddlewareBuilder::new(self, state)
  }
}
