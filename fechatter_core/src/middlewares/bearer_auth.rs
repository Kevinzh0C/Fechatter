use axum::{
  body::Body,
  extract::{FromRequestParts, Request, State},
  http::{HeaderMap, StatusCode},
  middleware::Next,
  response::{IntoResponse, Response},
};

use tracing::warn;

use crate::{
  error::CoreError,
  models::{
    jwt::{TokenConfigProvider, TokenManager, UserClaims},
    AuthUser, UserId, WorkspaceId,
  },
  TokenVerifier,
};

/// Generic `T` is any application state that implements
/// [`TokenVerifier`](crate::middlewares::TokenVerifier).  The function is intended to be
/// wrapped via `axum::middleware::from_fn_with_state` and therefore matches the
/// signature expected by that helper.
pub async fn verify_token_middleware<T>(
  State(state): State<T>,
  req: Request<Body>,
  next: Next,
) -> Response
where
  T: TokenVerifier + Clone + Send + Sync + 'static,
  AuthUser: From<T::Claims>,
{
  verify_token_middleware_with_user_type::<T, AuthUser>(State(state), req, next).await
}

/// Version of the middleware that allows specifying the user type
pub async fn verify_token_middleware_with_user_type<T, U>(
  State(state): State<T>,
  req: Request<Body>,
  next: Next,
) -> Response
where
  T: TokenVerifier + Clone + Send + Sync + 'static,
  U: From<T::Claims> + Clone + Send + Sync + 'static,
{
  let (mut parts, body) = req.into_parts();

  // Extract Authorization header manually
  let token = match parts.headers.get("authorization") {
    Some(header_value) => match header_value.to_str() {
      Ok(header_str) => {
        if header_str.starts_with("Bearer ") {
          header_str.strip_prefix("Bearer ").unwrap_or("").to_string()
        } else {
          let msg = "Authorization header must start with 'Bearer '".to_string();
          warn!("{}", msg);
          return (StatusCode::UNAUTHORIZED, msg).into_response();
        }
      }
      Err(e) => {
        let msg = format!("Invalid Authorization header format: {}", e);
        warn!("{}", msg);
        return (StatusCode::UNAUTHORIZED, msg).into_response();
      }
    },
    None => {
      let msg = "Authorization header missing".to_string();
      warn!("{}", msg);
      return (StatusCode::UNAUTHORIZED, msg).into_response();
    }
  };

  match state.verify_token(&token) {
    Ok(claims) => {
      let user: U = claims.into();
      let mut req = Request::from_parts(parts, body);
      req.extensions_mut().insert(user);
      next.run(req).await
    }
    Err(e) => (StatusCode::UNAUTHORIZED, format!("{:?}", e)).into_response(),
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    models::jwt::{TokenManager, UserClaims},
    models::{UserId, WorkspaceId},
  };

  use super::*;

  use anyhow::Result;
  use async_trait::async_trait;
  use axum::{body::Body, middleware::from_fn_with_state, routing::get, Router};

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
    use crate::error::CoreError;
    use crate::models::jwt::{
      RefreshToken, RefreshTokenRepository, ReplaceTokenPayload, StoreTokenPayload,
    };
    use crate::models::jwt::{TokenConfigProvider, TokenManager};
    use crate::models::User;
    use chrono::Utc;
    use std::sync::Arc;

    // Create a simplified mock RefreshTokenRepository
    struct MockRefreshTokenRepository;

    #[async_trait]
    impl RefreshTokenRepository for MockRefreshTokenRepository {
      async fn find_by_token(&self, _raw_token: &str) -> Result<Option<RefreshToken>, CoreError> {
        Ok(None)
      }

      async fn replace(&self, _payload: ReplaceTokenPayload) -> Result<RefreshToken, CoreError> {
        // Use a static string to avoid lifetime issues
        Err(CoreError::Internal("Not implemented".to_string()))
      }

      async fn revoke(&self, _token_id: i64) -> Result<(), CoreError> {
        Ok(())
      }

      async fn revoke_all_for_user(&self, _user_id: UserId) -> Result<(), CoreError> {
        Ok(())
      }

      async fn create(&self, _payload: StoreTokenPayload) -> Result<RefreshToken, CoreError> {
        Ok(RefreshToken {
          id: 1,
          user_id: UserId::new(1),
          token_hash: "test_hash".to_string(),
          expires_at: Utc::now() + chrono::Duration::hours(1),
          issued_at: Utc::now(),
          revoked: false,
          replaced_by: None,
          user_agent: None,
          ip_address: None,
          absolute_expires_at: Utc::now() + chrono::Duration::days(30),
        })
      }
    }

    // Define test constants
    const JWT_LEEWAY: u64 = 60;
    const JWT_AUDIENCE: &str = "fechatter-web";
    const JWT_ISSUER: &str = "fechatter-server";

    // Test TokenConfig - using keys read from files
    struct AuthConfig {
      sk: String,
      pk: String,
    }

    impl TokenConfigProvider for AuthConfig {
      fn get_encoding_key_pem(&self) -> &str {
        &self.sk
      }
      fn get_decoding_key_pem(&self) -> &str {
        &self.pk
      }
      fn get_jwt_leeway(&self) -> u64 {
        JWT_LEEWAY
      }
      fn get_jwt_audience(&self) -> Option<&str> {
        Some(JWT_AUDIENCE)
      }
      fn get_jwt_issuer(&self) -> Option<&str> {
        Some(JWT_ISSUER)
      }
    }

    // Intelligently detect key file locations
    let (encoding_path, decoding_path) = crate::middlewares::find_key_files();

    let auth_config = AuthConfig {
      sk: std::fs::read_to_string(encoding_path)?,
      pk: std::fs::read_to_string(decoding_path)?,
    };

    // Use mock repository
    let refresh_token_repository = Arc::new(MockRefreshTokenRepository);

    // Create TokenManager instance for testing
    let token_manager = match TokenManager::from_config(&auth_config, refresh_token_repository) {
      Ok(tm) => tm,
      Err(e) => {
        eprintln!("Failed to create TokenManager: {:?}", e);
        return Err(anyhow::anyhow!("Failed to create TokenManager: {:?}", e));
      }
    };

    // Create test application state
    let state = Appstate {
      inner: Arc::new(AppstateInner { token_manager }),
    };

    // Create test user
    let user = User {
      id: UserId::new(1),
      fullname: "Test User".to_string(),
      email: "test@example.com".to_string(),
      password_hash: Some("".to_string()),
      status: crate::models::UserStatus::Active,
      created_at: chrono::Utc::now(),
      workspace_id: WorkspaceId::new(1),
    };

    // Set up test router
    let app = Router::new()
      .route("/api", get(handler))
      .layer(from_fn_with_state(
        state.clone(),
        verify_token_middleware::<Appstate>,
      ));

    // Generate JWT token
    let token = match state.inner.token_manager.generate_token_for_user(&user) {
      Ok(t) => {
        println!("Successfully generated token: {}", t);
        t
      }
      Err(e) => {
        eprintln!("Failed to generate token: {:?}", e);
        return Err(anyhow::anyhow!("Failed to generate token: {:?}", e));
      }
    };

    // Verify token validity
    match state.inner.token_manager.verify_token(&token) {
      Ok(claims) => println!(
        "Token verification succeeded! Claims user id: {}",
        claims.id
      ),
      Err(e) => {
        eprintln!("ERROR: Token verification failed: {:?}", e);
        // Don't return error here, continue with request processing
      }
    }

    // Execute test request with Authorization header
    let req = Request::builder()
      .uri("/api")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())?;

    println!("Sending request with Authorization: Bearer {}", token);

    let response = app.oneshot(req).await.unwrap();
    println!("Response status: {:?}", response.status());

    // Verify response status is 200 OK
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
  }
}
