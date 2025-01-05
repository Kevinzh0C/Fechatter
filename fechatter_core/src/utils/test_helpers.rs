use crate::models::{AuthUser, User, UserStatus};
use chrono::Utc;

#[cfg(test)]
use crate::utils::MockTokenManagerTrait;

/// Create a test user with default values
pub fn create_test_user(id: i64, workspace_id: i64) -> User {
  User {
    id,
    fullname: format!("Test User {}", id),
    email: format!("user{}@test.com", id),
    password_hash: Some(
      "$argon2id$v=19$m=16,t=2,p=1$c29tZXNhbHQ$oixeQKqM1dOx4bKZUkJc1A".to_string(),
    ),
    status: UserStatus::Active,
    created_at: Utc::now(),
    workspace_id,
  }
}

/// Create an auth user from a user
pub fn create_test_auth_user(user: &User) -> AuthUser {
  AuthUser {
    id: user.id,
    fullname: user.fullname.clone(),
    email: user.email.clone(),
    status: user.status,
    created_at: user.created_at,
    workspace_id: user.workspace_id,
  }
}

#[cfg(test)]
mod tests {
  use crate::{CoreError, TokenManagerTrait as _, UserClaims};

  use super::*;

  pub fn create_test_token_manager() -> MockTokenManagerTrait {
    let mut mock = MockTokenManagerTrait::new();

    // Configure the mock to handle common test scenarios
    mock
      .expect_generate_token_from_user()
      .returning(|user| Ok(format!("test_token_for_user_{}", user.id)));

    mock
      .expect_generate_token_from_auth_user()
      .returning(|user| Ok(format!("test_token_for_auth_user_{}", user.id)));

    mock.expect_verify_token().returning(|token| {
      // Parse user ID from the token format we're using for testing
      if token.starts_with("test_token_for_user_") || token.starts_with("test_token_for_auth_user_")
      {
        let parts: Vec<&str> = token.rsplitn(2, '_').collect();
        if parts.len() == 2 {
          if let Ok(id) = parts[0].parse::<i64>() {
            return Ok(UserClaims {
              id,
              workspace_id: 1,
              fullname: format!("Test User {}", id),
              email: format!("user{}@test.com", id),
              status: UserStatus::Active,
              created_at: Utc::now(),
            });
          }
        }
      }

      Err(CoreError::Unauthorized("Invalid token".to_string()))
    });

    mock
  }

  #[test]
  fn test_create_test_user() {
    let user = create_test_user(1, 10);
    assert_eq!(user.id, 1);
    assert_eq!(user.workspace_id, 10);
    assert_eq!(user.fullname, "Test User 1");
    assert_eq!(user.email, "user1@test.com");
    assert_eq!(user.status, UserStatus::Active);
  }

  #[test]
  fn test_create_test_auth_user() {
    let user = create_test_user(2, 20);
    let auth_user = create_test_auth_user(&user);

    assert_eq!(auth_user.id, user.id);
    assert_eq!(auth_user.fullname, user.fullname);
    assert_eq!(auth_user.email, user.email);
    assert_eq!(auth_user.status, user.status);
    assert_eq!(auth_user.created_at, user.created_at);
    assert_eq!(auth_user.workspace_id, user.workspace_id);
  }

  #[test]
  fn test_create_token_manager() {
    let mock = create_test_token_manager();
    let user = create_test_user(1, 1);

    // Test token generation
    let token = mock.generate_token_from_user(&user).unwrap();
    assert_eq!(token, "test_token_for_user_1");

    // Test token verification
    let claims = mock.verify_token(&token).unwrap();
    assert_eq!(claims.id, 1);
  }
}
