use async_trait::async_trait;
use sqlx::{Acquire, PgPool};
use std::{mem, sync::Arc};

use argon2::{
  Argon2, PasswordHash, PasswordVerifier,
  password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};

use crate::{AppError, AppState};
use fechatter_core::{
  AuthUser, CreateUser, SigninUser, User, UserClaims, UserRepository, Workspace, error::CoreError,
};

// 用于连接AppState与UserRepository的适配器
pub struct FechatterUserRepository {
  pool: Arc<PgPool>,
}

impl FechatterUserRepository {
  pub fn new(pool: Arc<PgPool>) -> Self {
    Self { pool }
  }

  #[allow(unused)]
  fn pool(&self) -> &PgPool {
    &self.pool
  }

  /// Find workspace by name, or create a default one if it doesn't exist
  async fn find_workspace_by_name(
    &self,
    name: &str,
  ) -> Result<fechatter_core::Workspace, CoreError> {
    let mut conn = self.pool.acquire().await?;

    match sqlx::query_as::<_, fechatter_core::Workspace>(
      "SELECT id, name, owner_id, created_at FROM workspaces WHERE name = $1",
    )
    .bind(name)
    .fetch_optional(&mut *conn)
    .await?
    {
      Some(workspace) => Ok(workspace),
      None => {
        // Create a new workspace with owner_id 0 (will be updated later)
        sqlx::query_as::<_, fechatter_core::Workspace>(
          r#"
          INSERT INTO workspaces (name, owner_id)
          VALUES ($1, 0)
          RETURNING id, name, owner_id, created_at
          "#,
        )
        .bind(name)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| CoreError::Internal(e.to_string()))
      }
    }
  }
}

#[async_trait]
impl UserRepository for FechatterUserRepository {
  async fn find_by_id(&self, id: i64) -> Result<Option<User>, CoreError> {
    let user = sqlx::query_as::<_, User>(
      "SELECT id, fullname, email, password_hash, status, created_at, workspace_id FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&*self.pool)
    .await
    .map_err(|e| CoreError::Internal(e.to_string()))?;

    Ok(user)
  }

  async fn create(&self, input: &CreateUser) -> Result<User, CoreError> {
    // Check if email already exists
    let existing_user = self.email_user_exists(&input.email).await?;
    if existing_user.is_some() {
      return Err(CoreError::UserAlreadyExists(format!(
        "User with email {} already exists",
        input.email
      )));
    }

    // Check if workspace exists (or create default)
    let workspace = self.find_workspace_by_name(&input.workspace).await?;

    let mut tx = self
      .pool
      .begin()
      .await
      .map_err(|e| CoreError::Internal(e.to_string()))?;

    let conn = tx
      .acquire()
      .await
      .map_err(|e| CoreError::Internal(e.to_string()))?;

    let mut is_new_workspace = false;
    if workspace.owner_id == 0 {
      is_new_workspace = true;
    }

    let password_hash =
      hashed_password(&input.password).map_err(|e| CoreError::Internal(e.to_string()))?;

    let user = sqlx::query_as::<_, User>(
      r#"
      INSERT INTO users (workspace_id, email, fullname, password_hash)
      VALUES ($1, $2, $3, $4)
      RETURNING id, fullname, email, status, created_at, workspace_id, password_hash
      "#,
    )
    .bind(workspace.id)
    .bind(&input.email)
    .bind(&input.fullname)
    .bind(password_hash)
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| {
      if let Some(db_err) = e.as_database_error() {
        if db_err.is_unique_violation() {
          CoreError::Validation(format!("User with email {} already exists", input.email))
        } else {
          CoreError::Internal(e.to_string())
        }
      } else {
        CoreError::Internal(e.to_string())
      }
    })?;

    if is_new_workspace {
      let res = sqlx::query("UPDATE workspaces SET owner_id = $1 WHERE id = $2")
        .bind(user.id)
        .bind(workspace.id)
        .execute(&mut *conn)
        .await
        .map_err(|e| CoreError::Internal(e.to_string()))?;

      if res.rows_affected() == 0 {
        return Err(CoreError::NotFound(format!(
          "Workspace {} not found",
          input.workspace
        )));
      }
    }

    tx.commit()
      .await
      .map_err(|e| CoreError::Internal(e.to_string()))?;

    Ok(user)
  }

  async fn authenticate(&self, credentials: &SigninUser) -> Result<Option<User>, CoreError> {
    let user = sqlx::query_as::<_, User>(
      "SELECT id, fullname, email, password_hash, status, created_at, workspace_id FROM users WHERE email = $1",
    )
    .bind(&credentials.email)
    .fetch_optional(&*self.pool)
    .await
    .map_err(|e| CoreError::Internal(e.to_string()))?;

    match user {
      Some(mut user) => {
        let password_hash = match mem::take(&mut user.password_hash) {
          Some(h) => h,
          None => return Ok(None), // User has no password hash, so it's not authenticated
        };

        let is_valid = verify_password(&credentials.password, &password_hash)
          .map_err(|e| CoreError::Internal(e.to_string()))?;
        if is_valid {
          Ok(Some(user))
        } else {
          Ok(None) // Password is invalid, so it's not authenticated
        }
      }
      None => Ok(None), // User not found, so it's not authenticated
    }
  }

