use crate::models::AuthService;
use crate::models::AuthUser;
use crate::{AppState, ErrorOutput, SigninUser, error::AppError, models::CreateUser};
use axum::{
  Extension, Json,
  extract::State,
  http::{HeaderMap, HeaderValue, StatusCode, header},
  response::IntoResponse,
};
use axum_extra::extract::cookie::CookieJar;

use fechatter_core::utils::jwt::ACCESS_TOKEN_EXPIRATION;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize)]
struct AuthResponse {
  access_token: String,
  expires_in: usize,
  refresh_token: Option<String>,
}

fn set_refresh_token_cookie(
  headers: &mut HeaderMap,
  token_str: &str,
  expires_at: &chrono::DateTime<chrono::Utc>,
) -> Result<(), AppError> {
  let cookie = format!(
    "refresh_token={}; HttpOnly; Secure; SameSite=Lax; Path=/api; Expires={}",
    token_str,
    expires_at.format("%a, %d %b %Y %H:%M:%S GMT")
  );

  headers.append(header::SET_COOKIE, HeaderValue::from_str(&cookie)?);

  Ok(())
}

fn clear_refresh_token_cookie(headers: &mut HeaderMap) -> Result<(), AppError> {
  let cookie = "refresh_token=; HttpOnly; Secure; SameSite=Lax; Path=/api; Max-Age=0";

  headers.insert(header::SET_COOKIE, HeaderValue::from_str(cookie)?);

  Ok(())
}

pub(crate) async fn signup_handler(
  State(state): State<AppState>,
  headers: HeaderMap,
  Json(payload): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
  let user_agent = headers
    .get("user-agent")
    .and_then(|h| h.to_str().ok())
    .map(String::from);
  let ip_address = headers
    .get("x-forwarded-for")
    .and_then(|h| h.to_str().ok())
    .map(String::from);

  let auth_service = AuthService::new(&state.service_provider);
  let tokens = auth_service
    .signup(&payload, user_agent, ip_address)
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

  Ok((StatusCode::CREATED, response_headers, body).into_response())
}

