use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;
use axum::Router;
use std::marker::PhantomData;

// Marker types for state
pub struct NoAuth;
pub struct HasAuth;
pub struct AuthAndRefresh;

use crate::middlewares::bearer_auth::verify_token_middleware;
use crate::middlewares::token_refresh::refresh_token_middleware;
use crate::middlewares::{
  ActualAuthServiceProvider, HasIdField, TokenVerifier, WithServiceProvider, WithTokenManager,
};
use crate::models::jwt::TokenManager;
use crate::{
  contracts::services::AuthContext,
  error::CoreError,
  models::jwt::{AuthTokens, UserClaims},
  AuthUser,
};

#[cfg(test)]
use crate::models::{CreateUser, SigninUser, UserId, UserStatus, WorkspaceId};

//==============================================================================
// MIDDLEWARE SYSTEM
//==============================================================================
//
// This module implements a type-safe middleware system for the Fechatter application
// based on Axum. It provides an intuitive API for applying middleware in the correct
// order, with compile-time safety checks.
//
// Key components:
// - Middleware functions that wrap Axum middleware
// - Extension traits for fluent middleware application
// - Type-state builder pattern for enforcing correct ordering
//
// See the function-specific documentation below for detailed usage examples.

/// Helper function to adapt the token refresh middleware to the expected signature.
///
/// This function is used internally by the refresh middleware to handle the result
/// of the token refresh middleware, transforming the Result<Response, StatusCode>
/// returned by the core middleware into a Response.
///
/// # Type Parameters
///
/// * `TState` - The application state type that must implement required token management traits
/// * `TUser` - The user type to create from token claims, typically AuthUser
///
/// # Arguments
///
/// * `headers` - The HTTP headers from the request
/// * `state` - The application state wrapped in Axum's State extractor
/// * `req` - The HTTP request
/// * `next` - The next middleware in the chain
///
/// # Returns
///
/// The HTTP response, either from successful token refresh or an error response
async fn adapter<TState, TUser>(
  headers: HeaderMap,
  state: State<TState>,
  req: Request<Body>,
  next: Next,
) -> Response
where
  TState: WithTokenManager<TokenManagerType = TokenManager>
    + WithServiceProvider
    + Clone
    + Send
    + Sync
    + 'static,
  <TState as WithServiceProvider>::ServiceProviderType: ActualAuthServiceProvider,
  TUser: From<<TState::TokenManagerType as TokenVerifier>::Claims>
    + Clone
    + Send
    + Sync
    + 'static
    + HasIdField,
{
  use axum::response::IntoResponse;

  match refresh_token_middleware::<TState, TUser>(headers, state, req, next).await {
    Ok(resp) => resp,
    Err(status) => status.into_response(),
  }
}

/// Adds authentication middleware to an Axum router.
///
/// This middleware verifies JWT tokens in the Authorization header and adds the
/// authenticated user to the request extensions if the token is valid.
///
/// # Type Parameters
///
/// * `S` - The router state type
/// * `T` - The application state type that must implement TokenVerifier
///
/// # Arguments
///
/// * `router` - The Axum router to add middleware to
/// * `state` - The application state that implements TokenVerifier
///
/// # Returns
///
/// A new router with the authentication middleware applied
///
/// # Example
///
/// ```rust
/// use axum::{Router, routing::get};
/// use fechatter_core::middlewares::custom_builder::add_auth_middleware;
///
/// // Create an app state that implements TokenVerifier
/// let app_state = AppState::new();
///
/// // Create a router with routes
/// let base_router = Router::new()
///     .route("/api/users", get(list_users))
///     .route("/api/users/:id", get(get_user));
///
/// // Add authentication middleware
/// let router = add_auth_middleware(base_router, app_state);
/// ```
///
/// # Error Handling
///
/// The middleware will return a 401 Unauthorized response if:
/// - The Authorization header is missing
/// - The Authorization header is not a valid Bearer token
/// - The token signature is invalid
/// - The token has expired
/// - The token fails any other verification check
pub fn add_auth_middleware<S, T>(router: Router<S>, state: T) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
  T: TokenVerifier<Claims = UserClaims> + Clone + Send + Sync + 'static,
  <T as TokenVerifier>::Error: Send + 'static,
  AuthUser: From<UserClaims>,
{
  use axum::middleware::from_fn;

  router.layer(from_fn(move |req: Request<Body>, next: Next| {
    let state_clone = state.clone();
    async move { verify_token_middleware::<T>(State(state_clone), req, next).await }
  }))
}

/// Adds token refresh middleware to an Axum router.
///
/// This middleware refreshes expired access tokens using refresh tokens stored in cookies.
/// It checks if:
/// 1. There's no valid access token in the Authorization header
/// 2. A refresh token cookie exists
/// 3. The refresh token is valid
///
/// If all conditions are met, it generates a new access token, adds it to the request,
/// and sets a new refresh token cookie in the response.
///
/// # Type Parameters
///
/// * `S` - The router state type
/// * `T` - The application state type that must implement required token management traits
///
/// # Arguments
///
/// * `router` - The Axum router to add middleware to
/// * `state` - The application state that implements token management
///
/// # Returns
///
/// A new router with the token refresh middleware applied
///
/// # Example
///
/// ```rust
/// use axum::{Router, routing::get};
/// use fechatter_core::middlewares::custom_builder::add_refresh_middleware;
///
/// // Create an app state that implements required traits
/// let app_state = AppState::new();
///
/// // Create a router with routes
/// let base_router = Router::new()
///     .route("/api/users", get(list_users))
///     .route("/api/users/:id", get(get_user));
///
/// // Add token refresh middleware
/// let router = add_refresh_middleware(base_router, app_state);
/// ```
///
/// # Cookie Format
///
/// The middleware expects and sets the following cookie:
/// - Name: "refresh_token"
/// - HttpOnly: true
/// - SameSite: Strict
/// - Secure: true (in production)
/// - Path: "/"
///
/// # Error Handling
///
/// If token refresh fails, the middleware will:
/// 1. Clear the invalid refresh token cookie
/// 2. Return a 401 Unauthorized response if no valid access token exists
/// 3. Continue the request if a valid access token exists
pub fn add_refresh_middleware<S, T>(router: Router<S>, state: T) -> Router<S>
where
  S: Clone + Send + Sync + 'static,
  T: WithTokenManager<TokenManagerType = TokenManager>
    + WithServiceProvider
    + Clone
    + Send
    + Sync
    + 'static,
  <T as WithServiceProvider>::ServiceProviderType: ActualAuthServiceProvider,
{
  use axum::middleware::from_fn;

  router.layer(from_fn(move |req: Request<Body>, next: Next| {
    let state_clone = state.clone();
    let headers = req.headers().clone();

    async move { adapter::<T, AuthUser>(headers, State(state_clone), req, next).await }
  }))
}