  async fn email_user_exists(&self, email: &str) -> Result<Option<User>, CoreError> {
    let user = sqlx::query_as::<_, User>(
      "SELECT id, fullname, email, password_hash, status, created_at, workspace_id FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(&*self.pool)
    .await
    .map_err(|e| CoreError::Internal(e.to_string()))?;

    Ok(user)
  }

  async fn validate_users_exists_by_ids(&self, ids: &[i64]) -> Result<(), CoreError> {
    if ids.is_empty() {
      return Ok(());
    }

    let query = r#"
      SELECT id FROM UNNEST($1::bigint[]) AS ids(id)
      WHERE NOT EXISTS (SELECT 1 FROM users WHERE id = ids.id)
    "#;

    let missing_ids = sqlx::query_scalar(query)
      .bind(ids)
      .fetch_all(&*self.pool)
      .await
      .map_err(|e| CoreError::Internal(e.to_string()))?;

    if !missing_ids.is_empty() {
      let missing_ids_str = missing_ids
        .iter()
        .map(|id: &Option<i64>| id.unwrap().to_string())
        .collect::<Vec<String>>();
      return Err(CoreError::NotFound(format!(
        "Users not found: {}",
        missing_ids_str.join(", ")
      )));
    }

    Ok(())
  }
}

// Implementation for AppState to handle all database operations for User model
#[allow(unused)]
impl AppState {
  /// Create a new user account with workspace assignment
  pub async fn create_user(
    &self,
    input: &CreateUser,
    auth_context: Option<fechatter_core::services::AuthContext>,
  ) -> Result<User, AppError> {
    // We don't use auth_context here but keep the parameter for API compatibility
    let pool = self.pool().clone();
    let repository = FechatterUserRepository::new(Arc::new(pool));
    let user = repository.create(input).await?;
    Ok(user)
  }

  /// Find a user by ID
  pub async fn find_user_by_id(&self, id: i64) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
      "SELECT id, fullname, email, password_hash, status, created_at, workspace_id FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(self.pool())
    .await?;

    Ok(user)
  }

  /// Check if a user with the given email exists in the database
  pub async fn email_user_exists(&self, email: &str) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
      "SELECT id, fullname, email, password_hash, status, created_at, workspace_id FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(self.pool())
    .await?;

    Ok(user)
  }

  /// Authenticate a user with credentials
  pub async fn authenticate(&self, credentials: &SigninUser) -> Result<Option<User>, AppError> {
    let pool = self.pool().clone();
    let repository = FechatterUserRepository::new(Arc::new(pool));
    let user = repository.authenticate(credentials).await?;
    Ok(user)
  }

  /// Validate that all users in the given IDs list exist
  pub async fn validate_users_exists_by_ids(&self, ids: &[i64]) -> Result<(), AppError> {
    if ids.is_empty() {
      return Ok(());
    }

    let query = r#"
      SELECT id FROM UNNEST($1::bigint[]) AS ids(id)
      WHERE NOT EXISTS (SELECT 1 FROM users WHERE id = ids.id)
    "#;

    let missing_ids = sqlx::query_scalar(query)
      .bind(ids)
      .fetch_all(self.pool())
      .await?;

    if !missing_ids.is_empty() {
      let missing_ids_str = missing_ids
        .iter()
        .map(|id: &Option<i64>| id.unwrap().to_string())
        .collect::<Vec<String>>();
      return Err(AppError::NotFound(missing_ids_str));
    }

    Ok(())
  }

  /// Update user's profile information
  pub async fn update_user_profile(&self, user_id: i64, fullname: &str) -> Result<User, AppError> {
    let user = sqlx::query_as::<_, User>(
      r#"
      UPDATE users 
      SET fullname = $1
      WHERE id = $2
      RETURNING id, fullname, email, password_hash, status, created_at, workspace_id
      "#,
    )
    .bind(fullname)
    .bind(user_id)
    .fetch_one(self.pool())
    .await?;

    Ok(user)
  }

  /// Change user's password
  pub async fn change_password(
    &self,
    user_id: i64,
    current_password: &str,
    new_password: &str,
  ) -> Result<(), AppError> {
    // First get the user's current password hash
    let user = self
      .find_user_by_id(user_id)
      .await?
      .ok_or(AppError::NotFound(vec![user_id.to_string()]))?;

    let password_hash = user
      .password_hash
      .ok_or(AppError::InvalidInput("User has no password set".into()))?;

    // Verify current password
    let is_valid = verify_password(current_password, &password_hash)?;
    if !is_valid {
      return Err(AppError::InvalidInput(
        "Current password is incorrect".into(),
      ));
    }

    // Hash the new password
    let new_hash = hashed_password(new_password)?;

    // Update the password in the database
    let res = sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
      .bind(new_hash)
      .bind(user_id)
      .execute(self.pool())
      .await?;

    if res.rows_affected() == 0 {
      return Err(AppError::NotFound(vec![user_id.to_string()]));
    }

    Ok(())
  }

  /// Get users by workspace id
  pub async fn get_users_in_workspace(&self, workspace_id: i64) -> Result<Vec<User>, AppError> {
    let users = sqlx::query_as::<_, User>(
      "SELECT id, fullname, email, status, created_at, workspace_id, NULL as password_hash FROM users WHERE workspace_id = $1"
    )
    .bind(workspace_id)
    .fetch_all(self.pool())
    .await?;

    Ok(users)
  }

  /// Convert a UserClaims to AuthUser
  pub fn user_from_claims(&self, claims: UserClaims) -> AuthUser {
    claims.into()
  }

  /// Create a user (with testing defaults if needed)
  #[cfg(test)]
  pub async fn create(&self, input: &CreateUser) -> Result<User, AppError> {
    self.create_user(input, None).await
  }
}

