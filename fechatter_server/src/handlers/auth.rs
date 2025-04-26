use crate::models::AuthUser;
use crate::services::auth_service;
use crate::utils::jwt::{ACCESS_TOKEN_EXPIRATION, RefreshToken, generate_refresh_token};
use crate::{AppState, ErrorOutput, SigninUser, User, error::AppError, models::CreateUser};
use axum::{
  Extension, Json,
  extract::State,
  http::{HeaderMap, HeaderValue, StatusCode, header},
  response::IntoResponse,
};
use axum_extra::extract::cookie::CookieJar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct AuthResponse {
  access_token: String,
  expires_in: usize,
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
  Json(payload): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
  let tokens = auth_service::signup(&state.pool, &state.token_manager, &payload).await?;

  let mut headers = HeaderMap::new();
  set_refresh_token_cookie(
    &mut headers,
    &tokens.refresh_token.token,
    &tokens.refresh_token.expires_at,
  )?;

  let body = Json(AuthResponse {
    access_token: tokens.access_token,
    expires_in: ACCESS_TOKEN_EXPIRATION,
  });

  Ok((StatusCode::CREATED, headers, body).into_response())
}

pub(crate) async fn signin_handler(
  State(state): State<AppState>,
  Json(payload): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
  match auth_service::signin(&state.pool, &state.token_manager, &payload).await? {
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
  cookies: CookieJar,
) -> Result<impl IntoResponse, AppError> {
  let refresh_token_str = match cookies.get("refresh_token") {
    Some(cookie) => cookie.value().to_string(),
    None => {
      return Ok(
        (
          StatusCode::UNAUTHORIZED,
          Json(ErrorOutput::new("Refresh token not provided")),
        )
          .into_response(),
      );
    }
  };

  match auth_service::refresh_token(&state.pool, &state.token_manager, &refresh_token_str).await {
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
      });

      Ok((StatusCode::OK, headers, body).into_response())
    }
    Err(AppError::InvalidInput(msg)) => {
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
    Err(e) => Err(e),
  }
}

pub(crate) async fn logout_handler(
  State(state): State<AppState>,
  cookies: CookieJar,
  _auth_user: Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
  let mut headers = HeaderMap::new();
  clear_refresh_token_cookie(&mut headers)?;

  if let Some(cookie) = cookies.get("refresh_token") {
    let _ = auth_service::logout(&state.pool, cookie.value()).await;
  }

  Ok(
    (
      StatusCode::OK,
      headers,
      Json(serde_json::json!({
        "message": "Logged out successfully"
      })),
    )
      .into_response(),
  )
}
#[cfg(test)]
mod tests {
  use super::*;
  use crate::ErrorOutput;
  use crate::{assert_handler_error, assert_handler_success, setup_test_users};
  use anyhow::Result;
  use axum::{Json, http::StatusCode};
  use axum_extra::extract::cookie::{CookieJar, Cookie};
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

    let auth_response = assert_handler_success!(
      signup_handler(State(state), Json(payload)),
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

    let auth_response = assert_handler_success!(
      signin_handler(State(state), Json(input)),
      StatusCode::OK,
      AuthResponse
    );

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

    assert_handler_error!(
        signup_handler(State(state.clone()), Json(payload.clone())),
        AppError::UserAlreadyExists(email) if email == user1.email
    );

    Ok(())
  }

  #[tokio::test]
  async fn signin_non_existing_user_should_403() -> Result<()> {
    let (_tdb, state, _users) = setup_test_users!(0).await;

    let input = SigninUser::new("nonexistent@acme.test", "password");

    let response = signin_handler(State(state), Json(input))
      .await?
      .into_response();
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

    let tokens = state.token_manager
      .generate_auth_tokens(user, None, None, &state.pool)
      .await?;

    let mut jar = CookieJar::new();
    jar = jar.add(Cookie::new("refresh_token", tokens.refresh_token.token.clone()));

    let auth_response = assert_handler_success!(
      refresh_token_handler(State(state), jar),
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

    let response = refresh_token_handler(State(state), CookieJar::new())
      .await?
      .into_response();

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

    let response = refresh_token_handler(State(state), jar)
      .await;

    match response {
      Ok(resp) => {
        let resp = resp.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let body = BodyExt::collect(resp.into_body()).await?.to_bytes();
        let res: ErrorOutput = serde_json::from_slice(&body)?;
        assert_eq!(res.error, "Invalid or expired refresh token");
      },
      Err(_) => {
      }
    }

    Ok(())
  }

  #[tokio::test]
  async fn logout_handler_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = &users[0];

    let tokens = state.token_manager
      .generate_auth_tokens(user, None, None, &state.pool)
      .await?;

    let mut jar = CookieJar::new();
    jar = jar.add(Cookie::new("refresh_token", tokens.refresh_token.token.clone()));

    let auth_user = Extension(AuthUser {
      id: user.id,
      fullname: user.fullname.clone(),
      email: user.email.clone(),
      status: user.status,
      created_at: user.created_at,
      workspace_id: user.workspace_id,
    });

    let response = logout_handler(State(state.clone()), jar, auth_user)
      .await?
      .into_response();

    assert_eq!(response.status(), StatusCode::OK);
    let body = BodyExt::collect(response.into_body()).await?.to_bytes();
    let res: serde_json::Value = serde_json::from_slice(&body)?;
    assert_eq!(res["message"], "Logged out successfully");

    let mut jar2 = CookieJar::new();
    jar2 = jar2.add(Cookie::new("refresh_token", tokens.refresh_token.token));

    let refresh_response = refresh_token_handler(State(state), jar2)
      .await;

    match refresh_response {
      Ok(resp) => {
        let resp = resp.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
      },
      Err(_) => {
      }
    }

    Ok(())
  }
}
