use crate::models::AuthUser;
use crate::{AppState, ErrorOutput, SigninUser, error::AppError, models::CreateUser};
use axum::{
  Extension, Json,
  extract::State,
  http::{HeaderMap, HeaderValue, StatusCode, header},
  response::IntoResponse,
};

use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{DateTime, Utc};

use fechatter_core::models::jwt::ACCESS_TOKEN_EXPIRATION;
use fechatter_core::services::AuthContext;
use serde::{Deserialize, Serialize};

fn set_refresh_token_cookie(
  headers: &mut HeaderMap,
  token_str: &str,
  expires_at: &DateTime<Utc>,
) -> Result<(), AppError> {
  // Create cookie with required attributes
  let mut cookie = Cookie::new("refresh_token", token_str.to_string());
  cookie.set_http_only(true);
  cookie.set_secure(true);
  cookie.set_same_site(Some(SameSite::Lax));
  cookie.set_path("/");

  // Calculate seconds until expiration
  let now = Utc::now();
  let duration_seconds = expires_at.signed_duration_since(now).num_seconds();

  // Format expiration date in correct HTTP format (RFC 7231)
  let time_str = expires_at.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

  // Include both Max-Age and Expires in the cookie string for cross-browser compatibility
  // If duration_seconds <= 0 (e.g., client clock ahead of server), set Max-Age=0
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

  headers.append(header::SET_COOKIE, HeaderValue::from_str(&cookie_str)?);

  Ok(())
}

fn clear_refresh_token_cookie(headers: &mut HeaderMap) -> Result<(), AppError> {
  // Create cookie with base attributes
  let mut cookie = Cookie::new("refresh_token", "");
  cookie.set_http_only(true);
  cookie.set_secure(true);
  cookie.set_same_site(Some(SameSite::Lax));
  cookie.set_path("/");

  // Set both Max-Age=0 and Expires to past date for cross-browser compatibility
  let cookie_str = format!(
    "{}; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT",
    cookie.to_string()
  );

  headers.insert(header::SET_COOKIE, HeaderValue::from_str(&cookie_str)?);

  Ok(())
}

