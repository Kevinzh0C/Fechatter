use super::{HasIdField, WithServiceProvider, WithTokenManager};
use crate::{
  TokenVerifier,
  contracts::AuthContext,
  middlewares::ActualAuthServiceProvider,
  models::jwt::{RefreshTokenData, RefreshTokenService},
};
use axum::{
  body::Body,
  extract::State,
  http::{
    Request, StatusCode,
    header::{HeaderMap, HeaderValue, SET_COOKIE},
  },
  middleware::Next,
  response::{IntoResponse, Response},
};
use tracing::{debug, warn};

const AUTH_HEADER: &str = "Authorization";
const AUTH_COOKIE_NAME: &str = "refresh_token";

/// Refresh an expired **access token** using a `refresh_token` cookie.
///
/// Behaviour overview:
/// 1. If the request already carries a valid access token (Bearer header) the
///    middleware is a no-op.
/// 2. Otherwise it looks for `refresh_token` cookie, asks the configured auth
///    service to mint fresh tokens, validates the new access token and stores
///    an `AuthUser` extension for downstream handlers.
/// 3. On success it also sets an updated `Set-Cookie` header with the new
///    refresh token.
///
/// The function is highly generic so it can live close to domain code while
/// still being reusable:
/// * `AppState` must provide a token-manager & service-provider accessor.
/// * `UserType` is typically `AuthUser` but any type that implements
///   `From<Claims>` + [`HasIdField`] is accepted.
///
/// It returns a *plain* `Response` wrapped in `Result` so callers can decide
/// whether to translate failures to `IntoResponse` themselves.
/// Extract auth context from cookies and update tokens if necessary
pub async fn refresh_token_middleware<
  AppState, // Generic state type for the middleware
  UserType, // Generic user type that will be stored in extensions
