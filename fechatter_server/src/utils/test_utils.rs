#[cfg(test)]
#[macro_export]
macro_rules! setup_test_users {
  ($num_users:expr) => {{
    async {
      let config = $crate::config::AppConfig::minimal_dev_config()
        .expect("Failed to create minimal dev config");

      let mut test_config = config;
      test_config.server.db_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://postgres:password@localhost:5432/fechatter_test".to_string()
      });

      let state = $crate::AppState::new(test_config)
        .await
        .expect("Failed to create test state");

      let pool = state.infrastructure().database_pool();

      sqlx::query("SELECT 1")
        .execute(pool.as_ref())
        .await
        .expect("Failed to verify database connection");

      let mut users = Vec::with_capacity($num_users);
      let names = vec![
        "Alice", "Bob", "Charlie", "David", "Eve", "Frank", "Grace", "Hank", "Ivy", "Judy",
        "Kevin", "Linda", "Michael", "Nancy", "Oscar", "Peggy", "Quentin", "Ruth", "Steve",
        "Tracy", "Ursula", "Victor", "Wendy", "Xavier", "Yvonne", "Zoe",
      ];

      let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
      let process_id = std::process::id();
      let thread_id = std::thread::current().id();
      let thread_hash = format!("{:?}", thread_id).len() as u64;
      let unique_id = format!("{}{}{}", timestamp, process_id, thread_hash);

      for i in 0..($num_users as usize) {
        let fullname = names
          .get(i)
          .map(|&n| n.to_string())
          .unwrap_or_else(|| format!("User {}", i + 1));
        let email_name_part = fullname.to_lowercase().replace(' ', "");
        let email = format!("{}{}{}@acme.test", email_name_part, i + 1, unique_id);
        let password = "password";
        let workspace = "Acme";
        let user_payload = fechatter_core::CreateUser::new(&fullname, &email, &workspace, password);

        let auth_service = state.services().auth();
        let auth_tokens = auth_service
          .register_user(&user_payload)
          .await
          .expect(&format!("Failed to create user {}", fullname));

        let user = fechatter_core::User {
          id: auth_tokens.user_id,
          fullname: fullname.clone(),
          email: email.clone(),
          password_hash: Some("test_hash".to_string()),
          status: fechatter_core::UserStatus::Active,
          created_at: chrono::Utc::now(),
          workspace_id: auth_tokens.workspace_id,
          // Profile fields - set defaults for test users
          phone: None,
          title: None,
          department: None,
          avatar_url: None,
          bio: None,
          timezone: None,
          language: None,
          last_active_at: Some(chrono::Utc::now()),
        };
        users.push(user);
      }
      (state, users)
    }
  }};
}

#[cfg(test)]
#[macro_export]
macro_rules! create_new_test_chat {
    ($state:expr, $creator:expr, $chat_type:expr, $members:expr, $name:expr $(, $desc:expr)?) => {{
        async {
            // Convert members to Vec<fechatter_core::UserId>
            let member_ids: Vec<fechatter_core::UserId> = $members.iter().map(|u| u.id).collect();

            // Handle optional description
            let description = None $(.or(Some($desc.to_string())))?;

            // Use the new create_new_chat method signature
            let chat = $state.create_new_chat(
                $chat_type,
                Some($name.to_string()),
                description,
                $creator.id,
                member_ids,
            )
            .await
            .expect(&format!("Failed to create test chat '{}'", $name));

            chat
        }
    }};
}

