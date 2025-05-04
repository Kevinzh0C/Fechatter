mod bearer_auth;
mod builder;
mod chat;
mod token_refresh;
mod workspace;

use axum::{Router, middleware::from_fn};

use tower::ServiceBuilder;
use tower_http::{
  LatencyUnit,
  compression::CompressionLayer,
  trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use crate::models::AuthUser;

pub(crate) use self::bearer_auth::verify_token_middleware;
pub use self::builder::RouterExt;
pub(crate) use self::request_id::request_id_middleware;
pub(crate) use self::server_time::ServerTimeLayer;
use crate::AppState;

pub const REQUEST_ID_HEADER: &str = "x-request-id";
pub const SERVER_TIME_HEADER: &str = "x-server-time";

pub trait TokenVerifier {
  type Error: std::fmt::Debug;
  type Claims;
  fn verify_token(&self, token: &str) -> Result<Self::Claims, Self::Error>;
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
pub fn with_auth_layer<S, T>(router: Router<S>, state: T) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
  T: TokenVerifier + Clone + Send + Sync + 'static,
  AuthUser: From<T::Claims>,
{
  router
}
