use async_nats::Server;
use chrono::Utc;
use fechatter_core::{
  ChatId, Message, MessageId, UserId,
  contracts::events::{
    ChatMemberJoinedEvent, ChatMemberLeftEvent, DuplicateMessageEvent, EventVersion,
    HmacSha256Verifier, MessageEvent, MessageLifecycle, SignatureVerifier, subjects,
  },
};
use uuid::Uuid;

/// Test event serialization/deserialization compatibility
#[tokio::test]
async fn test_event_compatibility() {
  // Create a test message event, simulating an event sent from fechatter_server
  let message = Message {
    id: MessageId(123),
    chat_id: ChatId(456),
    sender_id: UserId(789),
    content: "Hello from integration test".to_string(),
    files: Some(vec!["test.txt".to_string()]),
    created_at: Utc::now(),
    idempotency_key: Some(Uuid::new_v4()),
  };

  let event = MessageEvent {
    version: EventVersion::default(),
    kind: MessageLifecycle::Created,
    msg: message.clone(),
    members: vec![UserId(1), UserId(2), UserId(3)],
    occurred_at: Utc::now(),
    sig: None,
  };

  // Serialize event (simulating fechatter_server sending)
  let serialized = serde_json::to_vec(&event).expect("Failed to serialize MessageEvent");

  // Deserialize event (simulating notify_server receiving)
  let deserialized: MessageEvent =
    serde_json::from_slice(&serialized).expect("Failed to deserialize MessageEvent");

  // Verify data integrity
  assert_eq!(deserialized.msg.id, message.id);
  assert_eq!(deserialized.msg.content, message.content);
  assert_eq!(deserialized.members.len(), 3);
  assert_eq!(deserialized.version, EventVersion::V1);
}

/// Test chat member joined event compatibility
#[tokio::test]
async fn test_chat_member_joined_compatibility() {
  let event = ChatMemberJoinedEvent {
    version: EventVersion::default(),
    chat_id: ChatId(100),
    user_id: UserId(200),
    occurred_at: Utc::now(),
    sig: Some("test_signature".to_string()),
  };

  let serialized = serde_json::to_vec(&event).expect("Failed to serialize ChatMemberJoinedEvent");
  let deserialized: ChatMemberJoinedEvent =
    serde_json::from_slice(&serialized).expect("Failed to deserialize ChatMemberJoinedEvent");

  assert_eq!(deserialized.chat_id, event.chat_id);
  assert_eq!(deserialized.user_id, event.user_id);
  assert_eq!(deserialized.sig, event.sig);
  assert_eq!(deserialized.version, EventVersion::V1);
}

/// Test duplicate message event compatibility
#[tokio::test]
async fn test_duplicate_message_compatibility() {
  let event = DuplicateMessageEvent {
    version: EventVersion::default(),
    idempotency_key: Uuid::new_v4(),
    chat_id: ChatId(300),
    sender_id: UserId(400),
    occurred_at: Utc::now(),
    sig: None,
  };

  let serialized = serde_json::to_vec(&event).expect("Failed to serialize DuplicateMessageEvent");
  let deserialized: DuplicateMessageEvent =
    serde_json::from_slice(&serialized).expect("Failed to deserialize DuplicateMessageEvent");

  assert_eq!(deserialized.idempotency_key, event.idempotency_key);
  assert_eq!(deserialized.chat_id, event.chat_id);
  assert_eq!(deserialized.sender_id, event.sender_id);
  assert_eq!(deserialized.version, EventVersion::V1);
}

/// Test complete NATS message flow
#[tokio::test]
async fn test_full_nats_message_flow() {
  // Start test NATS server
  let server = Server::new().await;
  let client = async_nats::connect(&server.client_url()).await.unwrap();

  // Subscribe to message topic
  let mut sub = client.subscribe(subjects::MESSAGE_CREATED).await.unwrap();

  // Create and send message event
  let message = Message {
    id: MessageId(999),
    chat_id: ChatId(888),
    sender_id: UserId(777),
    content: "Test NATS flow".to_string(),
    files: None,
    created_at: Utc::now(),
    idempotency_key: Some(Uuid::new_v4()),
  };

  let event = MessageEvent {
    version: EventVersion::default(),
    kind: MessageLifecycle::Created,
    msg: message.clone(),
    members: vec![UserId(1), UserId(2)],
    occurred_at: Utc::now(),
    sig: None,
  };

  let payload = serde_json::to_vec(&event).unwrap();
  client
    .publish(subjects::MESSAGE_CREATED, payload.into())
    .await
    .unwrap();

  // Receive and verify message
  use futures::StreamExt;
  let received = tokio::time::timeout(std::time::Duration::from_secs(1), sub.next())
    .await
    .unwrap()
    .unwrap();

  let received_event: MessageEvent = serde_json::from_slice(&received.payload).unwrap();
  assert_eq!(received_event.msg.id, message.id);
  assert_eq!(received_event.msg.content, message.content);

  server.close().await;
}

/// Test subject matching
#[test]
fn test_subject_matching() {
  // Ensure subject format is correct
  let subjects_list = vec![
    subjects::MESSAGE_CREATED,
    subjects::CHAT_MEMBER_JOINED,
    subjects::CHAT_MEMBER_LEFT,
    subjects::DUPLICATE_MESSAGE,
  ];

  for subject in subjects_list {
    assert!(subject.starts_with("fechatter."));
    assert!(!subject.contains("*"));
    assert!(!subject.contains(">"));
  }
}

/// Test signature verification
#[test]
fn test_signature_verification() {
  let verifier = HmacSha256Verifier;

  let secret = b"test-secret-key";
  let payload = b"test payload";

  // Create valid signature
  use hmac::{Hmac, Mac};
  use sha2::Sha256;

  let mut mac = Hmac::<Sha256>::new_from_slice(secret).unwrap();
  mac.update(payload);
  let signature = hex::encode(mac.finalize().into_bytes());

  // Verify signature
  assert!(verifier.verify_signature(payload, &signature, secret));

  // Verify invalid signatures
  assert!(!verifier.verify_signature(payload, "invalid_signature", secret));
  assert!(!verifier.verify_signature(payload, &signature, b"wrong_secret"));
}

/// Test event versioning
#[test]
fn test_event_versioning() {
  let event_v1 = MessageEvent {
    version: EventVersion::V1,
    kind: MessageLifecycle::Created,
    msg: Message {
      id: MessageId(1),
      chat_id: ChatId(1),
      sender_id: UserId(1),
      content: "test".to_string(),
      files: None,
      created_at: Utc::now(),
      idempotency_key: None,
    },
    members: vec![UserId(1)],
    occurred_at: Utc::now(),
    sig: None,
  };

  // Serialization includes version info
  let serialized = serde_json::to_string(&event_v1).unwrap();
  assert!(serialized.contains("\"version\":\"V1\""));

  // Default version when deserializing
  let without_version = r#"{
    "kind": "Created",
    "msg": {
      "id": 1,
      "chat_id": 1,
      "sender_id": 1,
      "content": "test",
      "files": null,
      "created_at": "2024-01-01T00:00:00Z",
      "idempotency_key": null
    },
    "members": [1],
    "occurred_at": "2024-01-01T00:00:00Z"
  }"#;

  let deserialized: MessageEvent = serde_json::from_str(without_version).unwrap();
  assert_eq!(deserialized.version, EventVersion::V1); // Default version
}
