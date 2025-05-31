//! # Composed Middleware Layer - Complex Authentication Flow Composition
//!
//! **Design Principles**:
//! - Create dedicated middleware functions for specific use cases
//! - Follow axum's middleware patterns: async fn(Request, Next) -> Response
//! - Implement composition logic internally, not through dynamic composition
//! - Each middleware has clear single responsibility

use crate::{
  AppState,
  middlewares::core::primitives::{
    extract_bearer_token, extract_chat_id, extract_refresh_token, extract_workspace_id,
    generate_request_id, is_valid_id, is_valid_token_format,
  },
};
use axum::{
  extract::{Request, State},
  http::{HeaderValue, StatusCode},
  middleware::Next,
  response::{IntoResponse, Response},
};
use fechatter_core::models::jwt::TokenService;
use fechatter_core::models::{AuthUser, UserClaims};

// =============================================================================
// Composed Authentication Middlewares - Single Responsibility: Complex Auth Flows
// =============================================================================

/// Authentication with Automatic Refresh Middleware
///
/// **Single Responsibility**: Try bearer auth, automatically attempt token refresh on failure
/// **Use Case**: API endpoints requiring seamless user experience
/// **Implementation**: Combines auth logic and refresh logic in one middleware
#[inline(always)]
pub async fn auth_with_refresh_middleware(
  State(state): State<AppState>,
  mut request: Request,
  next: Next,
) -> Response {
  // Step 1: Try standard bearer token authentication
  if let Some(token) = extract_bearer_token(request.headers()) {
    if is_valid_token_format(token) {
      if let Ok(claims) = state.token_manager().verify_token(token) {
        let user = create_auth_user_from_claims(claims);
        request.extensions_mut().insert(user);
        return next.run(request).await;
      }
    }
  }

  // Step 2: Try refresh token if bearer auth failed
  if let Some(refresh_token) = extract_refresh_token(request.headers()) {
    match state.refresh_token(&refresh_token, None).await {
      Ok(new_tokens) => {
        // Verify new access token
        if let Ok(claims) = state.token_manager().verify_token(&new_tokens.access_token) {
          let user = create_auth_user_from_claims(claims);

          // Update request header with new token
          if let Ok(auth_header) =
            HeaderValue::from_str(&format!("Bearer {}", &new_tokens.access_token))
          {
            request.headers_mut().insert("authorization", auth_header);
          }

          request.extensions_mut().insert(user);

          // Execute request and set new refresh cookie
          let mut response = next.run(request).await;
          set_refresh_cookie(&mut response, &new_tokens.refresh_token);
          return response;
        }
      }
      Err(_) => {
        // Clear invalid refresh token
        let mut response = StatusCode::UNAUTHORIZED.into_response();
        set_clear_refresh_cookie(&mut response);
        return response;
      }
    }
  }

  StatusCode::UNAUTHORIZED.into_response()
}

/// Complete Permission Validation Middleware
///
/// **Single Responsibility**: Execute auth -> workspace validation -> chat validation in sequence
/// **Use Case**: Chat-related APIs requiring complete permission checks
/// **Implementation**: Combines all permission checks in one middleware
#[inline(always)]
pub async fn full_auth_middleware(
  State(state): State<AppState>,
  mut request: Request,
  next: Next,
) -> Response {
  // Step 1: Authentication
  let token = match extract_bearer_token(request.headers()) {
    Some(token) if is_valid_token_format(token) => token,
    _ => return StatusCode::UNAUTHORIZED.into_response(),
  };

  let auth_user = match state.token_manager().verify_token(token) {
    Ok(claims) => create_auth_user_from_claims(claims),
    Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
  };

  // Step 2: Workspace validation
  let workspace_id = extract_workspace_id(&request).unwrap_or(i64::from(auth_user.workspace_id));

  if !is_valid_id(workspace_id) {
    return StatusCode::BAD_REQUEST.into_response();
  }

  if i64::from(auth_user.workspace_id) != workspace_id {
    return StatusCode::FORBIDDEN.into_response();
  }

  // Step 3: Chat validation
  let chat_id = match extract_chat_id(&request) {
    Some(id) if is_valid_id(id) => id,
    _ => return StatusCode::BAD_REQUEST.into_response(),
  };

  match state
    .user_can_access_chat(auth_user.id.into(), chat_id)
    .await
  {
    Ok(true) => {
      // Insert all credentials
      request.extensions_mut().insert(auth_user);
      request
        .extensions_mut()
        .insert(WorkspaceAccess { workspace_id });
      request.extensions_mut().insert(ChatAccess { chat_id });
      next.run(request).await
    }
    Ok(false) => StatusCode::FORBIDDEN.into_response(),
    Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
  }
}