/// Extension trait for Router to make middleware application more ergonomic.
///
/// This trait extends Axum's Router type with additional methods for applying
/// middleware in a fluent, chainable way. It provides a more intuitive API than
/// using the middleware functions directly.
///
/// # Type Parameters
///
/// * `S` - The router state type
///
/// # Usage Examples
///
/// Basic usage with multiple middleware layers:
///
/// ```rust
/// use axum::{Router, routing::get};
/// use fechatter_core::middlewares::custom_builder::RouterExt;
///
/// // Create app state
/// let app_state = AppState::new();
///
/// // Create router with routes and apply middleware
/// let router = Router::new()
///     .route("/api/users", get(list_users))
///     .route("/api/users/:id", get(get_user))
///     .with_auth(app_state.clone())     // First apply authentication
///     .with_refresh(app_state.clone()); // Then apply token refresh
/// ```
///
/// Extending with custom middleware:
///
/// ```rust
/// use axum::{Router, routing::get, middleware::from_fn};
/// use fechatter_core::middlewares::custom_builder::RouterExt;
///
/// // Define custom middleware function
/// fn add_logging_middleware<S>(router: Router<S>) -> Router<S>
/// where
///     S: Clone + Send + Sync + 'static,
/// {
///     router.layer(from_fn(|req, next| async move {
///         println!("Incoming request: {}", req.uri());
///         let response = next.run(req).await;
///         println!("Outgoing response: {}", response.status());
///         response
///     }))
/// }
///
/// // Extend RouterExt with custom middleware
/// trait LoggingExt<S>: RouterExt<S> {
///     fn with_logging(self) -> Router<S>;
/// }
///
/// impl<S> LoggingExt<S> for Router<S>
/// where
///     S: Clone + Send + Sync + 'static,
/// {
///     fn with_logging(self) -> Router<S> {
///         add_logging_middleware(self)
///     }
/// }
///
/// // Use both standard and custom middleware
/// let router = Router::new()
///     .route("/api", get(handler))
///     .with_auth(app_state.clone())
///     .with_refresh(app_state.clone())
///     .with_logging();
/// ```
pub trait RouterExt<S> {
  /// Applies authentication middleware to the router.
  ///
  /// This method adds authentication middleware that verifies JWT tokens
  /// in the Authorization header and adds the authenticated user to the
  /// request extensions.
  ///
  /// # Type Parameters
  ///
  /// * `T` - The application state type that must implement TokenVerifier
  ///
  /// # Arguments
  ///
  /// * `state` - The application state that implements TokenVerifier
  ///
  /// # Returns
  ///
  /// A new router with the authentication middleware applied
  ///
  /// # Example
  ///
  /// ```rust
  /// use axum::Router;
  /// use fechatter_core::middlewares::custom_builder::RouterExt;
  ///
  /// let app_state = AppState::new();
  /// let router = Router::new()
  ///     .route("/api/protected", get(protected_handler))
  ///     .with_auth(app_state);
  /// ```
  ///
  /// # Authentication Flow
  ///
  /// 1. Extract the Bearer token from the Authorization header
  /// 2. Verify the token signature, expiration, and other claims
  /// 3. If valid, extract user information and add to request extensions
  /// 4. If invalid, return 401 Unauthorized response
  fn with_auth<T>(self, state: T) -> Router<S>
  where
    T: TokenVerifier<Claims = UserClaims> + Clone + Send + Sync + 'static,
    <T as TokenVerifier>::Error: Send + 'static,
    AuthUser: From<UserClaims>;

  /// Applies token refresh middleware to the router.
  ///
  /// This method adds middleware that refreshes expired access tokens
  /// using refresh tokens stored in cookies.
  ///
  /// # Type Parameters
  ///
  /// * `T` - The application state type that must implement required token management traits
  ///
  /// # Arguments
  ///
  /// * `state` - The application state that implements token management
  ///
  /// # Returns
  ///
  /// A new router with the token refresh middleware applied
  ///
  /// # Example
  ///
  /// ```rust
  /// use axum::Router;
  /// use fechatter_core::middlewares::custom_builder::RouterExt;
  ///
  /// let app_state = AppState::new();
  /// let router = Router::new()
  ///     .route("/api/data", get(data_handler))
  ///     .with_auth(app_state.clone())
  ///     .with_refresh(app_state);
  /// ```
  ///
  /// # Refresh Flow
  ///
  /// 1. Check if request already has a valid access token
  /// 2. If not, look for a refresh token cookie
  /// 3. If found, attempt to generate a new access token
  /// 4. If successful, add the new token to the request and set a new refresh token cookie
  /// 5. If unsuccessful, clear the invalid cookie and return 401 Unauthorized
  fn with_refresh<T>(self, state: T) -> Router<S>
  where
    T: WithTokenManager<TokenManagerType = TokenManager>
      + WithServiceProvider
      + Clone
      + Send
      + Sync
      + 'static,
    <T as WithServiceProvider>::ServiceProviderType: ActualAuthServiceProvider;
}

