use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};

use tracing::warn;
use uuid::Uuid;

/// HTTP header for request ID
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/**
 * Request ID Middleware
 *
 * This middleware is responsible for generating and propagating a unique request ID for each HTTP request.
 * If the request already contains an x-request-id header, it uses that value; otherwise, it generates a new UUID v4.
 * The request ID is added to both request and response headers for request tracing and debugging purposes.
 *
 */
pub async fn request_id_middleware(mut req: Request, next: Next) -> Response {
  // Check if request already has an ID, if not generate one
  let req_id = if req.headers().get(REQUEST_ID_HEADER).is_none() {
    // Generate a new request ID using UUID v4
    let req_id = Uuid::new_v4().to_string();

    // Insert the request ID into headers
    if let Ok(header_value) = HeaderValue::from_str(&req_id) {
      req.headers_mut().insert(REQUEST_ID_HEADER, header_value);
    }
    Some(req_id)
  } else {
    None
  };

  let mut response = next.run(req).await;

  // Add request ID to response headers if we generated one
  if let Some(id) = req_id {
    if let Ok(header_value) = HeaderValue::from_str(&id) {
      response
        .headers_mut()
        .insert(REQUEST_ID_HEADER, header_value);
    }
  }

  response
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::from_fn,
    response::IntoResponse,
    routing::get,
    Router,
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

    let invalid_id = "invalid-header-value-123";
    let request = Request::builder()
      .uri("/")
      .header(REQUEST_ID_HEADER, invalid_id)
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
      invalid_id
    );
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
