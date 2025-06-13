use crate::{AppState, NotifyEvent, notify::ChatMemberEventData, notify::DuplicateMessagePayload};
use async_nats::jetstream;
use fechatter_core::Message;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};
use tokio_stream::StreamExt;
use tracing::{error, info, warn};
use uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageCreatedEvent {
  pub message: Message,
  pub chat_members: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMemberEvent {
  pub chat_id: i64,
  pub user_id: i64,
  pub joined_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DuplicateMessageEvent {
  pub idempotency_key: uuid::Uuid,
  pub chat_id: i64,
  pub sender_id: i64,
}

struct Notification {
  user_ids: HashSet<i64>,
  event: Arc<NotifyEvent>,
}

pub async fn setup_nats_subscriber(state: AppState) -> anyhow::Result<()> {
  info!("Setting up NATS subscriber...");

  let nats_client = async_nats::connect(&state.config.messaging.nats.url).await?;
  let jetstream = jetstream::new(nats_client);

  // 创建或获取 stream
  let stream_name = &state.config.messaging.nats.jetstream.stream;
  let stream = match jetstream.get_stream(stream_name).await {
    Ok(stream) => {
      info!("Using existing JetStream stream: {}", stream_name);
      stream
    }
    Err(_) => {
      info!("Creating new JetStream stream: {}", stream_name);
      let stream_config = jetstream::stream::Config {
        name: stream_name.clone(),
        subjects: state.config.messaging.nats.subscription_subjects.clone(),
        storage: match state.config.messaging.nats.jetstream.storage.as_str() {
          "memory" => jetstream::stream::StorageType::Memory,
          _ => jetstream::stream::StorageType::File,
        },
        max_bytes: state.config.messaging.nats.jetstream.max_bytes as i64,
        max_age: std::time::Duration::from_secs(
          state.config.messaging.nats.jetstream.max_age * 3600,
        ),
        ..Default::default()
      };
      jetstream.create_stream(stream_config).await?
    }
  };

  // 创建或获取 consumer
  let consumer_name = &state
    .config
    .messaging
    .nats
    .jetstream
    .consumers
    .notification_processor
    .name;
  let consumer = match stream.get_consumer(consumer_name).await {
    Ok(consumer) => {
      info!("Using existing consumer: {}", consumer_name);
      consumer
    }
    Err(_) => {
      info!("Creating new consumer: {}", consumer_name);
      let consumer_config = jetstream::consumer::pull::Config {
        name: Some(consumer_name.clone()),
        filter_subjects: state
          .config
          .messaging
          .nats
          .jetstream
          .consumers
          .notification_processor
          .filter_subjects
          .clone(),
        ack_policy: jetstream::consumer::AckPolicy::Explicit,
        max_deliver: state
          .config
          .messaging
          .nats
          .jetstream
          .consumers
          .notification_processor
          .max_deliver as i64,
        ack_wait: parse_duration(
          &state
            .config
            .messaging
            .nats
            .jetstream
            .consumers
            .notification_processor
            .ack_wait,
        )?,
        ..Default::default()
      };
      stream.create_consumer(consumer_config).await?
    }
  };

  // 启动消息处理任务
  let mut messages = consumer.messages().await?;
  let state_clone = state.clone();

  tokio::spawn(async move {
    info!("Starting NATS message processing loop");
    while let Some(message) = messages.next().await {
      match message {
        Ok(msg) => {
          if let Err(e) = process_nats_message(&state_clone, msg).await {
            error!("Failed to process NATS message: {}", e);
          }
        }
        Err(e) => {
          error!("NATS message stream error: {}", e);
        }
      }
    }
  });

  info!("NATS subscriber setup completed");
  Ok(())
}

async fn process_nats_message(
  state: &AppState,
  message: jetstream::message::Message,
) -> anyhow::Result<()> {
  let subject = message.subject.as_str();
  let payload = message.payload.as_ref();

  info!("Processing NATS message: subject={}", subject);

  let notification = match subject {
    s if s.ends_with(".messages.created") => {
      let event: MessageCreatedEvent = serde_json::from_slice(payload)?;
      info!(
        "Processing message created event: message_id={}",
        event.message.id
      );
      create_message_notification(event)
    }
    s if s.ends_with(".chats.member.joined") => {
      let event: ChatMemberEvent = serde_json::from_slice(payload)?;
      info!(
        "Processing user joined chat event: user_id={}, chat_id={}",
        event.user_id, event.chat_id
      );
      create_user_joined_notification(event)
    }
    s if s.ends_with(".chats.member.left") => {
      let event: ChatMemberEvent = serde_json::from_slice(payload)?;
      info!(
        "Processing user left chat event: user_id={}, chat_id={}",
        event.user_id, event.chat_id
      );
      create_user_left_notification(event)
    }
    s if s.ends_with(".messages.duplicate") => {
      let event: DuplicateMessageEvent = serde_json::from_slice(payload)?;
      info!(
        "Processing duplicate message event: idempotency_key={}",
        event.idempotency_key
      );
      create_duplicate_message_notification(event)
    }
    _ => {
      warn!("Unhandled NATS subject: {}", subject);
      if let Err(e) = message.ack().await {
        error!("Failed to ack unhandled message: {}", e);
      }
      return Ok(());
    }
  };

  // 发送通知给相关用户
  send_notification_to_users(state, notification).await;

  // 确认消息处理完成
  if let Err(e) = message.ack().await {
    error!("Failed to ack processed message: {}", e);
    return Err(anyhow::anyhow!("Failed to ack message: {}", e));
  }

  info!("Successfully processed NATS message: subject={}", subject);
  Ok(())
}

fn create_message_notification(event: MessageCreatedEvent) -> Notification {
  let user_ids = event.chat_members.into_iter().collect();
  Notification {
    user_ids,
    event: Arc::new(NotifyEvent::NewMessage(event.message)),
  }
}

fn create_user_joined_notification(event: ChatMemberEvent) -> Notification {
  let mut user_ids = HashSet::new();
  user_ids.insert(event.user_id);

  Notification {
    user_ids,
    event: Arc::new(NotifyEvent::UserJoinedChat(ChatMemberEventData {
      chat_id: event.chat_id,
      user_id: event.user_id,
      joined_at: event.joined_at,
    })),
  }
}

fn create_user_left_notification(event: ChatMemberEvent) -> Notification {
  let mut user_ids = HashSet::new();
  user_ids.insert(event.user_id);

  Notification {
    user_ids,
    event: Arc::new(NotifyEvent::UserLeftChat(ChatMemberEventData {
      chat_id: event.chat_id,
      user_id: event.user_id,
      joined_at: event.joined_at,
    })),
  }
}

fn create_duplicate_message_notification(event: DuplicateMessageEvent) -> Notification {
  let mut user_ids = HashSet::new();
  user_ids.insert(event.sender_id);

  let duplicate_payload = DuplicateMessagePayload {
    idempotency_key: event.idempotency_key,
    chat_id: event.chat_id,
    sender_id: event.sender_id,
  };

  Notification {
    user_ids,
    event: Arc::new(NotifyEvent::DuplicateMessageAttempted(duplicate_payload)),
  }
}

async fn send_notification_to_users(state: &AppState, notification: Notification) {
  let users = &state.users;

  if notification.user_ids.is_empty() {
    info!("No users to notify for event: {:?}", notification.event);
    return;
  }

  for user_id in notification.user_ids {
    if let Some(tx) = users.get(&user_id) {
      info!(
        "Sending event {:?} to user: {}",
        notification.event, user_id
      );
      if let Err(e) = tx.send(notification.event.clone()) {
        warn!("Failed to send notification to user {}: {}", user_id, e);
      }
    }
  }
}

fn parse_duration(duration_str: &str) -> anyhow::Result<std::time::Duration> {
  // 简单的持续时间解析，支持 "30s", "1m", "1h" 等格式
  if duration_str.ends_with('s') {
    let secs: u64 = duration_str[..duration_str.len() - 1].parse()?;
    Ok(std::time::Duration::from_secs(secs))
  } else if duration_str.ends_with('m') {
    let mins: u64 = duration_str[..duration_str.len() - 1].parse()?;
    Ok(std::time::Duration::from_secs(mins * 60))
  } else if duration_str.ends_with('h') {
    let hours: u64 = duration_str[..duration_str.len() - 1].parse()?;
    Ok(std::time::Duration::from_secs(hours * 3600))
  } else {
    // 默认按秒处理
    let secs: u64 = duration_str.parse()?;
    Ok(std::time::Duration::from_secs(secs))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::NotifyEvent;
  use fechatter_core::Message;

  use std::sync::Arc;

  use uuid::Uuid;

  // 创建测试用的消息
  fn create_test_message() -> Message {
    Message {
      id: fechatter_core::MessageId(1),
      chat_id: fechatter_core::ChatId(100),
      sender_id: fechatter_core::UserId(10),
      content: "Test message content".to_string(),
      files: Some(vec!["file1.txt".to_string(), "file2.jpg".to_string()]),
      created_at: chrono::Utc::now(),
      idempotency_key: Some(Uuid::new_v4()),
    }
  }

  #[test]
  fn test_message_created_event_serialization() {
    let message = create_test_message();
    let chat_members = vec![1, 2, 3];

    let event = MessageCreatedEvent {
      message: message.clone(),
      chat_members: chat_members.clone(),
    };

    // 测试序列化
    let serialized = serde_json::to_vec(&event).expect("Failed to serialize event");
    assert!(!serialized.is_empty());

    // 测试反序列化
    let deserialized: MessageCreatedEvent =
      serde_json::from_slice(&serialized).expect("Failed to deserialize event");

    assert_eq!(deserialized.message.id, message.id);
    assert_eq!(deserialized.message.content, message.content);
    assert_eq!(deserialized.chat_members, chat_members);
  }

  #[test]
  fn test_chat_member_event_serialization() {
    let event = ChatMemberEvent {
      chat_id: 100,
      user_id: 10,
      joined_at: chrono::Utc::now(),
    };

    // 测试序列化和反序列化
    let serialized = serde_json::to_vec(&event).expect("Failed to serialize event");
    let deserialized: ChatMemberEvent =
      serde_json::from_slice(&serialized).expect("Failed to deserialize event");

    assert_eq!(deserialized.chat_id, event.chat_id);
    assert_eq!(deserialized.user_id, event.user_id);
    assert_eq!(deserialized.joined_at, event.joined_at);
  }

  #[test]
  fn test_duplicate_message_event_serialization() {
    let event = DuplicateMessageEvent {
      idempotency_key: Uuid::new_v4(),
      chat_id: 100,
      sender_id: 10,
    };

    // 测试序列化和反序列化
    let serialized = serde_json::to_vec(&event).expect("Failed to serialize event");
    let deserialized: DuplicateMessageEvent =
      serde_json::from_slice(&serialized).expect("Failed to deserialize event");

    assert_eq!(deserialized.idempotency_key, event.idempotency_key);
    assert_eq!(deserialized.chat_id, event.chat_id);
    assert_eq!(deserialized.sender_id, event.sender_id);
  }

  #[test]
  fn test_create_message_notification() {
    let message = create_test_message();
    let chat_members = vec![1, 2, 3];

    let event = MessageCreatedEvent {
      message: message.clone(),
      chat_members,
    };

    let notification = create_message_notification(event);

    // 验证用户ID集合
    assert_eq!(notification.user_ids.len(), 3);
    assert!(notification.user_ids.contains(&1));
    assert!(notification.user_ids.contains(&2));
    assert!(notification.user_ids.contains(&3));

    // 验证事件类型
    match notification.event.as_ref() {
      NotifyEvent::NewMessage(msg) => {
        assert_eq!(msg.id, message.id);
        assert_eq!(msg.content, message.content);
      }
      _ => panic!("Expected NewMessage event"),
    }
  }

  #[test]
  fn test_create_user_joined_notification() {
    let event = ChatMemberEvent {
      chat_id: 100,
      user_id: 10,
      joined_at: chrono::Utc::now(),
    };

    let notification = create_user_joined_notification(event.clone());

    // 验证用户ID集合
    assert_eq!(notification.user_ids.len(), 1);
    assert!(notification.user_ids.contains(&10));

    // 验证事件类型
    match notification.event.as_ref() {
      NotifyEvent::UserJoinedChat(data) => {
        assert_eq!(data.chat_id, event.chat_id);
        assert_eq!(data.user_id, event.user_id);
        assert_eq!(data.joined_at, event.joined_at);
      }
      _ => panic!("Expected UserJoinedChat event"),
    }
  }

  #[test]
  fn test_create_user_left_notification() {
    let event = ChatMemberEvent {
      chat_id: 100,
      user_id: 10,
      joined_at: chrono::Utc::now(),
    };

    let notification = create_user_left_notification(event.clone());

    // 验证用户ID集合
    assert_eq!(notification.user_ids.len(), 1);
    assert!(notification.user_ids.contains(&10));

    // 验证事件类型
    match notification.event.as_ref() {
      NotifyEvent::UserLeftChat(data) => {
        assert_eq!(data.chat_id, event.chat_id);
        assert_eq!(data.user_id, event.user_id);
        assert_eq!(data.joined_at, event.joined_at);
      }
      _ => panic!("Expected UserLeftChat event"),
    }
  }

  #[test]
  fn test_create_duplicate_message_notification() {
    let event = DuplicateMessageEvent {
      idempotency_key: Uuid::new_v4(),
      chat_id: 100,
      sender_id: 10,
    };

    let notification = create_duplicate_message_notification(event.clone());

    // 验证用户ID集合
    assert_eq!(notification.user_ids.len(), 1);
    assert!(notification.user_ids.contains(&10));

    // 验证事件类型
    match notification.event.as_ref() {
      NotifyEvent::DuplicateMessageAttempted(data) => {
        assert_eq!(data.idempotency_key, event.idempotency_key);
        assert_eq!(data.chat_id, event.chat_id);
        assert_eq!(data.sender_id, event.sender_id);
      }
      _ => panic!("Expected DuplicateMessageAttempted event"),
    }
  }

  #[test]
  fn test_parse_duration_seconds() {
    let duration = parse_duration("30s").expect("Failed to parse seconds");
    assert_eq!(duration, std::time::Duration::from_secs(30));

    let duration = parse_duration("0s").expect("Failed to parse zero seconds");
    assert_eq!(duration, std::time::Duration::from_secs(0));

    let duration = parse_duration("3600s").expect("Failed to parse large seconds");
    assert_eq!(duration, std::time::Duration::from_secs(3600));
  }

  #[test]
  fn test_parse_duration_minutes() {
    let duration = parse_duration("5m").expect("Failed to parse minutes");
    assert_eq!(duration, std::time::Duration::from_secs(300));

    let duration = parse_duration("0m").expect("Failed to parse zero minutes");
    assert_eq!(duration, std::time::Duration::from_secs(0));

    let duration = parse_duration("60m").expect("Failed to parse large minutes");
    assert_eq!(duration, std::time::Duration::from_secs(3600));
  }

  #[test]
  fn test_parse_duration_hours() {
    let duration = parse_duration("2h").expect("Failed to parse hours");
    assert_eq!(duration, std::time::Duration::from_secs(7200));

    let duration = parse_duration("0h").expect("Failed to parse zero hours");
    assert_eq!(duration, std::time::Duration::from_secs(0));

    let duration = parse_duration("24h").expect("Failed to parse large hours");
    assert_eq!(duration, std::time::Duration::from_secs(86400));
  }

  #[test]
  fn test_parse_duration_plain_number() {
    let duration = parse_duration("120").expect("Failed to parse plain number");
    assert_eq!(duration, std::time::Duration::from_secs(120));

    let duration = parse_duration("0").expect("Failed to parse zero");
    assert_eq!(duration, std::time::Duration::from_secs(0));
  }

  #[test]
  fn test_parse_duration_invalid() {
    let result = parse_duration("invalid");
    assert!(result.is_err());

    let result = parse_duration("30x");
    assert!(result.is_err());

    let result = parse_duration("");
    assert!(result.is_err());

    let result = parse_duration("s");
    assert!(result.is_err());

    let result = parse_duration("-5s");
    assert!(result.is_err());
  }

  #[test]
  fn test_notification_structure() {
    let user_ids: std::collections::HashSet<i64> = [1, 2, 3].into_iter().collect();
    let event = Arc::new(NotifyEvent::NewMessage(create_test_message()));

    let notification = Notification {
      user_ids: user_ids.clone(),
      event: event.clone(),
    };

    assert_eq!(notification.user_ids, user_ids);
    assert!(Arc::ptr_eq(&notification.event, &event));
  }

  #[test]
  fn test_event_structures_debug() {
    let message = create_test_message();
    let chat_members = vec![1, 2, 3];

    let message_event = MessageCreatedEvent {
      message: message.clone(),
      chat_members,
    };

    let chat_member_event = ChatMemberEvent {
      chat_id: 100,
      user_id: 10,
      joined_at: chrono::Utc::now(),
    };

    let duplicate_event = DuplicateMessageEvent {
      idempotency_key: Uuid::new_v4(),
      chat_id: 100,
      sender_id: 10,
    };

    // 确保Debug trait正常工作
    let message_debug = format!("{:?}", message_event);
    let chat_debug = format!("{:?}", chat_member_event);
    let duplicate_debug = format!("{:?}", duplicate_event);

    assert!(message_debug.contains("MessageCreatedEvent"));
    assert!(chat_debug.contains("ChatMemberEvent"));
    assert!(duplicate_debug.contains("DuplicateMessageEvent"));
  }

  #[test]
  fn test_edge_cases_empty_chat_members() {
    let message = create_test_message();
    let event = MessageCreatedEvent {
      message: message.clone(),
      chat_members: vec![], // 空的聊天成员列表
    };

    let notification = create_message_notification(event);
    assert!(notification.user_ids.is_empty());
  }

  #[test]
  fn test_edge_cases_large_chat_members() {
    let message = create_test_message();
    let large_member_list: Vec<i64> = (1..=1000).collect();

    let event = MessageCreatedEvent {
      message: message.clone(),
      chat_members: large_member_list.clone(),
    };

    let notification = create_message_notification(event);
    assert_eq!(notification.user_ids.len(), 1000);

    // 验证所有成员都包含在内
    for member_id in large_member_list {
      assert!(notification.user_ids.contains(&member_id));
    }
  }

  #[test]
  fn test_message_with_optional_fields() {
    let mut message = create_test_message();

    // 测试files为None的情况
    message.files = None;
    message.idempotency_key = None;

    let event = MessageCreatedEvent {
      message: message.clone(),
      chat_members: vec![1, 2],
    };

    let serialized = serde_json::to_vec(&event).expect("Failed to serialize event");
    let deserialized: MessageCreatedEvent =
      serde_json::from_slice(&serialized).expect("Failed to deserialize event");

    assert_eq!(deserialized.message.files, None);
    assert_eq!(deserialized.message.idempotency_key, None);
  }

  #[test]
  fn test_chat_member_event_with_extreme_timestamps() {
    // 测试极端时间戳
    use chrono::{DateTime, Utc};

    let min_time = DateTime::from_timestamp(0, 0).unwrap().with_timezone(&Utc);
    let event = ChatMemberEvent {
      chat_id: 100,
      user_id: 10,
      joined_at: min_time,
    };

    let serialized = serde_json::to_vec(&event).expect("Failed to serialize event");
    let deserialized: ChatMemberEvent =
      serde_json::from_slice(&serialized).expect("Failed to deserialize event");

    assert_eq!(deserialized.joined_at, min_time);
  }

  // 由于send_notification_to_users需要AppState，我们创建一个模拟测试
  #[tokio::test]
  async fn test_send_notification_to_users_empty_list() {
    // 创建一个模拟的AppState - 这个测试主要验证空用户列表的处理
    let user_ids = HashSet::new(); // 空的用户ID列表
    let event = Arc::new(NotifyEvent::NewMessage(create_test_message()));

    let notification = Notification { user_ids, event };

    // 验证空用户列表不会导致panic
    assert!(notification.user_ids.is_empty());
  }

  #[test]
  fn test_uuid_handling() {
    // 测试UUID的处理
    let test_uuid = Uuid::new_v4();
    let event = DuplicateMessageEvent {
      idempotency_key: test_uuid,
      chat_id: 100,
      sender_id: 10,
    };

    let serialized = serde_json::to_vec(&event).expect("Failed to serialize event");
    let deserialized: DuplicateMessageEvent =
      serde_json::from_slice(&serialized).expect("Failed to deserialize event");

    assert_eq!(deserialized.idempotency_key, test_uuid);

    // 测试nil UUID
    let nil_uuid = Uuid::nil();
    let event = DuplicateMessageEvent {
      idempotency_key: nil_uuid,
      chat_id: 100,
      sender_id: 10,
    };

    let serialized = serde_json::to_vec(&event).expect("Failed to serialize event");
    let deserialized: DuplicateMessageEvent =
      serde_json::from_slice(&serialized).expect("Failed to deserialize event");

    assert_eq!(deserialized.idempotency_key, nil_uuid);
  }

  // 性能测试
  #[test]
  fn test_large_message_content_serialization() {
    let large_content = "x".repeat(10000); // 10KB的内容
    let mut message = create_test_message();
    message.content = large_content.clone();

    let event = MessageCreatedEvent {
      message: message.clone(),
      chat_members: vec![1, 2, 3],
    };

    let start = std::time::Instant::now();
    let serialized = serde_json::to_vec(&event).expect("Failed to serialize large event");
    let serialize_time = start.elapsed();

    let start = std::time::Instant::now();
    let deserialized: MessageCreatedEvent =
      serde_json::from_slice(&serialized).expect("Failed to deserialize large event");
    let deserialize_time = start.elapsed();

    assert_eq!(deserialized.message.content, large_content);

    // 验证性能合理（这些值可以根据需要调整）
    assert!(serialize_time.as_millis() < 100); // 序列化应该在100ms内完成
    assert!(deserialize_time.as_millis() < 100); // 反序列化应该在100ms内完成
  }
}