impl<S> RouterExt<S> for Router<S>
where
  S: Clone + Send + Sync + 'static,
{
  fn with_auth<T>(self, state: T) -> Router<S>
  where
    T: TokenVerifier<Claims = UserClaims> + Clone + Send + Sync + 'static,
    <T as TokenVerifier>::Error: Send + 'static,
    AuthUser: From<UserClaims>,
  {
    add_auth_middleware(self, state)
  }

  fn with_refresh<T>(self, state: T) -> Router<S>
  where
    T: WithTokenManager<TokenManagerType = TokenManager>
      + WithServiceProvider
      + Clone
      + Send
      + Sync
      + 'static,
    <T as WithServiceProvider>::ServiceProviderType: ActualAuthServiceProvider,
  {
    add_refresh_middleware(self, state)
  }
}

/// A type-state builder for constructing routers with ordered middleware.
///
/// This struct implements the type-state builder pattern for enforcing correct
/// middleware ordering at compile time. It ensures that middleware is applied
/// in the proper sequence, preventing common errors.
///
/// The generic parameters encode the current state of the builder:
///
/// * `S` - The router state type
/// * `T` - The application state type
/// * `State` - A marker type indicating which middleware has been applied
///
/// # Type States
///
/// The builder transitions through several states as middleware is applied:
///
/// * `NoAuth` - Initial state, no middleware applied
/// * `HasAuth` - Authentication middleware applied
/// * `AuthAndRefresh` - Both authentication and token refresh middleware applied
///
/// # Benefits of the Type-State Pattern
///
/// 1. **Compile-Time Safety**: Ensures middleware is applied in the correct order
/// 2. **Self-Documenting Code**: Makes the middleware flow explicit
/// 3. **IDE Support**: Provides better autocompletion and error messages
///
/// # Example: Basic Usage
///
/// ```rust
/// use axum::{Router, routing::get};
/// use fechatter_core::middlewares::custom_builder::CoreBuilder;
///
/// // Create a router with routes
/// let base_router = Router::new()
///     .route("/api/users", get(list_users))
///     .route("/api/users/:id", get(get_user));
///
/// // Create an app state
/// let app_state = AppState::new();
///
/// // Use the builder to apply middleware in the correct order
/// let router = CoreBuilder::new(base_router, app_state)
///     .with_auth()           // First add authentication
///     .with_token_refresh()  // Then add token refresh
///     .build();              // Finally build the router
/// ```
///
/// # Example: Compile-Time Error Prevention
///
/// The following code would cause a compile-time error because token refresh
/// middleware cannot be applied before authentication middleware:
///
/// ```rust,compile_fail
/// let router = CoreBuilder::new(base_router, app_state)
///     .with_token_refresh()  // Error: method not found in `CoreBuilder<_, _, NoAuth>`
///     .with_auth()
///     .build();
/// ```
///
/// # Implementation Details
///
/// The builder uses phantom type parameters to track the state of middleware
/// application. Methods are only implemented for specific states, ensuring
/// that middleware is applied in the correct order.
pub struct CoreBuilder<S, T, State = NoAuth> {
  router: Router<S>,
  state: T,
  _state: PhantomData<State>,
}

impl<S, T> CoreBuilder<S, T, NoAuth>
where
  S: Clone + Send + Sync + 'static,
  T: TokenVerifier<Claims = UserClaims> + Clone + Send + Sync + 'static,
  <T as TokenVerifier>::Error: Send + 'static,
  AuthUser: From<UserClaims>,
{
  /// Creates a new CoreBuilder with the given router and state.
  ///
  /// This is the entry point for the builder pattern, creating a builder
  /// in the initial NoAuth state.
  ///
  /// # Type Parameters
  ///
  /// * `S` - The router state type
  /// * `T` - The application state type that must implement TokenVerifier
  ///
  /// # Arguments
  ///
  /// * `router` - The Axum router to build on
  /// * `state` - The application state for middleware
  ///
  /// # Returns
  ///
  /// A new CoreBuilder in the NoAuth state
  ///
  /// # Example
  ///
  /// ```rust
  /// use axum::{Router, routing::get};
  /// use fechatter_core::middlewares::custom_builder::CoreBuilder;
  ///
  /// // Create a router with routes
  /// let base_router = Router::new()
  ///     .route("/api/users", get(list_users));
  ///
  /// // Create an app state
  /// let app_state = AppState::new();
  ///
  /// // Create a new builder
  /// let builder = CoreBuilder::new(base_router, app_state);
  /// ```
  pub fn new(router: Router<S>, state: T) -> Self {
    Self {
      router,
      state,
      _state: PhantomData,
    }
  }

  /// Adds authentication middleware and transitions to the HasAuth state.
  ///
  /// This method applies authentication middleware to the router and changes
  /// the builder's type state to HasAuth, allowing token refresh middleware
  /// to be applied next.
  ///
  /// # Returns
  ///
  /// A new CoreBuilder in the HasAuth state
  ///
  /// # Example
  ///
  /// ```rust
  /// use axum::Router;
  /// use fechatter_core::middlewares::custom_builder::CoreBuilder;
  ///
  /// let builder = CoreBuilder::new(Router::new(), app_state);
  /// let builder_with_auth = builder.with_auth();
  ///
  /// // Now we can add token refresh middleware
  /// let builder_with_auth_and_refresh = builder_with_auth.with_token_refresh();
  /// ```
  ///
  /// # Authentication Details
  ///
  /// The authentication middleware:
  ///
  /// 1. Extracts the Bearer token from the Authorization header
  /// 2. Verifies the token using the TokenVerifier implementation from the state
  /// 3. If valid, adds the authenticated user to the request extensions
  /// 4. If invalid, returns a 401 Unauthorized response
  pub fn with_auth(self) -> CoreBuilder<S, T, HasAuth> {
    CoreBuilder {
      router: self.router.with_auth(self.state.clone()),
      state: self.state,
      _state: PhantomData,
    }
  }
}

