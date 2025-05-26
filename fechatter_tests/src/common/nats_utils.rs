//! NATS测试工具
//!
//! 封装NATS相关的测试操作

use anyhow::Result;
use async_nats::jetstream;
use futures::StreamExt;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tokio::time::timeout;

/// NATS测试工具
pub struct NatsTestUtils {
  client: async_nats::Client,
}

impl NatsTestUtils {
  /// 创建新的NATS测试工具
  pub fn new(client: async_nats::Client) -> Self {
    Self { client }
  }

  /// 发布消息
  pub async fn publish(&self, subject: String, payload: Vec<u8>) -> Result<()> {
    self.client.publish(subject, payload.into()).await?;
    Ok(())
  }

  /// 发布JSON消息
  pub async fn publish_json<T: Serialize>(&self, subject: String, data: &T) -> Result<()> {
    let payload = serde_json::to_vec(data)?;
    self.publish(subject, payload).await
  }

  /// 订阅主题并等待消息
  pub async fn subscribe_and_wait(
    &self,
    subject: String,
    timeout_secs: u64,
  ) -> Result<Option<async_nats::Message>> {
    let mut subscriber = self.client.subscribe(subject).await?;

    match tokio::time::timeout(
      tokio::time::Duration::from_secs(timeout_secs),
      subscriber.next(),
    )
    .await
    {
      Ok(Some(msg)) => Ok(Some(msg)),
      Ok(None) => Ok(None),
      Err(_) => Ok(None),
    }
  }

  /// 订阅并收集多条消息
  pub async fn subscribe_and_collect(
    &self,
    subject: String,
    expected_count: usize,
    timeout_secs: u64,
  ) -> Result<Vec<async_nats::Message>> {
    let mut subscriber = self.client.subscribe(subject).await?;
    let mut messages = Vec::new();

    let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(timeout_secs);

    while messages.len() < expected_count && tokio::time::Instant::now() < deadline {
      match tokio::time::timeout_at(deadline, subscriber.next()).await {
        Ok(Some(msg)) => messages.push(msg),
        _ => break,
      }
    }

    Ok(messages)
  }

  /// 检查JetStream是否可用
  pub async fn check_jetstream(&self) -> Result<()> {
    let jetstream = async_nats::jetstream::new(self.client.clone());
    // 尝试获取一个不存在的stream来检查JetStream是否可用
    match jetstream.get_stream("_CHECK_JS_AVAILABLE_").await {
      Err(e) => {
        let err_str = e.to_string();
        if err_str.contains("not found") || err_str.contains("No stream found") {
          Ok(())
        } else {
          Err(anyhow::anyhow!("JetStream check failed: {}", e))
        }
      }
      Ok(_) => Ok(()), // 不太可能，但如果存在就认为JS可用
    }
  }

  /// 创建或获取JetStream流
  pub async fn ensure_stream(&self, name: &str, subjects: Vec<String>) -> Result<()> {
    let jetstream = jetstream::new(self.client.clone());

    let config = jetstream::stream::Config {
      name: name.to_string(),
      subjects,
      ..Default::default()
    };

    // 尝试获取现有流
    match jetstream.get_stream(name).await {
      Ok(_) => {
        log::info!("Using existing stream: {}", name);
        Ok(())
      }
      Err(_) => {
        // 创建新流
        log::info!("Creating new stream: {}", name);
        jetstream.create_stream(config).await?;
        Ok(())
      }
    }
  }

  /// 清理测试流
  pub async fn cleanup_stream(&self, name: &str) -> Result<()> {
    let jetstream = jetstream::new(self.client.clone());

    match jetstream.delete_stream(name).await {
      Ok(_) => {
        log::info!("Deleted test stream: {}", name);
        Ok(())
      }
      Err(e) => {
        log::warn!("Failed to delete test stream {}: {}", name, e);
        Ok(()) // 忽略错误，可能流不存在
      }
    }
  }
}

/// NATS测试事件验证器
pub struct NatsEventValidator;

impl NatsEventValidator {
  /// 验证消息创建事件
  pub fn validate_message_created_event(
    event_data: &serde_json::Value,
    expected_content: &str,
    expected_chat_id: i64,
    expected_sender_id: i64,
  ) -> Result<()> {
    let message = event_data
      .get("message")
      .ok_or_else(|| anyhow::anyhow!("Missing 'message' field in event"))?;

    let content = message
      .get("content")
      .and_then(|v| v.as_str())
      .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'content' field"))?;

    let chat_id = message
      .get("chat_id")
      .and_then(|v| v.as_i64())
      .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'chat_id' field"))?;

    let sender_id = message
      .get("sender_id")
      .and_then(|v| v.as_i64())
      .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'sender_id' field"))?;

    anyhow::ensure!(content == expected_content, "Content mismatch");
    anyhow::ensure!(chat_id == expected_chat_id, "Chat ID mismatch");
    anyhow::ensure!(sender_id == expected_sender_id, "Sender ID mismatch");

    Ok(())
  }

  /// 验证成员加入事件
  pub fn validate_member_joined_event(
    event_data: &serde_json::Value,
    expected_chat_id: i64,
    expected_user_id: i64,
  ) -> Result<()> {
    let chat_id = event_data
      .get("chat_id")
      .and_then(|v| v.as_i64())
      .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'chat_id' field"))?;

    let user_id = event_data
      .get("user_id")
      .and_then(|v| v.as_i64())
      .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'user_id' field"))?;

    anyhow::ensure!(chat_id == expected_chat_id, "Chat ID mismatch");
    anyhow::ensure!(user_id == expected_user_id, "User ID mismatch");

    Ok(())
  }
}
