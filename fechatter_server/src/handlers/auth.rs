//! # Authentication Handlers - Clean Architecture Implementation
//!
//! **Architecture Principles**: Clean Architecture + DDD
//! **Handler Responsibilities**: HTTP coordination + Service delegation + Response construction (≤20 lines/function)
//! **Dependency Direction**: Handler → Application Service → Domain Service → Infrastructure
//!
//! ## Correct Modern Architecture Implementation
//! -  Handlers only handle HTTP coordination, no business logic
//! -  Using existing AuthService (services/application/auth_app_service)
//! -  All business logic delegated to Service layer
//! -  Technical details (cookies, headers) extracted to utility functions
//! -  Follow proper dependency chain

use crate::{AppState, ErrorOutput, error::AppError};
use axum::{
  Extension, Json,
  extract::State,
  http::{HeaderMap, HeaderValue, StatusCode, header},
  response::IntoResponse,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{DateTime, Utc};
use fechatter_core::models::{AuthUser, CreateUser};
use fechatter_core::{
  AuthTokens, SigninUser, contracts::AuthContext, models::jwt::ACCESS_TOKEN_EXPIRATION,
};
use utoipa::ToSchema;

// =============================================================================
// UTILITY FUNCTIONS - Technical Implementation Details
// =============================================================================

/// Unified API Response wrapper (consistent with other handlers)
#[derive(Debug, serde::Serialize)]
pub struct ApiResponse<T> {
  pub data: T,
  pub status: String,
}

impl<T> ApiResponse<T> {
  pub fn success(data: T) -> Self {
    Self {
      data,
      status: "success".to_string(),
    }
  }
}

/// Set refresh token cookie in response headers
fn set_refresh_token_cookie(
  headers: &mut HeaderMap,
  token_str: &str,
  expires_at: &DateTime<Utc>,
) -> Result<(), AppError> {
  let mut cookie = Cookie::new("refresh_token", token_str.to_string());
  cookie.set_http_only(true);
  cookie.set_secure(false); // Set to true in production with HTTPS
  cookie.set_same_site(Some(SameSite::None));
  cookie.set_path("/");

  // Calculate seconds until expiration
  let now = Utc::now();
  let duration_seconds = expires_at.signed_duration_since(now).num_seconds();

  // Format expiration date in correct HTTP format (RFC 7231)
  let time_str = expires_at.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

  // Include both Max-Age and Expires in the cookie string for cross-browser compatibility
  let cookie_str = if duration_seconds <= 0 {
    format!("{}; Max-Age=0; Expires={}", cookie.to_string(), time_str)
  } else {
    format!(
      "{}; Max-Age={}; Expires={}",
      cookie.to_string(),
      duration_seconds,
      time_str
    )
  };

  match HeaderValue::from_str(&cookie_str) {
    Ok(hv) => {
      headers.insert(header::SET_COOKIE, hv);
      Ok(())
    }
    Err(e) => {
      tracing::error!("Failed to create header value from cookie: {}", e);
      Err(AppError::Internal(
        "Failed to set refresh token cookie".to_string(),
      ))
    }
  }
}

/// Clear refresh token cookie from response headers
fn clear_refresh_token_cookie(headers: &mut HeaderMap) -> Result<(), AppError> {
  let mut cookie = Cookie::new("refresh_token", "");
  cookie.set_http_only(true);
  cookie.set_secure(false); // Set to true in production with HTTPS
  cookie.set_same_site(Some(SameSite::None));
  cookie.set_path("/");

  // Set both Max-Age=0 and Expires to past date for cross-browser compatibility
  let cookie_str = format!(
    "{}; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT",
    cookie.to_string()
  );

  match HeaderValue::from_str(&cookie_str) {
    Ok(hv) => {
      headers.insert(header::SET_COOKIE, hv);
      Ok(())
    }
    Err(e) => {
      tracing::error!("Failed to create header value from cookie: {}", e);
      Err(AppError::Internal(
        "Failed to clear refresh token cookie".to_string(),
      ))
    }
  }
}

/// Authentication response DTO
#[derive(Debug, serde::Serialize)]
pub struct AuthResponse {
  pub access_token: String,
  pub expires_in: usize,
  pub refresh_token: Option<String>,
}

/// Extract AuthContext from request headers
fn extract_auth_context(headers: &HeaderMap) -> Option<AuthContext> {
  let user_agent = headers
    .get("user-agent")
    .and_then(|h| h.to_str().ok())
    .map(String::from);
  let ip_address = headers
    .get("x-forwarded-for")
    .and_then(|h| h.to_str().ok())
    .map(String::from);

  Some(AuthContext {
    user_agent,
    ip_address,
  })
}

// =============================================================================
// HANDLERS - HTTP Coordination Layer (Using Modern Architecture)
// =============================================================================

/// User Signup Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
#[utoipa::path(
    post,
    path = "/api/signup", 
    request_body = CreateUser,
    responses(
        (status = 201, description = "User registered successfully", body = AuthTokens),
        (status = 409, description = "User already exists", body = ErrorOutput),
        (status = 400, description = "Invalid input", body = ErrorOutput)
    ),
    tag = "auth"
)]
pub(crate) async fn signup_handler(
  State(state): State<AppState>,
  headers: HeaderMap,
  Json(payload): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
  let auth_context = extract_auth_context(&headers);

  match state.signup(&payload, auth_context).await {
    Ok(tokens) => {
      let mut response_headers = HeaderMap::new();
      set_refresh_token_cookie(
        &mut response_headers,
        &tokens.refresh_token.token,
        &tokens.refresh_token.expires_at,
      )?;

      let body = Json(AuthResponse {
        access_token: tokens.access_token,
        expires_in: ACCESS_TOKEN_EXPIRATION,
        refresh_token: Some(tokens.refresh_token.token),
      });

      Ok((StatusCode::CREATED, response_headers, body).into_response())
    }
    Err(fechatter_core::error::CoreError::UserAlreadyExists(msg)) => {
      Ok((StatusCode::CONFLICT, Json(ErrorOutput::new(msg))).into_response())
    }
    Err(e) => Err(AppError::from(e)),
  }
}

