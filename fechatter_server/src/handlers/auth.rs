use crate::{AppState, ErrorOutput, SigninUser, User, error::AppError, models::CreateUser};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct AuthResponse {
  token: String,
}

pub(crate) async fn signup_handler(
  State(state): State<AppState>,
  Json(payload): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
  let user = User::create(&payload, &state.pool).await?;
  let token = state.token_manager.generate_token(&user)?;
  // let mut header = HeaderMap::new();
  // header.insert("X-Token", HeaderValue::from_str(&token)?);
  // Ok((StatusCode::CREATED, header))
  let body = Json(AuthResponse { token });
  Ok((StatusCode::CREATED, body))
}

pub(crate) async fn signin_handler(
  State(state): State<AppState>,
  Json(payload): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
  let user = User::authenticate(&payload, &state.pool).await?;

  match user {
    Some(user) => {
      let token = state.token_manager.generate_token(&user)?;
      Ok((StatusCode::OK, Json(AuthResponse { token })).into_response())
    }
    None => {
      let body = Json(ErrorOutput::new("Invalid credentials"));
      Ok((StatusCode::FORBIDDEN, body).into_response())
    }
  }
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
      workspace: "Acme".to_string(),
    };

    let auth_response = assert_handler_success!(
      signup_handler(State(state), Json(payload)),
      StatusCode::CREATED,
      AuthResponse
    );

    assert_ne!(auth_response.token, "");
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

    assert_ne!(auth_response.token, "");
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
}
