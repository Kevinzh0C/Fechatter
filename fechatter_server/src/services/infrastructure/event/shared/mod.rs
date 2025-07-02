// Shared Event Publishing Components
//
// This module contains components shared between legacy and high-performance
// event publishing systems, including transport abstractions and utilities.

pub mod transport;
pub mod unified_publisher;

// Re-export transport components
pub use transport::{
    EventTransport, InMemoryTransport, KafkaConfig, KafkaSecurityConfig, KafkaTransport,
    NatsTransport, TransportConfig, TransportFactory,
};

// Re-export unified publisher (deprecated, use auto_degradation instead)
#[deprecated(note = "Use auto_degradation::AdaptivePublisher instead")]
pub use unified_publisher::*;
