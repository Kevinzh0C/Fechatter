use std::sync::Arc;
use tokio;

// Simple NATS connection test using the same configuration as fechatter_server
#[tokio::main]
async fn main() {
    println!("🔧 Debug: Testing NATS connection and downcasting...");
    
    let nats_url = "nats://localhost:4222";
    println!("📡 Connecting to NATS at: {}", nats_url);
    
    // Step 1: Direct NATS connection test
    match async_nats::connect(nats_url).await {
        Ok(client) => {
            println!("✅ Direct NATS connection successful!");
            println!("🔗 Connection state: {:?}", client.connection_state());
            
            // Step 2: Test publishing
            if let Err(e) = client.publish("test.debug", "hello".into()).await {
                println!("❌ Failed to publish test message: {}", e);
            } else {
                println!("✅ Test message published successfully!");
            }
            
            // Step 3: Test NatsTransport creation and downcasting
            println!("\n🔧 Testing EventTransport creation and downcasting...");
            
            // Create NatsTransport
            let nats_transport = Arc::new(NatsTransport::new(client.clone()));
            println!("✅ NatsTransport created successfully");
            
            // Test EventTransport trait methods
            println!("📋 Transport type: {}", nats_transport.transport_type());
            println!("❤️ Is healthy: {}", nats_transport.is_healthy().await);
            
            // Test downcasting - same as in AppState::nats_client()
            let transport: Arc<dyn EventTransport> = nats_transport.clone();
            
            if let Some(nats_transport_downcast) = transport
                .as_any()
                .downcast_ref::<NatsTransport>() 
            {
                println!("✅ Downcasting to NatsTransport successful!");
                let extracted_client = nats_transport_downcast.client().clone();
                println!("✅ Client extracted from downcast!");
                println!("🔗 Extracted client state: {:?}", extracted_client.connection_state());
            } else {
                println!("❌ Failed to downcast EventTransport to NatsTransport!");
            }
            
        }
        Err(e) => {
            println!("❌ NATS connection failed: {}", e);
            return;
        }
    }
    
    println!("\n🎯 Debug complete!");
}

// Copy the EventTransport trait and NatsTransport struct for standalone testing
use async_trait::async_trait;
use bytes::Bytes;  
use std::collections::HashMap;

#[async_trait]
pub trait EventTransport: Send + Sync {
    async fn publish(&self, subject: &str, payload: Bytes) -> Result<(), String>;
    async fn publish_with_headers(
        &self,
        subject: &str,
        headers: HashMap<String, String>,
        payload: Bytes,
    ) -> Result<(), String>;
    fn transport_type(&self) -> &'static str;
    async fn is_healthy(&self) -> bool;
    fn as_any(&self) -> &dyn std::any::Any;
}

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
    async fn publish(&self, subject: &str, payload: Bytes) -> Result<(), String> {
        self.client
            .publish(subject.to_string(), payload)
            .await
            .map_err(|e| e.to_string())
    }
    
    async fn publish_with_headers(
        &self,
        subject: &str,
        _headers: HashMap<String, String>,
        payload: Bytes,
    ) -> Result<(), String> {
        // Simplified implementation
        self.publish(subject, payload).await
    }
    
    fn transport_type(&self) -> &'static str {
        "NATS"
    }
    
    async fn is_healthy(&self) -> bool {
        match self.client.connection_state() {
            async_nats::connection::State::Connected => true,
            _ => false,
        }
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
} 