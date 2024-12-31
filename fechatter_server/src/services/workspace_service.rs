use sqlx::{Executor, PgPool, Postgres};

use crate::{
  AppError,
  models::{ChatUser, Workspace},
};

/// 工作区服务，处理所有工作区相关的业务逻辑
pub struct WorkspaceService {
  pool: PgPool,
}

impl WorkspaceService {
  /// 创建新的工作区服务实例
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }

  /// 创建新工作区
  pub async fn create_workspace(&self, name: &str, user_id: i64) -> Result<Workspace, AppError> {
    Workspace::create(name, user_id, &self.pool).await
  }

  /// 更新工作区所有者
  pub async fn update_workspace_owner(
    &self,
    workspace_id: i64,
    owner_id: i64,
  ) -> Result<Workspace, AppError> {
    let workspace = self
      .find_workspace_by_id(workspace_id)
      .await?
      .ok_or_else(|| AppError::NotFound(vec![workspace_id.to_string()]))?;

    workspace.update_owner(owner_id, &self.pool).await
  }

  /// 获取工作区所有用户
  pub async fn get_workspace_users(&self, workspace_id: i64) -> Result<Vec<ChatUser>, AppError> {
    Workspace::fetch_all_users(workspace_id, &self.pool).await
  }

  /// 通过名称查找工作区
  pub async fn find_workspace_by_name(&self, name: &str) -> Result<Option<Workspace>, AppError> {
    Workspace::find_by_name(name, &self.pool).await
  }

  /// 通过ID查找工作区
  pub async fn find_workspace_by_id(&self, id: i64) -> Result<Option<Workspace>, AppError> {
    Workspace::find_by_id(id, &self.pool).await
  }

  /// 将用户添加到工作区
  pub async fn add_user_to_workspace(
    &self,
    workspace_id: i64,
    user_id: i64,
  ) -> Result<Workspace, AppError> {
    let workspace = self
      .find_workspace_by_id(workspace_id)
      .await?
      .ok_or_else(|| AppError::NotFound(vec![workspace_id.to_string()]))?;

    workspace.add_to_workspace(user_id, &self.pool).await
  }
}
