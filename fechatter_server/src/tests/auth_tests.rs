#[cfg(test)]
mod cookie_refresh_tests {
  use anyhow::Result;
  use axum::{
    Json,
    body::to_bytes,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::IntoResponse,
  };
  use chrono::{Duration, Utc};
  use fechatter_core::{AuthTokens, RefreshTokenData};

  // Basic function to create mock auth tokens for tests
  fn create_mock_tokens() -> AuthTokens {
    let expires_at = Utc::now() + Duration::days(7);
    let refresh_token_data = RefreshTokenData {
      token: "test_refresh_token".to_string(),
      expires_at,
      absolute_expires_at: expires_at + Duration::days(30),
    };

    AuthTokens {
      access_token: "test_access_token".to_string(),
      refresh_token: refresh_token_data,
    }
  }

  #[allow(dead_code)]
  fn get_cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
      .get_all(header::SET_COOKIE)
      .iter()
      .filter_map(|v| v.to_str().ok())
      .find_map(|cookie_str| {
        if cookie_str.starts_with(&format!("{}=", name)) {
          let parts: Vec<&str> = cookie_str.split(';').collect();
          if !parts.is_empty() {
            let cookie_pair = parts[0].trim();
            let mut cookie_parts = cookie_pair.split('=');
            if cookie_parts.next() == Some(name) {
              return cookie_parts.next().map(|v| v.to_string());
            }
          }
        }
        None
      })
  }

  #[tokio::test]
  async fn test_cookie_value_extraction() -> Result<()> {
    let mut headers = HeaderMap::new();
    headers.insert(
      header::SET_COOKIE,
      HeaderValue::from_str("refresh_token=test_token; Path=/; HttpOnly").unwrap(),
    );

    let cookie_value = get_cookie_value(&headers, "refresh_token");
    assert_eq!(cookie_value, Some("test_token".to_string()));

    let non_existent = get_cookie_value(&headers, "non_existent");
    assert_eq!(non_existent, None);

    Ok(())
  }

  fn create_success_response(tokens: &AuthTokens) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    let cookie = format!(
      "refresh_token={}; Path=/; HttpOnly; SameSite=Strict; Secure",
      tokens.refresh_token.token
    );
    headers.insert(header::SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());

    (
      StatusCode::OK,
      headers,
      Json(serde_json::json!({
        "access_token": tokens.access_token,
        "refresh_token": tokens.refresh_token.token
      })),
    )
  }

  fn create_error_response(error_message: &str) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    let clear_cookie = "refresh_token=; Path=/; Max-Age=0; HttpOnly; SameSite=Strict; Secure";
    headers.insert(
      header::SET_COOKIE,
      HeaderValue::from_str(clear_cookie).unwrap(),
    );

    (
      StatusCode::UNAUTHORIZED,
      headers,
      Json(serde_json::json!({
        "error": error_message
      })),
    )
  }

  #[tokio::test]
  async fn test_success_response_with_refresh_token() -> Result<()> {
    let tokens = create_mock_tokens();
    let response = create_success_response(&tokens).into_response();

    assert_eq!(response.status(), StatusCode::OK);

    let cookie_value = get_cookie_value(response.headers(), "refresh_token");
    assert_eq!(cookie_value, Some("test_refresh_token".to_string()));

    let body_bytes = to_bytes(response.into_body(), usize::MAX).await?;
    let body: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    assert_eq!(body["access_token"], "test_access_token");

    Ok(())
  }

  #[tokio::test]
  async fn test_error_response_clears_cookie() -> Result<()> {
    let response = create_error_response("Invalid token").into_response();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let cookie_header = response
      .headers()
      .get(header::SET_COOKIE)
      .unwrap()
      .to_str()
      .unwrap();
    assert!(cookie_header.contains("Max-Age=0"));

    let body_bytes = to_bytes(response.into_body(), usize::MAX).await?;
    let body: serde_json::Value = serde_json::from_slice(&body_bytes)?;
    assert_eq!(body["error"], "Invalid token");

    Ok(())
  }
}
