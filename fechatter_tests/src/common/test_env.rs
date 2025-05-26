//! 测试环境管理器
//!
//! 提供统一的测试环境设置和清理功能

use anyhow::Result;
use fechatter_core::models::User;
use fechatter_server::AppState;
use sqlx_db_tester::TestPg;
use std::sync::Arc;

/// 测试环境配置
pub struct TestEnvironment {
  pub test_db: TestPg,
  pub app_state: AppState,
  nats_client: Option<async_nats::Client>,
  cleanup_tasks: Vec<Box<dyn FnOnce() + Send>>,
}

impl TestEnvironment {
  /// 创建新的测试环境
  pub async fn new() -> Result<Self> {
    let (test_db, app_state) = AppState::test_new().await?;

    Ok(Self {
      test_db,
      app_state,
      nats_client: None,
      cleanup_tasks: Vec::new(),
    })
  }

  /// 创建带NATS的测试环境
  pub async fn new_with_nats() -> Result<Self> {
    let (test_db, app_state) = AppState::test_new().await?;

    // 尝试连接NATS（如果可用）
    let nats_client = match async_nats::connect("nats://localhost:4222").await {
      Ok(client) => {
        log::info!("Connected to NATS for testing");
        Some(client)
      }
      Err(e) => {
        log::warn!("NATS not available for testing: {}", e);
        None
      }
    };

    Ok(Self {
      test_db,
      app_state,
      nats_client,
      cleanup_tasks: Vec::new(),
    })
  }

  /// 获取数据库连接池
  pub fn pool(&self) -> &sqlx::PgPool {
    self.app_state.pool()
  }

  /// 获取NATS客户端
  pub fn nats_client(&self) -> Option<&async_nats::Client> {
    self.nats_client.as_ref()
  }

  /// 检查NATS是否可用
  pub fn is_nats_available(&self) -> bool {
    self.nats_client.is_some()
  }

  /// 创建测试用户
  pub async fn create_test_user(&mut self, prefix: &str) -> Result<User> {
    use super::test_fixtures::TestFixtures;

    let user_data = TestFixtures::create_user(prefix);
    let user = self.app_state.create_user(&user_data, None).await?;
    Ok(user)
  }

  /// 批量创建测试用户
  pub async fn create_test_users(&mut self, count: usize) -> Result<Vec<User>> {
    use super::test_fixtures::TestFixtures;

    let mut users = Vec::with_capacity(count);
    for i in 0..count {
      let user_data = TestFixtures::create_user(&format!("test_{}", i));
      let user = self.app_state.create_user(&user_data, None).await?;
      users.push(user);
    }
    Ok(users)
  }

  /// 添加清理任务
  pub fn add_cleanup_task<F>(&mut self, task: F)
  where
    F: FnOnce() + Send + 'static,
  {
    self.cleanup_tasks.push(Box::new(task));
  }

  /// 清理测试环境
  pub async fn cleanup(&mut self) -> Result<()> {
    // 清理NATS连接
    if let Some(nats_client) = self.nats_client.take() {
      // NATS client 会在 drop 时自动清理，不需要显式关闭
      drop(nats_client);
    }

    // 清理数据库将在 TestDB drop 时自动执行
    Ok(())
  }
}

impl Drop for TestEnvironment {
  fn drop(&mut self) {
    // 确保清理任务被执行
    for task in self.cleanup_tasks.drain(..) {
      task();
    }
  }
}