pub(crate) async fn signup_handler(
  State(state): State<AppState>,
  headers: HeaderMap,
  Json(payload): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
  // Extract auth context from headers
  let user_agent = headers
    .get("user-agent")
    .and_then(|h| h.to_str().ok())
    .map(String::from);
  let ip_address = headers
    .get("x-forwarded-for")
    .and_then(|h| h.to_str().ok())
    .map(String::from);

  let auth_context = Some(AuthContext {
    user_agent,
    ip_address,
  });

  // Delegate to the auth service for signup and handle errors explicitly
  match state.signup(&payload, auth_context).await {
    Ok(tokens) => {
      // Set refresh token cookie in response
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

pub(crate) async fn signin_handler(
  State(state): State<AppState>,
  headers: HeaderMap,
  Json(payload): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
  // Extract auth context from headers
  let user_agent = headers
    .get("user-agent")
    .and_then(|h| h.to_str().ok())
    .map(String::from);
  let ip_address = headers
    .get("x-forwarded-for")
    .and_then(|h| h.to_str().ok())
    .map(String::from);

  let auth_context = Some(AuthContext {
    user_agent,
    ip_address,
  });

  // Delegate to the auth service for signin
  match state.signin(&payload, auth_context).await? {
    Some(tokens) => {
      // Set refresh token cookie
      let mut headers = HeaderMap::new();
      set_refresh_token_cookie(
        &mut headers,
        &tokens.refresh_token.token,
        &tokens.refresh_token.expires_at,
      )?;

      let body = Json(AuthResponse {
        access_token: tokens.access_token,
        expires_in: ACCESS_TOKEN_EXPIRATION,
        refresh_token: Some(tokens.refresh_token.token),
      });

      Ok((StatusCode::OK, headers, body).into_response())
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

pub(crate) async fn refresh_token_handler(
  State(state): State<AppState>,
  headers: HeaderMap,
  cookies: CookieJar,
  _auth_user: Option<Extension<AuthUser>>,
) -> Result<impl IntoResponse, AppError> {
  // Extract auth context from headers
  let user_agent = headers
    .get("user-agent")
    .and_then(|h| h.to_str().ok())
    .map(String::from);
  let ip_address = headers
    .get("x-forwarded-for")
    .and_then(|h| h.to_str().ok())
    .map(String::from);

  // Write debug logs
  println!(
    "!! Debug refresh_token_handler START - request ID: {:?}",
    headers.get("x-request-id")
  );
  println!("!! Debug refresh_token_handler - headers: {:?}", headers);
  println!(
    "!! Debug refresh_token_handler - user_agent: {:?}, ip_address: {:?}",
    user_agent, ip_address
  );
  println!("!! Debug refresh_token_handler - cookies: {:?}", cookies);

  let auth_context = Some(AuthContext {
    user_agent,
    ip_address,
  });

  // Check if user is already authenticated via extension
  if let Some(auth_user) = _auth_user {
    println!("!! Debug: User already authenticated, generating new tokens");
    // User is authenticated, we can just generate new tokens
    let tokens = state
      .generate_new_tokens_for_user(auth_user.id, auth_context)
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

    println!("!! Debug refresh_token_handler END - success (auth user)");
    return Ok((StatusCode::OK, response_headers, body).into_response());
  }

  // Extract refresh token from cookie or Authorization header
  let refresh_token = if let Some(cookie) = cookies.get("refresh_token") {
    println!(
      "!! Debug: Found refresh_token cookie with value: {}",
      cookie.value()
    );
    cookie.value().to_string()
  } else if let Some(auth_header) = headers.get("Authorization") {
    // Try to get from Authorization header
    let auth_value = auth_header
      .to_str()
      .map_err(|_| AppError::InvalidInput("Invalid Authorization header".to_string()))?;

    if auth_value.starts_with("Bearer ") {
      auth_value[7..].to_string()
    } else {
      return Ok(
        (
          StatusCode::UNAUTHORIZED,
          Json(ErrorOutput::new(
            "Invalid Authorization format, expected 'Bearer {token}'",
          )),
        )
          .into_response(),
      );
    }
  } else {
    println!("!! Debug: No refresh token found in cookies or Authorization header");
    return Ok(
      (
        StatusCode::UNAUTHORIZED,
        Json(ErrorOutput::new("No refresh token provided")),
      )
        .into_response(),
    );
  };

  println!(
    "!! Debug: Using refresh token: {} (length: {})",
    refresh_token,
    refresh_token.len()
  );

  // Call refresh token service
  let result = state.refresh_token(&refresh_token, auth_context).await;

  match result {
    Ok(tokens) => {
      println!(
        "!! Debug: Token refresh successful - new token: {}",
        tokens.refresh_token.token
      );
      // Set refresh token cookie in response
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

      println!("!! Debug refresh_token_handler END - success");
      Ok((StatusCode::OK, response_headers, body).into_response())
    }
    Err(e) => {
      println!("!! Debug: Token refresh failed: {:?}", e);
      // For tests we need to map all errors to proper format with expected error messages
      let mut headers = HeaderMap::new();
      clear_refresh_token_cookie(&mut headers)?;

      println!("!! Debug refresh_token_handler END - error");
      // Map specific errors to expected error messages
      match e {
        fechatter_core::error::CoreError::InvalidToken(_) => Ok(
          (
            StatusCode::UNAUTHORIZED,
            headers,
            Json(ErrorOutput::new("Invalid or expired refresh token")),
          )
            .into_response(),
        ),
        fechatter_core::error::CoreError::Validation(_) => Ok(
          (
            StatusCode::UNAUTHORIZED,
            headers,
            Json(ErrorOutput::new("Invalid or expired refresh token")),
          )
            .into_response(),
        ),
        _ => Err(AppError::from(e)),
      }
    }
  }
}

pub(crate) async fn logout_handler(
  State(state): State<AppState>,
  cookies: CookieJar,
  headers: HeaderMap,
  _auth_user: Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
  let mut response_headers = HeaderMap::new();
  clear_refresh_token_cookie(&mut response_headers)?;

  // Try to get refresh token from cookie
  let refresh_token_str = if let Some(cookie) = cookies.get("refresh_token") {
    let token = cookie.value().to_string();
    state.logout(&token).await?;
    Some(token)
  } else {
    None
  };

  // If not in cookie, try to get from Authorization header
  if refresh_token_str.is_none() {
    if let Some(auth_header) = headers.get("Authorization") {
      if let Ok(auth_value) = auth_header.to_str() {
        if auth_value.starts_with("Bearer ") {
          let token = auth_value[7..].to_string();
          state.logout(&token).await?;
        }
      }
    }
  }

  Ok(
    (
      StatusCode::OK,
      response_headers,
      Json(serde_json::json!({
        "message": "Logged out successfully"
      })),
    )
      .into_response(),
  )
}

pub(crate) async fn logout_all_handler(
  State(state): State<AppState>,
  cookies: CookieJar,
  headers: HeaderMap,
  auth_user: Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
  let mut response_headers = HeaderMap::new();
  // Clear refresh_token cookie
  clear_refresh_token_cookie(&mut response_headers)?;

  let user_id = auth_user.id;

  // Log the operation
  if cookies.get("refresh_token").is_some() {
    tracing::info!(
      "Logout all sessions with token from cookie for user {}",
      user_id
    );
  } else if headers.get("Authorization").is_some() {
    tracing::info!(
      "Logout all sessions with token from header for user {}",
      user_id
    );
  } else {
    tracing::info!("Logout all sessions without token for user {}", user_id);
  }

  // Delegate to service to revoke all tokens
  match state.logout_all(user_id).await {
    Ok(_) => {
      tracing::info!("All sessions for user {} revoked successfully", user_id);
    }
    Err(e) => {
      tracing::warn!("Error revoking all sessions for user {}: {}", user_id, e);
    }
  }

  Ok(
    (
      StatusCode::OK,
      response_headers,
      Json(serde_json::json!({
        "message": "Logged out from all sessions successfully"
      })),
    )
      .into_response(),
  )
}

#[derive(Serialize, Deserialize)]
struct AuthResponse {
  access_token: String,
  expires_in: usize,
  refresh_token: Option<String>,
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

    let payload = CreateUser {
      fullname: "Test User".to_string(),
      email: "testuser1@acme.test".to_string(),
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

  #[tokio::test]
  async fn signup_user_duplicate_should_409() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user1 = users.into_iter().next().unwrap();

    let payload = CreateUser {
      fullname: user1.fullname.clone(),
      email: user1.email.clone(),
      password: "newpassword".to_string(),
      workspace: "Acme".to_string(),
    };

    let test_handler = |state, payload| async {
      signup_handler(State(state), HeaderMap::new(), Json(payload)).await
    };

    // 测试通过state直接调用，确保返回的是正确的核心错误类型
    let core_result = state.signup(&payload, None).await;
    match core_result {
      Err(fechatter_core::error::CoreError::UserAlreadyExists(msg)) => {
        assert!(msg.contains(&user1.email));
      }
      _ => {
        panic!(
          "Expected CoreError::UserAlreadyExists, but got: {:?}",
          core_result
        );
      }
    }

    // 测试handler的响应码
    // 由于handler会将UserAlreadyExists映射到CONFLICT响应
    match test_handler(state.clone(), payload.clone()).await {
      Ok(resp) => {
        let http_resp = resp.into_response();
        assert_eq!(http_resp.status(), StatusCode::CONFLICT);

        let body = BodyExt::collect(http_resp.into_body()).await?.to_bytes();
        let error_output: ErrorOutput = serde_json::from_slice(&body)?;
        assert!(error_output.error.contains("already exists"));
      }
      Err(err) => {
        panic!(
          "Handler should return Ok with CONFLICT status, not Err. Got: {:?}",
          err
        );
      }
    }

    Ok(())
  }

  #[tokio::test]
  async fn signin_non_existing_user_should_403() -> Result<()> {
    let (_tdb, state, _users) = setup_test_users!(0).await;

    let input = SigninUser::new("nonexistent@acme.test", "password");

    let test_handler =
      |state, input| async { signin_handler(State(state), HeaderMap::new(), Json(input)).await };

    let response = test_handler(state, input).await?.into_response();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = BodyExt::collect(response.into_body()).await?.to_bytes();
    let res: ErrorOutput = serde_json::from_slice(&body)?;
    assert_eq!(res.error, "Invalid credentials");

    Ok(())
  }

  #[tokio::test]
  async fn refresh_token_handler_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    // 创建一个登录用户的token
    let signin_payload = SigninUser::new(&user.email, "password");
    let signin_result = state.signin(&signin_payload, None).await.unwrap().unwrap();

    // 使用真实的refresh token
    let refresh_token = signin_result.refresh_token.token;

    // 创建cookie jar模拟请求
    let mut jar = CookieJar::new();
    jar = jar.add(Cookie::new("refresh_token", refresh_token));

    // 创建handler
    let test_handler =
      |state, jar| async { refresh_token_handler(State(state), HeaderMap::new(), jar, None).await };

    // 验证成功响应
    let auth_response = assert_handler_success!(
      test_handler(state.clone(), jar),
      StatusCode::OK,
      AuthResponse
    );

    assert_ne!(auth_response.access_token, "");
    assert_eq!(auth_response.expires_in, ACCESS_TOKEN_EXPIRATION);

    Ok(())
  }

  #[tokio::test]
  async fn refresh_token_handler_should_fail_without_refresh_token() -> Result<()> {
    let (_tdb, state, _users) = setup_test_users!(1).await;

    let test_handler =
      |state, jar| async { refresh_token_handler(State(state), HeaderMap::new(), jar, None).await };

    let response = test_handler(state, CookieJar::new()).await?.into_response();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = BodyExt::collect(response.into_body()).await?.to_bytes();
    let res: ErrorOutput = serde_json::from_slice(&body)?;
    assert_eq!(res.error, "No refresh token provided");

    Ok(())
  }

  #[tokio::test]
  async fn refresh_token_handler_should_fail_with_invalid_refresh_token() -> Result<()> {
    let (_tdb, state, _users) = setup_test_users!(1).await;

    let mut jar = CookieJar::new();
    jar = jar.add(Cookie::new("refresh_token", "invalid_token"));

    let test_handler =
      |state, jar| async { refresh_token_handler(State(state), HeaderMap::new(), jar, None).await };

    let response = test_handler(state, jar).await;

    match response {
      Ok(resp) => {
        let resp = resp.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let body = BodyExt::collect(resp.into_body()).await?.to_bytes();
        let res: ErrorOutput = serde_json::from_slice(&body)?;
        assert_eq!(res.error, "Invalid or expired refresh token");
      }
      Err(_) => {
        panic!("Expected response with UNAUTHORIZED status, but got an error");
      }
    }

    Ok(())
  }

  #[tokio::test]
  async fn logout_handler_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    // 创建一个登录用户的token
    let signin_payload = SigninUser::new(&user.email, "password");
    let signin_result = state.signin(&signin_payload, None).await.unwrap().unwrap();

    // 使用真实的refresh token
    let refresh_token = signin_result.refresh_token.token;

    // 创建cookie jar
    let mut jar = CookieJar::new();
    jar = jar.add(Cookie::new("refresh_token", refresh_token.clone()));

    // 设置auth user Extension
    let auth_user = Extension(AuthUser {
      id: user.id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
      workspace_id: user.workspace_id,
    });

    // 测试logout
    let response = logout_handler(State(state.clone()), jar, HeaderMap::new(), auth_user)
      .await?
      .into_response();

    assert_eq!(response.status(), StatusCode::OK);

    let body = BodyExt::collect(response.into_body()).await?.to_bytes();
    let res: serde_json::Value = serde_json::from_slice(&body)?;
    assert_eq!(res["message"], "Logged out successfully");

    // 尝试使用已经登出的token
    let mut jar2 = CookieJar::new();
    jar2 = jar2.add(Cookie::new("refresh_token", refresh_token));

    let test_refresh =
      |state, jar| async { refresh_token_handler(State(state), HeaderMap::new(), jar, None).await };

    let refresh_response = test_refresh(state.clone(), jar2).await;

    match refresh_response {
      Ok(resp) => {
        let resp = resp.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
      }
      Err(_) => {
        panic!("Expected UNAUTHORIZED response, but got an error");
      }
    }

    Ok(())
  }
}
