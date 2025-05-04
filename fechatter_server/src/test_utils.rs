#[cfg(test)]
#[macro_export]
macro_rules! setup_test_users {
  ($num_users:expr) => {{
    async {
      let (tdb, state) = $crate::AppState::test_new()
        .await
        .expect("Failed to create test state");

      // Add a longer delay to ensure database is ready
      tokio::time::sleep(std::time::Duration::from_millis(200)).await;

      // Verify database connection
      sqlx::query("SELECT 1")
        .execute(&state.pool)
        .await
        .expect("Failed to verify database connection");

      let mut users = Vec::with_capacity($num_users);
      let names = vec![
        "Alice", "Bob", "Charlie", "David", "Eve", "Frank", "Grace", "Hank", "Ivy", "Judy",
        "Kevin", "Linda", "Michael", "Nancy", "Oscar", "Peggy", "Quentin", "Ruth", "Steve",
        "Tracy", "Ursula", "Victor", "Wendy", "Xavier", "Yvonne", "Zoe",
      ];

      for i in 0..($num_users as usize) {
        let fullname = names
          .get(i)
          .map(|&n| n.to_string())
          .unwrap_or_else(|| format!("User {}", i + 1));
        let email_name_part = fullname.to_lowercase().replace(' ', "");
        // Use index for uniqueness within the test run
        let email = format!("{}{}@acme.test", email_name_part, i + 1);
        let password = "password";
        let workspace = "Acme";
        let user_payload = $crate::models::CreateUser::new(&fullname, &email, &workspace, password);
        let user = state
          .create(&user_payload)
          .await
          .expect(&format!("Failed to create user {}", fullname));
        users.push(user);
      }
      (tdb, state, users)
    }
  }};
}

#[cfg(test)]
#[macro_export]
macro_rules! create_new_test_chat {
    ($state:expr, $creator:expr, $chat_type:expr, $members:expr, $name:expr $(, $desc:expr)?) => {{
        async {
            use crate::models::chat::create_new_chat;

            // Convert members Vec<&User> or Vec<User> to Vec<i64>
            let member_ids: Vec<i64> = $members.iter().map(|u| u.id).collect();
            // Handle optional description
            let description_opt: Option<&str> = None $(.or(Some($desc)))?;

            $state.create_new_chat(
                $creator.id,
                $name,
                $chat_type,
                Some(member_ids),
                description_opt,
                $creator.workspace_id
            ).await.expect(&format!("Failed to create test chat '{}'", $name))
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
      $crate::handlers::list_chats_handler(
        axum::extract::State($state.clone()),
        axum::extract::Extension($auth_user.clone())
      ),
      axum::http::StatusCode::OK,
      Vec<fechatter_core::models::chat::ChatSidebar> // Using CoreChatSidebar
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
      $crate::handlers::list_chat_members_handler(
        axum::extract::State($state.clone()),
        axum::extract::Extension($auth_user.clone()),
        axum::extract::Path($chat_id)
      ),
      axum::http::StatusCode::OK,
      Vec<fechatter_core::ChatMember> // Using core ChatMember
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
    let (_, _, users) = setup_test_users!(0).await;
    assert!(users.is_empty());
  }
}
