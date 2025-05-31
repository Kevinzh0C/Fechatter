use crate::error::PublishError;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::sleep;

/// Retry strategy trait for handling transient errors
#[async_trait::async_trait]
pub trait RetryStrategy: Send + Sync {
  /// Execute a function with retry logic
  async fn retry_operation(
    &self,
    operation: Box<
      dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), PublishError>> + Send>> + Send + Sync,
    >,
  ) -> Result<(), PublishError>;
}

/// Exponential backoff retry strategy
#[derive(Debug, Clone)]
pub struct ExponentialBackoffRetry {
  /// Base delay in milliseconds
  pub base_delay_ms: u64,
  /// Maximum number of retries
  pub max_retries: u32,
  /// Backoff multiplier
  pub backoff_multiplier: f64,
  /// Maximum delay in milliseconds
  pub max_delay_ms: u64,
}

impl Default for ExponentialBackoffRetry {
  fn default() -> Self {
    Self {
      base_delay_ms: 1000, // 1秒
      max_retries: 3,
      backoff_multiplier: 2.0,
      max_delay_ms: 30000, // 30秒
    }
  }
}

impl ExponentialBackoffRetry {
  pub fn new(base_delay_ms: u64, max_retries: u32) -> Self {
    Self {
      base_delay_ms,
      max_retries,
      ..Default::default()
    }
  }

  /// Calculate delay for a given attempt
  fn calculate_delay(&self, attempt: u32) -> Duration {
    let delay_ms =
      (self.base_delay_ms as f64 * self.backoff_multiplier.powi(attempt as i32)) as u64;
    let capped_delay = delay_ms.min(self.max_delay_ms);
    Duration::from_millis(capped_delay)
  }

  /// Execute a function with retry logic (generic version for direct use)
  pub async fn retry<F, Fut, T>(&self, operation: F) -> Result<T, PublishError>
  where
    F: Fn() -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<T, PublishError>> + Send,
    T: Send,
  {
    let mut last_error: Option<PublishError> = None;

    for attempt in 0..=self.max_retries {
      match operation().await {
        Ok(result) => return Ok(result),
        Err(error) => {
          // 如果错误不可重试，立即返回
          if !error.is_retryable() {
            return Err(error);
          }

          last_error = Some(error.clone());

          // 如果这是最后一次尝试，返回错误
          if attempt == self.max_retries {
            return Err(error);
          }

          // 计算延迟并等待
          let delay = self.calculate_delay(attempt);
          tracing::warn!(
            "Publish attempt {} failed: {}. Retrying in {:?}",
            attempt + 1,
            error,
            delay
          );
          sleep(delay).await;
        }
      }
    }

    // 理论上不应该到达这里，但为了安全起见
    Err(last_error.unwrap_or_else(|| PublishError::Network("Max retries exceeded".to_string())))
  }
}

#[async_trait::async_trait]
impl RetryStrategy for ExponentialBackoffRetry {
  async fn retry_operation(
    &self,
    operation: Box<
      dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), PublishError>> + Send>> + Send + Sync,
    >,
  ) -> Result<(), PublishError> {
    let mut last_error: Option<PublishError> = None;

    for attempt in 0..=self.max_retries {
      match operation().await {
        Ok(result) => return Ok(result),
        Err(error) => {
          if !error.is_retryable() {
            return Err(error);
          }

          last_error = Some(error.clone());

          if attempt == self.max_retries {
            return Err(error);
          }

          let delay = self.calculate_delay(attempt);
          tracing::warn!(
            "Publish attempt {} failed: {}. Retrying in {:?}",
            attempt + 1,
            error,
            delay
          );
          sleep(delay).await;
        }
      }
    }

    Err(last_error.unwrap_or_else(|| PublishError::Network("Max retries exceeded".to_string())))
  }
}

/// Publisher trait with retry capabilities
#[async_trait::async_trait]
pub trait RetryablePublisher {
  /// Publish with automatic retry for Network and Timeout errors
  async fn publish_with_retry<T: serde::Serialize + Send + Sync>(
    &self,
    topic: &str,
    event: &T,
    retry_strategy: Option<&dyn RetryStrategy>,
  ) -> Result<(), PublishError>;
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::Arc;
  use std::sync::atomic::{AtomicU32, Ordering};

  #[tokio::test]
  async fn test_exponential_backoff_success() {
    let retry_strategy = ExponentialBackoffRetry::new(100, 3);
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result: Result<&str, PublishError> = retry_strategy
      .retry(|| {
        let counter = counter_clone.clone();
        async move {
          let count = counter.fetch_add(1, Ordering::SeqCst);
          if count < 2 {
            Err(PublishError::Network("test error".to_string()))
          } else {
            Ok("success")
          }
        }
      })
      .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
    assert_eq!(counter.load(Ordering::SeqCst), 3);
  }

  #[tokio::test]
  async fn test_non_retryable_error() {
    let retry_strategy = ExponentialBackoffRetry::new(100, 3);
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result: Result<&str, PublishError> = retry_strategy
      .retry(|| {
        let counter = counter_clone.clone();
        async move {
          counter.fetch_add(1, Ordering::SeqCst);
          Err(PublishError::Serialize("serialize error".to_string()))
        }
      })
      .await;

    assert!(result.is_err());
    // 序列化错误不重试，所以只应该调用一次
    assert_eq!(counter.load(Ordering::SeqCst), 1);
  }

  #[tokio::test]
  async fn test_max_retries_exceeded() {
    let retry_strategy = ExponentialBackoffRetry::new(50, 2); // 快速测试
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result: Result<&str, PublishError> = retry_strategy
      .retry(|| {
        let counter = counter_clone.clone();
        async move {
          counter.fetch_add(1, Ordering::SeqCst);
          Err(PublishError::Network("persistent error".to_string()))
        }
      })
      .await;

    assert!(result.is_err());
    // 应该尝试 max_retries + 1 次 (初始尝试 + 2次重试)
    assert_eq!(counter.load(Ordering::SeqCst), 3);
  }

  #[test]
  fn test_delay_calculation() {
    let retry_strategy = ExponentialBackoffRetry::new(1000, 5);

    assert_eq!(
      retry_strategy.calculate_delay(0),
      Duration::from_millis(1000)
    );
    assert_eq!(
      retry_strategy.calculate_delay(1),
      Duration::from_millis(2000)
    );
    assert_eq!(
      retry_strategy.calculate_delay(2),
      Duration::from_millis(4000)
    );
    assert_eq!(
      retry_strategy.calculate_delay(3),
      Duration::from_millis(8000)
    );
  }
}
