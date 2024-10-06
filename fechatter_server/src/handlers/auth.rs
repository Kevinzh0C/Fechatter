use crate::{error::AppError, models::CreateUser, AppState, ErrorOutput, SigninUser, User};
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
  use crate::AppConfig;
  use anyhow::Result;
  use http_body_util::BodyExt;

  #[tokio::test]
  async fn signup_handler_should_work() -> Result<()> {
    let config = AppConfig::load()?;
    let (_tdb, state) = AppState::test_new(config).await?;

    let payload = CreateUser {
      fullname: "test".to_string(),
      email: "test@test.com".to_string(),
      password: "password".to_string(),
    };

    let response = signup_handler(State(state), Json(payload))
      .await
      .into_response();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body();
    let bytes = body.collect().await?.to_bytes();
    let body: AuthResponse = serde_json::from_slice(&bytes).unwrap();
    assert_ne!(body.token, "");
    Ok(())
  }

  #[tokio::test]
  async fn signin_handler_should_work() -> Result<()> {
    let config = AppConfig::load()?;

    let (_tdb, state) = AppState::test_new(config).await?;
    let name = "Alice";
    let email = "alice@acme.com";
    let password = "password";
    let user = CreateUser::new(name, email, password);
    User::create(&user, &state.pool).await?;

    let input = SigninUser::new(email, password);

    let ret = signin_handler(State(state), Json(input))
      .await
      .into_response();
    assert_eq!(ret.status(), StatusCode::OK);

    let body = ret.into_body().collect().await?.to_bytes();
    let ret: AuthResponse = serde_json::from_slice(&body).unwrap();
    assert_ne!(ret.token, "");
    Ok(())
  }

  #[tokio::test]
  async fn signup_user_duplicate_should_409() -> Result<()> {
    let config = AppConfig::load()?;
    let (_tdb, state) = AppState::test_new(config).await?;

    let payload = CreateUser {
      fullname: "Alice".to_string(),
      email: "alice@acme.com".to_string(),
      password: "password".to_string(),
    };

    signup_handler(State(state.clone()), Json(payload.clone())).await?;

    let response = signup_handler(State(state.clone()), Json(payload.clone()))
      .await
      .into_response();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    let body = response.into_body();
    let bytes = body.collect().await?.to_bytes();
    let res: ErrorOutput = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(res.error, format!("user already exists: {}", payload.email));
    Ok(())
  }

  #[tokio::test]
  async fn signin_non_existing_user_should_403() -> Result<()> {
    let config = AppConfig::load()?;
    let (_tdb, state) = AppState::test_new(config).await?;

    let input = SigninUser::new("bob@acme.com", "newclaude");
    let response = signin_handler(State(state), Json(input))
      .await
      .into_response();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = response.into_body().collect().await?.to_bytes();
    let res: ErrorOutput = serde_json::from_slice(&body).unwrap();
    assert_eq!(res.error, "Invalid credentials");
    Ok(())
  }
}
