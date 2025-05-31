use sqlx::PgPool;
use std::sync::Arc;

use fechatter_core::{
  error::CoreError,
  models::{UserId, Workspace, WorkspaceId},
};

/// Workspace repository implementation
pub struct WorkspaceRepositoryImpl {
  pool: Arc<PgPool>,
}

impl WorkspaceRepositoryImpl {
  pub fn new(pool: Arc<PgPool>) -> Self {
    Self { pool }
  }

  /// Find workspace by name or create new one
  pub async fn find_or_create_by_name(&self, name: &str) -> Result<Workspace, CoreError> {
    // First try to find existing workspace
    if let Some(workspace) = self.find_by_name(name).await? {
      return Ok(workspace);
    }

    // Create new workspace if not found
    let workspace = sqlx::query_as::<_, Workspace>(
      r#"
      INSERT INTO workspaces (name, owner_id, created_at)
      VALUES ($1, 0, NOW())
      RETURNING id, name, owner_id, created_at, created_at as updated_at
      "#,
    )
    .bind(name)
    .fetch_one(&*self.pool)
    .await
    .map_err(|e| CoreError::Database(e.to_string()))?;

    Ok(workspace)
  }

  /// Find workspace by name
  pub async fn find_by_name(&self, name: &str) -> Result<Option<Workspace>, CoreError> {
    let workspace = sqlx::query_as::<_, Workspace>(
      "SELECT id, name, owner_id, created_at, created_at as updated_at FROM workspaces WHERE name = $1",
    )
    .bind(name)
    .fetch_optional(&*self.pool)
    .await
    .map_err(|e| CoreError::Database(e.to_string()))?;

    Ok(workspace)
  }

  /// Find workspace by ID
  pub async fn find_by_id(&self, id: WorkspaceId) -> Result<Option<Workspace>, CoreError> {
    let workspace = sqlx::query_as::<_, Workspace>(
      "SELECT id, name, owner_id, created_at, created_at as updated_at FROM workspaces WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&*self.pool)
    .await
    .map_err(|e| CoreError::Database(e.to_string()))?;

    Ok(workspace)
  }

  /// Update workspace owner
  pub async fn update_owner(
    &self,
    id: WorkspaceId,
    new_owner_id: UserId,
  ) -> Result<Workspace, CoreError> {
    let workspace = sqlx::query_as::<_, Workspace>(
      r#"
      UPDATE workspaces 
      SET owner_id = $1
      WHERE id = $2
      RETURNING id, name, owner_id, created_at, created_at as updated_at
      "#,
    )
    .bind(i64::from(new_owner_id))
    .bind(i64::from(id))
    .fetch_one(&*self.pool)
    .await
    .map_err(|e| CoreError::Database(e.to_string()))?;

    Ok(workspace)
  }
}
