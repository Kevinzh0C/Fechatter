use crate::models::AuthUser;
use crate::utils::jwt::{ACCESS_TOKEN_EXPIRATION, RefreshToken, generate_refresh_token};
use crate::{AppState, ErrorOutput, SigninUser, User, error::AppError, models::CreateUser};
use axum::{
  Json,
  extract::{Extension, State},
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
  refresh_token: &str,
  expires_at: &chrono::DateTime<chrono::Utc>,
  headers: &mut HeaderMap,
) -> Result<(), AppError> {
  let cookie = format!(
    "refresh_token={}; HttpOnly; Secure; SameSite=Lax; Path=/api; Expires={}",
    refresh_token,
    expires_at.format("%a, %d %b %Y %H:%M:%S GMT")
  );

  headers.insert(header::SET_COOKIE, HeaderValue::from_str(&cookie)?);

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
  let user = User::create(&payload, &state.pool).await?;

  let user_agent = None;
  let ip_address = None;

  let tokens = state
    .token_manager
    .generate_auth_tokens(&user, user_agent, ip_address, &state.pool)
    .await?;

  let mut headers = HeaderMap::new();
  set_refresh_token_cookie(
    &tokens.refresh_token.token,
    &tokens.refresh_token.expires_at,
    &mut headers,
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
  let user = User::authenticate(&payload, &state.pool).await?;

  match user {
    Some(user) => {
      let user_agent = None;
      let ip_address = None;

      let tokens = state
        .token_manager
        .generate_auth_tokens(&user, user_agent, ip_address, &state.pool)
        .await?;

      let mut headers = HeaderMap::new();
      set_refresh_token_cookie(
        &tokens.refresh_token.token,
        &tokens.refresh_token.expires_at,
        &mut headers,
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
  let refresh_token = match cookies.get("refresh_token") {
    Some(cookie) => cookie.value(),
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

  let token_record = match RefreshToken::find_by_token(refresh_token, &state.pool).await? {
    Some(token) => token,
    None => {
      return Ok(
        (
          StatusCode::UNAUTHORIZED,
          Json(ErrorOutput::new("Invalid or expired refresh token")),
        )
          .into_response(),
      );
    }
  };

  let user = match User::find_by_id(token_record.user_id, &state.pool).await? {
    Some(user) => user,
    None => {
      return Ok(
        (
          StatusCode::UNAUTHORIZED,
          Json(ErrorOutput::new("User not found")),
        )
          .into_response(),
      );
    }
  };

  let user_agent = None;
  let ip_address = None;

  let new_refresh_token = generate_refresh_token();
  let new_token_record = token_record
    .replace(&new_refresh_token, user_agent, ip_address, &state.pool)
    .await?;

  let access_token = state.token_manager.generate_token(&user)?;

  let mut headers = HeaderMap::new();
  set_refresh_token_cookie(
    &new_refresh_token,
    &new_token_record.expires_at,
    &mut headers,
  )?;

  let body = Json(AuthResponse {
    access_token,
    expires_in: ACCESS_TOKEN_EXPIRATION,
  });

  Ok((StatusCode::OK, headers, body).into_response())
}

pub(crate) async fn logout_handler(
  State(state): State<AppState>,
  cookies: CookieJar,
  _auth_user: Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
  let mut headers = HeaderMap::new();
  clear_refresh_token_cookie(&mut headers)?;

  if let Some(cookie) = cookies.get("refresh_token") {
    if let Some(token_record) = RefreshToken::find_by_token(cookie.value(), &state.pool).await? {
      token_record.revoke(&state.pool).await?;
    }
  }

  // RefreshToken::revoke_all_for_user(auth_user.id, &state.pool).await?;

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
  use http_body_util::BodyExt;

  #[tokio::test]
  async fn signup_handler_should_work() -> Result<()> {
    let (_tdb, state, _users) = setup_test_users!(0).await;

    let payload = CreateUser {
      fullname: "Test User".to_string(),
      email: "testuser1@acme.test".to_string(),
      password: "password".to_string(),
    };

    let auth_response = assert_handler_success!(
      signup_handler(State(state), Json(payload)),
      StatusCode::CREATED,
      AuthResponse
    );

    assert_ne!(auth_response.access_token, "");
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
}
