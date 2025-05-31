use super::{TokenVerifier, WithServiceProvider, WithTokenManager};
use crate::models::jwt::TokenManager;
use crate::{error::CoreError, models::AuthUser, models::jwt::UserClaims};
use axum::{
  body::Body,
  extract::{Request, State},
  http::{HeaderMap, StatusCode},
  middleware::Next,
  response::Response,
};

// Marker trait for simple auth middleware (Bearer token verification)
#[async_trait::async_trait]
pub trait AuthMw<S>: Send + Sync + 'static {
  type State: TokenVerifier<Claims = UserClaims> + Clone + Send + Sync + 'static;

  async fn call(state: State<Self::State>, req: Request<Body>, next: Next) -> Response;
}

// Marker trait for refresh-token middleware
#[async_trait::async_trait]
pub trait RefreshMw<S>: Send + Sync + 'static {
  type State: WithTokenManager<TokenManagerType = TokenManager>
    + WithServiceProvider
    + Clone
    + Send
    + Sync
    + 'static;

  async fn call(
    headers: HeaderMap,
    state: State<Self::State>,
    req: Request<Body>,
    next: Next,
  ) -> Result<Response, StatusCode>;
}

// ---------------- wrapper structs implementing marker traits ----------------
use super::{bearer_auth, token_refresh};

pub struct BearerAuthMw;

#[async_trait::async_trait]
impl<S> AuthMw<S> for BearerAuthMw
where
  S: TokenVerifier<Claims = UserClaims> + Clone + Send + Sync + 'static,
  <S as TokenVerifier>::Error: Send + Sync + std::fmt::Debug,
  AuthUser: From<UserClaims>,
{
  type State = S;

  async fn call(state: State<Self::State>, req: Request<Body>, next: Next) -> Response {
    bearer_auth::verify_token_middleware(state, req, next).await
  }
}

pub struct RefreshTokenMw;

#[async_trait::async_trait]
impl<S> RefreshMw<S> for RefreshTokenMw
where
  S: WithTokenManager<TokenManagerType = TokenManager>
    + WithServiceProvider
    + Clone
    + Send
    + Sync
    + 'static,
  <S as WithServiceProvider>::ServiceProviderType: super::ActualAuthServiceProvider,
  AuthUser: From<<TokenManager as TokenVerifier>::Claims>,
  // No additional bounds on <S as TokenVerifier>::Error are required here because S isn't necessarily a TokenVerifier.
{
  type State = S;

  async fn call(
    headers: HeaderMap,
    state: State<Self::State>,
    req: Request<Body>,
    next: Next,
  ) -> Result<Response, StatusCode> {
    token_refresh::refresh_token_middleware::<S, AuthUser>(headers, state, req, next).await
  }
}