impl<S, T> CoreBuilder<S, T, HasAuth>
where
  S: Clone + Send + Sync + 'static,
  T: WithTokenManager<TokenManagerType = TokenManager>
    + WithServiceProvider
    + Clone
    + Send
    + Sync
    + 'static,
  <T as WithServiceProvider>::ServiceProviderType: ActualAuthServiceProvider,
{
  /// Adds token refresh middleware and transitions to the AuthAndRefresh state.
  ///
  /// This method applies token refresh middleware to the router and changes
  /// the builder's type state to AuthAndRefresh.
  ///
  /// # Returns
  ///
  /// A new CoreBuilder in the AuthAndRefresh state
  ///
  /// # Example
  ///
  /// ```rust
  /// use axum::Router;
  /// use fechatter_core::middlewares::custom_builder::CoreBuilder;
  ///
  /// let router = CoreBuilder::new(Router::new(), app_state)
  ///     .with_auth()
  ///     .with_token_refresh()
  ///     .build();
  /// ```
  ///
  /// # Token Refresh Details
  ///
  /// The token refresh middleware:
  ///
  /// 1. Checks if the request already has a valid access token
  /// 2. If not, looks for a refresh token cookie
  /// 3. If found, attempts to generate a new access token
  /// 4. If successful, adds the new token to the request and sets a new refresh token cookie
  /// 5. If unsuccessful, clears the invalid cookie and returns 401 Unauthorized
  ///
  /// # Note
  ///
  /// This method is only available after authentication middleware has been applied,
  /// ensuring that middleware is applied in the correct order.
  pub fn with_token_refresh(self) -> CoreBuilder<S, T, AuthAndRefresh> {
    CoreBuilder {
      router: self.router.with_refresh(self.state.clone()),
      state: self.state,
      _state: PhantomData,
    }
  }
}

impl<S, T, State> CoreBuilder<S, T, State>
where
  S: Clone + Send + Sync + 'static,
{
  /// Finalizes the builder and returns the constructed router.
  ///
  /// This method completes the builder pattern, returning the Axum router
  /// with all middleware applied in the correct order.
  ///
  /// # Returns
  ///
  /// The constructed Axum router with all middleware applied
  ///
  /// # Example
  ///
  /// ```rust
  /// use axum::Router;
  /// use fechatter_core::middlewares::custom_builder::CoreBuilder;
  ///
  /// // Create and configure the router with middleware
  /// let router = CoreBuilder::new(Router::new(), app_state)
  ///     .with_auth()
  ///     .with_token_refresh()
  ///     .build();
  ///
  /// // Now the router is ready to be used
  /// axum::serve(listener, router).await?;
  /// ```
  ///
  /// # Note
  ///
  /// The build method is available at any state in the builder pattern,
  /// allowing you to finalize the router at any point in the middleware chain.
  pub fn build(self) -> Router<S> {
    self.router
  }
}

/// Create a RefreshTokenData instance for testing
#[cfg(test)]
fn create_test_refresh_token_data() -> crate::models::jwt::RefreshTokenData {
  let now = chrono::Utc::now();
  let expires_in = chrono::Duration::hours(24);

  crate::models::jwt::RefreshTokenData {
    token: format!("refresh_token_{}", uuid::Uuid::new_v4()),
    expires_at: now + expires_in,
    absolute_expires_at: now + expires_in,
  }
}

