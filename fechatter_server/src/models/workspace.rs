use sqlx::{Executor, PgPool, Postgres, query_as};

use crate::AppError;

use super::{ChatUser, Workspace};

#[allow(dead_code)]
impl Workspace {
  pub async fn create<'e, E>(name: &str, user_id: i64, executor: E) -> Result<Self, AppError>
  where
    E: Executor<'e, Database = Postgres> + Copy,
  {
    if name.trim().is_empty() {
      return Err(AppError::InvalidInput(
        "Workspace name cannot be empty".into(),
      ));
    }

    let workspace = sqlx::query_as::<_, Workspace>(
      r#"
      INSERT INTO workspaces (name, owner_id)
      VALUES ($1, $2)
      RETURNING id, name, owner_id, created_at
      "#,
    )
    .bind(name)
    .bind(user_id)
    .fetch_one(executor)
    .await
    .map_err(|e| {
      if let Some(db_err) = e.as_database_error() {
        if db_err.is_unique_violation() {
          AppError::WorkspaceAlreadyExists(name.to_string())
        } else {
          e.into()
        }
      } else {
        e.into()
      }
    })?;

    if user_id != 0 {
      let rows = sqlx::query("UPDATE users SET workspace_id = $1 WHERE id = $2")
        .bind(workspace.id)
        .bind(user_id)
        .execute(executor)
        .await?
        .rows_affected();

      if rows == 0 {
        return Err(AppError::NotFound(vec![user_id.to_string()]));
      }
    }

    Ok(workspace)
  }

  pub async fn update_owner<'e, E>(&self, owner_id: i64, executor: E) -> Result<Self, AppError>
  where
    E: Executor<'e, Database = Postgres>,
  {
    let workspace = sqlx::query_as::<_, Workspace>(
      r#"
      UPDATE workspaces
      SET owner_id = $1
      WHERE id = $2 AND EXISTS (SELECT 1 FROM users u WHERE u.id = $1 AND u.workspace_id = $2)
      RETURNING id, name, owner_id, created_at
      "#,
    )
    .bind(owner_id)
    .bind(self.id)
    .fetch_one(executor)
    .await?;

    Ok(workspace)
  }

  pub async fn fetch_all_users<'e, E>(id: i64, executor: E) -> Result<Vec<ChatUser>, AppError>
  where
    E: Executor<'e, Database = Postgres>,
  {
    let users = query_as::<_, ChatUser>(
      r#"
      SELECT u.id, u.fullname, u.email 
      FROM users u
      WHERE u.workspace_id = $1
      "#,
    )
    .bind(id)
    .fetch_all(executor)
    .await?;

    Ok(users)
  }

  pub async fn find_by_name<'e, E>(name: &str, executor: E) -> Result<Option<Self>, AppError>
  where
    E: Executor<'e, Database = Postgres>,
  {
    let workspace = sqlx::query_as::<_, Workspace>(
      r#"
      SELECT * FROM workspaces WHERE name = $1
      "#,
    )
    .bind(name)
    .fetch_optional(executor)
    .await?;

    Ok(workspace)
  }

  pub async fn find_by_id<'e, E>(id: i64, executor: E) -> Result<Option<Self>, AppError>
  where
    E: Executor<'e, Database = Postgres>,
  {
    let workspace = sqlx::query_as::<_, Workspace>(
      r#"
      SELECT * FROM workspaces WHERE id = $1
      "#,
    )
    .bind(id)
    .fetch_optional(executor)
    .await?;

    Ok(workspace)
  }

  pub async fn add_to_workspace<'e, E>(&self, user_id: i64, executor: E) -> Result<Self, AppError>
  where
    E: Executor<'e, Database = Postgres> + Copy,
  {
    let workspace = sqlx::query_as::<_, Workspace>(
      r#"
      UPDATE workspaces 
      SET owner_id = CASE 
          WHEN owner_id = 0 THEN $1
          ELSE owner_id              
      END
      WHERE id = $2
      RETURNING id, name, owner_id, created_at
      "#,
    )
    .bind(user_id)
    .bind(self.id)
    .fetch_one(executor)
    .await?;

    let rows = sqlx::query("UPDATE users SET workspace_id = $1 WHERE id = $2")
      .bind(self.id)
      .bind(user_id)
      .execute(executor)
      .await?
      .rows_affected();

    if rows == 0 {
      return Err(AppError::NotFound(vec![user_id.to_string()]));
    }

    Ok(workspace)
  }
}

#[allow(dead_code)]
impl Workspace {
  pub async fn create_with_pool(name: &str, user_id: i64, pool: &PgPool) -> Result<Self, AppError> {
    Self::create(name, user_id, pool).await
  }

  pub async fn update_owner_with_pool(
    &self,
    owner_id: i64,
    pool: &PgPool,
  ) -> Result<Self, AppError> {
    self.update_owner(owner_id, pool).await
  }

  pub async fn fetch_all_users_with_pool(
    workspace_id: i64,
    pool: &PgPool,
  ) -> Result<Vec<ChatUser>, AppError> {
    Self::fetch_all_users(workspace_id, pool).await
  }

  pub async fn find_by_name_with_pool(name: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
    Self::find_by_name(name, pool).await
  }

  pub async fn find_by_id_with_pool(
    workspace_id: i64,
    pool: &PgPool,
  ) -> Result<Option<Self>, AppError> {
    Self::find_by_id(workspace_id, pool).await
  }

  pub async fn add_to_workspace_with_pool(
    &self,
    user_id: i64,
    pool: &PgPool,
  ) -> Result<Self, AppError> {
    self.add_to_workspace(user_id, pool).await
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::setup_test_users;
  use anyhow::{Ok, Result};

  #[tokio::test]
  async fn workspace_should_create_and_set_owner() -> Result<()> {
    let (_tdb, state, _users) = setup_test_users!(1).await;
    let user_id = _users[0].id;

    let workspace = Workspace::create("PWQ", 0, &state.pool).await?;
    assert_eq!(workspace.name, "PWQ");

    workspace.add_to_workspace(user_id, &state.pool).await?;

    let workspace_id = sqlx::query_scalar::<_, i64>("SELECT workspace_id FROM users WHERE id = $1")
      .bind(user_id)
      .fetch_one(&state.pool)
      .await?;

    assert_eq!(workspace.id, workspace_id);

    let updated_workspace = workspace.update_owner(user_id, &state.pool).await?;
    assert_eq!(updated_workspace.owner_id, user_id);

    Ok(())
  }

  #[tokio::test]
  async fn workspace_should_find_by_name() -> Result<()> {
    let (_tdb, state, _users) = setup_test_users!(1).await;

    let workspace = Workspace::find_by_name("Acme", &state.pool).await?;
    assert_eq!(workspace.unwrap().name, "Acme");

    let workspace = Workspace::find_by_name("NonExistentWorkspace", &state.pool).await?;
    assert!(workspace.is_none());

    Ok(())
  }

  #[tokio::test]
  async fn workspace_should_fetch_all_users() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(5).await;

    let workspace = Workspace::fetch_all_users(users[0].workspace_id, &state.pool).await?;
    assert_eq!(workspace.len(), 5);

    Ok(())
  }

  #[tokio::test]
  async fn workspace_should_update_owner() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(2).await;
    let user1 = users[0].clone();
    let user2 = users[1].clone();

    let workspace = Workspace::find_by_id(user1.workspace_id, &state.pool)
      .await?
      .unwrap();

    let updated_workspace = workspace.update_owner(user2.id, &state.pool).await?;

    assert_eq!(updated_workspace.owner_id, user2.id);
    Ok(())
  }
}
