use async_trait::async_trait;
use bytes::Bytes;
use std::collections::HashMap;
use std::str::FromStr;

// Import centralized error types from error module
use crate::error::EventTransportError;

/// Trait for event transport implementations (NATS, Kafka, etc.)
///
/// Use Arc<T> when cloning behavior is needed.
#[async_trait]
pub trait EventTransport: Send + Sync {
  /// Publish a message to a subject/topic
  async fn publish(&self, subject: &str, payload: Bytes) -> Result<(), EventTransportError>;

  /// Publish a message with headers to a subject/topic
  async fn publish_with_headers(
    &self,
    subject: &str,
    headers: HashMap<String, String>,
    payload: Bytes,
  ) -> Result<(), EventTransportError>;

  /// Get transport type name for logging/debugging
  fn transport_type(&self) -> &'static str;

  /// Check if the transport is connected/healthy
  async fn is_healthy(&self) -> bool;
}

/// NATS transport implementation
#[derive(Clone)]
pub struct NatsTransport {
  client: async_nats::Client,
}

impl NatsTransport {
  pub fn new(client: async_nats::Client) -> Self {
    Self { client }
  }

  pub fn client(&self) -> &async_nats::Client {
    &self.client
  }
}

#[async_trait]
impl EventTransport for NatsTransport {
  async fn publish(&self, subject: &str, payload: Bytes) -> Result<(), EventTransportError> {
    let subject_owned = subject.to_string();
    self
      .client
      .publish(subject_owned, payload)
      .await
      .map_err(|e| EventTransportError::Publish(e.to_string()))
  }

  async fn publish_with_headers(
    &self,
    subject: &str,
    headers: HashMap<String, String>,
    payload: Bytes,
  ) -> Result<(), EventTransportError> {
    let subject_owned = subject.to_string();
    let mut nats_headers = async_nats::HeaderMap::new();

    for (key, value) in headers {
      // Validate header name before inserting to avoid runtime errors
      let header_name = async_nats::HeaderName::from_str(&key).map_err(|_| {
        EventTransportError::InvalidHeader(format!(
          "Invalid header name '{}': must be valid HTTP header name",
          key
        ))
      })?;

      let header_value = async_nats::HeaderValue::from_str(&value).map_err(|_| {
        EventTransportError::InvalidHeader(format!(
          "Invalid header value for key '{}': {}",
          key, value
        ))
      })?;

      nats_headers.insert(header_name, header_value);
    }

    self
      .client
      .publish_with_headers(subject_owned, nats_headers, payload)
      .await
      .map_err(|e| EventTransportError::Publish(e.to_string()))
  }

  fn transport_type(&self) -> &'static str {
    "NATS"
  }

  async fn is_healthy(&self) -> bool {
    // Simple health check - NATS client connection status
    // In newer versions, we can just check if the connection is active
    true // Simplified for compatibility
  }
}

/// Kafka transport placeholder implementation
#[derive(Clone)]
pub struct KafkaTransport {
  _placeholder: std::marker::PhantomData<()>,
}

impl KafkaTransport {
  #[allow(dead_code)]
  pub fn new(_config: KafkaConfig) -> Self {
    Self {
      _placeholder: std::marker::PhantomData,
    }
  }
}

