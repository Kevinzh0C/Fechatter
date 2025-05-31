//! # Core Middleware Layer - Single Responsibility Middleware Functions
//!
//! **Design Principles**:
//! - Each middleware function handles one specific functionality
//! - Depends on Layer 0 atomic operations
//! - Zero-cost abstraction, inline optimization
//! - Clear responsibilities, no functional overlap

use crate::{
  AppState,
  middlewares::core::primitives::{
    calculate_cookie_max_age, extract_bearer_token, extract_chat_id, extract_refresh_token,
    extract_workspace_id, format_bearer_token, generate_request_id, is_valid_id,
    is_valid_token_format,
  },
};
use axum::{
  extract::{Request, State},
  http::{HeaderValue, StatusCode},
  middleware::Next,
  response::{IntoResponse, Response},
};
use fechatter_core::models::{AuthUser, UserClaims};
// Import TokenService trait to use verify_token method
use fechatter_core::models::jwt::TokenService;

// =============================================================================
// Authentication Middleware - Single Responsibility: Handle User Authentication
// =============================================================================

/// Pure Authentication Middleware
///
/// **Single Responsibility**: Verify user identity without refresh logic
/// **Function**: Extract and verify Bearer token, create AuthUser
/// **Dependencies**: extract_bearer_token
/// **Output**: Insert AuthUser into request or return 401
#[inline(always)]
pub async fn auth_middleware(
  State(state): State<AppState>,
  mut request: Request,
  next: Next,
) -> Response {
  // Extract token - using atomic operation
  let token = match extract_bearer_token(request.headers()) {
    Some(token) if is_valid_token_format(token) => token,
    _ => return StatusCode::UNAUTHORIZED.into_response(),
  };

  // Verify token
  let user = match state.token_manager().verify_token(token) {
    Ok(claims) => create_auth_user_from_claims(claims),
    Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
  };

  // Insert user info
  request.extensions_mut().insert(user);
  next.run(request).await
}

/// Pure Token Refresh Middleware
///
/// **Single Responsibility**: Handle refresh token refresh without auth logic
/// **Function**: Use refresh token to get new access token
/// **Dependencies**: extract_refresh_token
/// **Output**: Update token and set new cookie, or return 401
#[inline(always)]
pub async fn token_refresh_middleware(
  State(state): State<AppState>,
  mut request: Request,
  next: Next,
) -> Response {
  // Extract refresh token - using atomic operation
  let refresh_token = match extract_refresh_token(request.headers()) {
    Some(token) => token,
    None => return StatusCode::UNAUTHORIZED.into_response(),
  };

  // Refresh token
  let new_tokens = match state.refresh_token(&refresh_token, None).await {
    Ok(tokens) => tokens,
    Err(_) => {
      // Clear invalid refresh token
      let mut response = StatusCode::UNAUTHORIZED.into_response();
      set_clear_refresh_cookie(&mut response);
      return response;
    }
  };

  // Verify new token and set user info
  let user = match state.token_manager().verify_token(&new_tokens.access_token) {
    Ok(claims) => create_auth_user_from_claims(claims),
    Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  };

  // Update request header
  if let Ok(auth_header) = HeaderValue::from_str(&format_bearer_token(&new_tokens.access_token)) {
    request.headers_mut().insert("authorization", auth_header);
  }

  // Insert user info
  request.extensions_mut().insert(user);

  // Execute request and set new cookie
  let mut response = next.run(request).await;
  set_refresh_cookie(&mut response, &new_tokens.refresh_token);
  response
}

// =============================================================================
// Permission Verification Middleware - Single Responsibility: Handle Permission Checks
// =============================================================================

/// Workspace Permission Middleware
///
/// **Single Responsibility**: Verify user's workspace access permission
/// **Precondition**: User is authenticated
/// **Function**: Extract workspace ID and verify permission
/// **Dependencies**: extract_workspace_id
#[inline(always)]
pub async fn workspace_middleware(mut request: Request, next: Next) -> Response {
  // Ensure user is authenticated
  let auth_user = match request.extensions().get::<AuthUser>() {
    Some(user) => user.clone(),
    None => return StatusCode::UNAUTHORIZED.into_response(),
  };

  // Extract workspace ID - using atomic operation
  let workspace_id = extract_workspace_id(&request).unwrap_or(i64::from(auth_user.workspace_id));

  // Verify ID validity
  if !is_valid_id(workspace_id) {
    return StatusCode::BAD_REQUEST.into_response();
  }

  // Verify workspace access permission (simplified logic)
  if i64::from(auth_user.workspace_id) != workspace_id {
    return StatusCode::FORBIDDEN.into_response();
  }

  // Record workspace ID in extensions
  request
    .extensions_mut()
    .insert(WorkspaceAccess { workspace_id });
  next.run(request).await
}

/// Chat Permission Middleware
///
/// **Single Responsibility**: Verify user's chat access permission
/// **Precondition**: User is authenticated and workspace verified
/// **Function**: Extract chat ID and verify permission
/// **Dependencies**: extract_chat_id
#[inline(always)]
pub async fn chat_middleware(
  State(state): State<AppState>,
  mut request: Request,
  next: Next,
) -> Response {
  // Ensure user is authenticated
  let auth_user = match request.extensions().get::<AuthUser>() {
    Some(user) => user.clone(),
    None => return StatusCode::UNAUTHORIZED.into_response(),
  };

  // Extract chat ID - using atomic operation
  let chat_id = match extract_chat_id(&request) {
    Some(id) if is_valid_id(id) => id,
    _ => return StatusCode::BAD_REQUEST.into_response(),
  };

  // Verify chat access permission
  match state
    .user_can_access_chat(auth_user.id.into(), chat_id)
    .await
  {
    Ok(true) => {
      request.extensions_mut().insert(ChatAccess { chat_id });
      next.run(request).await
    }
    Ok(false) => StatusCode::FORBIDDEN.into_response(),
    Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  }
}

