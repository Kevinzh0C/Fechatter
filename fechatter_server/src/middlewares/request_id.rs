use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};

use tracing::warn;
use uuid::Uuid;

use super::REQUEST_ID_HEADER;

/**
 * Request ID Middleware
 *
 * This middleware is responsible for generating and propagating a unique request ID for each HTTP request.
 * If the request already contains an x-request-id header, it uses that value; otherwise, it generates a new UUID v7.
 * The request ID is added to both request and response headers for request tracing and debugging purposes.
 *
 */
pub async fn request_id_middleware(mut req: Request, next: Next) -> Response {
  let id = match req.headers().get(REQUEST_ID_HEADER) {
    Some(v) => Some(v.to_owned()),
    None => {
      let req_id = Uuid::now_v7().to_string();
      match HeaderValue::from_str(&req_id) {
        Ok(v) => {
          req.headers_mut().insert(REQUEST_ID_HEADER, v.clone());
          Some(v)
        }
        Err(e) => {
          warn!("Failed to parse generated request ID: {}", e);
          None
        }
      }
    }
  };

  // Execute subsequent middleware and handlers
  let mut res = next.run(req).await;

  // Add the request ID to response headers
  let Some(id) = id else {
    return res;
  };
  res.headers_mut().insert(REQUEST_ID_HEADER, id);
  res
}
