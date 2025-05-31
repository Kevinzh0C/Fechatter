// 简化的消息流服务 - 针对200DAU企业聊天优化
// 移除过度复杂的Redis Streams，使用WebSocket + 数据库

use crate::AppError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMessage {
  pub id: String,
  pub chat_id: i64,
  pub sender_id: i64,
  pub content: String,
  pub files: Vec<String>,
  pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStatusUpdate {
  pub message_id: i64,
  pub user_id: i64,
  pub status: String, // "delivered", "read"
  pub timestamp: i64,
}

pub struct MessageStreamService {
  message_sender: broadcast::Sender<StreamMessage>,
  status_sender: broadcast::Sender<MessageStatusUpdate>,
}

impl MessageStreamService {
  pub fn new() -> Self {
    let (message_sender, _) = broadcast::channel(1000);
    let (status_sender, _) = broadcast::channel(1000);

    Self {
      message_sender,
      status_sender,
    }
  }

  pub async fn publish_message(&self, message: StreamMessage) -> Result<(), AppError> {
    match self.message_sender.send(message.clone()) {
      Ok(receiver_count) => {
        info!(
          "Message {} broadcasted to {} receivers",
          message.id, receiver_count
        );
        Ok(())
      }
      Err(_) => {
        warn!("No active receivers for message {}", message.id);
        Ok(()) // 200DAU下，没有接收者不是错误
      }
    }
  }

  pub async fn publish_status_update(&self, status: MessageStatusUpdate) -> Result<(), AppError> {
    match self.status_sender.send(status.clone()) {
      Ok(receiver_count) => {
        info!(
          "Status update for message {} sent to {} receivers",
          status.message_id, receiver_count
        );
        Ok(())
      }
      Err(_) => {
        warn!(
          "No active receivers for status update of message {}",
          status.message_id
        );
        Ok(())
      }
    }
  }

  pub fn subscribe_messages(&self) -> broadcast::Receiver<StreamMessage> {
    self.message_sender.subscribe()
  }

  pub fn subscribe_status_updates(&self) -> broadcast::Receiver<MessageStatusUpdate> {
    self.status_sender.subscribe()
  }

  pub fn get_message_subscriber_count(&self) -> usize {
    self.message_sender.receiver_count()
  }

  pub fn get_status_subscriber_count(&self) -> usize {
    self.status_sender.receiver_count()
  }
}

/// 消息消费者 - 简化版本
pub struct MessageConsumer {
  stream_service: Arc<MessageStreamService>,
}

impl MessageConsumer {
  pub fn new(stream_service: Arc<MessageStreamService>) -> Self {
    Self { stream_service }
  }

  /// 启动消息消费 - 简化版本，返回JoinHandle供调用者管理
  pub fn start(&self) -> tokio::task::JoinHandle<()> {
    let stream_service = self.stream_service.clone();

    tokio::spawn(async move {
      let mut receiver = stream_service.subscribe_messages();

      loop {
        match receiver.recv().await {
          Ok(message) => {
            if let Err(e) = Self::process_message(&message).await {
              error!("Failed to process message {}: {}", message.id, e);
            }
          }
          Err(broadcast::error::RecvError::Closed) => {
            info!("Message stream closed, stopping consumer");
            break;
          }
          Err(broadcast::error::RecvError::Lagged(skipped)) => {
            warn!("Consumer lagged, skipped {} messages", skipped);
          }
        }
      }
    })
  }

  /// 处理单个消息 - 简化版本
  async fn process_message(message: &StreamMessage) -> Result<(), AppError> {
    info!(
      "Processing message: {} in chat {}",
      message.id, message.chat_id
    );

    // 在200DAU场景下，我们可以简化处理逻辑
    // 实际使用时，这里可以调用具体的业务逻辑

    // 模拟处理时间
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    info!("Message {} processed successfully", message.id);
    Ok(())
  }
}

impl MessageStreamService {
  pub fn create_message_event(
    message_id: i64,
    chat_id: i64,
    sender_id: i64,
    content: String,
    files: Vec<String>,
  ) -> StreamMessage {
    StreamMessage {
      id: message_id.to_string(),
      chat_id,
      sender_id,
      content,
      files,
      timestamp: chrono::Utc::now().timestamp(),
    }
  }

  pub fn create_status_event(message_id: i64, user_id: i64, status: &str) -> MessageStatusUpdate {
    MessageStatusUpdate {
      message_id,
      user_id,
      status: status.to_string(),
      timestamp: chrono::Utc::now().timestamp(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_simple_message_flow() {
    let service = MessageStreamService::new();
    let mut receiver = service.subscribe_messages();

    let test_message = StreamMessage {
      id: "test-1".to_string(),
      chat_id: 1,
      sender_id: 100,
      content: "Hello, World!".to_string(),
      files: vec![],
      timestamp: chrono::Utc::now().timestamp(),
    };

    // 发送消息
    service.publish_message(test_message.clone()).await.unwrap();

    // 接收消息
    let received = receiver.recv().await.unwrap();
    assert_eq!(received.id, test_message.id);
    assert_eq!(received.content, test_message.content);
  }

  #[test]
  fn test_subscriber_count() {
    let service = MessageStreamService::new();
    assert_eq!(service.get_message_subscriber_count(), 0);

    let _receiver1 = service.subscribe_messages();
    assert_eq!(service.get_message_subscriber_count(), 1);

    let _receiver2 = service.subscribe_messages();
    assert_eq!(service.get_message_subscriber_count(), 2);
  }
}
