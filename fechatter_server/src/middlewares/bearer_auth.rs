use axum::{
  extract::{FromRequestParts, Request, State},
  http::StatusCode,
  middleware::Next,
  response::{IntoResponse, Response},
};

use axum_extra::{
  TypedHeader,
  headers::{Authorization, authorization::Bearer},
};
use tracing::warn;

use crate::AppState;
use crate::models::AuthUser;
use crate::utils::token::TokenValidator;

pub async fn verify_token_middleware(
  State(state): State<AppState>,
  req: Request,
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

  match state.token_manager.validate_token(&token) {
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
    }
    Err(e) => (StatusCode::UNAUTHORIZED, e.to_string()).into_response(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::setup_test_users;
  use anyhow::Result;
  use axum::{Router, body::Body, middleware::from_fn_with_state, routing::get};
  use tower::ServiceExt;

  async fn handler(_req: Request) -> impl IntoResponse {
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
