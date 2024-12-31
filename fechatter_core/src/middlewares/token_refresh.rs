use axum::{
  body::Body,
  extract::State,
  http::{
    Request, StatusCode,
    header::{HeaderMap, HeaderValue, SET_COOKIE},
  },
  middleware::Next,
  response::Response,
};

use crate::{
  AppError, AppState, services::auth_service::AuthService, utils::jwt::RefreshTokenData,
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
  // Get refresh token from cookie header
  let refresh_token = match get_cookie_value(&headers, AUTH_COOKIE_NAME) {
    Some(value) => value,
    None => {
      // No refresh token, continue without refreshing
      return Ok(next.run(request).await);
    }
  };

  // Check if there's an access token
  let has_access_token = headers
    .get(AUTH_HEADER)
    .map(|h| h.to_str().unwrap_or_default().starts_with("Bearer "))
    .unwrap_or(false);

  // If we have both tokens, no need to refresh yet
  if has_access_token {
    return Ok(next.run(request).await);
  }

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

  // Create auth service
  let auth_service = AuthService::new(&state.pool, &state.token_manager);

  // Try to refresh the token
  match auth_service
    .refresh_token(&refresh_token, user_agent, ip_address)
    .await
  {
    Ok(tokens) => {
      // Set the new auth token in request headers
      let auth_header = format!("Bearer {}", tokens.access_token);
      request.headers_mut().insert(
        AUTH_HEADER,
        HeaderValue::from_str(&auth_header).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
      );

      // Set new refresh token in cookie to be sent back
      let refresh_cookie = create_refresh_cookie(&tokens.refresh_token);
      let mut response = next.run(request).await;
      response.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_str(&refresh_cookie).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
      );

      Ok(response)
    }
    Err(_) => {
      // Token refresh failed, clear the cookie and continue
      let expired_cookie = format!(
        "{}=; Path=/; Max-Age=0; HttpOnly; SameSite=Strict; Secure",
        AUTH_COOKIE_NAME
      );
      let mut response = next.run(request).await;
      response.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_str(&expired_cookie).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
      );

      Ok(response)
    }
  }
}

/// Get a cookie value from the headers
fn get_cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
  headers
    .get("cookie")?
    .to_str()
    .ok()?
    .split(';')
    .map(|s| s.trim())
    .find_map(|cookie| {
      let mut parts = cookie.split('=');
      match (parts.next(), parts.next()) {
        (Some(n), Some(v)) if n == name => Some(v.to_string()),
        _ => None,
      }
    })
}

/// Create a secure cookie with refresh token
fn create_refresh_cookie(token_data: &RefreshTokenData) -> String {
  let expires = token_data
    .expires_at
    .format("%a, %d %b %Y %T GMT")
    .to_string();

  format!(
    "{}={}; Path=/; Expires={}; HttpOnly; SameSite=Strict; Secure",
    AUTH_COOKIE_NAME, token_data.token, expires
  )
}
