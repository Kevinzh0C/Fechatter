//! 测试通用模块
//!
//! 包含测试环境设置、HTTP客户端、NATS测试工具等

// 测试配置
pub mod test_config;
pub use test_config::TestConfig;

// 测试环境相关
pub mod test_env;
pub use test_env::TestEnvironment;

// HTTP客户端
pub mod http_client;
pub use http_client::HttpClient;

// NATS测试工具
pub mod nats_utils;
pub use nats_utils::{NatsEventValidator, NatsTestUtils};

// 测试数据fixtures
pub mod test_fixtures;
pub use test_fixtures::{TestAssertions, TestFixtures};

// 测试上下文
pub mod test_context;
pub use test_context::TestContext;

// 测试工具函数
pub mod test_utils {
  use anyhow::Result;

  /// 等待异步操作完成
  pub async fn wait_for_async(duration_ms: u64) {
    tokio::time::sleep(tokio::time::Duration::from_millis(duration_ms)).await;
  }

  /// 重试操作直到成功或超时
  pub async fn retry_until_success<F, T, E>(
    mut operation: F,
    max_attempts: usize,
    delay_ms: u64,
  ) -> Result<T>
  where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Display,
  {
    for attempt in 1..=max_attempts {
      match operation() {
        Ok(result) => return Ok(result),
        Err(e) if attempt < max_attempts => {
          log::debug!("Attempt {} failed: {}, retrying...", attempt, e);
          wait_for_async(delay_ms).await;
        }
        Err(e) => {
          return Err(anyhow::anyhow!(
            "Failed after {} attempts: {}",
            max_attempts,
            e
          ));
        }
      }
    }
    unreachable!()
  }
}
