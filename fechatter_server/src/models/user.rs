use sqlx::{Acquire, PgPool};
use std::mem;

use argon2::{
  Argon2, PasswordHash, PasswordVerifier,
  password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};

use crate::{AppError, AppState};
use fechatter_core::{User, Workspace};

#[allow(unused)]
impl AppState {
  /// Check if a user with the given email exists in the database.
  pub async fn email_user_exists(
    &self,
    email: &str,
    pool: &PgPool,
  ) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
      "SELECT id, fullname, email, status, created_at, workspace_id FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(&self.pool)
    .await?;

    Ok(user)
  }

  pub async fn validate_users_exists_by_ids(
    &self,
    ids: &[i64],
    pool: &PgPool,
  ) -> Result<(), AppError> {
    if ids.is_empty() {
      return Ok(());
    }

    let missing_ids = sqlx::query_scalar!(
      r#"
      SELECT id FROM UNNEST($1::bigint[]) AS ids(id)
      WHERE NOT EXISTS (SELECT 1 FROM users WHERE id = ids.id)
      "#,
      ids
    )
    .fetch_all(&self.pool)
    .await?;

    if !missing_ids.is_empty() {
      let missing_ids_str = missing_ids
        .iter()
        .map(|id| id.unwrap().to_string())
        .collect::<Vec<String>>();
      return Err(AppError::NotFound(missing_ids_str));
    }

    Ok(())
  }

  /// Create a new user in the database.
  pub async fn create(&self, input: &fechatter_core::CreateUser) -> Result<User, AppError> {
    let mut tx = self.pool.begin().await?;

    let conn = tx.acquire().await?;

    let mut is_new_workspace = false;
    let workspace = match find_workspace_by_name(&input.workspace, &self.pool).await? {
      Some(workspace) => {
        if workspace.owner_id == 0 {
          is_new_workspace = true;
        }
        workspace
      }
      None => {
        is_new_workspace = true;
        sqlx::query_as::<_, Workspace>(
          r#"
          INSERT INTO workspaces (name, owner_id)
          VALUES ($1, 0)
          RETURNING id, name, owner_id, created_at
          "#,
        )
        .bind(&input.workspace)
        .fetch_one(&mut *conn)
        .await?
      }
    };

    let password_hash = hashed_password(&input.password)?;

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
          AppError::UserAlreadyExists(input.email.clone())
        } else {
          e.into()
        }
      } else {
        e.into()
      }
    })?;

    if is_new_workspace {
      let res = sqlx::query("UPDATE workspaces SET owner_id = $1 WHERE id = $2")
        .bind(user.id)
        .bind(workspace.id)
        .execute(&mut *conn)
        .await?;

      if res.rows_affected() == 0 {
        return Err(AppError::NotFound(vec![input.workspace.clone()]));
      }
    }

    tx.commit().await?;

    Ok(user)
  }

  /// Authenticate a user with email and password.
  /// Returns the user if authentication is successful.
  pub async fn authenticate(
    &self,
    input: &fechatter_core::SigninUser,
  ) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
      "SELECT id, fullname, email, password_hash, status, created_at, workspace_id FROM users WHERE email = $1",
    )
    .bind(&input.email)
    .fetch_optional(&self.pool)
    .await?;

    match user {
      Some(mut user) => {
        let password_hash = match mem::take(&mut user.password_hash) {
          Some(h) => h,
          None => return Ok(None), // User has no password hash, so it's not authenticated
        };

        let is_valid = verify_password(&input.password, &password_hash)?;
        if is_valid {
          Ok(Some(user))
        } else {
          Ok(None) // Password is invalid, so it's not authenticated
        }
      }
      None => Ok(None), // User not found, so it's not authenticated
    }
  }

  /// Find a user by their ID
  pub async fn find_user_by_id(&self, id: i64) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
      "SELECT id, fullname, email, password_hash, status, created_at, workspace_id FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(user)
  }
}

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

#[allow(unused)]
fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
  let argon2 = Argon2::default();
  let parsed_hash = PasswordHash::new(password_hash)?;

  let is_valid = argon2
    .verify_password(password.as_bytes(), &parsed_hash)
    .is_ok();

  Ok(is_valid)
}

impl ChatUser {
  #[allow(dead_code)]
  pub async fn get_users_in_workspace(&self, pool: &PgPool) -> Result<Vec<User>, AppError> {
    let workspace_id = sqlx::query_scalar::<_, i64>("SELECT workspace_id FROM users LIMIT 1")
      .fetch_optional(pool)
      .await?
      .unwrap_or(0);

    let users = sqlx::query_as::<_, User>(
      "SELECT id, fullname, email, status, created_at, workspace_id FROM users WHERE workspace_id = $1"
    )
    .bind(workspace_id)
    .fetch_all(pool)
    .await?;

    Ok(users)
  }
}

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
    let _pool = &state.pool;

    let input = CreateUser::new("Alice", "alice1@acme.test", "Acme", "hunter4332");
    let user = state.create(&input).await?;

    assert_eq!(user.email, "alice1@acme.test");
    assert_eq!(user.fullname, "Alice");
    assert!(user.id > 0);

    let user_check = state.email_user_exists(&input.email, pool).await?;
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
    let user1 = users.into_iter().next().unwrap();

    let duplicate_input = CreateUser::new("Another Alice", &user1.email, "acme", "hunter4332");
    let result = state.create(&duplicate_input).await;
    match result {
      Err(e) => {
        let app_error = AppError::from(e);
        if let AppError::UserAlreadyExists(email_msg) = app_error {
          assert!(
            email_msg.contains(&user1.email),
            "Error message '{}' should contain email '{}'",
            email_msg,
            user1.email
          );
        } else {
          panic!(
            "Expected AppError::UserAlreadyExists error, got {:?}",
            app_error
          );
        }
      }
      _ => panic!("Expected error for duplicate user"),
    }

    Ok(())
  }
}