>(
  headers: HeaderMap,
  State(state): State<AppState>,
  mut request: Request<Body>,
  next: Next,
) -> Result<Response, StatusCode>
where
  AppState: Clone + Send + Sync + 'static + WithServiceProvider + WithTokenManager,
  UserType: From<<<AppState as WithTokenManager>::TokenManagerType as TokenVerifier>::Claims>
    + Clone
    + Send
    + Sync
    + 'static
    + HasIdField,
  <<AppState as WithTokenManager>::TokenManagerType as TokenVerifier>::Error: std::fmt::Debug,
  // To use claims.id directly, this bound would be needed:
  // <<(AppState as WithTokenManager)::TokenManagerType as TokenVerifier)>::Claims: HasIdField + std::fmt::Debug,
{
  // Debug header information
  debug!("Headers in refresh_token_middleware: {:?}", headers);

  // Check if AuthUser extension already exists (from previous middleware)
  let existing_user = request.extensions().get::<UserType>().cloned();
  if let Some(_) = &existing_user {
    // If AuthUser struct (not the generic param) has id:
    // debug!("AuthUser extension already exists for user_id={}, preserving it", user_ext.id());
    // If AuthUser is just a generic param, it might not have .id. This needs clarification.
    // For now, assume AuthUser is the concrete struct and this debug line will be reviewed if AuthUser changes.
    return Ok(next.run(request).await);
  }

  // Check if cookie header exists and log it
  if let Some(cookie_header) = headers.get("cookie") {
    if let Ok(cookie_str) = cookie_header.to_str() {
      debug!("Cookie header: {}", cookie_str);
    }
  } else {
    debug!("No cookie header found");
  }

  // Get refresh token from cookie header
  let refresh_token = match get_cookie_value(&headers, AUTH_COOKIE_NAME) {
    Some(value) => {
      debug!("Found refresh_token cookie: {}", value);
      value
    }
    None => {
      // No refresh token, continue without refreshing
      debug!("No refresh_token cookie found, continuing without refresh");
      return Ok(next.run(request).await);
    }
  };

  // Check if there's already a valid access token
  let has_access_token = headers
    .get(AUTH_HEADER)
    .map(|h| h.to_str().unwrap_or_default().starts_with("Bearer "))
    .unwrap_or(false);

  // If we have an access token, no need to refresh
  if has_access_token {
    debug!("Access token present, no need to refresh");
    return Ok(next.run(request).await);
  }

  debug!("No access token found, attempting to refresh with cookie");

  // Extract user agent and IP for security checks
  let user_agent = request
    .headers()
    .get("user-agent")
    .and_then(|h| h.to_str().ok())
    .map(String::from);

  let ip_address = request
    .headers()
    .get("x-forwarded-for")
    .and_then(|h| h.to_str().ok())
    .map(String::from);

  // Create auth service using trait-based approach
  let auth_service = state.service_provider().create_service();

  // Try to refresh the token
  match auth_service
    .refresh_token(
      &refresh_token,
      Some(AuthContext {
        user_agent,
        ip_address,
      }),
    )
    .await
  {
    Ok(tokens) => {
      debug!("Token refresh successful");
      // Set the new auth token in request headers
      let auth_header = format!("Bearer {}", tokens.access_token);
      request.headers_mut().insert(
        AUTH_HEADER,
        HeaderValue::from_str(&auth_header).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
      );

      // Validate the newly generated token and add AuthUser extension
      match state.token_manager().verify_token(&tokens.access_token) {
        Ok(claims) => {
          // claims is of type <<T_mw as WithTokenManager>::TokenManagerType as TokenVerifier>::Claims
          // This Claims type needs an `id` field for the debug log, or the log needs to change.
          // debug!("Adding AuthUser extension from refreshed token for user_id={}", claims.id);
          let user_to_insert: UserType = claims.into();
          request.extensions_mut().insert(user_to_insert);
          debug!("AuthUser extension successfully added to request");
        }
        Err(e) => {
          warn!("Refreshed token validation failed: {:?}", e);
          return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
      }

      let refresh_cookie = create_refresh_cookie(&tokens.refresh_token);
      let mut response = next.run(request).await;
      response.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_str(&refresh_cookie).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
      );

      Ok(response)
    }
    Err(e) => {
      warn!("Token refresh failed: {:?}", e);
      if !has_access_token {
        debug!("No valid refresh token and no authorization token, returning 401");
        let expired_cookie = format!(
          "{}=; Path=/; Max-Age=0; HttpOnly; SameSite=Strict; Secure",
          AUTH_COOKIE_NAME
        );
        let mut response = StatusCode::UNAUTHORIZED.into_response();
        response.headers_mut().insert(
          SET_COOKIE,
          HeaderValue::from_str(&expired_cookie).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        );
        return Ok(response);
      }
      Ok(next.run(request).await)
    }
  }
}

/// Create a cookie string for the refresh token
fn create_refresh_cookie(token_data: &RefreshTokenData) -> String {
  let max_age = (token_data.expires_at - chrono::Utc::now())
    .num_seconds()
    .max(0)
    .try_into()
    .unwrap_or(0);

  // Set HTTP-only cookie with secure flag
  format!(
    "{}={}; Path=/; Max-Age={}; HttpOnly; SameSite=Strict; Secure",
    AUTH_COOKIE_NAME, token_data.token, max_age
  )
}

/// Get cookie value by name from headers
#[allow(dead_code)]
fn get_cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
  let cookie_header = headers.get("cookie")?;
  let cookie_str = cookie_header.to_str().ok()?;

  debug!("Parsing cookies from: {}", cookie_str);

  // Parse cookies
  for cookie_part in cookie_str.split(';') {
    let trimmed = cookie_part.trim();
    debug!("Checking cookie segment: {}", trimmed);

    // Look for cookie with the specified name - more robust parsing logic
    // Handle name=value exactly and also name = value with potential spaces
    let mut parts = trimmed.splitn(2, '=');
    if let Some(cookie_name) = parts.next() {
      if cookie_name.trim() == name {
        if let Some(value) = parts.next() {
          debug!("Found cookie {} with value: {}", name, value.trim());
          return Some(value.trim().to_string());
        }
      }
    }
  }

  debug!("Cookie {} not found in header", name);
  None
}
