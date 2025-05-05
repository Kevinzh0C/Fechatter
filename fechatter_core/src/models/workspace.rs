use sqlx::query_as;

use crate::{
  error::CoreError,
  models::{ChatUser, DatabaseModel, Workspace},
  state::WithDbPool,
};

impl DatabaseModel for Workspace {
  type CreateType = String; // Workspace name
  type UpdateType = i64; // New owner ID
  type IdType = i64; // Workspace ID

  async fn create<S: WithDbPool + Sync>(
    input: &Self::CreateType,
    state: &S,
  ) -> Result<Self, CoreError> {
    if input.trim().is_empty() {
      return Err(CoreError::Validation(
        "Workspace name cannot be empty".into(),
      ));
    }

    let workspace = sqlx::query_as::<_, Workspace>(
      r#"
      INSERT INTO workspaces (name, owner_id)
      VALUES ($1, 0)
      RETURNING id, name, owner_id, created_at
      "#,
    )
    .bind(input)
    .fetch_one(state.db_pool())
    .await
    .map_err(|e| {
      if let Some(db_err) = e.as_database_error() {
        if db_err.is_unique_violation() {
          CoreError::Conflict(format!("Workspace with name {} already exists", input))
        } else {
          CoreError::Database(e)
        }
      } else {
        CoreError::Database(e)
      }
    })?;

    Ok(workspace)
  }

  async fn find_by_id<S: WithDbPool + Sync>(
    id: Self::IdType,
    state: &S,
  ) -> Result<Option<Self>, CoreError> {
    let workspace = sqlx::query_as::<_, Workspace>(
      r#"
      SELECT * FROM workspaces WHERE id = $1
      "#,
    )
    .bind(id)
    .fetch_optional(state.db_pool())
    .await
    .map_err(CoreError::Database)?;

    Ok(workspace)
  }

  async fn update<S: WithDbPool + Sync>(
    id: Self::IdType,
    owner_id: &Self::UpdateType,
    state: &S,
  ) -> Result<Self, CoreError> {
    let workspace = sqlx::query_as::<_, Workspace>(
      r#"
      UPDATE workspaces
      SET owner_id = $1
      WHERE id = $2 AND EXISTS (SELECT 1 FROM users u WHERE u.id = $1 AND u.workspace_id = $2)
      RETURNING id, name, owner_id, created_at
      "#,
    )
    .bind(owner_id)
    .bind(id)
    .fetch_one(state.db_pool())
    .await
    .map_err(CoreError::Database)?;

    Ok(workspace)
  }
}

impl Workspace {
  pub async fn find_by_name<S: WithDbPool + Sync>(
    name: &str,
    state: &S,
  ) -> Result<Option<Self>, CoreError> {
    let workspace = sqlx::query_as::<_, Workspace>(
      r#"
      SELECT * FROM workspaces WHERE name = $1
      "#,
    )
    .bind(name)
    .fetch_optional(state.db_pool())
    .await
    .map_err(CoreError::Database)?;

    Ok(workspace)
  }

  pub async fn fetch_all_users<S: WithDbPool + Sync>(
    workspace_id: i64,
    state: &S,
  ) -> Result<Vec<ChatUser>, CoreError> {
    let users = query_as::<_, ChatUser>(
      r#"
      SELECT u.id, u.fullname, u.email 
      FROM users u
      WHERE u.workspace_id = $1
      "#,
    )
    .bind(workspace_id)
    .fetch_all(state.db_pool())
    .await
    .map_err(CoreError::Database)?;

    Ok(users)
  }

  pub async fn add_to_workspace<S: WithDbPool + Sync>(
    &self,
    user_id: i64,
    state: &S,
  ) -> Result<Self, CoreError> {
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
    .fetch_one(state.db_pool())
    .await
    .map_err(CoreError::Database)?;

    let rows = sqlx::query("UPDATE users SET workspace_id = $1 WHERE id = $2")
      .bind(self.id)
      .bind(user_id)
      .execute(state.db_pool())
      .await
      .map_err(CoreError::Database)?
      .rows_affected();

    if rows == 0 {
      return Err(CoreError::NotFound(format!("User {} not found", user_id)));
    }

    Ok(workspace)
  }
}