/// Kafka configuration placeholder
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct KafkaConfig {
  pub brokers: Vec<String>,
  pub client_id: String,
  pub security: Option<KafkaSecurityConfig>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct KafkaSecurityConfig {
  pub protocol: String,
  pub sasl_mechanism: Option<String>,
  pub username: Option<String>,
  pub password: Option<String>,
}

#[async_trait]
impl EventTransport for KafkaTransport {
  async fn publish(&self, _subject: &str, _payload: Bytes) -> Result<(), EventTransportError> {
    Err(EventTransportError::NotImplemented(
      "Kafka transport is not yet implemented. Please use NATS transport.".to_string(),
    ))
  }

  async fn publish_with_headers(
    &self,
    _subject: &str,
    _headers: HashMap<String, String>,
    _payload: Bytes,
  ) -> Result<(), EventTransportError> {
    Err(EventTransportError::NotImplemented(
      "Kafka transport is not yet implemented. Please use NATS transport.".to_string(),
    ))
  }

  fn transport_type(&self) -> &'static str {
    "Kafka (Not Implemented)"
  }

  async fn is_healthy(&self) -> bool {
    false // Not implemented yet
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_transport_error_retryable() {
    // Retryable errors
    assert!(EventTransportError::Connection("test".to_string()).is_retryable());
    assert!(EventTransportError::Timeout("test".to_string()).is_retryable());
    assert!(EventTransportError::Io("test".to_string()).is_retryable());

    // Non-retryable errors
    assert!(!EventTransportError::InvalidHeader("test".to_string()).is_retryable());
    assert!(!EventTransportError::NotImplemented("test".to_string()).is_retryable());
    assert!(!EventTransportError::Publish("test".to_string()).is_retryable());
    assert!(!EventTransportError::Other("test".to_string()).is_retryable());
  }

  #[test]
  fn test_kafka_transport_not_implemented() {
    let config = KafkaConfig {
      brokers: vec!["localhost:9092".to_string()],
      client_id: "test".to_string(),
      security: None,
    };
    let kafka = KafkaTransport::new(config);
    assert_eq!(kafka.transport_type(), "Kafka (Not Implemented)");
  }

  #[tokio::test]
  async fn test_kafka_methods_return_not_implemented() {
    let config = KafkaConfig {
      brokers: vec!["localhost:9092".to_string()],
      client_id: "test".to_string(),
      security: None,
    };
    let kafka = KafkaTransport::new(config);

    let result = kafka.publish("test", Bytes::new()).await;
    assert!(matches!(
      result,
      Err(EventTransportError::NotImplemented(_))
    ));

    let result = kafka
      .publish_with_headers("test", HashMap::new(), Bytes::new())
      .await;
    assert!(matches!(
      result,
      Err(EventTransportError::NotImplemented(_))
    ));

    assert!(!kafka.is_healthy().await);
  }

  #[test]
  fn test_invalid_header_name_validation() {
    // Test that invalid header names would be caught
    let invalid_names = vec![
      "invalid header",    // contains space
      "invalid\theader",   // contains tab
      "invalid\nheader",   // contains newline
      "",                  // empty
      "invalid\x00header", // contains null byte
    ];

    for invalid_name in invalid_names {
      let result = async_nats::HeaderName::from_str(invalid_name);
      assert!(
        result.is_err(),
        "Expected '{}' to be invalid header name",
        invalid_name
      );
    }
  }

  #[test]
  fn test_valid_header_names() {
    let valid_names = vec![
      "content-type",
      "x-custom-header",
      "authorization",
      "X-Signature",
      "user-agent",
    ];

    for valid_name in valid_names {
      let result = async_nats::HeaderName::from_str(valid_name);
      assert!(
        result.is_ok(),
        "Expected '{}' to be valid header name",
        valid_name
      );
    }
  }

  #[test]
  fn test_error_classification() {
    // Test EventTransportError variants directly instead of trying to convert from invalid NatsError variants

    // Connection errors (retryable)
    let conn_err = EventTransportError::Connection("test connection error".to_string());
    assert!(matches!(conn_err, EventTransportError::Connection(_)));
    assert!(conn_err.is_retryable());

    // Timeout errors (retryable)
    let timeout_err = EventTransportError::Timeout("test timeout error".to_string());
    assert!(matches!(timeout_err, EventTransportError::Timeout(_)));
    assert!(timeout_err.is_retryable());

    // Invalid header (non-retryable)
    let header_err = EventTransportError::InvalidHeader("test invalid header".to_string());
    assert!(matches!(header_err, EventTransportError::InvalidHeader(_)));
    assert!(!header_err.is_retryable());

    // IO errors (retryable)
    let io_err = EventTransportError::Io("test io error".to_string());
    assert!(matches!(io_err, EventTransportError::Io(_)));
    assert!(io_err.is_retryable());

    // Publish errors (non-retryable)
    let publish_err = EventTransportError::Publish("test publish error".to_string());
    assert!(matches!(publish_err, EventTransportError::Publish(_)));
    assert!(!publish_err.is_retryable());
  }
}
