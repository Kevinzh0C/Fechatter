mod bearer_auth;
mod builder;
mod request_id;
mod server_time;
mod token_refresh;
pub mod workspace;

use axum::{
  Router,
  middleware::{from_fn, from_fn_with_state},
};

use tower::ServiceBuilder;
use tower_http::{
  LatencyUnit,
  compression::CompressionLayer,
  trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

pub(crate) use self::bearer_auth::verify_token_middleware;
pub use self::builder::RouterExt;
pub(crate) use self::request_id::request_id_middleware;
pub(crate) use self::server_time::ServerTimeLayer;
pub(crate) use self::token_refresh::refresh_token_middleware;
pub use self::workspace::{WorkspaceContext, with_workspace_context};
use crate::AppState;

pub const REQUEST_ID_HEADER: &str = "x-request-id";
pub const SERVER_TIME_HEADER: &str = "x-server-time";

/// Apply common middleware to a router
pub trait SetLayer {
  fn set_layer(self) -> Self;
}

impl<S> SetLayer for Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn set_layer(self) -> Self {
    self.layer(
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
        .layer(CompressionLayer::new().gzip(true).br(true).deflate(true))
        .layer(from_fn(request_id_middleware))
        .layer(ServerTimeLayer),
    )
  }
}

/// Apply authentication middleware to a router
pub trait SetAuthLayer {
  fn set_auth_layer(self, state: AppState) -> Self;
}

impl<S> SetAuthLayer for Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn set_auth_layer(self, state: AppState) -> Self {
    self
      .layer(from_fn_with_state(state.clone(), refresh_token_middleware))
      .layer(from_fn_with_state(state, verify_token_middleware))
  }
}

/// Apply workspace middleware to a router
pub trait SetWorkspaceLayer {
  fn set_workspace_layer(self, state: AppState) -> Self;
}

impl<S> SetWorkspaceLayer for Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn set_workspace_layer(self, state: AppState) -> Self {
    self
      .layer(from_fn_with_state(state.clone(), refresh_token_middleware))
      .layer(from_fn_with_state(state.clone(), verify_token_middleware))
      .layer(from_fn_with_state(state, with_workspace_context))
  }
}

/// Apply token refresh middleware to a router
pub trait SetTokenRefreshLayer {
  fn set_token_refresh_layer(self, state: AppState) -> Self;
}

impl<S> SetTokenRefreshLayer for Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn set_token_refresh_layer(self, state: AppState) -> Self {
    self.layer(from_fn_with_state(state, refresh_token_middleware))
  }
}
