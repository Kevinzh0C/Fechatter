use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;

use super::entities::Notification;
use fechatter_core::error::CoreError;

/// 通知仓储接口
#[async_trait]
pub trait NotificationRepository: Send + Sync {
  /// 保存通知到数据库
  async fn save_notification(&self, notification: &Notification) -> Result<i64, CoreError>;

  /// 根据ID和用户ID查找通知
  async fn find_by_id_and_user(
    &self,
    notification_id: i64,
    user_id: i64,
  ) -> Result<Option<Notification>, CoreError>;

  /// 更新通知
  async fn update_notification(&self, notification: &Notification) -> Result<(), CoreError>;

  /// 获取用户的未读通知
  async fn get_unread_by_user(
    &self,
    user_id: i64,
    limit: i32,
  ) -> Result<Vec<Notification>, CoreError>;

  /// 获取用户的所有通知
  async fn get_user_notifications(&self, user_id: i64) -> Result<Vec<Notification>, CoreError>;

  /// 删除通知
  async fn delete_notification(&self, notification_id: i64, user_id: i64) -> Result<(), CoreError>;

  /// 获取未读通知数量
  async fn get_unread_count(&self, user_id: i64) -> Result<i64, CoreError>;

  /// 标记所有通知为已读
  async fn mark_all_as_read(&self, user_id: i64) -> Result<(), CoreError>;

  /// 删除过期通知
  async fn delete_expired_notifications(&self) -> Result<i64, CoreError>;
}

/// PostgreSQL实现
pub struct PostgresNotificationRepository {
  pool: Arc<PgPool>,
}

impl PostgresNotificationRepository {
  pub fn new(pool: Arc<PgPool>) -> Self {
    Self { pool }
  }
}

#[async_trait]
impl NotificationRepository for PostgresNotificationRepository {
  async fn save_notification(&self, notification: &Notification) -> Result<i64, CoreError> {
    // TODO: 当通知表创建后实现
    // 现在返回模拟ID
    Ok(1)
  }

  async fn find_by_id_and_user(
    &self,
    notification_id: i64,
    user_id: i64,
  ) -> Result<Option<Notification>, CoreError> {
    // TODO: 当通知表创建后实现
    Ok(None)
  }

  async fn update_notification(&self, notification: &Notification) -> Result<(), CoreError> {
    // TODO: 当通知表创建后实现
    Ok(())
  }

  async fn get_unread_by_user(
    &self,
    user_id: i64,
    limit: i32,
  ) -> Result<Vec<Notification>, CoreError> {
    // TODO: 当通知表创建后实现
    Ok(Vec::new())
  }

  async fn get_user_notifications(&self, user_id: i64) -> Result<Vec<Notification>, CoreError> {
    // TODO: 当通知表创建后实现
    Ok(Vec::new())
  }

  async fn delete_notification(&self, notification_id: i64, user_id: i64) -> Result<(), CoreError> {
    // TODO: 当通知表创建后实现
    Ok(())
  }

  async fn get_unread_count(&self, user_id: i64) -> Result<i64, CoreError> {
    // TODO: 当通知表创建后实现
    Ok(0)
  }

  async fn mark_all_as_read(&self, user_id: i64) -> Result<(), CoreError> {
    // TODO: 当通知表创建后实现
    Ok(())
  }

  async fn delete_expired_notifications(&self) -> Result<i64, CoreError> {
    // TODO: 当通知表创建后实现
    Ok(0)
  }
}
