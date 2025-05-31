use chrono::Utc;
use fechatter_core::{ChatId, Message, MessageId, UserId};
use fechatter_server::config::NatsSubjectsConfig;
use fechatter_server::services::infrastructure::event::{
  DynEventPublisher, EventPublisher, EventTransport, KafkaConfig, KafkaTransport,
  NatsEventPublisher, NatsTransport, RetryConfig,
};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("🚀 EventPublisher Pluggable Transport Demo");
  println!("📝 Now featuring object-safe trait objects!");

  // Create test subjects configuration
  let subjects = NatsSubjectsConfig {
    message_created: "demo.message.created".to_string(),
    message_updated: "demo.message.updated".to_string(),
    message_deleted: "demo.message.deleted".to_string(),
    chat_created: "demo.chat.created".to_string(),
    chat_updated: "demo.chat.updated".to_string(),
    user_joined_chat: "demo.chat.joined".to_string(),
    user_left_chat: "demo.chat.left".to_string(),
    duplicate_message_attempted: "demo.msg.dup".to_string(),
  };

  // Demo 1: NATS Transport (working implementation)
  println!("\n📡 Demo 1: NATS Transport");
  demo_nats_transport(subjects.clone()).await?;

  // Demo 2: Kafka Transport (placeholder implementation)
  println!("\n📡 Demo 2: Kafka Transport (Placeholder)");
  demo_kafka_transport(subjects.clone()).await?;

  // Demo 3: Custom Retry Configuration
  println!("\n⚙️  Demo 3: Custom Retry Configuration");
  demo_custom_retry_config(subjects.clone()).await?;

  // Demo 4: Object-Safe Trait Objects
  println!("\n🎯 Demo 4: Object-Safe Trait Objects");
  demo_object_safe_transports(subjects.clone()).await?;

  // Demo 5: Shared Transport with Arc
  println!("\n🔗 Demo 5: Shared Transport with Arc");
  demo_shared_transport(subjects.clone()).await?;

  // Demo 6: Generic Transport Usage
  println!("\n🔧 Demo 6: Generic Transport Usage");
  demo_generic_transport_usage(subjects).await?;

  println!("\n✅ All demos completed successfully!");
  Ok(())
}

async fn demo_nats_transport(
  subjects: NatsSubjectsConfig,
) -> Result<(), Box<dyn std::error::Error>> {
  // This would normally connect to a real NATS server
  // For demo purposes, we'll show the API usage

  println!("  • Creating NATS EventPublisher with backward-compatible API");

  // Method 1: Backward compatible constructor (would need real NATS client)
  // let nats_client = async_nats::connect("nats://localhost:4222").await?;
  // let publisher = NatsEventPublisher::new(nats_client, subjects);

  // Method 2: Using the new transport-based constructor
  // let transport = NatsTransport::new(nats_client);
  // let publisher = EventPublisher::with_transport(transport, subjects);

  println!("  • NATS transport supports:");
  println!("    - Message publishing with subjects");
  println!("    - Header-based signatures with validation");
  println!("    - Automatic retry with exponential backoff");
  println!("    - Health checking via connection status");
  println!("    - Pre-validated header names to prevent runtime errors");

  Ok(())
}

async fn demo_kafka_transport(
  subjects: NatsSubjectsConfig,
) -> Result<(), Box<dyn std::error::Error>> {
  println!("  • Creating Kafka EventPublisher (placeholder implementation)");

  let kafka_config = KafkaConfig {
    brokers: vec!["localhost:9092".to_string()],
    client_id: "fechatter-demo".to_string(),
    security: None,
  };

  let kafka_transport = KafkaTransport::new(kafka_config);
  let publisher = EventPublisher::with_transport(kafka_transport, subjects);

  println!(
    "  • Transport type: {}",
    publisher.transport().transport_type()
  );
  println!(
    "  • Health status: {}",
    publisher.transport().is_healthy().await
  );

  // Attempting to publish would return NotImplemented error
  let test_message = create_test_message();
  match publisher.publish_search_delete(&test_message.id).await {
    Err(e) => println!("  • Expected error: {}", e),
    Ok(_) => println!("  • Unexpected success"),
  }

  println!("  • Kafka transport is ready for future implementation");
  println!("  • All interfaces are defined and type-safe");

  Ok(())
}

async fn demo_custom_retry_config(
  subjects: NatsSubjectsConfig,
) -> Result<(), Box<dyn std::error::Error>> {
  println!("  • Creating EventPublisher with custom retry configuration");

  let kafka_transport = KafkaTransport::new(KafkaConfig {
    brokers: vec!["localhost:9092".to_string()],
    client_id: "fechatter-retry-demo".to_string(),
    security: None,
  });

  // Custom retry configuration with builder pattern
  let custom_retry = RetryConfig::new()
    .with_max_retries(3)
    .with_backoff_range(50, 1000); // 50ms to 1s

  let publisher =
    EventPublisher::with_transport(kafka_transport, subjects).with_retry_config(custom_retry);

  println!("  • Max retries: 3");
  println!("  • Backoff range: 50ms - 1000ms");
  println!("  • Retry configuration is transport-agnostic");
  println!("  • Refined error classification for better retry decisions");

  Ok(())
}

