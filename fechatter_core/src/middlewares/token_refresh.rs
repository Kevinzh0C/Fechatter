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

use crate::{
  models::AuthUser, services::AuthServiceTrait, utils::{jwt::RefreshTokenData, token::TokenValidator}, AppState
};

const AUTH_HEADER: &str = "Authorization";
const AUTH_COOKIE_NAME: &str = "refresh_token";

/// Extract auth context from cookies and update tokens if necessary
pub async fn refresh_token_middleware(
  headers: HeaderMap,
  State(state): State<AppState>,
  mut request: Request<Body>,
  next: Next,
) -> Result<Response, StatusCode> {
  // Debug header information
  debug!("Headers in refresh_token_middleware: {:?}", headers);

  // Check if AuthUser extension already exists (from previous middleware)
  let existing_auth_user = request.extensions().get::<AuthUser>().cloned();
  if let Some(user) = &existing_auth_user {
    debug!(
      "AuthUser extension already exists for user_id={}, preserving it",
      user.id
    );
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

  let auth_service: Box<dyn AuthServiceTrait> = state.service_provider.create_service();

  let auth_service: Box<dyn AuthServiceTrait> =
    state.service_provider.create_service();

  // Try to refresh the token
  match auth_service
    .refresh_token(&refresh_token, user_agent, ip_address)
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
      match state.token_manager.validate_token(&tokens.access_token) {
        Ok(claims) => {
          debug!(
            "Adding AuthUser extension from refreshed token for user_id={}",
            claims.id
          );
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
          debug!("AuthUser extension successfully added to request");
        }
        Err(e) => {
          warn!("Refreshed token validation failed: {}", e);
          // Odd case - refresh succeeded but validation failed
          return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
      }

      // Set new refresh token in cookie to be sent back
      let refresh_cookie = create_refresh_cookie(&tokens.refresh_token);
      let mut response = next.run(request).await;
      response.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_str(&refresh_cookie).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
      );

      Ok(response)
    }
    Err(e) => {
      // Token refresh failed, clear the cookie and continue
      warn!("Token refresh failed: {:?}", e);

      // Since the refresh token is invalid and there's no authorization token,
      // we can't continue - return 401 to force a login
      if !has_access_token {
        debug!("No valid refresh token and no authorization token, returning 401");
        // Clear the cookie in the response
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

      // Otherwise continue with the request (the bearer auth will handle it)
      Ok(next.run(request).await)
    }
  }
}

/// Create a cookie string for the refresh token
fn create_refresh_cookie(token_data: &RefreshTokenData) -> String {
  let max_age = (token_data.expires_at - chrono::Utc::now())
    .num_seconds()
    .max(0) as usize;

  // Set HTTP-only cookie with secure flag
  format!(
    "{}={}; Path=/; Max-Age={}; HttpOnly; SameSite=Strict; Secure",
    AUTH_COOKIE_NAME, token_data.token, max_age
  )
}

/// Get cookie value by name from headers
fn get_cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
  let cookie_header = headers.get("cookie")?;
  let cookie_str = cookie_header.to_str().ok()?;

  debug!("Parsing cookies from: {}", cookie_str);

  // Parse cookies
  for cookie_part in cookie_str.split(';') {
    let trimmed = cookie_part.trim();
    debug!("Checking cookie segment: {}", trimmed);

    // Look for cookie with the specified name
    if trimmed.starts_with(&format!("{}=", name)) {
      debug!("Found cookie {}", trimmed);
      let cookie_value = trimmed[name.len() + 1..].to_string();
      return Some(cookie_value);
    }
  }

  None
}