// Helper functions - these can be used independently of the AppState

fn hashed_password(password: &str) -> Result<String, AppError> {
  let salt = SaltString::generate(OsRng);

  // Argon2 with default params (Argon2id v19)
  let argon2 = Argon2::default();

  // Hash password to PHC string ($argon2id$v=19$...)
  let password_hash = argon2
    .hash_password(password.as_bytes(), &salt)?
    .to_string();

  Ok(password_hash)
}

fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
  let argon2 = Argon2::default();
  let parsed_hash = PasswordHash::new(password_hash)?;

  let is_valid = argon2
    .verify_password(password.as_bytes(), &parsed_hash)
    .is_ok();

  Ok(is_valid)
}

#[allow(unused)]
async fn find_workspace_by_name(name: &str, pool: &PgPool) -> Result<Option<Workspace>, AppError> {
  let workspace = sqlx::query_as::<_, Workspace>(
    "SELECT id, name, owner_id, created_at FROM workspaces WHERE name = $1",
  )
  .bind(name)
  .fetch_optional(pool)
  .await?;

  Ok(workspace)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::error::AppError;
  use crate::setup_test_users;
  use anyhow::Result;
  use fechatter_core::{CreateUser, SigninUser};

  #[test]
  fn hashed_password_should_work() -> Result<()> {
    let password = "hunter4332";
    let password_hash = hashed_password(password)?;
    assert_eq!(password_hash.len(), 97);
    let is_valid = verify_password(password, &password_hash)?;
    assert!(is_valid);
    Ok(())
  }

  #[tokio::test]
  async fn create_user_should_work() -> Result<()> {
    let (_tdb, state, _users) = setup_test_users!(0).await;

    let input = CreateUser::new("Alice", "alice1@acme.test", "Acme", "hunter4332");
    let user = state.create_user(&input, None).await?;

    assert_eq!(user.email, "alice1@acme.test");
    assert_eq!(user.fullname, "Alice");
    assert!(user.id > 0);

    let user_check = state.email_user_exists(&input.email).await?;
    assert!(user_check.is_some());
    let user_check_unwrapped = user_check.unwrap();
    assert_eq!(user_check_unwrapped.email, input.email);
    assert_eq!(user_check_unwrapped.fullname, input.fullname);

    let signin_user = SigninUser::new(&input.email, &input.password);

    let auth_result = state.authenticate(&signin_user).await?;
    assert!(auth_result.is_some());
    let auth_user_unwrapped = auth_result.unwrap();
    assert_eq!(auth_user_unwrapped.email, input.email);
    assert_eq!(auth_user_unwrapped.fullname, input.fullname);

    Ok(())
  }

  #[tokio::test]
  async fn create_duplicate_user_should_fail() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user1 = users[0].clone();

    let duplicate_input = CreateUser::new("Another Alice", &user1.email, "acme", "hunter4332");
    let result = state.create_user(&duplicate_input, None).await;
    match result {
      Err(e) => {
        if let AppError::UserAlreadyExists(email_msg) = e {
          assert!(
            email_msg.contains(&user1.email),
            "Error message '{}' should contain email '{}'",
            email_msg,
            user1.email
          );
        } else {
          panic!("Expected AppError::UserAlreadyExists error, got {:?}", e);
        }
      }
      _ => panic!("Expected error for duplicate user"),
    }

    Ok(())
  }

  #[tokio::test]
  async fn password_change_should_work() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let user = users[0].clone();

    // Change password
    state
      .change_password(user.id, "password", "newpassword")
      .await?;

    // Try to authenticate with new password
    let signin_with_new = SigninUser::new(&user.email, "newpassword");
    let auth_result = state.authenticate(&signin_with_new).await?;
    assert!(auth_result.is_some());

    // Old password should fail
    let signin_with_old = SigninUser::new(&user.email, "password");
    let auth_result = state.authenticate(&signin_with_old).await?;
    assert!(auth_result.is_none());

    Ok(())
  }
}
