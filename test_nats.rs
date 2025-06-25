use tokio;

#[tokio::main]
async fn main() {
    println!("🧪 Testing NATS connection...");
    
    let nats_url = "nats://localhost:4222";
    println!("📡 Attempting to connect to: {}", nats_url);
    
    match async_nats::connect(nats_url).await {
        Ok(client) => {
            println!("✅ NATS connection successful!");
            println!("🔗 Connection state: {:?}", client.connection_state());
            
            // Try to publish a test message
            match client.publish("test.connection", "hello world".into()).await {
                Ok(_) => println!("✅ Test message published successfully!"),
                Err(e) => println!("❌ Failed to publish test message: {}", e),
            }
        }
        Err(e) => {
            println!("❌ NATS connection failed: {}", e);
            println!("🔍 Error details: {:?}", e);
        }
    }
} 