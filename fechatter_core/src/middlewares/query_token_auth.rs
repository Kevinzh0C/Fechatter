use axum::{
  body::Body,
  extract::{Query, Request, State},
  http::StatusCode,
  middleware::Next,
  response::{IntoResponse, Response},
};
use serde::Deserialize;
use tracing::warn;

use crate::{models::AuthUser, TokenVerifier};

#[derive(Deserialize)]
pub struct TokenQuery {
  pub access_token: String,
}

/// Token verification middleware that supports query parameter authentication
/// Specifically designed for SSE connections where headers cannot be customized
pub async fn verify_query_token_middleware<T>(
  State(state): State<T>,
  req: Request<Body>,
  next: Next,
) -> Response
where
  T: TokenVerifier + Clone + Send + Sync + 'static,
  AuthUser: From<T::Claims>,
{
  let (mut parts, body) = req.into_parts();

  // Extract query parameters from the request
  let query = match Query::<TokenQuery>::try_from_uri(&parts.uri) {
    Ok(Query(q)) => q,
    Err(e) => {
      let msg = format!("Failed to parse access_token from query: {}", e);
      warn!("{}", msg);
      return (StatusCode::UNAUTHORIZED, msg).into_response();
    }
  };

  let token = query.access_token;

  match state.verify_token(&token) {
    Ok(claims) => {
      let user: AuthUser = claims.into();
      let mut req = Request::from_parts(parts, body);
      req.extensions_mut().insert(user);
      next.run(req).await
    }
    Err(e) => {
      let msg = format!("Token verification failed: {:?}", e);
      warn!("{}", msg);
      (StatusCode::UNAUTHORIZED, msg).into_response()
    }
  }
}