/// User Signin Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
#[utoipa::path(
    post,
    path = "/api/signin",
    request_body = SigninUser,
    responses(
        (status = 200, description = "Login successful", body = AuthTokens),
        (status = 403, description = "Invalid credentials", body = ErrorOutput),
        (status = 400, description = "Invalid input", body = ErrorOutput)
    ),
    tag = "auth"
)]
pub(crate) async fn signin_handler(
  State(state): State<AppState>,
  headers: HeaderMap,
  Json(payload): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
  let auth_context = extract_auth_context(&headers);

  match state.signin(&payload, auth_context).await? {
    Some(tokens) => {
      let mut response_headers = HeaderMap::new();
      set_refresh_token_cookie(
        &mut response_headers,
        &tokens.refresh_token.token,
        &tokens.refresh_token.expires_at,
      )?;

      let body = Json(AuthResponse {
        access_token: tokens.access_token,
        expires_in: ACCESS_TOKEN_EXPIRATION,
        refresh_token: Some(tokens.refresh_token.token),
      });

      Ok((StatusCode::OK, response_headers, body).into_response())
    }
    None => Ok(
      (
        StatusCode::FORBIDDEN,
        Json(ErrorOutput::new("Invalid credentials")),
      )
        .into_response(),
    ),
  }
}

/// Refresh Token Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
#[utoipa::path(
    post,
    path = "/api/refresh",
    security(("refresh_token" = [])),
    responses(
        (status = 200, description = "Token refreshed successfully", body = AuthTokens),
        (status = 401, description = "Invalid or expired refresh token", body = ErrorOutput)
    ),
    tag = "auth"
)]
pub(crate) async fn refresh_token_handler(
  State(state): State<AppState>,
  headers: HeaderMap,
  cookies: CookieJar,
  auth_user: Option<Extension<AuthUser>>,
) -> Result<impl IntoResponse, AppError> {
  let auth_context = extract_auth_context(&headers);

  if let Some(Extension(user)) = auth_user {
    let tokens = state
      .generate_new_tokens_for_user(user.id.into(), auth_context)
      .await?;
    let mut response_headers = HeaderMap::new();
    set_refresh_token_cookie(
      &mut response_headers,
      &tokens.refresh_token.token,
      &tokens.refresh_token.expires_at,
    )?;

    let body = Json(AuthResponse {
      access_token: tokens.access_token,
      expires_in: ACCESS_TOKEN_EXPIRATION,
      refresh_token: Some(tokens.refresh_token.token),
    });
    return Ok((StatusCode::OK, response_headers, body).into_response());
  }

  let refresh_token = extract_refresh_token(&cookies, &headers)?;

  match state.refresh_token(&refresh_token, auth_context).await {
    Ok(tokens) => {
      let mut response_headers = HeaderMap::new();
      set_refresh_token_cookie(
        &mut response_headers,
        &tokens.refresh_token.token,
        &tokens.refresh_token.expires_at,
      )?;

      let body = Json(AuthResponse {
        access_token: tokens.access_token,
        expires_in: ACCESS_TOKEN_EXPIRATION,
        refresh_token: Some(tokens.refresh_token.token),
      });
      Ok((StatusCode::OK, response_headers, body).into_response())
    }
    Err(_) => {
      let mut headers = HeaderMap::new();
      clear_refresh_token_cookie(&mut headers)?;
      Ok(
        (
          StatusCode::UNAUTHORIZED,
          headers,
          Json(ErrorOutput::new("Invalid or expired refresh token")),
        )
          .into_response(),
      )
    }
  }
}