/// Enhanced Authentication Middleware (Auth + Workspace)
///
/// **Single Responsibility**: Auth with refresh + workspace validation
/// **Use Case**: Workspace-level APIs requiring seamless user experience
/// **Implementation**: Combines auth with refresh and workspace validation
#[inline(always)]
pub async fn enhanced_auth_middleware(
  State(state): State<AppState>,
  mut request: Request,
  next: Next,
) -> Response {
  // Step 1: Try authentication with auto-refresh
  let auth_user = match try_auth_with_refresh(&state, &mut request).await {
    Ok(user) => user,
    Err(response) => return response,
  };

  // Step 2: Workspace validation
  let workspace_id = extract_workspace_id(&request).unwrap_or(i64::from(auth_user.workspace_id));

  if !is_valid_id(workspace_id) {
    return StatusCode::BAD_REQUEST.into_response();
  }

  if i64::from(auth_user.workspace_id) != workspace_id {
    return StatusCode::FORBIDDEN.into_response();
  }

  // Insert credentials
  request.extensions_mut().insert(auth_user);
  request
    .extensions_mut()
    .insert(WorkspaceAccess { workspace_id });
  next.run(request).await
}

// =============================================================================
// Observability Middlewares - Single Responsibility: Monitoring and Error Handling
// =============================================================================

/// Standard Observability Middleware
///
/// **Single Responsibility**: Add request tracking + error handling
/// **Use Case**: All API endpoints requiring monitoring
/// **Implementation**: Combines tracking and error handling in one middleware
#[inline(always)]
pub async fn standard_observability_middleware(mut request: Request, next: Next) -> Response {
  // Add request tracking
  let request_id = request
    .headers()
    .get("x-request-id")
    .and_then(|h| h.to_str().ok())
    .map(String::from)
    .unwrap_or_else(generate_request_id);

  if let Ok(header_value) = HeaderValue::from_str(&request_id) {
    request
      .headers_mut()
      .insert("x-request-id", header_value.clone());

    // Execute request with error handling
    let response = next.run(request).await;

    // Add tracking to response and handle errors
    let mut final_response =
      if response.status().is_client_error() || response.status().is_server_error() {
        let mut error_response = response;
        if let Ok(error_id) = HeaderValue::from_str(&generate_request_id()) {
          error_response.headers_mut().insert("x-error-id", error_id);
        }
        error_response
      } else {
        response
      };

    final_response
      .headers_mut()
      .insert("x-request-id", header_value);
    final_response
  } else {
    next.run(request).await
  }
}

/// Authentication + Observability Middleware
///
/// **Single Responsibility**: Combine authentication and observability features
/// **Use Case**: Authentication APIs requiring monitoring
/// **Implementation**: Combines observability and auth in one middleware
#[inline(always)]
pub async fn auth_with_observability_middleware(
  State(state): State<AppState>,
  mut request: Request,
  next: Next,
) -> Response {
  // Add request tracking first
  let request_id = request
    .headers()
    .get("x-request-id")
    .and_then(|h| h.to_str().ok())
    .map(String::from)
    .unwrap_or_else(generate_request_id);

  if let Ok(header_value) = HeaderValue::from_str(&request_id) {
    request
      .headers_mut()
      .insert("x-request-id", header_value.clone());
  }

  // Then perform authentication
  let token = match extract_bearer_token(request.headers()) {
    Some(token) if is_valid_token_format(token) => token,
    _ => {
      let mut response = StatusCode::UNAUTHORIZED.into_response();
      if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", header_value);
      }
      return response;
    }
  };

  let user = match state.token_manager().verify_token(token) {
    Ok(claims) => create_auth_user_from_claims(claims),
    Err(_) => {
      let mut response = StatusCode::UNAUTHORIZED.into_response();
      if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", header_value);
      }
      return response;
    }
  };

  request.extensions_mut().insert(user);

  // Execute request and add response headers
  let mut response = next.run(request).await;
  if let Ok(header_value) = HeaderValue::from_str(&request_id) {
    response.headers_mut().insert("x-request-id", header_value);
  }
  response
}

// =============================================================================
// Helper Structures and Functions - Single Responsibility: Support Operations
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

/// Helper function for auth with refresh logic
///
/// **Single Responsibility**: Handle authentication with automatic token refresh
async fn try_auth_with_refresh(
  state: &AppState,
  request: &mut Request,
) -> Result<AuthUser, Response> {
  // Try bearer token first
  if let Some(token) = extract_bearer_token(request.headers()) {
    if is_valid_token_format(token) {
      if let Ok(claims) = state.token_manager().verify_token(token) {
        return Ok(create_auth_user_from_claims(claims));
      }
    }
  }

  // Try refresh token if bearer auth failed
  if let Some(refresh_token) = extract_refresh_token(request.headers()) {
    match state.refresh_token(&refresh_token, None).await {
      Ok(new_tokens) => {
        if let Ok(claims) = state.token_manager().verify_token(&new_tokens.access_token) {
          // Update request header
          if let Ok(auth_header) =
            HeaderValue::from_str(&format!("Bearer {}", &new_tokens.access_token))
          {
            request.headers_mut().insert("authorization", auth_header);
          }
          return Ok(create_auth_user_from_claims(claims));
        }
      }
      Err(_) => {
        let mut response = StatusCode::UNAUTHORIZED.into_response();
        set_clear_refresh_cookie(&mut response);
        return Err(response);
      }
    }
  }

  Err(StatusCode::UNAUTHORIZED.into_response())
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
  use crate::middlewares::core::primitives::calculate_cookie_max_age;

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
