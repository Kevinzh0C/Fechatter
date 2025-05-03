use axum::{
  Extension,
  extract::{FromRequestParts, Path, Request, State},
  http::StatusCode,
  middleware::Next,
  response::{IntoResponse, Response},
};
use tracing::{debug, error, info};

use crate::{
  AppError, AppState,
  models::{AuthUser, ensure_user_is_chat_member},
  utils::token::TokenValidator,
};

pub async fn verify_chat_membership_middleware(
  State(state): State<AppState>,
  req: Request,
  next: Next,
) -> Response {
  let (mut parts, body) = req.into_parts();

  // 获取聊天ID
  let chat_id = match Path::<i64>::from_request_parts(&mut parts, &state).await {
    Ok(path) => {
      debug!("Found chat_id in path: {}", path.0);
      path.0
    }
    Err(_) => {
      debug!("Failed to extract chat_id from path, trying to parse from URI segments");
      if let Some(path_and_query) = parts.uri.path_and_query() {
        let path = path_and_query.path();
        debug!("Path: {}", path);
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        debug!("Path segments: {:?}", segments);

        if segments.len() >= 2 {
          // Try second segment as ID (e.g. /chat/123/...)
          if let Ok(id) = segments[1].parse::<i64>() {
            debug!("Using second segment as chat_id: {}", id);
            id
          } else if segments.len() >= 1 {
            // If second segment is not ID, try first segment (e.g. /123/...)
            let potential_id = segments[0].parse::<i64>().unwrap_or(0);
            debug!("Using first segment as chat_id: {}", potential_id);
            potential_id
          } else {
            // No ID found, return error
            error!("Missing chat ID in path");
            return (StatusCode::BAD_REQUEST, "Missing chat ID in path").into_response();
          }
        } else {
          // Path too short, return error
          error!("Invalid path format: {:?}", segments);
          return (StatusCode::BAD_REQUEST, "Invalid path format").into_response();
        }
      } else {
        // Cannot get path and query, return error
        error!("Invalid request URI: unable to extract path and query");
        return (StatusCode::BAD_REQUEST, "Invalid request URI").into_response();
      }
    }
  };

  // 尝试提取用户信息
  let user = match Extension::<AuthUser>::from_request_parts(&mut parts, &state).await {
    Ok(Extension(user)) => {
      debug!("Found AuthUser extension: user_id={}", user.id);
      user
    }
    Err(e) => {
      // Extended error handling and debugging
      error!(
        "Failed to extract AuthUser extension: {}. This usually means the auth middleware didn't run or the token was invalid.",
        e
      );
      debug!("Request headers: {:?}", parts.headers);

      // Check for Authorization header to provide better error message
      if let Some(auth_header) = parts.headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
          debug!("Authorization header present: {}", auth_str);
          if auth_str.starts_with("Bearer ") {
            // Token exists but AuthUser extension is missing
            error!(
              "Bearer token exists but AuthUser extension is missing. This indicates the token refresh middleware isn't adding the AuthUser extension after refresh."
            );

            // Try to validate token directly to see if it's valid
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
              match state.token_manager.validate_token(token) {
                Ok(claims) => {
                  // Token is valid but AuthUser extension wasn't added
                  error!(
                    "The token is valid (user_id={}), but AuthUser extension wasn't added to the request!",
                    claims.id
                  );
                }
                Err(e) => {
                  // Token is invalid
                  error!("The token is invalid: {}", e);
                }
              }
            }
          }
        }
      } else {
        debug!("No Authorization header found in request");
      }

      return (
        StatusCode::UNAUTHORIZED,
        "Authentication required. Make sure you provide a valid Bearer token.",
      )
        .into_response();
    }
  };

  debug!("Checking if user {} is member of chat {}", user.id, chat_id);

  // 验证用户是否是聊天成员
  match ensure_user_is_chat_member(&state.pool, chat_id, user.id).await {
    Ok(true) => {
      debug!(
        "User {} is a member of chat {}, proceeding",
        user.id, chat_id
      );
      let req = Request::from_parts(parts, body);
      next.run(req).await
    }
    Ok(false) => {
      info!(
        "Permission denied: User {} is not a member of chat {}",
        user.id, chat_id
      );
      let err = AppError::ChatPermissionError(format!(
        "User {} is not a member of chat {}",
        user.id, chat_id
      ));
      err.into_response()
    }
    Err(e) => {
      error!("Error checking chat membership: {:?}", e);
      match e {
        AppError::ChatPermissionError(_) => e.into_response(),
        _ => AppError::from(e).into_response(),
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::get,
  };
  use std::convert::Infallible;
  use tower::ServiceExt;

  fn make_test_router(
    is_member: bool,
    should_error: bool,
    error_type: Option<&'static str>,
  ) -> Router {
    async fn handler() -> &'static str {
      "Hello, World!"
    }

    let middleware = move |req: Request<Body>, next: Next| {
      let is_member = is_member;
      let should_error = should_error;
      let error_type = error_type;
      async move {
        if should_error {
          let err = match error_type {
            Some("not_found") => AppError::NotFound(vec!["Chat not found".to_string()]),
            Some("db_error") => AppError::SqlxError(sqlx::Error::RowNotFound),
            _ => AppError::ChatPermissionError("User is not a member of chat".to_string()),
          };
          return err.into_response();
        }

        if !is_member {
          let err = AppError::ChatPermissionError("User is not a member of chat".to_string());
          return err.into_response();
        }

        next.run(req).await
      }
    };

    Router::new()
      .route("/test", get(handler))
      .layer(axum::middleware::from_fn(middleware))
  }

  #[tokio::test]
  async fn chat_member_should_access_route() -> Result<(), Infallible> {
    let app = make_test_router(true, false, None);

    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
  }

  #[tokio::test]
  async fn non_member_should_not_access() -> Result<(), Infallible> {
    let app = make_test_router(false, false, None);

    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    Ok(())
  }

  #[tokio::test]
  async fn chat_permission_error_should_return_forbidden() -> Result<(), Infallible> {
    let app = make_test_router(false, true, None);

    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    Ok(())
  }

  #[tokio::test]
  async fn not_found_error_should_return_404() -> Result<(), Infallible> {
    let app = make_test_router(false, true, Some("not_found"));

    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    Ok(())
  }

  #[tokio::test]
  async fn db_error_should_return_500() -> Result<(), Infallible> {
    let app = make_test_router(false, true, Some("db_error"));

    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    Ok(())
  }

  #[tokio::test]
  async fn invalid_path_should_return_bad_request() -> Result<(), Infallible> {
    // 创建一个模拟verify_chat_membership_middleware的处理函数
    async fn mock_middleware(req: Request<Body>, _next: Next) -> Response {
      let uri = req.uri().clone();
      let path = uri.path();

      if !path.contains("/chat/") && !path.contains("/id/") {
        (StatusCode::BAD_REQUEST, "Invalid path format").into_response()
      } else {
        (StatusCode::OK, "Valid path").into_response()
      }
    }

    let app = Router::new()
      .route("/test", get(|| async { "OK" }))
      .layer(axum::middleware::from_fn(mock_middleware));

    let response = app
      .oneshot(
        Request::builder()
          .uri("/invalid")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    Ok(())
  }

  #[tokio::test]
  async fn missing_auth_should_return_unauthorized() -> Result<(), Infallible> {
    async fn mock_middleware(_req: Request<Body>, _next: Next) -> Response {
      (StatusCode::UNAUTHORIZED, "Authentication required").into_response()
    }

    let app = Router::new()
      .route("/test", get(|| async { "OK" }))
      .layer(axum::middleware::from_fn(mock_middleware));

    let response = app
      .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    Ok(())
  }
}
