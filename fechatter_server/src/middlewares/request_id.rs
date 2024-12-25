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

#[cfg(test)]
mod tests {
  use super::*;
  use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    middleware::from_fn,
    response::IntoResponse,
    routing::get,
  };
  use tower::ServiceExt;

  async fn test_handler() -> impl IntoResponse {
    StatusCode::OK
  }

  #[tokio::test]
  async fn test_with_existing_header() {
    let app = Router::new()
      .route("/", get(test_handler))
      .layer(from_fn(request_id_middleware));

    let existing_id = "test-request-id-123";
    let request = Request::builder()
      .uri("/")
      .header(REQUEST_ID_HEADER, existing_id)
      .body(Body::empty())
      .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
      response
        .headers()
        .get(REQUEST_ID_HEADER)
        .unwrap()
        .to_str()
        .unwrap(),
      existing_id
    );
  }

  #[tokio::test]
  async fn test_without_header() {
    let app = Router::new()
      .route("/", get(test_handler))
      .layer(from_fn(request_id_middleware));

    let request = Request::builder().uri("/").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.unwrap();

    let response_id = response.headers().get(REQUEST_ID_HEADER).unwrap();
    assert!(!response_id.is_empty());

    let id_str = response_id.to_str().unwrap();
    assert!(Uuid::parse_str(id_str).is_ok());
  }

  #[tokio::test]
  async fn test_propagation_to_handler() {
    async fn check_header_handler(req: Request<Body>) -> impl IntoResponse {
      let has_header = req.headers().contains_key(REQUEST_ID_HEADER);
      if has_header {
        StatusCode::OK
      } else {
        StatusCode::BAD_REQUEST
      }
    }

    let app = Router::new()
      .route("/", get(check_header_handler))
      .layer(from_fn(request_id_middleware));

    let request = Request::builder().uri("/").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    assert!(response.headers().contains_key(REQUEST_ID_HEADER));
  }

  #[tokio::test]
  async fn test_with_invalid_header_value() {
    let app = Router::new()
      .route("/", get(test_handler))
      .layer(from_fn(request_id_middleware));

    let request = Request::builder().uri("/").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert!(response.headers().contains_key(REQUEST_ID_HEADER));
  }

  #[tokio::test]
  async fn test_header_consistency() {
    let app = Router::new()
      .route("/", get(test_handler))
      .layer(from_fn(request_id_middleware));

    let mut request_ids = Vec::new();

    for _ in 0..5 {
      let request = Request::builder().uri("/").body(Body::empty()).unwrap();

      let response = app.clone().oneshot(request).await.unwrap();
      let id = response
        .headers()
        .get(REQUEST_ID_HEADER)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
      request_ids.push(id);
    }

    let unique_ids: std::collections::HashSet<_> = request_ids.iter().collect();
    assert_eq!(unique_ids.len(), request_ids.len());

    for id in &request_ids {
      assert!(Uuid::parse_str(id).is_ok());
    }
  }
}
