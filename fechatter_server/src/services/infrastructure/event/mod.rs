// Event service implementations
// Provides pluggable transport layer for NATS, Kafka, etc.
// Now supports object-safe trait objects with Arc<dyn EventTransport>

pub mod event_publisher;
pub mod transport;

pub use event_publisher::{
  ChatInfo, ChatMemberJoined, ChatMemberLeft, DuplicateMessageEvent, DynEventPublisher,
  EventPublisher, MessageEvent, MsgLifecycle, NatsEventPublisher, RetryConfig, SearchEvent,
  SearchOp, Signable,
};

pub use transport::{
  EventTransport, KafkaConfig, KafkaSecurityConfig, KafkaTransport, NatsTransport,
};

pub use crate::error::EventTransportError;
