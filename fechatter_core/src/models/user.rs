use serde::{Deserialize, Serialize};

use argon2::{
  Argon2, PasswordHash, PasswordVerifier,
  password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};

use crate::{
  error::CoreError,
  models::{User, UserStatus},
};

use super::{CreateUser, SigninUser};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
  pub id: i64,
  pub fullname: String,
  pub email: String,
  pub status: UserStatus,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub workspace_id: i64,
}

pub trait UserRepository: Send + Sync {
  fn create(
    &self,
    input: &CreateUser,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<User, CoreError>> + Send>>;
  fn find_by_id(
    &self,
    id: i64,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<User>, CoreError>> + Send>>;
  fn email_user_exists(
    &self,
    email: &str,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<User>, CoreError>> + Send>>;
  fn validate_users_exists_by_ids(
    &self,
    ids: &[i64],
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), CoreError>> + Send>>;
  fn authenticate(
    &self,
    input: &SigninUser,
  ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<User>, CoreError>> + Send>>;
}

/// Utility functions for User model that don't require database access
impl User {
  /// Convert User to AuthUser for authentication purposes
  pub fn into_auth_user(self) -> AuthUser {
    AuthUser {
      id: self.id,
      fullname: self.fullname,
      email: self.email,
      status: self.status,
      created_at: self.created_at,
      workspace_id: self.workspace_id,
    }
  }
}

/// Authentication helper functions (pure algorithm parts)
pub fn hashed_password(password: &str) -> Result<String, CoreError> {
  let salt = SaltString::generate(OsRng);

  // Argon2 with default params (Argon2id v19)
  let argon2 = Argon2::default();

  // Hash password to PHC string ($argon2id$v=19$...)
  let password_hash = argon2
    .hash_password(password.as_bytes(), &salt)
    .map_err(|e| CoreError::Internal(e.into()))?
    .to_string();

  Ok(password_hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, CoreError> {
  let argon2 = Argon2::default();
  let parsed_hash = PasswordHash::new(password_hash).map_err(|e| CoreError::Internal(e.into()))?;

  let is_valid = argon2
    .verify_password(password.as_bytes(), &parsed_hash)
    .is_ok();

  Ok(is_valid)
}

#[cfg(test)]
mod tests {
  use super::*;
  use anyhow::Result;

  #[test]
  fn hashed_password_should_work() -> Result<()> {
    let password = "hunter4332";
    let password_hash = hashed_password(password)?;
    assert_eq!(password_hash.len(), 97);
    let is_valid = verify_password(password, &password_hash)?;
    assert!(is_valid);
    Ok(())
  }
}
