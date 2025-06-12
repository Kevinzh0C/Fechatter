use sqlx::{PgPool, Row};
use std::sync::Arc;

use fechatter_core::{
  error::CoreError,
  models::{UserId, UserStatus, Workspace, WorkspaceId},
};

/// User within a workspace context
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct WorkspaceUser {
  pub id: UserId,
  pub fullname: String,
  pub email: String,
  pub status: UserStatus,
}

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

  /// List all users in a workspace
  pub async fn list_users(
    &self,
    workspace_id: WorkspaceId,
  ) -> Result<Vec<WorkspaceUser>, CoreError> {
    let users = sqlx::query_as::<_, WorkspaceUser>(
      r#"
      SELECT id, fullname, email, status
      FROM users 
      WHERE workspace_id = $1 
      ORDER BY fullname
      "#,
    )
    .bind(i64::from(workspace_id))
    .fetch_all(&*self.pool)
    .await
    .map_err(|e| CoreError::Database(e.to_string()))?;

    Ok(users)
  }

  /// Add members to workspace
  pub async fn add_members(
    &self,
    workspace_id: WorkspaceId,
    member_ids: &[UserId],
  ) -> Result<Vec<WorkspaceUser>, CoreError> {
    // Update users' workspace_id to add them to the workspace
    for member_id in member_ids {
      sqlx::query(
        r#"
        UPDATE users 
        SET workspace_id = $1 
        WHERE id = $2
        "#,
      )
      .bind(i64::from(workspace_id))
      .bind(i64::from(*member_id))
      .execute(&*self.pool)
      .await
      .map_err(|e| CoreError::Database(e.to_string()))?;
    }

    // Return the updated list of workspace users
    self.list_users(workspace_id).await
  }

  /// Check if users exist
  pub async fn check_users_exist(&self, user_ids: &[UserId]) -> Result<Vec<UserId>, CoreError> {
    let ids: Vec<i64> = user_ids.iter().map(|id| i64::from(*id)).collect();

    let existing_users = sqlx::query(
      r#"
      SELECT id 
      FROM users 
      WHERE id = ANY($1)
      "#,
    )
    .bind(&ids)
    .fetch_all(&*self.pool)
    .await
    .map_err(|e| CoreError::Database(e.to_string()))?;

    Ok(
      existing_users
        .into_iter()
        .map(|row| {
          let id: i64 = row
            .try_get(0)
            .map_err(|e| CoreError::Database(e.to_string()))?;
          Ok(UserId(id))
        })
        .collect::<Result<Vec<_>, CoreError>>()?,
    )
  }
}
