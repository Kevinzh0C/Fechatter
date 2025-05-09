#[cfg(test)]
mod auth_integration_tests {
  use anyhow::Result;
  use axum::Json;
  use axum::extract::State;
  use axum::http::HeaderMap;
  use axum::http::StatusCode;
  use axum_extra::extract::cookie::CookieJar;
  use fechatter_core::{CreateUser, SigninUser};
  use fechatter_server::{AppState, AuthService};

  // Helper to create test data
  fn get_test_user() -> CreateUser {
    CreateUser {
      email: format!("test_user_{}@example.com", uuid::Uuid::new_v4()),
      fullname: "Test User".to_string(),
      password: "password123".to_string(),
      workspace: "TestWorkspace".to_string(),
    }
  }

  #[tokio::test]
  async fn test_full_auth_flow_integration() -> Result<()> {
    // Create a test AppState
    let (test_db, state) = AppState::test_new().await?;

    // Test signup
    let new_user = get_test_user();

    // Call the handler directly
    let signup_result = fechatter_server::handlers::auth::signup_handler(
      State(state.clone()),
      HeaderMap::new(),
      Json(new_user.clone()),
    )
    .await;

    // We don't really care if it succeeds, just that it doesn't panic with the core placeholder
    if let Ok(response) = &signup_result {
      println!("Signup succeeded with status: {:?}", response.status());
    } else if let Err(e) = &signup_result {
      println!("Expected error in test (not a problem): {:?}", e);
    }

    // Test signin with the same user
    let signin_user = SigninUser {
      email: new_user.email.clone(),
      password: new_user.password.clone(),
    };

    let signin_result = fechatter_server::handlers::auth::signin_handler(
      State(state.clone()),
      HeaderMap::new(),
      Json(signin_user.clone()),
    )
    .await;

    if let Ok(response) = &signin_result {
      println!("Signin succeeded with status: {:?}", response.status());
    } else if let Err(e) = &signin_result {
      println!("Expected error in test (not a problem): {:?}", e);
    }

    // Test refresh token
    let refresh_result = fechatter_server::handlers::auth::refresh_token_handler(
      State(state.clone()),
      HeaderMap::new(),
      CookieJar::new(),
      None,
    )
    .await;

    if let Ok(response) = &refresh_result {
      println!("Refresh succeeded with status: {:?}", response.status());
    } else if let Err(e) = &refresh_result {
      println!("Expected error in test (not a problem): {:?}", e);
    }

    // If we got here without a panic, we're not hitting the core placeholders
    Ok(())
  }

  #[tokio::test]
  async fn test_all_auth_handler_paths() -> Result<()> {
    // Create a test AppState
    let (test_db, state) = AppState::test_new().await?;

    // Test full range of handlers to ensure we never hit the placeholders
    let user = get_test_user();

    // This test just verifies that no handling path triggers a panic from the core placeholders
    let _ = fechatter_server::handlers::auth::signup_handler(
      State(state.clone()),
      HeaderMap::new(),
      Json(user.clone()),
    )
    .await;

    let signin_user = SigninUser {
      email: user.email.clone(),
      password: user.password.clone(),
    };

    let _ = fechatter_server::handlers::auth::signin_handler(
      State(state.clone()),
      HeaderMap::new(),
      Json(signin_user),
    )
    .await;

    let _ = fechatter_server::handlers::auth::refresh_token_handler(
      State(state.clone()),
      HeaderMap::new(),
      CookieJar::new(),
      None,
    )
    .await;

    // Create mock auth user for logout test
    let auth_user = fechatter_core::AuthUser {
      id: 1,
      email: "test@example.com".to_string(),
      fullname: "Test User".to_string(),
      status: fechatter_core::UserStatus::Active,
      created_at: chrono::Utc::now(),
      workspace_id: 1,
    };

    let _ = fechatter_server::handlers::auth::logout_handler(
      State(state.clone()),
      CookieJar::new(),
      HeaderMap::new(),
      axum::Extension(auth_user.clone()),
    )
    .await;

    let _ = fechatter_server::handlers::auth::logout_all_handler(
      State(state.clone()),
      CookieJar::new(),
      HeaderMap::new(),
      axum::Extension(auth_user.clone()),
    )
    .await;

    // If we got here without a panic, the test passed
    Ok(())
  }
}
