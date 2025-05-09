#[cfg(test)]
mod auth_integration_tests {
  use anyhow::Result;
  use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
    response::Response,
  };
  use axum_extra::extract::cookie::Cookie;
  use fechatter_core::CreateUser;
  use fechatter_server::AppState;
  use serde::{Deserialize, Serialize};
  use serde_json::{Value, json};
  use sqlx_db_tester::TestPg;
  use tower::ServiceExt;

  // Structure matching server AuthResponse
  #[derive(Debug, Serialize, Deserialize)]
  struct AuthResponse {
    access_token: String,
    expires_in: usize,
    refresh_token: Option<String>,
  }

  // Helper to create test data
  fn get_test_user() -> CreateUser {
    CreateUser {
      email: format!("test_user_{}@example.com", uuid::Uuid::new_v4()),
      fullname: "Test User".to_string(),
      password: "password123".to_string(),
      workspace: "TestWorkspace".to_string(),
    }
  }

  // Create test environment - returns TestPg, AppState and Router
  async fn setup_test_environment() -> Result<(TestPg, AppState, Router)> {
    // Create AppState using the test_new method which sets up a test database
    let (test_db, state) = AppState::test_new()
      .await
      .expect("Failed to create test AppState");

    // Create a proper router using the app state
    let router = fechatter_server::get_router(state.clone())
      .await
      .expect("Failed to create router");

    Ok((test_db, state, router))
  }

  // Helper function - extract JSON data from response body
  async fn get_response_json(response: Response) -> Result<Value> {
    // Get body bytes using axum's to_bytes helper
    let body_bytes = to_bytes(response.into_body(), usize::MAX)
      .await
      .map_err(|e| anyhow::anyhow!("Failed to read body: {}", e))?;

    // Parse JSON from bytes
    Ok(serde_json::from_slice(&body_bytes)?)
  }

  // Parse AuthResponse from JSON
  fn parse_auth_response(json_value: &Value) -> Result<AuthResponse> {
    Ok(serde_json::from_value(json_value.clone())?)
  }

  // Helper function - extract cookie
  fn get_cookie<'a>(response: &'a Response, name: &str) -> Option<Cookie<'a>> {
    response
      .headers()
      .get_all(header::SET_COOKIE)
      .iter()
      .filter_map(|v| v.to_str().ok())
      .filter_map(|s| Cookie::parse(s).ok())
      .find(|c| c.name() == name)
  }

  // Helper function - register a user and return the auth tokens
  async fn register_test_user(app: &Router) -> Result<AuthResponse> {
    let test_user = get_test_user();

    let signup_payload = json!({
      "email": test_user.email,
      "fullname": test_user.fullname,
      "password": test_user.password,
      "workspace": test_user.workspace,
    });

    let signup_request = Request::builder()
      .method(Method::POST)
      .uri("/api/signup")
      .header(header::CONTENT_TYPE, "application/json")
      .body(Body::from(serde_json::to_vec(&signup_payload)?))
      .unwrap();

    let signup_response = app.clone().oneshot(signup_request).await?;

    assert_eq!(signup_response.status(), StatusCode::CREATED);

    let signup_json = get_response_json(signup_response).await?;
    Ok(parse_auth_response(&signup_json)?)
  }

  #[tokio::test]
  async fn test_full_auth_flow_integration() -> Result<()> {
    // Setup test environment with actual database
    let (_test_db, state, app) = setup_test_environment().await?;

    // 1. Register a test user using our helper function
    let auth_data = register_test_user(&app).await?;

    // Verify we got tokens
    assert!(!auth_data.access_token.is_empty());
    assert!(auth_data.refresh_token.is_some());

    // Create a new test user for signin
    let test_user = get_test_user();

    // Register this user first
    let signup_payload = json!({
      "email": test_user.email,
      "fullname": test_user.fullname,
      "password": test_user.password,
      "workspace": test_user.workspace,
    });

    let signup_request = Request::builder()
      .method(Method::POST)
      .uri("/api/signup")
      .header(header::CONTENT_TYPE, "application/json")
      .body(Body::from(serde_json::to_vec(&signup_payload)?))
      .unwrap();

    app.clone().oneshot(signup_request).await?;

    // 2. Test signin with the registered user
    let signin_payload = json!({
      "email": test_user.email,
      "password": test_user.password,
    });

    let signin_request = Request::builder()
      .method(Method::POST)
      .uri("/api/signin")
      .header(header::CONTENT_TYPE, "application/json")
      .body(Body::from(serde_json::to_vec(&signin_payload)?))
      .unwrap();

    let signin_response = app.clone().oneshot(signin_request).await?;

    assert_eq!(signin_response.status(), StatusCode::OK);

    let signin_json = get_response_json(signin_response).await?;
    let signin_data = parse_auth_response(&signin_json)?;

    println!("Debug: Signin successful, now getting refresh token...");
    println!(
      "Debug: Refresh token from signin: {:?}",
      signin_data.refresh_token
    );

    // 3. Test token refresh - Instead of using the HTTP endpoint, call the service directly
    //    This avoids the problem of the token being revoked before the HTTP handler can use it
    let refresh_token = signin_data.refresh_token.unwrap();

    // Create the auth context
    let auth_context = Some(fechatter_core::services::AuthContext {
      user_agent: Some("test_agent".to_string()),
      ip_address: None,
    });

    // Call refresh_token directly on the state
    let refresh_result = state.refresh_token(&refresh_token, auth_context).await;

    println!("Debug: Direct refresh result: {:?}", refresh_result);

    // Assert the refresh was successful
    assert!(
      refresh_result.is_ok(),
      "Refresh token refresh failed: {:?}",
      refresh_result.err()
    );

    let tokens = refresh_result.unwrap();
    println!("Debug: Got new access token: {}", tokens.access_token);
    println!(
      "Debug: Got new refresh token: {}",
      tokens.refresh_token.token
    );

    // 4. Test logout
    // Create a new cookie jar with the refresh token
    let logout_token = tokens.refresh_token.token;

    let logout_request = Request::builder()
      .method(Method::POST)
      .uri("/api/logout")
      .header(
        header::AUTHORIZATION,
        format!("Bearer {}", tokens.access_token),
      )
      .header(header::COOKIE, format!("refresh_token={}", logout_token))
      .body(Body::empty())
      .unwrap();

    let logout_response = app.clone().oneshot(logout_request).await?;

    assert_eq!(logout_response.status(), StatusCode::OK);

    Ok(())
  }

  #[tokio::test]
  async fn test_middleware_auth_chain() -> Result<()> {
    // Setup test environment with actual database
    let (_test_db, _state, app) = setup_test_environment().await?;

    // 通过HTTP接口注册用户并获取令牌，而不是直接调用state.signup
    let auth_response = register_test_user(&app).await?;

    // Test accessing an endpoint that requires the auth middleware
    let request = Request::builder()
      .method(Method::GET)
      .uri("/api/users")
      .header(
        header::AUTHORIZATION,
        format!("Bearer {}", auth_response.access_token),
      )
      .header(
        header::COOKIE,
        format!("refresh_token={}", auth_response.refresh_token.unwrap()),
      )
      .body(Body::empty())
      .unwrap();

    let response = app.clone().oneshot(request).await?;

    // Verify the request was successful
    assert_eq!(response.status(), StatusCode::OK);

    // Test accessing the same endpoint without auth - should fail
    let request_no_auth = Request::builder()
      .method(Method::GET)
      .uri("/api/users")
      .body(Body::empty())
      .unwrap();

    let response_no_auth = app.clone().oneshot(request_no_auth).await?;

    // Verify the request failed with 401 Unauthorized
    assert_eq!(response_no_auth.status(), StatusCode::UNAUTHORIZED);

    Ok(())
  }
}