// =============================================================================
// Observability Middleware - Single Responsibility: Handle Monitoring and Tracing
// =============================================================================

/// Request Tracking Middleware
///
/// **Single Responsibility**: Add unique tracking ID for each request
/// **Function**: Generate or extract request ID, add to request and response headers
/// **Dependencies**: generate_request_id
#[inline(always)]
pub async fn request_tracking_middleware(mut request: Request, next: Next) -> Response {
  // Generate or extract request ID - using atomic operation
  let request_id = request
    .headers()
    .get("x-request-id")
    .and_then(|h| h.to_str().ok())
    .map(String::from)
    .unwrap_or_else(generate_request_id);

  // Set request header
  if let Ok(header_value) = HeaderValue::from_str(&request_id) {
    request
      .headers_mut()
      .insert("x-request-id", header_value.clone());

    // Execute request and set response header
    let mut response = next.run(request).await;
    response.headers_mut().insert("x-request-id", header_value);
    response
  } else {
    next.run(request).await
  }
}

/// Server Time Middleware
///
/// **Single Responsibility**: Add server timestamp in response header
/// **Function**: Record server time of request processing
#[inline(always)]
pub async fn server_time_middleware(request: Request, next: Next) -> Response {
  let server_time = chrono::Utc::now().to_rfc3339();
  let mut response = next.run(request).await;

  if let Ok(time_header) = HeaderValue::from_str(&server_time) {
    response.headers_mut().insert("x-server-time", time_header);
  }

  response
}

// =============================================================================
// Error Handling Middleware - Single Responsibility: Handle Error Responses
// =============================================================================

/// Unified Error Handling Middleware
///
/// **Single Responsibility**: Catch and standardize error response format
/// **Function**: Convert various errors to unified HTTP response
#[inline(always)]
pub async fn error_handling_middleware(request: Request, next: Next) -> Response {
  let response = next.run(request).await;

  // If error response, add unified error header
  if response.status().is_client_error() || response.status().is_server_error() {
    let mut new_response = response;
    if let Ok(error_id) = HeaderValue::from_str(&generate_request_id()) {
      new_response.headers_mut().insert("x-error-id", error_id);
    }
    new_response
  } else {
    response
  }
}

// =============================================================================
// Helper Structures and Functions - Single Responsibility: Support Middleware Operations
// =============================================================================

/// Workspace Access Credential
#[derive(Debug, Clone)]
pub struct WorkspaceAccess {
  pub workspace_id: i64,
}

/// Chat Access Credential
#[derive(Debug, Clone)]
pub struct ChatAccess {
  pub chat_id: i64,
}

/// Create AuthUser from Claims
///
/// **Single Responsibility**: Type conversion, Claims -> AuthUser
fn create_auth_user_from_claims(claims: UserClaims) -> AuthUser {
  AuthUser {
    id: claims.id,
    fullname: claims.fullname,
    email: claims.email,
    status: claims.status,
    created_at: claims.created_at,
    workspace_id: claims.workspace_id,
  }
}

/// Set Refresh Token Cookie
///
/// **Single Responsibility**: Set refresh token cookie in response
fn set_refresh_cookie(
  response: &mut Response,
  token: &fechatter_core::models::jwt::RefreshTokenData,
) {
  let max_age = calculate_cookie_max_age(token.expires_at);
  let cookie = format!(
    "refresh_token={}; Path=/; Max-Age={}; HttpOnly; SameSite=Strict; Secure",
    token.token, max_age
  );

  if let Ok(cookie_header) = HeaderValue::from_str(&cookie) {
    response.headers_mut().insert("set-cookie", cookie_header);
  }
}

/// Clear Refresh Token Cookie
///
/// **Single Responsibility**: Clear invalid refresh token cookie
fn set_clear_refresh_cookie(response: &mut Response) {
  let clear_cookie = "refresh_token=; Path=/; Max-Age=0; HttpOnly; SameSite=Strict; Secure";
  if let Ok(cookie_header) = HeaderValue::from_str(clear_cookie) {
    response.headers_mut().insert("set-cookie", cookie_header);
  }
}

// =============================================================================
// Test Module
// =============================================================================

#[cfg(test)]
mod tests {
  use super::*;
  use axum::http::{HeaderMap, HeaderValue};

  #[test]
  fn test_create_auth_user_from_claims() {
    let claims = UserClaims {
      id: 1.into(),
      fullname: "Test User".to_string(),
      email: "test@example.com".to_string(),
      status: fechatter_core::UserStatus::Active,
      created_at: chrono::Utc::now(),
      workspace_id: 1,
    };

    let user = create_auth_user_from_claims(claims.clone());
    assert_eq!(user.id, claims.id);
    assert_eq!(user.email, claims.email);
  }

  #[test]
  fn test_workspace_access() {
    let access = WorkspaceAccess { workspace_id: 123 };
    assert_eq!(access.workspace_id, 123);
  }

  #[test]
  fn test_chat_access() {
    let access = ChatAccess { chat_id: 456 };
    assert_eq!(access.chat_id, 456);
  }
}
