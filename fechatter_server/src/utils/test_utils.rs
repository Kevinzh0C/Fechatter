#[macro_export]
macro_rules! setup_test_users {
  ($num_users:expr) => {{
    async {
      let config = $crate::AppConfig::load().expect("Failed to load config");
      let (tdb, state) = $crate::AppState::test_new(config)
        .await
        .expect("Failed to create test state");

      tokio::time::sleep(std::time::Duration::from_millis(50)).await;
      let mut users = Vec::with_capacity($num_users);
      let names = [
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
        let user_payload = $crate::models::CreateUser::new(&fullname, &email, password);
        let user = $crate::models::User::create(&user_payload, &state.pool)
          .await
          .expect(&format!("Failed to create user {}", fullname));
        users.push(user);
      }
      (tdb, state, users)
    }
  }};
}

#[macro_export]
macro_rules! create_new_test_chat {
    ($state:expr, $creator:expr, $chat_type:expr, $members:expr, $name:expr $(, $desc:expr)?) => {{
        async {
            // Convert members Vec<&User> or Vec<User> to Vec<i64>
            let member_ids: Vec<i64> = $members.iter().map(|u| u.id).collect();
            // Handle optional description
            let description_opt: Option<&str> = None $(.or(Some($desc)))?;

            $crate::models::create_new_chat(
                &$state,
                $creator.id,
                $name,
                $chat_type,
                Some(member_ids),
                description_opt
            ).await.expect(&format!("Failed to create test chat '{}'", $name))
        }
    }};
}

// Macro to assert handler success and deserialize response
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

// Macro to assert handler failure with a specific AppError variant
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

// Macro to assert the number of chats listed for a user
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
      Vec<$crate::models::ChatSidebar> // Expecting Vec<ChatSidebar>
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

// Macro to assert the number of members in a specific chat
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
      Vec<$crate::models::ChatMember> // Expecting Vec<ChatMember>
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
pub fn add(left: usize, right: usize) -> usize {
  left + right
}

#[cfg(test)]
mod tests {

  use super::add;

  #[test]
  fn it_works() {
    let result = add(2, 2);
    assert_eq!(result, 4);
  }
}