pub(crate) async fn signin_handler(
  State(state): State<AppState>,
  headers: HeaderMap,
  Json(payload): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
  let user_agent = headers
    .get("user-agent")
    .and_then(|h| h.to_str().ok())
    .map(String::from);
  let ip_address = headers
    .get("x-forwarded-for")
    .and_then(|h| h.to_str().ok())
    .map(String::from);

  let auth_service: Box<dyn AuthServiceTrait> =
    state.service_provider.create_service::<AuthService>();
  match auth_service
    .signin(&payload, user_agent, ip_address)
    .await?
  {
    Some(tokens) => {
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
    None => {
      let body = Json(ErrorOutput::new("Invalid credentials"));
      Ok((StatusCode::FORBIDDEN, body).into_response())
    }
  }
}

pub(crate) async fn refresh_token_handler(
  State(state): State<AppState>,
  headers: HeaderMap,
  cookies: CookieJar,
) -> Result<impl IntoResponse, AppError> {
  let user_agent = headers
    .get("user-agent")
    .and_then(|h| h.to_str().ok())
    .map(String::from);
  let ip_address = headers
    .get("x-forwarded-for")
    .and_then(|h| h.to_str().ok())
    .map(String::from);

  let refresh_token_str = match cookies.get("refresh_token") {
    Some(cookie) => {
      let token = cookie.value().to_string();
      token
    }
    None => {
      if let Some(auth_header) = headers.get("Authorization") {
        let auth_value = auth_header
          .to_str()
          .map_err(|_| AppError::InvalidInput("Invalid Authorization header".to_string()))?;

        if auth_value.starts_with("Bearer ") {
          let token = auth_value[7..].to_string();
          token
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
        return Ok(
          (
            StatusCode::UNAUTHORIZED,
            Json(ErrorOutput::new("Refresh token not provided")),
          )
            .into_response(),
        );
      }
    }
  };

  let auth_service: Box<dyn AuthServiceTrait> =
    state.service_provider.create_service::<AuthService>();
  match auth_service
    .refresh_token(&refresh_token_str, user_agent, ip_address)
    .await
  {
    Ok(tokens) => {
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
    Err(e) => match e {
      fechatter_core::CoreError::Validation(msg) => {
        let mut headers = HeaderMap::new();
        clear_refresh_token_cookie(&mut headers)?;
        Ok(
          (
            StatusCode::UNAUTHORIZED,
            headers,
            Json(ErrorOutput::new(msg)),
          )
            .into_response(),
        )
      }
      _ => Err(AppError::from(e)),
    },
  }
}

pub(crate) async fn logout_handler(
  State(state): State<AppState>,
  cookies: CookieJar,
  headers: HeaderMap,
  _auth_user: Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
  // Authentication is already checked by middleware, so we can proceed
  // The auth_user extension is already provided, so we don't need to check it again

  let mut response_headers = HeaderMap::new();
  clear_refresh_token_cookie(&mut response_headers)?;

  let auth_service: Box<dyn AuthServiceTrait> =
    state.service_provider.create_service::<AuthService>();

  // First try to get refresh token from cookie
  let refresh_token_str = if let Some(cookie) = cookies.get("refresh_token") {
    let token = cookie.value().to_string();
    let _ = auth_service.logout(&token).await;
    Some(token)
  } else {
    None
  };

  // If not found in cookie, try to get from Authorization header
  if refresh_token_str.is_none() {
    if let Some(auth_header) = headers.get("Authorization") {
      if let Ok(auth_value) = auth_header.to_str() {
        if auth_value.starts_with("Bearer ") {
          let token = auth_value[7..].to_string();
          let _ = auth_service.logout(&token).await;
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
  _auth_user: Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
  let mut response_headers = HeaderMap::new();
  // Clear refresh_token cookie
  clear_refresh_token_cookie(&mut response_headers)?;

  let auth_service: Box<dyn AuthServiceTrait> =
    state.service_provider.create_service::<AuthService>();
  let user_id = _auth_user.id;

  // Try to get refresh token from cookie
  let refresh_token_from_cookie = cookies
    .get("refresh_token")
    .map(|cookie| cookie.value().to_string());

  // Try to get refresh token from Authorization header
  let refresh_token_from_header = match headers.get("Authorization") {
    Some(auth_header) => {
      if let Ok(auth_value) = auth_header.to_str() {
        if auth_value.starts_with("Bearer ") {
          Some(auth_value[7..].to_string())
        } else {
          None
        }
      } else {
        None
      }
    }
    None => None,
  };

  match auth_service.logout_all(user_id).await {
    Ok(_) => {
      info!("All sessions for user {} revoked successfully", user_id);
    }
    Err(e) => {
      warn!("Error revoking all sessions for user {}: {}", user_id, e);
    }
  }

  // Log for debugging
  if refresh_token_from_cookie.is_some() {
    info!(
      "Logout all sessions with token from cookie for user {}",
      user_id
    );
  } else if refresh_token_from_header.is_some() {
    info!(
      "Logout all sessions with token from header for user {}",
      user_id
    );
  } else {
    info!("Logout all sessions without token for user {}", user_id);
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

    let response = test_handler(state.clone(), payload.clone()).await?;
    let response = response.into_response();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    let body = BodyExt::collect(response.into_body()).await?.to_bytes();
    let error_output: ErrorOutput = serde_json::from_slice(&body)?;
    assert!(error_output.error.contains("already exists"));

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

    let auth_service: Box<dyn AuthServiceTrait> =
      state.service_provider.create_service::<AuthService>();
    let tokens = auth_service.generate_auth_tokens(user, None, None).await?;

    let mut jar = CookieJar::new();
    jar = jar.add(Cookie::new(
      "refresh_token",
      tokens.refresh_token.token.clone(),
    ));

    let test_handler =
      |state, jar| async { refresh_token_handler(State(state), HeaderMap::new(), jar).await };

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
      |state, jar| async { refresh_token_handler(State(state), HeaderMap::new(), jar).await };

    let response = test_handler(state, CookieJar::new()).await?.into_response();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = BodyExt::collect(response.into_body()).await?.to_bytes();
    let res: ErrorOutput = serde_json::from_slice(&body)?;
    assert_eq!(res.error, "Refresh token not provided");

    Ok(())
  }

  #[tokio::test]
  async fn refresh_token_handler_should_fail_with_invalid_refresh_token() -> Result<()> {
    let (_tdb, state, _users) = setup_test_users!(1).await;

    let mut jar = CookieJar::new();
    jar = jar.add(Cookie::new("refresh_token", "invalid_token"));

    let test_handler =
      |state, jar| async { refresh_token_handler(State(state), HeaderMap::new(), jar).await };

    let response = test_handler(state, jar).await;

    match response {
      Ok(resp) => {
        let resp = resp.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let body = BodyExt::collect(resp.into_body()).await?.to_bytes();
        let res: ErrorOutput = serde_json::from_slice(&body)?;
        assert_eq!(res.error, "Invalid or expired refresh token");
      }
      Err(_) => {}
    }

    Ok(())
  }

  #[tokio::test]
  async fn logout_handler_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    let auth_service: Box<dyn AuthServiceTrait> =
      state.service_provider.create_service::<AuthService>();
    let tokens = auth_service.generate_auth_tokens(user, None, None).await?;

    let mut jar = CookieJar::new();
    jar = jar.add(Cookie::new(
      "refresh_token",
      tokens.refresh_token.token.clone(),
    ));

    // Set up auth user Extension directly for the test
    let auth_user = Extension(AuthUser {
      id: user.id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
      workspace_id: user.workspace_id,
    });

    // Create a simplified test that just passes the user directly
    let response = logout_handler(State(state.clone()), jar, HeaderMap::new(), auth_user)
      .await?
      .into_response();

    println!("Logout response status: {:?}", response.status());
    assert_eq!(response.status(), StatusCode::OK);

    let body = BodyExt::collect(response.into_body()).await?.to_bytes();
    let res: serde_json::Value = serde_json::from_slice(&body)?;
    assert_eq!(res["message"], "Logged out successfully");

    let mut jar2 = CookieJar::new();
    jar2 = jar2.add(Cookie::new("refresh_token", tokens.refresh_token.token));

    let test_refresh =
      |state, jar| async { refresh_token_handler(State(state), HeaderMap::new(), jar).await };

    let refresh_response = test_refresh(state.clone(), jar2).await;

    match refresh_response {
      Ok(resp) => {
        let resp = resp.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
      }
      Err(_) => {}
    }

    Ok(())
  }
}
