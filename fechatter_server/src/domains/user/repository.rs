use async_trait::async_trait;
use sqlx::{Acquire, PgPool};
use std::{mem, sync::Arc};

use crate::domains::workspace::repository::WorkspaceRepositoryImpl;
use fechatter_core::{
  CreateUser, SigninUser, User, UserId, WorkspaceId, contracts::UserRepository, error::CoreError,
};

use super::password::{hashed_password, verify_password};

/// User repository - 纯数据访问层
pub struct UserRepositoryImpl {
  pool: Arc<PgPool>,
  workspace_repo: Arc<WorkspaceRepositoryImpl>,
}

impl UserRepositoryImpl {
  pub fn new(pool: Arc<PgPool>) -> Self {
    let workspace_repo = Arc::new(WorkspaceRepositoryImpl::new(pool.clone()));
    Self {
      pool,
      workspace_repo,
    }
  }

  pub fn new_with_workspace_repo(
    pool: Arc<PgPool>,
    workspace_repo: Arc<WorkspaceRepositoryImpl>,
  ) -> Self {
    Self {
      pool,
      workspace_repo,
    }
  }
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
  async fn create(&self, input: &CreateUser) -> Result<User, CoreError> {
    // Check if email already exists
    let existing_user = self.exists_by_email(&input.email).await?;
    if existing_user {
      return Err(CoreError::UserAlreadyExists(format!(
        "User with email {} already exists",
        input.email
      )));
    }

    // Check if workspace exists (or create default)
    let workspace = self
      .workspace_repo
      .find_or_create_by_name(&input.workspace)
      .await?;

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
    if workspace.owner_id == UserId(0) {
      is_new_workspace = true;
    }

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
          CoreError::Validation(format!("User with email {} already exists", input.email))
        } else {
          CoreError::Internal(e.to_string())
        }
      } else {
        CoreError::Internal(e.to_string())
      }
    })?;

    if is_new_workspace {
      let _res = sqlx::query("UPDATE workspaces SET owner_id = $1 WHERE id = $2")
        .bind(user.id)
        .bind(workspace.id)
        .execute(&mut *conn)
        .await
        .map_err(|e| CoreError::Internal(e.to_string()))?;
    }

    tx.commit()
      .await
      .map_err(|e| CoreError::Internal(e.to_string()))?;

    Ok(user)
  }

  async fn find_by_id(&self, id: UserId) -> Result<Option<User>, CoreError> {
    let user = sqlx::query_as::<_, User>(
      "SELECT id, fullname, email, password_hash, status, created_at, workspace_id FROM users WHERE id = $1",
    )
    .bind(i64::from(id))
    .fetch_optional(&*self.pool)
    .await
    .map_err(|e| CoreError::Database(e.to_string()))?;

    Ok(user)
  }

  async fn find_by_email(&self, email: &str) -> Result<Option<User>, CoreError> {
    let user = sqlx::query_as::<_, User>(
      "SELECT id, fullname, email, password_hash, status, created_at, workspace_id FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(&*self.pool)
    .await
    .map_err(|e| CoreError::Database(e.to_string()))?;

    Ok(user)
  }

  async fn authenticate(&self, credentials: &SigninUser) -> Result<Option<User>, CoreError> {
    let user = self.find_by_email(&credentials.email).await?;

    match user {
      Some(mut user) => {
        let password_hash = match mem::take(&mut user.password_hash) {
          Some(h) => h,
          None => return Ok(None),
        };

        let is_valid = verify_password(&credentials.password, &password_hash)?;
        if is_valid { Ok(Some(user)) } else { Ok(None) }
      }
      None => Ok(None),
    }
  }

  async fn update(&self, id: UserId, user_data: &User) -> Result<User, CoreError> {
    let updated_user = sqlx::query_as::<_, User>(
      r#"UPDATE users 
         SET fullname = $1, email = $2, status = $3, updated_at = NOW()
         WHERE id = $4
         RETURNING id, fullname, email, password_hash, status, created_at, workspace_id"#,
    )
    .bind(&user_data.fullname)
    .bind(&user_data.email)
    .bind(&user_data.status)
    .bind(i64::from(id))
    .fetch_one(&*self.pool)
    .await
    .map_err(|e| CoreError::Database(e.to_string()))?;

    Ok(updated_user)
  }

  async fn exists_by_email(&self, email: &str) -> Result<bool, CoreError> {
    let user = self.find_by_email(email).await?;
    Ok(user.is_some())
  }
}

// Repository扩展方法 - 仅数据访问，不是trait方法
impl UserRepositoryImpl {
  /// 验证用户ID列表是否都存在
  pub async fn validate_users_exist_by_ids(&self, ids: &[UserId]) -> Result<(), CoreError> {
    let query = r#"
      SELECT COUNT(DISTINCT id) as count FROM users WHERE id = ANY($1)
    "#;

    let count: i64 = sqlx::query_scalar(query)
      .bind(&ids.iter().map(|id| i64::from(*id)).collect::<Vec<_>>())
      .fetch_one(&*self.pool)
      .await
      .map_err(|e| CoreError::Database(e.to_string()))?;

    if count != ids.len() as i64 {
      let missing_ids = ids
        .iter()
        .map(|id| i64::from(*id).to_string())
        .collect::<Vec<_>>()
        .join(", ");

      return Err(CoreError::NotFound(format!(
        "Some users not found. IDs: {}",
        missing_ids
      )));
    }

    Ok(())
  }

  /// 更新用户密码哈希
  pub async fn update_password_hash(
    &self,
    user_id: UserId,
    new_hash: String,
  ) -> Result<(), CoreError> {
    let _result =
      sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(new_hash)
        .bind(i64::from(user_id))
        .execute(&*self.pool)
        .await
        .map_err(|e| CoreError::Database(e.to_string()))?;

    Ok(())
  }

  /// 检查用户邮箱是否存在 (别名方法)
  pub async fn email_user_exists(&self, email: &str) -> Result<Option<User>, CoreError> {
    self.find_by_email(email).await
  }

  /// 根据ID查找用户 (公共方法)
  pub async fn find_by_id_ext(&self, id: UserId) -> Result<Option<User>, CoreError> {
    self.find_by_id(id).await
  }

  /// 切换工作空间
  pub async fn switch_workspace(
    &self,
    user_id: UserId,
    workspace_id: WorkspaceId,
  ) -> Result<User, CoreError> {
    let updated_user = sqlx::query_as::<_, User>(
      r#"UPDATE users 
         SET workspace_id = $1, updated_at = NOW()
         WHERE id = $2
         RETURNING id, fullname, email, password_hash, status, created_at, workspace_id"#,
    )
    .bind(workspace_id)
    .bind(i64::from(user_id))
    .fetch_one(&*self.pool)
    .await
    .map_err(|e| CoreError::Database(e.to_string()))?;

    Ok(updated_user)
  }
}
