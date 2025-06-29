# PublishError 和重试机制使用指南

## 概述

新定义的 `PublishError` 错误类型提供了细粒度的事件发布错误分类，支持智能重试策略。这个系统区分了不同类型的错误，并为可重试的错误提供了自动重试机制。

## PublishError 错误类型

```rust
use fechatter_core::PublishError;

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum PublishError {
    /// 序列化错误 - 不可重试
    #[error("Failed to serialize event: {0}")]
    Serialize(String),

    /// 网络错误 - 可重试，指数退避
    #[error("Network error during publish: {0}")]
    Network(String),

    /// 超时错误 - 可重试，调整超时参数
    #[error("Publish operation timed out: {0}")]
    Timeout(String),
}
```

### 错误类型特性

每种错误类型都有不同的重试策略：

- **Serialize**: 不可重试，立即失败
- **Network**: 可重试，默认重试3次，1秒基础延迟
- **Timeout**: 可重试，默认重试2次，2秒基础延迟

## 基本用法

### 1. 错误检查

```rust
use fechatter_core::PublishError;

async fn handle_publish_error(error: PublishError) {
    if error.is_retryable() {
        println!("可重试错误: {}", error);
        if let Some(delay) = error.retry_delay_ms() {
            println!("建议延迟: {}ms", delay);
        }
        println!("最大重试次数: {}", error.max_retries());
    } else {
        println!("不可重试错误: {}", error);
    }
}
```

### 2. 指数退避重试策略

```rust
use fechatter_core::{ExponentialBackoffRetry, PublishError};

async fn example_with_retry() -> Result<(), PublishError> {
    // 创建重试策略：基础延迟1000ms，最多重试3次
    let retry_strategy = ExponentialBackoffRetry::new(1000, 3);
    
    // 使用重试机制执行操作
    retry_strategy.retry(|| async {
        // 模拟可能失败的发布操作
        simulate_publish_operation().await
    }).await
}

async fn simulate_publish_operation() -> Result<String, PublishError> {
    // 模拟网络错误
    Err(PublishError::Network("连接超时".to_string()))
}
```

### 3. 自定义重试策略

```rust
use fechatter_core::ExponentialBackoffRetry;

// 自定义重试策略
let custom_retry = ExponentialBackoffRetry {
    base_delay_ms: 500,      // 基础延迟500ms
    max_retries: 5,          // 最多重试5次
    backoff_multiplier: 1.5, // 退避倍数1.5
    max_delay_ms: 10000,     // 最大延迟10秒
};
```

## 在EventPublisher中的使用

### 1. 基础发布（带重试）

```rust
use fechatter_core::{PublishError, RetryablePublisher};

impl EventPublisher {
    pub async fn publish_message_created(
        &self,
        message: &Message,
        chat_members: Vec<i64>,
    ) -> Result<(), AppError> {
        let event = MessageCreatedEvent {
            message: message.clone(),
            chat_members,
        };

        match self
            .publish_with_retry(&self.subjects.message_created, &event, None)
            .await
        {
            Ok(()) => {
                info!("事件发布成功: message_id={}", message.id);
                Ok(())
            }
            Err(PublishError::Serialize(e)) => {
                error!("序列化失败，不重试: {}", e);
                Err(AppError::PublishError(PublishError::Serialize(e)))
            }
            Err(e) => {
                error!("发布失败，已重试: {}", e);
                Err(AppError::PublishError(e))
            }
        }
    }
}
```

### 2. 自定义重试策略发布

```rust
use fechatter_core::ExponentialBackoffRetry;

impl EventPublisher {
    pub async fn publish_critical_event(
        &self,
        event: &CriticalEvent,
    ) -> Result<(), PublishError> {
        // 对关键事件使用更激进的重试策略
        let critical_retry = ExponentialBackoffRetry::new(500, 5);
        
        self.publish_with_retry(
            "critical.events",
            event,
            Some(&critical_retry),
        ).await
    }
}
```

## 错误转换

系统提供了从常见错误类型到 `PublishError` 的自动转换：

```rust
// 自动转换示例
async fn publish_example() -> Result<(), PublishError> {
    // serde_json::Error 自动转换为 PublishError::Serialize
    let payload = serde_json::to_vec(&event)?;
    
    // std::io::Error 根据错误类型转换为 Network 或 Timeout
    publish_to_nats(payload).await?;
    
    Ok(())
}
```

## 监控和日志

重试机制会自动记录重试过程：

```
WARN publish attempt 1 failed: Network error during publish: 连接超时. Retrying in 1s
WARN publish attempt 2 failed: Network error during publish: 连接超时. Retrying in 2s  
INFO 事件发布成功: message_id=12345
```

## 测试支持

提供了完整的测试工具：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_retry_until_success() {
        let retry_strategy = ExponentialBackoffRetry::new(100, 3);
        let counter = Arc::new(AtomicU32::new(0));
        
        let result = retry_strategy.retry(|| async {
            let count = counter.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                Err(PublishError::Network("test error".to_string()))
            } else {
                Ok("success")
            }
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }
}
```

## 最佳实践

### 1. 错误分类
- 使用 `PublishError::Serialize` 处理数据格式错误
- 使用 `PublishError::Network` 处理网络连接问题
- 使用 `PublishError::Timeout` 处理超时问题

### 2. 重试策略选择
- **普通事件**: 使用默认重试策略
- **关键事件**: 使用更长延迟和更多重试次数
- **实时事件**: 使用较短延迟和较少重试次数

### 3. 监控集成
```rust
use tracing::{error, warn, info};

match publish_result {
    Ok(()) => info!("事件发布成功"),
    Err(PublishError::Serialize(e)) => error!("序列化错误: {}", e),
    Err(PublishError::Network(e)) => warn!("网络错误: {}", e),
    Err(PublishError::Timeout(e)) => warn!("超时错误: {}", e),
}
```

## 架构优势

1. **类型安全**: 编译时错误分类
2. **智能重试**: 自动识别可重试错误
3. **可配置**: 灵活的重试策略配置
4. **可观测**: 完整的重试过程日志
5. **测试友好**: 易于单元测试和集成测试

这个架构为事件发布系统提供了强大的错误处理和恢复能力，确保系统在面对临时性故障时的健壮性。 