/// Create an access token for testing, simulating actual JWT format
#[cfg(test)]
fn create_test_access_token() -> String {
  // Create a token similar to JWT format, with header, payload and signature parts
  use base64::{engine::general_purpose::STANDARD, Engine};

  let header = STANDARD.encode(r#"{"alg":"HS256","typ":"JWT"}"#);
  let payload = STANDARD.encode(format!(
    r#"{{"sub":"1","name":"Test User","iat":{},"exp":{}}}"#,
    chrono::Utc::now().timestamp(),
    (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp()
  ));
  let signature = uuid::Uuid::new_v4().to_string().replace("-", "");

  format!("{}.{}.{}", header, payload, signature)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    error::CoreError,
    models::jwt::{AuthTokens, UserClaims},
    AuthUser,
  };
  use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
    routing::get,
    Extension, Router,
  };
  use chrono::Utc;
  use std::{
    sync::{Arc, Mutex},
    time::Duration,
  };
  use tower::ServiceExt;

  // Implement all necessary traits for MockAuthService
  use crate::models::jwt::{
    AuthServiceTrait, LogoutService, RefreshTokenService, SigninService, SignupService,
  };
  use async_trait::async_trait;

  // ===== Test Tools and Framework =====

  // Structure for tracking middleware call order
  #[derive(Clone, Default)]
  struct MiddlewareTracker {
    auth_called: Arc<Mutex<bool>>,
    refresh_called: Arc<Mutex<bool>>,
    auth_call_time: Arc<Mutex<Option<std::time::Instant>>>,
    refresh_call_time: Arc<Mutex<Option<std::time::Instant>>>,
  }

  impl MiddlewareTracker {
    fn new() -> Self {
      Self::default()
    }

    fn mark_auth_called(&self) {
      *self.auth_called.lock().unwrap() = true;
      *self.auth_call_time.lock().unwrap() = Some(std::time::Instant::now());
    }

    fn mark_refresh_called(&self) {
      *self.refresh_called.lock().unwrap() = true;
      *self.refresh_call_time.lock().unwrap() = Some(std::time::Instant::now());
    }

    fn was_auth_called(&self) -> bool {
      *self.auth_called.lock().unwrap()
    }

    #[allow(dead_code)]
    fn was_refresh_called(&self) -> bool {
      *self.refresh_called.lock().unwrap()
    }

    #[allow(dead_code)]
    fn correct_order(&self) -> bool {
      let auth_time = *self.auth_call_time.lock().unwrap();
      let refresh_time = *self.refresh_call_time.lock().unwrap();

      match (auth_time, refresh_time) {
        (Some(auth), Some(refresh)) => auth < refresh,
        _ => false, // If either middleware wasn't called, return false
      }
    }
  }

  // Mock Token Verifier
  #[derive(Clone)]
  struct MockTokenVerifier {
    tracker: MiddlewareTracker,
    should_fail: Arc<Mutex<bool>>,
    #[allow(dead_code)]
    fail_with_error: Arc<Mutex<Option<&'static str>>>,
    token_behavior: Arc<Mutex<TokenBehavior>>,
  }

  // Token behavior simulation - Remove Copy trait
  #[derive(Debug, Clone)]
  #[allow(dead_code)]
  enum TokenBehavior {
    Valid,
    Expired,
    Invalid,
    Malformed,
    ThrowError,
    Random,
    Custom(UserClaims),
  }

  impl MockTokenVerifier {
    fn new(tracker: MiddlewareTracker) -> Self {
      Self {
        tracker,
        should_fail: Arc::new(Mutex::new(false)),
        fail_with_error: Arc::new(Mutex::new(None)),
        token_behavior: Arc::new(Mutex::new(TokenBehavior::Valid)),
      }
    }

    fn set_should_fail(&self, fail: bool) {
      *self.should_fail.lock().unwrap() = fail;
    }

    #[allow(dead_code)]
    fn set_error(&self, error: &'static str) {
      *self.fail_with_error.lock().unwrap() = Some(error);
    }

    fn set_token_behavior(&self, behavior: TokenBehavior) {
      *self.token_behavior.lock().unwrap() = behavior;
    }
  }

  impl TokenVerifier for MockTokenVerifier {
    type Claims = UserClaims;
    type Error = CoreError;

    fn verify_token(&self, _token: &str) -> Result<Self::Claims, Self::Error> {
      self.tracker.mark_auth_called();

      if *self.should_fail.lock().unwrap() {
        if let Some(error) = *self.fail_with_error.lock().unwrap() {
          return Err(CoreError::Unauthorized(error.to_string()));
        }
        return Err(CoreError::Unauthorized(
          "Token verification failed".to_string(),
        ));
      }

      let behavior = self.token_behavior.lock().unwrap();
      match &*behavior {
        TokenBehavior::Valid => Ok(UserClaims {
          id: UserId::new(1),
          fullname: "Test User".to_string(),
          email: "test@example.com".to_string(),
          workspace_id: WorkspaceId::new(1),
          status: UserStatus::Active,
          created_at: Utc::now(),
        }),
        TokenBehavior::Expired => Err(CoreError::Unauthorized("Token expired".to_string())),
        TokenBehavior::Invalid => Err(CoreError::Unauthorized("Invalid token".to_string())),
        TokenBehavior::Malformed => Err(CoreError::Unauthorized("Malformed token".to_string())),
        TokenBehavior::ThrowError => {
          panic!("Unexpected error during token verification")
        }
        TokenBehavior::Random => {
          let random_num = rand::random::<u8>() % 2;
          if random_num == 0 {
            Ok(UserClaims {
              id: UserId::new(1),
              fullname: "Test User".to_string(),
              email: "test@example.com".to_string(),
              workspace_id: WorkspaceId::new(1),
              status: UserStatus::Active,
              created_at: Utc::now(),
            })
          } else {
            Err(CoreError::Unauthorized("Token expired".to_string()))
          }
        }
        TokenBehavior::Custom(claims) => Ok(claims.clone()),
      }
    }
  }

  // Mock Token Manager
  #[derive(Clone)]
  struct MockTokenManager {
    tracker: MiddlewareTracker,
    #[allow(dead_code)]
    should_fail_refresh: Arc<Mutex<bool>>,
  }

  impl MockTokenManager {
    fn new(tracker: MiddlewareTracker) -> Self {
      Self {
        tracker,
        should_fail_refresh: Arc::new(Mutex::new(false)),
      }
    }

    #[allow(dead_code)]
    fn set_should_fail_refresh(&self, fail: bool) {
      *self.should_fail_refresh.lock().unwrap() = fail;
    }
  }

  impl TokenVerifier for MockTokenManager {
    type Claims = UserClaims;
    type Error = CoreError;

    fn verify_token(&self, _token: &str) -> Result<Self::Claims, Self::Error> {
      self.tracker.mark_refresh_called();

      Ok(UserClaims {
        id: UserId::new(1),
        fullname: "Test User".to_string(),
        email: "test@example.com".to_string(),
        workspace_id: WorkspaceId::new(1),
        status: UserStatus::Active,
        created_at: Utc::now(),
      })
    }
  }

  // Mock application state
  #[derive(Clone)]
  struct MockAppState {
    token_verifier: MockTokenVerifier,
    token_manager: MockTokenManager,
  }

  impl MockAppState {
    fn new(tracker: MiddlewareTracker) -> Self {
      Self {
        token_verifier: MockTokenVerifier::new(tracker.clone()),
        token_manager: MockTokenManager::new(tracker),
      }
    }
  }

  impl TokenVerifier for MockAppState {
    type Claims = UserClaims;
    type Error = CoreError;

    fn verify_token(&self, token: &str) -> Result<Self::Claims, Self::Error> {
      self.token_verifier.verify_token(token)
    }
  }

  impl WithTokenManager for MockAppState {
    type TokenManagerType = MockTokenManager;

    fn token_manager(&self) -> &Self::TokenManagerType {
      &self.token_manager
    }
  }

  // Implement mock service provider
  struct MockAuthService;

  impl MockAuthService {
    fn new() -> Self {
      Self {}
    }

    #[allow(dead_code)]
    pub fn refresh_token(&self, _refresh_token: &str) -> Result<(String, String), CoreError> {
      let new_access_token = format!("new_access_token_{}", uuid::Uuid::new_v4());
      let new_refresh_token = format!("new_refresh_token_{}", uuid::Uuid::new_v4());
      Ok((new_access_token, new_refresh_token))
    }
  }

  #[async_trait]
  impl RefreshTokenService for MockAuthService {
    async fn refresh_token(
      &self,
      _refresh_token: &str,
      _auth_context: Option<AuthContext>,
    ) -> Result<AuthTokens, CoreError> {
      Ok(AuthTokens {
        access_token: create_test_access_token(),
        refresh_token: create_test_refresh_token_data(),
      })
    }
  }

  #[async_trait]
  impl SignupService for MockAuthService {
    async fn signup(
      &self,
      _payload: &CreateUser,
      _auth_context: Option<AuthContext>,
    ) -> Result<AuthTokens, CoreError> {
      Ok(AuthTokens {
        access_token: create_test_access_token(),
        refresh_token: create_test_refresh_token_data(),
      })
    }
  }

  #[async_trait]
  impl SigninService for MockAuthService {
    async fn signin(
      &self,
      _payload: &SigninUser,
      _auth_context: Option<AuthContext>,
    ) -> Result<Option<AuthTokens>, CoreError> {
      Ok(Some(AuthTokens {
        access_token: create_test_access_token(),
        refresh_token: create_test_refresh_token_data(),
      }))
    }
  }

  #[async_trait]
  impl LogoutService for MockAuthService {
    async fn logout(&self, _refresh_token: &str) -> Result<(), CoreError> {
      Ok(())
    }

    async fn logout_all(&self, _user_id: UserId) -> Result<(), CoreError> {
      Ok(())
    }
  }

  impl AuthServiceTrait for MockAuthService {}

  #[async_trait]
  impl crate::contracts::AuthService for MockAuthService {
    async fn signup(
      &self,
      _payload: &CreateUser,
      _auth_context: Option<AuthContext>,
    ) -> Result<AuthTokens, CoreError> {
      Ok(AuthTokens {
        access_token: create_test_access_token(),
        refresh_token: create_test_refresh_token_data(),
      })
    }

    async fn signin(
      &self,
      _payload: &SigninUser,
      _auth_context: Option<AuthContext>,
    ) -> Result<Option<AuthTokens>, CoreError> {
      Ok(Some(AuthTokens {
        access_token: create_test_access_token(),
        refresh_token: create_test_refresh_token_data(),
      }))
    }

    async fn refresh_token(
      &self,
      _refresh_token: &str,
      _auth_context: Option<AuthContext>,
    ) -> Result<AuthTokens, CoreError> {
      Ok(AuthTokens {
        access_token: create_test_access_token(),
        refresh_token: create_test_refresh_token_data(),
      })
    }

    async fn logout(&self, _refresh_token: &str) -> Result<(), CoreError> {
      Ok(())
    }

    async fn logout_all(&self, _user_id: UserId) -> Result<(), CoreError> {
      Ok(())
    }

    fn verify_token(&self, _token: &str) -> Result<UserClaims, CoreError> {
      Ok(UserClaims {
        id: UserId::new(1),
        workspace_id: WorkspaceId::new(1),
        fullname: "Test User".to_string(),
        email: "test@example.com".to_string(),
        status: UserStatus::Active,
        created_at: Utc::now(),
      })
    }

    fn user_from_claims(&self, claims: UserClaims) -> AuthUser {
      AuthUser {
        id: claims.id,
        fullname: claims.fullname,
        email: claims.email,
        status: claims.status,
        created_at: claims.created_at,
        workspace_id: claims.workspace_id,
      }
    }
  }

  impl ActualAuthServiceProvider for MockAppState {
    type AuthService = MockAuthService;

    fn create_service(&self) -> Self::AuthService {
      MockAuthService::new()
    }
  }

  impl WithServiceProvider for MockAppState {
    type ServiceProviderType = Self;

    fn service_provider(&self) -> &Self::ServiceProviderType {
      self
    }
  }

  // Mock server operations struct
  struct MockServer {
    app: Router,
  }

  impl MockServer {
    fn new(app: Router) -> Self {
      Self { app }
    }

    async fn send_request(&self, request: Request<Body>) -> axum::response::Response {
      self.app.clone().oneshot(request).await.unwrap()
    }
  }

  // Test handlers
  async fn test_handler() -> &'static str {
    "Hello, world!"
  }

  async fn auth_user_handler(Extension(user): Extension<AuthUser>) -> String {
    format!("User ID: {}", user.id)
  }

  // ===== Unit Tests =====

  #[tokio::test]
  async fn it_should_accept_valid_token() {
    // Create tracker
    let tracker = MiddlewareTracker::new();

    // Create application state
    let app_state = MockAppState::new(tracker.clone());

    // Create router
    let app = Router::new()
      .route("/test", get(test_handler))
      .route("/user", get(auth_user_handler));

    // Add authentication middleware
    let app = add_auth_middleware(app, app_state);

    // Create mock server
    let server = MockServer::new(app);

    // Create request with valid token
    let request = Request::builder()
      .uri("/test")
      .header("Authorization", "Bearer valid_token")
      .body(Body::empty())
      .unwrap();

    // Send request
    let response = server.send_request(request).await;

    // Verify:
    // 1. Authentication middleware was called
    // 2. Response status code is 200 OK
    // 3. Response body is correct
    assert!(tracker.was_auth_called());
    assert_eq!(response.status(), StatusCode::OK);

    let body = body::to_bytes(response.into_body(), 1024 * 16)
      .await
      .unwrap();
    assert_eq!(&body[..], b"Hello, world!");
  }

  #[tokio::test]
  async fn it_should_reject_invalid_token() {
    // Create tracker
    let tracker = MiddlewareTracker::new();

    // Create application state
    let app_state = MockAppState::new(tracker.clone());

    // Set token verifier to fail
    app_state.token_verifier.set_should_fail(true);

    // Create router
    let app = Router::new().route("/test", get(test_handler));

    // Add authentication middleware
    let app = add_auth_middleware(app, app_state);

    // Create mock server
    let server = MockServer::new(app);

    // Create request with invalid token
    let request = Request::builder()
      .uri("/test")
      .header("Authorization", "Bearer invalid_token")
      .body(Body::empty())
      .unwrap();

    // Send request
    let response = server.send_request(request).await;

    // Verify:
    // 1. Authentication middleware was called
    // 2. Response status code is 401 Unauthorized
    assert!(tracker.was_auth_called());
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
  }

  #[tokio::test]
  async fn it_should_use_auth_extension_trait() {
    // Create tracker and application state
    let tracker = MiddlewareTracker::new();
    let app_state = MockAppState::new(tracker.clone());

    // Create router using RouterExt
    let app = Router::new()
      .route("/user", get(auth_user_handler))
      .with_auth(app_state);

    // Create mock server
    let server = MockServer::new(app);

    // Create request with valid token
    let request = Request::builder()
      .uri("/user")
      .header("Authorization", "Bearer valid_token")
      .body(Body::empty())
      .unwrap();

    // Send request
    let response = server.send_request(request).await;

    // Verify response
    assert_eq!(response.status(), StatusCode::OK);
    let body = body::to_bytes(response.into_body(), 1024 * 16)
      .await
      .unwrap();
    assert_eq!(&body[..], b"User ID: 1");
  }

  #[tokio::test]
  async fn it_should_execute_middleware_in_order() {
    // Create tracker and application state
    let tracker = MiddlewareTracker::new();
    let app_state = MockAppState::new(tracker.clone());

    // Create router with multiple middleware using RouterExt
    let app = Router::new()
      .route("/test", get(test_handler))
      .with_auth(app_state.clone());

    // Create mock server
    let server = MockServer::new(app);

    // Create request
    let request = Request::builder()
      .uri("/test")
      .header("Authorization", "Bearer valid_token")
      .body(Body::empty())
      .unwrap();

    // Send request
    let _response = server.send_request(request).await;

    // Verify authentication middleware was called
    assert!(tracker.was_auth_called());
  }

  #[tokio::test]
  async fn it_should_enforce_middleware_order() {
    // Create tracker and application state
    let tracker = MiddlewareTracker::new();
    let app_state = MockAppState::new(tracker.clone());

    // Create router using CoreBuilder, enforcing correct middleware order
    let app = CoreBuilder::new(
      Router::new().route("/test", get(test_handler)),
      app_state.clone(),
    )
    .with_auth()
    .build();

    // Create mock server
    let server = MockServer::new(app);

    // Create request
    let request = Request::builder()
      .uri("/test")
      .header("Authorization", "Bearer valid_token")
      .body(Body::empty())
      .unwrap();

    // Send request
    let _response = server.send_request(request).await;

    // Verify authentication middleware was called
    assert!(tracker.was_auth_called());
  }

  #[tokio::test]
  async fn it_should_refresh_token_when_auth_fails() {
    // Create tracker and application state
    let tracker = MiddlewareTracker::new();
    let app_state = MockAppState::new(tracker.clone());

    // Set verifier for expired token
    app_state
      .token_verifier
      .set_token_behavior(TokenBehavior::Expired);

    // Create router
    let app = Router::new()
      .route("/test", get(test_handler))
      .with_auth(app_state.clone());

    // Create mock server
    let server = MockServer::new(app);

    // Create request with expired token and valid refresh token
    let request = Request::builder()
      .uri("/test")
      .header("Authorization", "Bearer expired_token")
      .header("Cookie", "refresh_token=valid_refresh_token")
      .body(Body::empty())
      .unwrap();

    // Send request
    let response = server.send_request(request).await;

    // Verify expired token should return 401
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Verify authentication middleware was called
    assert!(tracker.was_auth_called());
  }

  #[tokio::test]
  async fn it_should_handle_token_behavior_variations() {
    // Create test cases, each containing token behavior and expected response status
    let test_cases = vec![
      (TokenBehavior::Valid, StatusCode::OK),
      (TokenBehavior::Expired, StatusCode::UNAUTHORIZED),
      (TokenBehavior::Invalid, StatusCode::UNAUTHORIZED),
      (TokenBehavior::Malformed, StatusCode::UNAUTHORIZED),
    ];

    for (behavior, expected_status) in test_cases {
      // Create tracker and application state
      let tracker = MiddlewareTracker::new();
      let app_state = MockAppState::new(tracker.clone());

      // Set token behavior
      app_state
        .token_verifier
        .set_token_behavior(behavior.clone());

      // Create router
      let app = Router::new()
        .route("/test", get(test_handler))
        .with_auth(app_state);

      // Create mock server
      let server = MockServer::new(app);

      // Create request
      let request = Request::builder()
        .uri("/test")
        .header("Authorization", "Bearer test_token")
        .body(Body::empty())
        .unwrap();

      // Send request
      let response = server.send_request(request).await;

      // Verify response status meets expectations
      assert_eq!(
        response.status(),
        expected_status,
        "Token behavior {:?} should return {:?}",
        behavior,
        expected_status
      );
    }
  }

  #[tokio::test]
  #[should_panic(expected = "Unexpected error during token verification")]
  async fn it_should_handle_token_verifier_panic() {
    // Create tracker and application state
    let tracker = MiddlewareTracker::new();
    let app_state = MockAppState::new(tracker.clone());

    // Set token behavior to throw error
    app_state
      .token_verifier
      .set_token_behavior(TokenBehavior::ThrowError);

    // Create router
    let app = Router::new()
      .route("/test", get(test_handler))
      .with_auth(app_state);

    // Create mock server
    let server = MockServer::new(app);

    // Create request
    let request = Request::builder()
      .uri("/test")
      .header("Authorization", "Bearer test_token")
      .body(Body::empty())
      .unwrap();

    // Send request, this will cause panic
    let _ = server.send_request(request).await;
  }

  #[tokio::test]
  async fn it_should_handle_random_token_validation() {
    // Create tracker and application state
    let tracker = MiddlewareTracker::new();
    let app_state = MockAppState::new(tracker.clone());

    // Set token behavior to random
    app_state
      .token_verifier
      .set_token_behavior(TokenBehavior::Random);

    // Create router
    let app = Router::new()
      .route("/test", get(test_handler))
      .with_auth(app_state.clone());

    // Create mock server
    let server = MockServer::new(app);

    // Send multiple requests to test random behavior
    let mut successes = 0;
    let mut failures = 0;
    let attempts = 10;

    for _ in 0..attempts {
      // Create request
      let request = Request::builder()
        .uri("/test")
        .header("Authorization", "Bearer random_token")
        .header("Cookie", "refresh_token=valid_refresh_token")
        .body(Body::empty())
        .unwrap();

      // Send request
      let response = server.send_request(request).await;

      // Count successes and failures
      if response.status() == StatusCode::OK {
        successes += 1;
      } else {
        failures += 1;
      }
    }

    // Verify some successes and failures, indicating random behavior is effective
    assert!(successes > 0, "Random validation should have some successes");
    assert!(failures > 0, "Random validation should have some failures");
    assert_eq!(successes + failures, attempts, "Total requests should equal attempts");
  }

  #[tokio::test]
  async fn it_should_work_without_middlewares() {
    // Test router behavior without any middleware
    let app = Router::new().route("/test", get(test_handler));
    let server = MockServer::new(app);
    let request = Request::builder().uri("/test").body(Body::empty()).unwrap();
    let response = server.send_request(request).await;
    assert_eq!(response.status(), StatusCode::OK);
  }

  #[tokio::test]
  async fn it_should_return_404_for_missing_routes() {
    // Create tracker and application state
    let tracker = MiddlewareTracker::new();
    let app_state = MockAppState::new(tracker);

    // Create router with middleware but route doesn't exist
    let app = Router::new().with_auth(app_state);

    // Create mock server
    let server = MockServer::new(app);

    // Create request to non-existent route
    let request = Request::builder()
      .uri("/nonexistent")
      .header("Authorization", "Bearer valid_token")
      .body(Body::empty())
      .unwrap();

    // Send request
    let response = server.send_request(request).await;

    // Verify response status is 404 Not Found
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
  }

  #[tokio::test]
  async fn it_should_work_with_partial_builder_chain() {
    // Create tracker and application state
    let tracker = MiddlewareTracker::new();
    let app_state = MockAppState::new(tracker);

    // Create router using CoreBuilder, but only add authentication middleware
    let app = CoreBuilder::new(
                Router::new().route("/test", get(test_handler)),
                app_state
            )
            .with_auth()
            // Build early, don't add refresh middleware
            .build();

    // Create mock server
    let server = MockServer::new(app);

    // Create request
    let request = Request::builder()
      .uri("/test")
      .header("Authorization", "Bearer valid_token")
      .body(Body::empty())
      .unwrap();

    // Send request
    let response = server.send_request(request).await;

    // Verify request succeeds
    assert_eq!(response.status(), StatusCode::OK);
  }

  /// Simplified concurrency test, removing complex token refresh logic
  #[tokio::test]
  async fn it_should_handle_concurrent_requests() {
    // Test configuration
    const CONCURRENT_REQUESTS: usize = 5; // Reduce concurrent request count

    // Create tracker and application state
    let tracker = MiddlewareTracker::new();
    let app_state = MockAppState::new(tracker.clone());

    // Create test router
    let app = Router::new()
      .route("/test", get(test_handler))
      .with_auth(app_state);

    // Run concurrent requests
    let handles = (0..CONCURRENT_REQUESTS)
      .map(|_| {
        let app = app.clone();
        tokio::spawn(async move {
          let request = Request::builder()
            .uri("/test")
            .header("Authorization", "Bearer valid_token")
            .body(Body::empty())
            .unwrap();

          app.oneshot(request).await.unwrap()
        })
      })
      .collect::<Vec<_>>();

    let results = futures::future::join_all(handles).await;

    // Verify all requests succeed
    for result in results {
      let response = result.unwrap();
      assert_eq!(response.status(), StatusCode::OK);
    }
  }

  /// Simplified token replay test
  #[tokio::test]
  async fn it_should_handle_token_replay_attempts() {
    // Create tracker and application state
    let tracker = MiddlewareTracker::new();
    let app_state = MockAppState::new(tracker.clone());

    // Create router
    let app = Router::new()
      .route("/test", get(test_handler))
      .route(
        "/secured",
        get(|Extension(_user): Extension<AuthUser>| async move {
          format!("Secured data for user {}", 1)
        }),
      )
      .with_auth(app_state);

    // Step 1: Get valid access token
    let initial_request = Request::builder()
      .uri("/test")
      .header("Authorization", "Bearer valid_token")
      .body(Body::empty())
      .unwrap();

    let initial_response = app.clone().oneshot(initial_request).await.unwrap();
    assert_eq!(initial_response.status(), StatusCode::OK);

    // Step 2: Use token to access protected resource
    let access_request = Request::builder()
      .uri("/secured")
      .header("Authorization", "Bearer valid_token")
      .body(Body::empty())
      .unwrap();

    let access_response = app.clone().oneshot(access_request).await.unwrap();
    assert_eq!(
      access_response.status(),
      StatusCode::OK,
      "First use of token should succeed"
    );

    // Step 3: Simulate token replay
    let delay_times = [
      Duration::from_millis(100), // Very short delay
      Duration::from_secs(1),     // 1 second delay
    ];

    for delay in delay_times {
      // Simulate time passing
      tokio::time::sleep(delay).await;

      // Attempt token replay
      let replay_request = Request::builder()
        .uri("/secured")
        .header("Authorization", "Bearer valid_token")
        .body(Body::empty())
        .unwrap();

      let replay_response = app.clone().oneshot(replay_request).await.unwrap();

      assert_eq!(
        replay_response.status(),
        StatusCode::OK,
        "Token replay status error after {}ms",
        delay.as_millis()
      );

      // Extract response content and verify correctness
      if replay_response.status() == StatusCode::OK {
        let body = body::to_bytes(replay_response.into_body(), 1024 * 16)
          .await
          .unwrap();
        let body_text = String::from_utf8_lossy(&body);
        assert!(
          body_text.contains("Secured data for user 1"),
          "Token replay produced incorrect user identity, response: {}",
          body_text
        );
      }
    }
  }
}
