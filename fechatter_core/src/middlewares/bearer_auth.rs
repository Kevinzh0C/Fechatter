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

use crate::{TokenVerifier, models::AuthUser};
use axum::body::Body;

pub async fn verify_token_middleware<T>(
  State(state): State<T>,
  req: Request<Body>,
  next: Next,
) -> Response
where
  T: TokenVerifier + Clone + Send + Sync + 'static,
  AuthUser: From<T::Claims>,
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
      let user: AuthUser = claims.into();
      let mut req = Request::from_parts(parts, body);
      req.extensions_mut().insert(user);
      next.run(req).await
    }
    Err(e) => (StatusCode::UNAUTHORIZED, format!("{:?}", e)).into_response(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    jwt::{TokenManager, UserClaims},
    setup_test_users,
  };
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
    type Claims = UserClaims;
    type Error = anyhow::Error;

    fn verify_token(&self, token: &str) -> Result<UserClaims, Self::Error> {
      self
        .inner
        .token_manager
        .verify_token(token)
        .map_err(|e| anyhow::anyhow!("{:?}", e))
    }
  }

  async fn handler(_req: Request) -> impl IntoResponse {
    (StatusCode::OK, "OK")
  }

  #[tokio::test]
  async fn verify_token_middleware_should_work() -> Result<()> {
    use crate::models::User;
    use crate::utils::jwt::AuthConfig;

    let auth_config = AuthConfig {
      sk: "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEIJ+DYvh6SEqVTm50DFtMDoQikTmiCqirVv9mWG9qfSnF\n-----END PRIVATE KEY-----".to_string(),
      pk: "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAHrnbu7wEfAP9cGBOAHHwmH4Wsot1ciXBHmCRcXLBUUQ=\n-----END PUBLIC KEY-----".to_string(),
    };

    let token_manager = TokenManager::from_config(&auth_config)?;

    let state = Appstate {
      inner: Arc::new(AppstateInner { token_manager }),
    };

    let user = User {
      id: 1,
      fullname: "Test User".to_string(),
      email: "test@example.com".to_string(),
      password_hash: Some("".to_string()),
      status: crate::models::UserStatus::Active,
      created_at: chrono::Utc::now(),
      workspace_id: 1,
    };

    let app = Router::new()
      .route("/api", get(handler))
      .layer(from_fn_with_state(
        state.clone(),
        verify_token_middleware::<Appstate>,
      ));

    let token = state
      .inner
      .token_manager
      .generate_token(&user)
      .map_err(|_| anyhow::anyhow!("Failed to generate token"))?;

    let req = Request::builder()
      .uri("/api")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())?;

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
  }
}
