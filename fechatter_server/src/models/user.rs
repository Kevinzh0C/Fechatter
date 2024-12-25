use std::mem;

use serde::{Deserialize, Serialize};
use sqlx::{Acquire, PgPool};

use argon2::{
  Argon2, PasswordHash, PasswordVerifier,
  password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};

use crate::{AppError, User};

use super::{ChatUser, UserStatus, Workspace};

impl User {
  /// Check if a user with the given email exists in the database.
  pub async fn email_user_exists(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
    let user = sqlx::query_as::<_, User>(
      "SELECT id, fullname, email, status, created_at, workspace_id FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(user)
  }

  pub async fn validate_users_exists_by_ids(ids: &[i64], pool: &PgPool) -> Result<(), AppError> {
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
    .fetch_all(pool)
    .await?;

    if !missing_ids.is_empty() {
      let missing_ids_str = missing_ids
        .iter()
        .map(|id| id.unwrap().to_string())
        .collect::<Vec<String>>();
      return Err(AppError::UsersNotExist(missing_ids_str));
    }

    Ok(())
  }

  /// Create a new user in the database.
  pub async fn create(input: &CreateUser, pool: &PgPool) -> Result<Self, AppError> {
    let mut tx = pool.begin().await?;

    let conn = tx.acquire().await?;

    let mut is_new_workspace = false;
    let workspace = match Workspace::find_by_name(&input.workspace, &mut *conn).await? {
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
        return Err(AppError::WorkspaceNotFound(input.workspace.clone()));
      }
    }

    tx.commit().await?;

    Ok(user)
  }

  /// Authenticate a user with email and password.
  /// Returns the user if authentication is successful.
  pub async fn authenticate(input: &SigninUser, pool: &PgPool) -> Result<Option<Self>, AppError> {
    let user = sqlx::query_as::<_, User>(
      "SELECT id, fullname, email, password_hash, status, created_at, workspace_id FROM users WHERE email = $1",
    )
    .bind(&input.email)
    .fetch_optional(pool)
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
  pub async fn fetch_all(user: &User, pool: &PgPool) -> Result<Vec<Self>, AppError> {
    let users = Workspace::fetch_all_users(user.workspace_id, pool).await?;

    Ok(users)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
  pub fullname: String,
  pub email: String,
  pub workspace: String,
  pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
  pub id: i64,
  pub fullname: String,
  pub email: String,
  pub status: UserStatus,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub workspace_id: i64,
}

#[cfg(test)]
impl CreateUser {
  pub fn new(fullname: &str, email: &str, workspace: &str, password: &str) -> Self {
    Self {
      fullname: fullname.to_string(),
      email: email.to_string(),
      workspace: workspace.to_string(),
      password: password.to_string(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigninUser {
  pub email: String,
  pub password: String,
}

#[cfg(test)]
impl SigninUser {
  pub fn new(email: &str, password: &str) -> Self {
    Self {
      email: email.to_string(),
      password: password.to_string(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::setup_test_users;
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

  #[tokio::test]
  async fn create_user_should_work() -> Result<()> {
    let (_tdb, state, _users) = setup_test_users!(0).await;
    let pool = &state.pool;

    let input = CreateUser::new("Alice", "alice1@acme.test", "Acme", "hunter4332");
    let user = User::create(&input, pool).await?;

    assert_eq!(user.email, "alice1@acme.test");
    assert_eq!(user.fullname, "Alice");
    assert!(user.id > 0);

    let user_check = User::email_user_exists(&input.email, pool).await?;
    assert!(user_check.is_some());
    let user_check_unwrapped = user_check.unwrap();
    assert_eq!(user_check_unwrapped.email, input.email);
    assert_eq!(user_check_unwrapped.fullname, input.fullname);

    let signin_user = SigninUser::new(&input.email, &input.password);

    let auth_result = User::authenticate(&signin_user, pool).await?;
    assert!(auth_result.is_some());
    let auth_user_unwrapped = auth_result.unwrap();
    assert_eq!(auth_user_unwrapped.email, input.email);
    assert_eq!(auth_user_unwrapped.fullname, input.fullname);

    Ok(())
  }

  #[tokio::test]
  async fn create_duplicate_user_should_fail() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1).await;
    let pool = &state.pool;
    let user1 = users.into_iter().next().unwrap();

    let duplicate_input = CreateUser::new("Another Alice", &user1.email, "acme", "hunter4332");
    let result = User::create(&duplicate_input, pool).await;
    match result {
      Err(AppError::UserAlreadyExists(email)) => {
        assert_eq!(email, user1.email);
      }
      _ => panic!("Expected UserAlreadyExists error"),
    }

    Ok(())
  }
}
