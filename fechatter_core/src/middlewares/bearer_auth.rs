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

use crate::{AuthUser, middlewares::TokenVerifier};

pub async fn verify_token_middleware<T>(
  State(state): State<T>,
  req: Request,
  next: Next,
) -> Response
where
  T: TokenVerifier + Clone + Send + Sync + 'static,
{
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

  match state.verify_token(&token) {
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
  use crate::{jwt::{TokenManager, UserClaims}, setup_test_users};
  use anyhow::Result;
  use axum::{Router, body::Body, middleware::from_fn_with_state, routing::get};
  use std::sync::Arc;
  use tower::ServiceExt;

  #[derive(Clone)]
  struct Appstate {
    inner: Arc<AppstateInner>,
  }

  struct AppstateInner {
    token_manager: TokenManager,
  }

  impl TokenVerifier for Appstate {
    type Error = ();

    fn verify_token(&self, token: &str) -> Result<UserClaims, Self::Error> {
      self.inner.token_manager.verify_token(token)
    }
  }

  async fn handler(_req: Request) -> impl IntoResponse {
    (StatusCode::OK, "OK")
  }

  #[tokio::test]
  async fn verify_token_middleware_should_work() -> Result<()> {
    
    let token_manager: TokenManager = TokenManager {
      encoding_key: EncodingKey::new(b"secret"),
      decoding_key: DecodingKey::new(b"secret"),
      validation: Validation::default(),
    }


    let state = Appstate {
      inner: Arc::new(AppstateInner {
        token_manager,
      }),
    };

    let app = Router::new()
      .route("/api", get(handler))
      .layer(from_fn_with_state(state.clone(), verify_token_middleware::<Appstate>));

    let token = state.token_manager.generate_token(&user1)?;
    let req = Request::builder()
      .uri("/api")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())?;

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
  }
}
