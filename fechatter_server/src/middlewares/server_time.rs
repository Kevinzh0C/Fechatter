use axum::{extract::Request, response::Response};
use futures::future::BoxFuture;
use std::task::{Context, Poll};
use tokio::time::Instant;
use tower::{Layer, Service};
use tracing::warn;

use super::{REQUEST_ID_HEADER, SERVER_TIME_HEADER};

/**
 * ServerTimeLayer
 *
 * A middleware layer that measures and adds server processing time to response headers.
 * It records the time when a request starts processing and adds the elapsed time
 * in microseconds to the response headers.
 */
#[derive(Clone)]
pub struct ServerTimeLayer;

impl<S> Layer<S> for ServerTimeLayer {
  type Service = ServerTimeMiddleware<S>;

  fn layer(&self, inner: S) -> Self::Service {
    ServerTimeMiddleware { inner }
  }
}

/**
 * ServerTimeMiddleware
 *
 * The actual middleware implementation that wraps the inner service.
 * It measures the time taken to process each request and adds this information
 * to the response headers.
 */
#[derive(Clone)]
pub struct ServerTimeMiddleware<S> {
  inner: S,
}

impl<S> Service<Request> for ServerTimeMiddleware<S>
where
  S: Service<Request, Response = Response> + Send + 'static,
  S::Future: Send + 'static,
{
  type Response = S::Response;
  type Error = S::Error;
  type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

  fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    self.inner.poll_ready(cx)
  }

  fn call(&mut self, request: Request) -> Self::Future {
    // Record the start time of request processing
    let start = Instant::now();
    let future = self.inner.call(request);

    Box::pin(async move {
      let mut resp: Response = future.await?;
      // Calculate elapsed time in microseconds
      let elapsed = format!("{}us", start.elapsed().as_micros());

      // Add the elapsed time to response headers
      match elapsed.parse() {
        Ok(v) => {
          resp.headers_mut().insert(SERVER_TIME_HEADER, v);
        }
        Err(e) => {
          // Log warning if header insertion fails, including the request ID for tracing
          warn!(
            "Failed to add elapsed time: {} for request: {:?}",
            e,
            resp.headers().get(REQUEST_ID_HEADER)
          );
        }
      }

      Ok(resp)
    })
  }
}