/// User Logout Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
#[utoipa::path(
    post,
    path = "/api/logout",
    security(("access_token" = [])),
    responses(
        (status = 200, description = "Logged out successfully"),
        (status = 401, description = "Unauthorized", body = ErrorOutput)
    ),
    tag = "auth"
)]
pub(crate) async fn logout_handler(
  State(state): State<AppState>,
  cookies: CookieJar,
  headers: HeaderMap,
  _auth_user: Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
  let mut response_headers = HeaderMap::new();
  clear_refresh_token_cookie(&mut response_headers)?;

  if let Ok(refresh_token) = extract_refresh_token(&cookies, &headers) {
    state.logout(&refresh_token).await?;
  }

  Ok(
    (
      StatusCode::OK,
      response_headers,
      Json(serde_json::json!({"message": "Logged out successfully"})),
    )
      .into_response(),
  )
}

/// Logout All Sessions Handler
///
/// **Modern Architecture**: Handler → Application Service → Domain Service
#[utoipa::path(
    post,
    path = "/api/logout-all",
    security(("access_token" = [])),
    responses(
        (status = 200, description = "Logged out from all sessions successfully"),
        (status = 401, description = "Unauthorized", body = ErrorOutput)
    ),
    tag = "auth"
)]
pub(crate) async fn logout_all_handler(
  State(state): State<AppState>,
  _cookies: CookieJar,
  _headers: HeaderMap,
  auth_user: Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
  let mut response_headers = HeaderMap::new();
  clear_refresh_token_cookie(&mut response_headers)?;

  state.logout_all(auth_user.id).await.ok(); // Ignore errors for user experience

  Ok(
    (
      StatusCode::OK,
      response_headers,
      Json(serde_json::json!({"message": "Logged out from all sessions successfully"})),
    )
      .into_response(),
  )
}

// =============================================================================
// UTILITY FUNCTIONS - Token Extraction
// =============================================================================

/// Extract refresh token from cookies or Authorization header
fn extract_refresh_token(cookies: &CookieJar, headers: &HeaderMap) -> Result<String, AppError> {
  // Try cookie first
  if let Some(cookie) = cookies.get("refresh_token") {
    return Ok(cookie.value().to_string());
  }

  // Try Authorization header
  if let Some(auth_header) = headers.get("Authorization") {
    let auth_value = auth_header
      .to_str()
      .map_err(|_| AppError::InvalidInput("Invalid Authorization header".to_string()))?;
    if auth_value.starts_with("Bearer ") {
      return Ok(auth_value[7..].to_string());
    }
  }

  Err(AppError::InvalidInput(
    "No refresh token provided".to_string(),
  ))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ErrorOutput;
  use crate::assert_handler_success;
  use crate::setup_test_users;
  use anyhow::Result;
  use axum::{Json, http::StatusCode};
  use axum_extra::extract::cookie::{Cookie, CookieJar};
  use http_body_util::BodyExt;

  #[tokio::test]
  async fn signup_handler_should_work() -> Result<()> {
    let (_tdb, state, _users) = setup_test_users!(0).await;

    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos();
    let unique_email = format!("testuser1{}@acme.test", timestamp);

    let payload = CreateUser {
      fullname: "Test User".to_string(),
      email: unique_email,
      password: "password".to_string(),
      workspace: "Acme".to_string(),
    };

    let test_handler = |state, payload| async {
      signup_handler(State(state), HeaderMap::new(), Json(payload)).await
    };

    let auth_response = assert_handler_success!(
      test_handler(state, payload),
      StatusCode::CREATED,
      AuthResponse
    );

    assert_ne!(auth_response.access_token, "");
    assert_eq!(auth_response.expires_in, ACCESS_TOKEN_EXPIRATION);
    Ok(())
  }

  #[tokio::test]
  async fn signin_handler_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user1 = &users[0];

    let input = SigninUser::new(&user1.email, "password");
    let test_handler =
      |state, input| async { signin_handler(State(state), HeaderMap::new(), Json(input)).await };

    let auth_response =
      assert_handler_success!(test_handler(state, input), StatusCode::OK, AuthResponse);

    assert_ne!(auth_response.access_token, "");
    assert_eq!(auth_response.expires_in, ACCESS_TOKEN_EXPIRATION);
    Ok(())
  }
}