#[cfg(test)]
#[macro_export]
macro_rules! auth_user {
  ($user:expr) => {{
    fechatter_core::AuthUser {
      id: $user.id,
      fullname: $user.fullname.clone(),
      email: $user.email.clone(),
      status: $user.status.clone(),
      created_at: $user.created_at,
      workspace_id: $user.workspace_id,
    }
  }};
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_handler_success {
  ($handler_call:expr, $expected_status:expr, $response_type:ty) => {{
    let response = $handler_call
      .await
      .expect("Handler call failed unexpectedly")
      .into_response();
    assert_eq!(
      response.status(),
      $expected_status,
      "Expected status code {:?} but got {:?}",
      $expected_status,
      response.status()
    );
    let body = http_body_util::BodyExt::collect(response.into_body())
      .await
      .expect("Failed to collect response body")
      .to_bytes();
    let result: $response_type = serde_json::from_slice(&body).expect(&format!(
      "Failed to deserialize response body into {}: {:?}",
      stringify!($response_type),
      String::from_utf8_lossy(&body)
    ));
    result
  }};

  ($handler_call:expr, $expected_status:expr) => {{
    let response = $handler_call
      .await
      .expect("Handler call failed unexpectedly")
      .into_response();
    assert_eq!(
      response.status(),
      $expected_status,
      "Expected status code {:?} but got {:?}",
      $expected_status,
      response.status()
    );
  }};
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_handler_error {
    ($handler_call:expr, $expected_error:pat $(if $guard:expr)?) => {{
        let result = $handler_call.await;
        assert!(result.is_err(), "Handler call expected to fail, but succeeded.");
        if let Err(err) = result {
             match err {
                $expected_error $(if $guard)? => { /* Test passed */ },
                _ => panic!("Handler failed with unexpected error type: {:?}, expected pattern: {}", err, stringify!($expected_error)),
             }
        }
    }};
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_chat_list_count {
  ($state:expr, $auth_user:expr, $expected_count:expr) => {{
    // Use assert_handler_success! internally to check status and get the list
    let chats = $crate::assert_handler_success!(
      $crate::list_chats_handler(
        axum::extract::State($state.clone()),
        axum::extract::Extension($auth_user.clone())
      ),
      axum::http::StatusCode::OK,
      Vec<$crate::domains::dtos::ChatSidebar>
    );
    assert_eq!(
      chats.len(),
      $expected_count,
      "Expected {} chats for user {}, but found {}",
      $expected_count,
      $auth_user.id,
      chats.len()
    );
  }};
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_chat_member_count {
  ($state:expr, $auth_user:expr, $chat_id:expr, $expected_count:expr) => {{
    let members = $crate::assert_handler_success!(
      $crate::list_chat_members_handler(
        axum::extract::State($state.clone()),
        axum::extract::Extension($auth_user.clone()),
        axum::extract::Path($chat_id)
      ),
      axum::http::StatusCode::OK,
      Vec<$crate::handlers::chat_members::ChatMemberDto>
    );
    assert_eq!(
      members.len(),
      $expected_count,
      "Expected {} members in chat {}, but found {}",
      $expected_count,
      $chat_id,
      members.len()
    );
  }};
}

#[cfg(test)]
pub mod cookie_helpers {
  use axum::http::HeaderMap;
  use axum_extra::extract::cookie::{Cookie, CookieJar};

  #[allow(dead_code)]
  pub fn extract_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers.get_all("set-cookie").iter().find_map(|v| {
      let cookie_str = v.to_str().ok()?;
      if cookie_str.starts_with(&format!("{}=", name)) {
        Some(cookie_str.to_string())
      } else {
        None
      }
    })
  }

  #[allow(dead_code)]
  pub fn create_cookie_jar(cookie_value: &str) -> CookieJar {
    let mut jar = CookieJar::new();
    if let Ok(cookie) = Cookie::parse(cookie_value.to_string()) {
      jar = jar.add(cookie);
    }
    jar
  }
}

#[cfg(test)]
mod tests {

  #[tokio::test]
  async fn zero_users_ok() {
    let (_, users) = setup_test_users!(0).await;
    assert!(users.is_empty());
  }
}

use fechatter_core::models::jwt::TokenConfigProvider;
use once_cell::sync::Lazy;

// Generate test JWT signing keys for tests - using simple strings for tests
static TEST_JWT_KEYS: Lazy<(String, String)> = Lazy::new(|| {
  // For tests, we use a simple pair of EdDSA-like strings as keys
  // Using the same key for both encoding and decoding to avoid key mismatch in tests
  let key = "TEST_CONSISTENT_KEY_FOR_BOTH_SIGNING_AND_VERIFICATION".to_string();

  (key.clone(), key.clone())
});

/// A test-specific TokenConfigProvider that uses consistent in-memory test keys
pub struct TestTokenConfig;

impl TokenConfigProvider for TestTokenConfig {
  fn get_encoding_key_pem(&self) -> &str {
    &TEST_JWT_KEYS.0
  }

  fn get_decoding_key_pem(&self) -> &str {
    &TEST_JWT_KEYS.1
  }

  // Smaller leeway for tests
  fn get_jwt_leeway(&self) -> u64 {
    5 // 5 seconds for tests
  }
}
