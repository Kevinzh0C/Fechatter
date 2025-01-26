//! 测试配置管理
//!
//! 统一管理测试环境的配置参数

use std::time::Duration;

/// 测试配置
#[derive(Debug, Clone)]
pub struct TestConfig {
  /// NATS连接URL
  pub nats_url: String,
  /// 数据库连接超时
  pub db_timeout: Duration,
  /// NATS消息超时
  pub nats_timeout: Duration,
  /// HTTP请求超时
  pub http_timeout: Duration,
  /// 测试数据清理延迟
  pub cleanup_delay: Duration,
  /// 并发测试的消息数量
  pub concurrent_message_count: usize,
  /// 性能测试的消息数量
  pub performance_message_count: usize,
}

impl Default for TestConfig {
  fn default() -> Self {
    Self {
      nats_url: "nats://localhost:4222".to_string(),
      db_timeout: Duration::from_secs(30),
      nats_timeout: Duration::from_secs(5),
      http_timeout: Duration::from_secs(10),
      cleanup_delay: Duration::from_millis(100),
      concurrent_message_count: 10,
      performance_message_count: 50,
    }
  }
}

impl TestConfig {
  /// 从环境变量创建配置
  pub fn from_env() -> Self {
    let mut config = Self::default();

    if let Ok(nats_url) = std::env::var("TEST_NATS_URL") {
      config.nats_url = nats_url;
    }

    if let Ok(timeout_str) = std::env::var("TEST_NATS_TIMEOUT") {
      if let Ok(timeout_secs) = timeout_str.parse::<u64>() {
        config.nats_timeout = Duration::from_secs(timeout_secs);
      }
    }

    if let Ok(count_str) = std::env::var("TEST_CONCURRENT_MESSAGES") {
      if let Ok(count) = count_str.parse::<usize>() {
        config.concurrent_message_count = count;
      }
    }

    if let Ok(count_str) = std::env::var("TEST_PERFORMANCE_MESSAGES") {
      if let Ok(count) = count_str.parse::<usize>() {
        config.performance_message_count = count;
      }
    }

    config
  }

  /// 获取全局测试配置
  pub fn global() -> &'static Self {
    static CONFIG: std::sync::OnceLock<TestConfig> = std::sync::OnceLock::new();
    CONFIG.get_or_init(|| Self::from_env())
  }
}
