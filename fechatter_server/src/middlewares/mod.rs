mod bearer_auth;
mod builder;
mod chat;
mod token_refresh;
mod workspace;

pub use bearer_auth::verify_token_middleware;
pub use builder::RouterExt;
pub use chat::verify_chat_membership_middleware;
pub use token_refresh::refresh_token_middleware;
pub use workspace::{WorkspaceContext, with_workspace_context};

use axum::{Router, middleware::from_fn_with_state};


// Middleware ordering traits for better type safety
pub trait WithAuth {}
pub trait WithTokenRefresh {}
pub trait WithWorkspace {}
pub trait WithChatMembership {}

pub(crate) use self::bearer_auth::verify_token_middleware;
pub use self::builder::RouterExt;
pub(crate) use self::request_id::request_id_middleware;
pub(crate) use self::server_time::ServerTimeLayer;
pub(crate) use self::token_refresh::refresh_token_middleware;
pub(crate) use self::workspace::{WorkspaceContext, with_workspace_context};
use crate::AppState;

/// Apply authentication middleware to a router
pub trait SetAuthLayer {
  fn with_auth(self, state: &crate::AppState) -> Self;
  fn with_token_refresh(self, state: &crate::AppState) -> Self;
  fn with_workspace(self, state: &crate::AppState) -> Self;
  fn with_chat_membership(self, state: &crate::AppState) -> Self;
}

impl<S> SetAuthLayer for Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn with_auth(self, state: &crate::AppState) -> Self {
    self.layer(from_fn_with_state(state.clone(), verify_token_middleware))
  }

  fn with_token_refresh(self, state: &crate::AppState) -> Self {
    self.layer(from_fn_with_state(state.clone(), refresh_token_middleware))
  }

  fn with_workspace(self, state: &crate::AppState) -> Self {
    self.layer(from_fn_with_state(state.clone(), with_workspace_context))
  }

  fn with_chat_membership(self, state: &crate::AppState) -> Self {
    self.layer(from_fn_with_state(
      state.clone(),
      verify_chat_membership_middleware,
    ))
  }
}

/// Apply common middleware to a router
pub trait SetLayer {
  fn set_layer(self) -> Self;
}

impl<S> SetLayer for Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn set_layer(self) -> Self {
    // Use the common middleware layers from tower and tower_http
    self
  }
}
