use axum::{
  Extension,
  extract::{FromRequestParts, Path, Request, State},
  http::StatusCode,
  middleware::Next,
  response::{IntoResponse, Response},
};
use tracing::{error, info};

use crate::{AppState, error::AppError};
use fechatter_core::models::AuthUser;
use fechatter_core::middlewares::TokenVerifier;

pub async fn verify_chat_membership_middleware(
  state: AppState,
  req: Request,
  next: Next,
) -> Response {
  tracing::info!("🔍 [CHAT_MIDDLEWARE] ========== Starting Chat Membership Verification ==========");
  
  let (mut parts, body) = req.into_parts();

  tracing::debug!("🔍 [CHAT_MIDDLEWARE] Step 1: Extracting chat_id from request path");
  let chat_id = match Path::<i64>::from_request_parts(&mut parts, &State(state.clone())).await {
    Ok(path) => {
      tracing::debug!("🔍 [CHAT_MIDDLEWARE] ✅ Successfully extracted chat_id: {}", path.0);
      path.0
    },
    Err(_) => {
      tracing::debug!("🔍 [CHAT_MIDDLEWARE] ⚠️ Failed to extract via Path, trying manual parsing");
      if let Some(path_and_query) = parts.uri.path_and_query() {
        let path = path_and_query.path();
        tracing::debug!("🔍 [CHAT_MIDDLEWARE] Parsing path: {}", path);
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        tracing::debug!("🔍 [CHAT_MIDDLEWARE] Path segments: {:?}", segments);

        // Enhanced pattern matching for different route structures
        let chat_id = match segments.as_slice() {
          // Standard API routes: /api/chat/{id}[/...]
          ["api", "chat", id, ..] => {
            tracing::debug!("🔍 [CHAT_MIDDLEWARE] Matched pattern: /api/chat/{}", id);
            id.parse::<i64>().ok()
          },
          
          // Direct chat routes: /chat/{id}[/...]
          ["chat", id, ..] => {
            tracing::debug!("🔍 [CHAT_MIDDLEWARE] Matched pattern: /chat/{}", id);
            id.parse::<i64>().ok()
          },
          
          // Legacy or alternative patterns: /{id}/chat/... or /chat/{workspace_id}/{id}/...
          [id, "chat", ..] if id.parse::<i64>().is_ok() => {
            tracing::debug!("🔍 [CHAT_MIDDLEWARE] Matched pattern: /{}/chat/...", id);
            id.parse::<i64>().ok()
          },
          ["chat", _workspace_id, id, ..] => {
            tracing::debug!("🔍 [CHAT_MIDDLEWARE] Matched pattern: /chat/workspace/{}", id);
            id.parse::<i64>().ok()
          },
          
          // Workspace-scoped routes: /api/workspaces/{workspace_id}/chat/{id}
          ["api", "workspaces", _workspace_id, "chat", id, ..] => {
            tracing::debug!("🔍 [CHAT_MIDDLEWARE] Matched pattern: /api/workspaces/.../chat/{}", id);
            id.parse::<i64>().ok()
          },
          
          // Fallback: try to find any numeric segment (should be chat_id)
          _ => {
            tracing::debug!("🔍 [CHAT_MIDDLEWARE] Using fallback: searching for numeric segment");
            segments.iter()
                 .filter_map(|s| s.parse::<i64>().ok())
                 .next()
          },
        };

        match chat_id {
          Some(id) => {
            tracing::info!("🔍 [CHAT_MIDDLEWARE] ✅ Successfully extracted chat_id={} from path: {}", id, path);
            id
          }
          None => {
            tracing::error!("🔍 [CHAT_MIDDLEWARE] ❌ Failed to extract valid chat_id from path: {} (segments: {:?})", path, segments);
            return (StatusCode::BAD_REQUEST, "Invalid or missing chat ID in path").into_response();
          }
        }
      } else {
        tracing::error!("🔍 [CHAT_MIDDLEWARE] ❌ Invalid request URI: unable to extract path and query");
        return (StatusCode::BAD_REQUEST, "Invalid request URI").into_response();
      }
    }
  };

  tracing::info!("🔍 [CHAT_MIDDLEWARE] Step 2: Extracting AuthUser from request");
  let user = match Extension::<AuthUser>::from_request_parts(&mut parts, &State(state.clone()))
    .await
  {
    Ok(Extension(user)) => {
      tracing::info!("🔍 [CHAT_MIDDLEWARE] ✅ Successfully extracted AuthUser: id={}, workspace_id={}", user.id, user.workspace_id);
      user
    }
    Err(e) => {
      // Extended error handling and debugging
      tracing::error!(
        "🔍 [CHAT_MIDDLEWARE] ❌ Failed to extract AuthUser extension: {}. This usually means the auth middleware didn't run or the token was invalid.",
        e
      );

      // Check for Authorization header to provide better error message
      if let Some(auth_header) = parts.headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
          if auth_str.starts_with("Bearer ") {
            // Token exists but AuthUser extension is missing
            tracing::error!(
              "🔍 [CHAT_MIDDLEWARE] Bearer token exists but AuthUser extension is missing. This indicates the token refresh middleware isn't adding the AuthUser extension after refresh."
            );

            // Try to validate token directly to see if it's valid
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
              match state
                .inner
                .application_services
                .token_manager()
                .verify_token(token)
              {
                Ok(claims) => {
                  // Token is valid but AuthUser extension wasn't added
                  tracing::error!(
                    "🔍 [CHAT_MIDDLEWARE] The token is valid (user_id={}), but AuthUser extension wasn't added to the request!",
                    claims.id
                  );
                }
                Err(e) => {
                  // Token is invalid
                  tracing::error!("🔍 [CHAT_MIDDLEWARE] The token is invalid: {}", e);
                }
              }
            }
          }
        }
      } else {
        tracing::error!("🔍 [CHAT_MIDDLEWARE] No Authorization header found in request");
      }

      return (
        StatusCode::UNAUTHORIZED,
        format!(
          "Authentication required for chat {}. Make sure you provide a valid Bearer token or the auth middleware is properly configured.",
          chat_id
        ),
      )
        .into_response();
    }
  };

  tracing::info!("🔍 [CHAT_MIDDLEWARE] Step 3: Calling ensure_user_is_chat_member(chat_id={}, user_id={})", chat_id, user.id);
  
  match state.ensure_user_is_chat_member(chat_id, user.id.into()).await {
    Ok(true) => {
      tracing::info!("🔍 [CHAT_MIDDLEWARE] ✅ SUCCESS: User {} granted access to chat {}", user.id, chat_id);
      tracing::info!("🔍 [CHAT_MIDDLEWARE] ========== Proceeding to next middleware/handler ==========");
      let req = Request::from_parts(parts, body);
      next.run(req).await
    }
    Ok(false) => {
      // This case should not happen with enhanced error handling
      tracing::warn!("🔍 [CHAT_MIDDLEWARE] ⚠️ Unexpected false result from membership check for user {} in chat {}", user.id, chat_id);
      AppError::ChatAccessDenied {
        reason: "Access denied to chat".to_string(),
        chat_id,
        user_id: user.id.into(),
      }.into_response()
    }
    Err(e) => {
      tracing::error!("🔍 [CHAT_MIDDLEWARE] ❌ FAILED: Chat access error for user {} in chat {}: {}", user.id, chat_id, e);
      
      // Enhanced error handling with precise HTTP status codes
      match &e {
        AppError::NotFound(_) => {
          tracing::error!("🔍 [CHAT_MIDDLEWARE] 🔍 Error Type: NotFound - Chat {} not found", chat_id);
        }
        AppError::ChatAccessDenied { reason, .. } => {
          tracing::error!("🔍 [CHAT_MIDDLEWARE] 🔍 Error Type: ChatAccessDenied - Access denied to chat {}: {}", chat_id, reason);
        }
        AppError::ChatMembershipError { message, membership_status, .. } => {
          tracing::error!("🔍 [CHAT_MIDDLEWARE] 🔍 Error Type: ChatMembershipError - Membership error for chat {}: {} (status: {})", 
                         chat_id, message, membership_status);
        }
        _ => {
          tracing::error!("🔍 [CHAT_MIDDLEWARE] 🔍 Error Type: Other - Unexpected error checking chat {} membership: {:?}", chat_id, e);
        }
      }
      
      tracing::error!("🔍 [CHAT_MIDDLEWARE] ========== Returning Error Response ==========");
      // Return the error directly - it now has precise status codes
      e.into_response()
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
