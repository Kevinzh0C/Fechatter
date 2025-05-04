use axum::body::Body;
use axum::http::Request;
use axum::{extract::State, http::StatusCode, middleware::Next, response::Response};
use tracing::{debug, warn};

<<<<<<< HEAD
use axum_extra::{
  TypedHeader,
  headers::{Authorization, authorization::Bearer},
};
use tracing::warn;

use crate::AppState;
use crate::models::AuthUser;

/// Get authentication from Bearer token and add user to request context
pub async fn verify_token_middleware(
  State(state): State<AppState>,
  mut request: Request<Body>,
  next: Next,
) -> Response {
  let (mut parts, body) = req.into_parts();
  let token =
    match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
      Ok(bearer) => bearer.token().to_string(),
      Err(e) => {
        let msg = format!("parse Bearer token failed: {}", e);
        warn!("{}", msg);
        return (StatusCode::UNAUTHORIZED, msg).into_response();
      }
    };

  match state.token_manager.verify_token(&token) {
    Ok(claims) => {
      let user = AuthUser {
        id: claims.id,
        fullname: claims.fullname,
        email: claims.email,
        status: claims.status,
        created_at: claims.created_at,
        workspace_id: claims.workspace_id,
      };
      let mut req = Request::from_parts(parts, body);
      req.extensions_mut().insert(user);
      next.run(req).await
=======
  match auth_header {
    Some(auth) if auth.starts_with("Bearer ") => {
      debug!("Bearer token extracted successfully");
      let token = &auth[7..]; // Skip "Bearer " prefix

      debug!("Validating token...");
      match state.token_manager.validate_token(token) {
        Ok(claims) => {
          debug!("Token validated successfully for user: {}", claims.id);
          let user = AuthUser {
            id: claims.id,
            fullname: claims.fullname,
            email: claims.email,
            status: claims.status,
            created_at: claims.created_at,
            workspace_id: claims.workspace_id,
          };

          // Add user info to request extensions
          request.extensions_mut().insert(user);
          debug!("AuthUser extension added to request");

          // Continue with the next middleware
          let response = next.run(request).await;
          Ok(response)
        }
        Err(e) => {
          warn!("Token validation failed: {}", e);
          // Return 401 Unauthorized status
          Err(StatusCode::UNAUTHORIZED)
        }
      }
    }
    _ => {
      // No token provided, check if AuthUser extension is already present (might be added by token_refresh middleware)
      if request.extensions().get::<AuthUser>().is_some() {
        debug!("No token in header but AuthUser is already present, continuing...");
        let response = next.run(request).await;
        Ok(response)
      } else {
        debug!("No authentication token provided");
        // Return 401 Unauthorized status
        Err(StatusCode::UNAUTHORIZED)
      }
>>>>>>> 19b2301 (refactor: middleware refresh_token & auth cleanup (#20))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::setup_test_users;
  use anyhow::Result;
  use axum::{
    Router, body::Body, middleware::from_fn_with_state, response::IntoResponse, routing::get,
  };
  use tower::ServiceExt;

  async fn handler(_req: Request<Body>) -> impl IntoResponse {
    (StatusCode::OK, "OK")
  }

  #[tokio::test]
  async fn verify_token_middleware_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user1 = users.into_iter().next().unwrap();

    let app = Router::new()
      .route("/api", get(handler))
      .layer(from_fn_with_state(state.clone(), verify_token_middleware));

    let token = state.token_manager.generate_token(&user1)?;
    let req = Request::builder()
      .uri("/api")
      .header("Authorization", format!("Bearer {token}"))
      .body(Body::empty())?;

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
  }
}
