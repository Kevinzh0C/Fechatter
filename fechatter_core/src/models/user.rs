use std::mem;

use serde::{Deserialize, Serialize};
use sqlx::{Acquire, PgPool};

use argon2::{
  Argon2, PasswordHash, PasswordVerifier,
  password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};

use crate::{
  error::CoreError,
  models::{ChatUser, DatabaseModel, User, UserStatus, Workspace},
  state::WithDbPool,
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

impl DatabaseModel for User {
  type CreateType = CreateUser;
  type UpdateType = (); // No update method currently defined
  type IdType = i64;

  async fn create<S: WithDbPool>(input: &Self::CreateType, state: &S) -> Result<Self, CoreError> {
    let pool = state.db_pool();
    let mut tx = pool.begin().await.map_err(CoreError::Database)?;
    let mut conn = tx.acquire().await.map_err(CoreError::Database)?;

    let mut is_new_workspace = false;
    let workspace = match sqlx::query_as::<_, Workspace>(
      r#"
      SELECT * FROM workspaces WHERE name = $1
      "#,
    )
    .bind(&input.workspace)
    .fetch_optional(&mut *conn)
    .await
    .map_err(CoreError::Database)?
    {
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
        .await
        .map_err(CoreError::Database)?
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
          CoreError::Conflict(format!("User with email {} already exists", input.email))
        } else {
          CoreError::Database(e)
        }
      } else {
        CoreError::Database(e)
      }
    })?;

    if is_new_workspace {
      let res = sqlx::query("UPDATE workspaces SET owner_id = $1 WHERE id = $2")
        .bind(user.id)
        .bind(workspace.id)
        .execute(&mut *conn)
        .await
        .map_err(CoreError::Database)?;

      if res.rows_affected() == 0 {
        return Err(CoreError::NotFound(format!(
          "Workspace {} not found",
          input.workspace
        )));
      }
    }

    tx.commit().await.map_err(CoreError::Database)?;

    Ok(user)
  }

  async fn find_by_id<S: WithDbPool + Sync>(
    id: Self::IdType,
    state: &S,
  ) -> Result<Option<Self>, CoreError> {
    let user = sqlx::query_as::<_, User>(
      "SELECT id, fullname, email, password_hash, status, created_at, workspace_id FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(CoreError::Database)?;

    Ok(user)
  }

  async fn update<S: WithDbPool + Sync>(
    _id: Self::IdType,
    _input: &Self::UpdateType,
    _state: &S,
  ) -> Result<Self, CoreError> {
    Err(CoreError::Internal(anyhow::anyhow!(
      "User update not implemented"
    )))
  }
}

impl User {
  /// Check if a user with the given email exists in the database.
  pub async fn email_user_exists<S: WithDbPool + Sync>(
    email: &str,
    state: &S,
  ) -> Result<Option<Self>, CoreError> {
    let user = sqlx::query_as::<_, User>(
      "SELECT id, fullname, email, status, created_at, workspace_id FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(state.db_pool())
    .await
    .map_err(CoreError::Database)?;

    Ok(user)
  }

  pub async fn validate_users_exists_by_ids<S: WithDbPool + Sync>(
    ids: &[i64],
    state: &S,
  ) -> Result<(), CoreError> {
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
    .fetch_all(state.db_pool())
    .await
    .map_err(CoreError::Database)?;

    if !missing_ids.is_empty() {
      let missing_ids_str = missing_ids
        .iter()
        .map(|id| id.unwrap().to_string())
        .collect::<Vec<String>>()
        .join(", ");
      return Err(CoreError::NotFound(format!(
        "Users not found: {}",
        missing_ids_str
      )));
    }

    Ok(())
  }

  /// Authenticate a user with email and password.
  /// Returns the user if authentication is successful.
  pub async fn authenticate<S: WithDbPool + Sync>(
    input: &SigninUser,
    state: &S,
  ) -> Result<Option<Self>, CoreError> {
    let user = sqlx::query_as::<_, User>(
      "SELECT id, fullname, email, password_hash, status, created_at, workspace_id FROM users WHERE email = $1",
    )
    .bind(&input.email)
    .fetch_optional(state.db_pool())
    .await
    .map_err(CoreError::Database)?;

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

fn hashed_password(password: &str) -> Result<String, CoreError> {
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

fn verify_password(password: &str, password_hash: &str) -> Result<bool, CoreError> {
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
    let (_tdb, state, _users) = setup_test_users!(0);
    let pool = &state.pool;

    let input = CreateUser::new("Alice", "alice1@acme.test", "Acme", "hunter4332");
    let user = User::create(&input, &state).await?;

    assert_eq!(user.email, "alice1@acme.test");
    assert_eq!(user.fullname, "Alice");
    assert!(user.id > 0);

    let user_check = User::email_user_exists(&input.email, &state).await?;
    assert!(user_check.is_some());
    let user_check_unwrapped = user_check.unwrap();
    assert_eq!(user_check_unwrapped.email, input.email);
    assert_eq!(user_check_unwrapped.fullname, input.fullname);

    let signin_user = SigninUser::new(&input.email, &input.password);

    let auth_result = User::authenticate(&signin_user, &state).await?;
    assert!(auth_result.is_some());
    let auth_user_unwrapped = auth_result.unwrap();
    assert_eq!(auth_user_unwrapped.email, input.email);
    assert_eq!(auth_user_unwrapped.fullname, input.fullname);

    Ok(())
  }

  #[tokio::test]
  async fn create_duplicate_user_should_fail() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(1);
    let pool = &state.pool;
    let user1 = users.into_iter().next().unwrap();

    let duplicate_input = CreateUser::new("Another Alice", &user1.email, "acme", "hunter4332");
    let result = User::create(&duplicate_input, pool).await;
    match result {
      Err(CoreError::Conflict(email)) => {
        assert_eq!(email, user1.email);
      }
      _ => panic!("Expected UserAlreadyExists error"),
    }

    Ok(())
  }
}