async fn demo_object_safe_transports(
  subjects: NatsSubjectsConfig,
) -> Result<(), Box<dyn std::error::Error>> {
  println!("  • Demonstrating object-safe trait objects");

  // Create different transport types
  let kafka_transport = KafkaTransport::new(KafkaConfig {
    brokers: vec!["localhost:9092".to_string()],
    client_id: "fechatter-object-safe".to_string(),
    security: None,
  });

  // Store in a heterogeneous collection using trait objects
  let transports: Vec<Arc<dyn EventTransport>> = vec![
    Arc::new(kafka_transport),
    // Arc::new(NatsTransport::new(nats_client)), // Would work with real client
  ];

  println!("  • Created heterogeneous collection of transports");
  println!("  • Transport types:");

  for (i, transport) in transports.iter().enumerate() {
    println!(
      "    {}. {} (healthy: {})",
      i + 1,
      transport.transport_type(),
      transport.is_healthy().await
    );

    // Create publisher from trait object
    let publisher = DynEventPublisher::with_dyn_transport(transport.clone(), subjects.clone());

    let test_message = create_test_message();
    match publisher.publish_search_delete(&test_message.id).await {
      Ok(_) => println!("      ✅ Published successfully"),
      Err(e) => println!("      ❌ Error: {}", e),
    }
  }

  println!("  • Object safety allows `dyn EventTransport` trait objects");
  println!("  • Perfect for plugin architectures and runtime selection");

  Ok(())
}

async fn demo_shared_transport(
  subjects: NatsSubjectsConfig,
) -> Result<(), Box<dyn std::error::Error>> {
  println!("  • Demonstrating shared transport with Arc");

  let kafka_transport = Arc::new(KafkaTransport::new(KafkaConfig {
    brokers: vec!["localhost:9092".to_string()],
    client_id: "fechatter-shared".to_string(),
    security: None,
  }));

  // Create multiple publishers sharing the same transport instance
  let publisher1 = EventPublisher::with_shared_transport(kafka_transport.clone(), subjects.clone());
  let publisher2 = EventPublisher::with_shared_transport(kafka_transport.clone(), subjects.clone());

  println!("  • Created 2 publishers sharing the same transport instance");
  println!(
    "  • Transport type: {}",
    publisher1.transport().transport_type()
  );
  println!("  • Both publishers use the same underlying connection");
  println!("  • Memory efficient: single transport, multiple publishers");

  // Both publishers share the same transport reference
  assert!(Arc::ptr_eq(
    publisher1.shared_transport(),
    publisher2.shared_transport()
  ));
  println!("  • ✅ Verified: Publishers share the same transport instance");

  Ok(())
}

async fn demo_generic_transport_usage(
  subjects: NatsSubjectsConfig,
) -> Result<(), Box<dyn std::error::Error>> {
  println!("  • Demonstrating generic transport usage");

  // Function that works with any EventTransport
  async fn publish_with_any_transport<T: EventTransport>(
    transport: T,
    subjects: NatsSubjectsConfig,
  ) -> Result<(), Box<dyn std::error::Error>> {
    let publisher = EventPublisher::with_transport(transport, subjects);

    println!(
      "    - Transport type: {}",
      publisher.transport().transport_type()
    );
    println!(
      "    - Health status: {}",
      publisher.transport().is_healthy().await
    );

    // This would work with any transport implementation
    let test_message = create_test_message();
    match publisher.publish_search_delete(&test_message.id).await {
      Ok(_) => println!("    - Successfully published event"),
      Err(e) => println!("    - Publish result: {}", e),
    }

    Ok(())
  }

  // Test with Kafka transport
  let kafka_transport = KafkaTransport::new(KafkaConfig {
    brokers: vec!["localhost:9092".to_string()],
    client_id: "fechatter-generic-demo".to_string(),
    security: None,
  });

  publish_with_any_transport(kafka_transport, subjects).await?;

  println!("  • Generic functions work with any transport");
  println!("  • Type safety enforced at compile time");
  println!("  • Easy to add new transport implementations");
  println!("  • Clone constraint removed for object safety");

  Ok(())
}

fn create_test_message() -> Message {
  Message {
    id: MessageId(42),
    chat_id: ChatId(100),
    sender_id: UserId(10),
    content: "Hello from demo!".to_string(),
    files: None,
    created_at: Utc::now(),
    idempotency_key: Some(Uuid::new_v4()),
  }
}
