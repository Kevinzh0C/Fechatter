use sqlx::{Executor, Postgres, query_as};

use crate::{AppError, AppState};

use fechatter_core::{ChatUser, Workspace};

#[allow(unused)]
impl AppState {
  pub async fn create_workspace<'e, E>(
    &self,
    name: &str,
    user_id: i64,
    executor: E,
  ) -> Result<Workspace, AppError>
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

  pub async fn update_owner<'e, E>(
    &self,
    workspace_id: i64,
    owner_id: i64,
    executor: E,
  ) -> Result<Workspace, AppError>
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
    .bind(workspace_id)
    .fetch_one(executor)
    .await?;

    Ok(workspace)
  }

  pub async fn fetch_workspace_users<'e, E>(
    &self,
    workspace_id: i64,
    executor: E,
  ) -> Result<Vec<ChatUser>, AppError>
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
    .bind(workspace_id)
    .fetch_all(executor)
    .await?;

    Ok(users)
  }

  pub async fn find_by_name<'e, E>(
    &self,
    name: &str,
    executor: E,
  ) -> Result<Option<Workspace>, AppError>
  where
    E: Executor<'e, Database = Postgres>,
  {
    let workspace = sqlx::query_as::<_, Workspace>(
      r#"
      SELECT id, name, owner_id, created_at FROM workspaces WHERE name = $1
      "#,
    )
    .bind(name)
    .fetch_optional(executor)
    .await?;

    Ok(workspace)
  }

  pub async fn find_by_id<'e, E>(&self, id: i64, executor: E) -> Result<Option<Workspace>, AppError>
  where
    E: Executor<'e, Database = Postgres>,
  {
    let workspace = sqlx::query_as::<_, Workspace>(
      r#"
      SELECT id, name, owner_id, created_at FROM workspaces WHERE id = $1
      "#,
    )
    .bind(id)
    .fetch_optional(executor)
    .await?;

    Ok(workspace)
  }

  pub async fn add_to_workspace<'e, E>(
    &self,
    workspace_id: i64,
    user_id: i64,
    executor: E,
  ) -> Result<Workspace, AppError>
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
    .bind(workspace_id)
    .fetch_one(executor)
    .await?;

    let rows = sqlx::query("UPDATE users SET workspace_id = $1 WHERE id = $2")
      .bind(workspace_id)
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

#[allow(unused)]
impl AppState {
  pub async fn create_workspace_with_pool(
    &self,
    name: &str,
    user_id: i64,
  ) -> Result<Workspace, AppError> {
    self.create_workspace(name, user_id, self.pool()).await
  }

  /// Update the owner of a workspace.
  pub async fn update_workspace_owner<'e, E>(
    &self,
    workspace_id: i64,
    owner_id: i64,
  ) -> Result<Workspace, AppError> {
    self.update_owner(workspace_id, owner_id, self.pool()).await
  }

  pub async fn fetch_workspace_users_with_pool(
    &self,
    workspace_id: i64,
  ) -> Result<Vec<ChatUser>, AppError> {
    self.fetch_workspace_users(workspace_id, self.pool()).await
  }

  pub async fn find_by_name_with_pool(&self, name: &str) -> Result<Option<Workspace>, AppError> {
    self.find_by_name(name, self.pool()).await
  }

  pub async fn find_by_id_with_pool(
    &self,
    workspace_id: i64,
  ) -> Result<Option<Workspace>, AppError> {
    let workspace = sqlx::query_as::<_, Workspace>(
      "SELECT id, name, owner_id, created_at FROM workspaces WHERE id = $1",
    )
    .bind(workspace_id)
    .fetch_optional(self.pool())
    .await?;

    Ok(workspace)
  }

  pub async fn add_user_to_workspace<'e, E>(
    &self,
    workspace_id: i64,
    user_id: i64,
  ) -> Result<Workspace, AppError> {
    self
      .add_to_workspace(workspace_id, user_id, self.pool())
      .await
  }

  pub async fn add_to_workspace_with_pool(
    &self,
    workspace_id: i64,
    user_id: i64,
  ) -> Result<Workspace, AppError> {
    self
      .add_to_workspace(workspace_id, user_id, self.pool())
      .await
  }

  pub async fn update_owner_with_pool(
    &self,
    workspace_id: i64,
    owner_id: i64,
  ) -> Result<Workspace, AppError> {
    self.update_owner(workspace_id, owner_id, self.pool()).await
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

    let workspace = state.create_workspace_with_pool("PWQ", user_id).await?;
    assert_eq!(workspace.name, "PWQ");

    state
      .add_to_workspace_with_pool(workspace.id, user_id)
      .await?;

    let workspace_id = sqlx::query_scalar::<_, i64>("SELECT workspace_id FROM users WHERE id = $1")
      .bind(user_id)
      .fetch_one(state.pool())
      .await?;

    assert_eq!(workspace.id, workspace_id);

    let updated_workspace = state.update_owner_with_pool(workspace.id, user_id).await?;
    assert_eq!(updated_workspace.owner_id, user_id);

    Ok(())
  }

  #[tokio::test]
  async fn workspace_should_find_by_name() -> Result<()> {
    let (_tdb, state, _users) = setup_test_users!(1).await;

    let workspace = state.find_by_name_with_pool("Acme").await?;
    assert_eq!(workspace.unwrap().name, "Acme");

    let workspace = state.find_by_name_with_pool("NonExistentWorkspace").await?;
    assert!(workspace.is_none());

    Ok(())
  }

  #[tokio::test]
  async fn workspace_should_fetch_all_users() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(5).await;

    let workspace = state
      .fetch_workspace_users_with_pool(users[0].workspace_id)
      .await?;
    assert_eq!(workspace.len(), 5);

    Ok(())
  }

  #[tokio::test]
  async fn workspace_should_update_owner() -> Result<()> {
    let (_tdb, state, users) = setup_test_users!(2).await;
    let user1 = users[0].clone();
    let user2 = users[1].clone();

    let workspace = state.find_by_id_with_pool(user1.workspace_id).await?;
    let workspace = workspace.unwrap();

    let updated_workspace = state.update_owner_with_pool(workspace.id, user2.id).await?;

    assert_eq!(updated_workspace.owner_id, user2.id);
    Ok(())
  }
}
