# Notify Server Implementation Guide

This document provides practical guidance on implementing the Notify Server as described in the architecture document. It includes future plans for NATS JetStream integration, code examples, configuration details, and best practices.

## Current Implementation

The current implementation uses a simpler message distribution system. This document outlines our future plans to enhance the system with NATS JetStream for improved scalability and reliability.

## Future Implementation: Setup and Dependencies

When implementing NATS JetStream support in the future, the following dependencies will be required:

```toml
[dependencies]
# NATS client with JetStream support
async-nats = { version = "0.32", features = ["jetstream"] }

# WebSocket support
tokio-tungstenite = "0.20"
futures-util = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Utilities
dashmap = "5.5"  # Concurrent map for connection tracking
tracing = "0.1"  # Logging and telemetry
```

### Future NATS Server Configuration

When implementing NATS, a configuration file like this will be used:

```conf
# Basic server configuration
port: 4222
http_port: 8222

# JetStream configuration
jetstream {
  store_dir: "./data/jetstream"
  max_memory: 2GB
  max_file: 10GB
}

# Cluster configuration for horizontal scaling
cluster {
  name: "fechatter-cluster"
  port: 6222
  routes: [
    "nats://nats-1:6222",
    "nats://nats-2:6222",
    "nats://nats-3:6222"
  ]
}
```

## Future NATS JetStream Integration

The following code examples show how JetStream would be integrated in the future:

```rust
use async_nats::jetstream::{self, Context, stream::{Config, Stream}};

async fn setup_jetstream(js_context: &Context) -> Result<Stream, async_nats::Error> {
    // Create a stream for messages
    let stream_config = Config {
        name: "MESSAGES".to_string(),
        subjects: vec![
            "workspace.*.chat.*.messages".to_string(),
            "workspace.*.user.*.notifications".to_string(),
        ],
        max_age: std::time::Duration::from_secs(86400 * 7), // 7 days retention
        storage: jetstream::stream::StorageType::File,
        ..Default::default()
    };
    
    // Create or update the stream
    let stream = match js_context.get_stream("MESSAGES").await {
        Ok(stream) => stream.update(stream_config).await?,
        Err(_) => js_context.create_stream(stream_config).await?,
    };
    
    // Create a durable consumer for the notify server
    let consumer_config = jetstream::consumer::pull::Config {
        durable_name: Some(format!("notify-server-{}", uuid::Uuid::new_v4())),
        ack_policy: jetstream::consumer::AckPolicy::Explicit,
        max_deliver: 5,
        ack_wait: std::time::Duration::from_secs(30),
        filter_subject: "workspace.*.chat.*.messages".to_string(),
        ..Default::default()
    };
    
    let _consumer = stream.create_consumer(consumer_config).await?;
    
    Ok(stream)
}
```

## Current Connection Management Implementation

The current implementation uses a connection management approach similar to this:

```rust
use dashmap::DashMap;
use std::{sync::Arc, collections::HashSet};
use futures_util::StreamExt;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

// Client connection state
struct ClientConnection {
    user_id: i64,
    workspace_id: i64,
    subscribed_chats: HashSet<i64>,
    sender: mpsc::UnboundedSender<Message>,
}

// Connection manager
struct ConnectionManager {
    connections: DashMap<String, ClientConnection>,
    workspace_subscribers: DashMap<i64, HashSet<String>>,
    chat_subscribers: DashMap<i64, HashSet<String>>,
}

impl ConnectionManager {
    fn new() -> Self {
        Self {
            connections: DashMap::new(),
            workspace_subscribers: DashMap::new(),
            chat_subscribers: DashMap::new(),
        }
    }
    
    fn add_connection(&self, conn_id: String, connection: ClientConnection) {
        // Add to workspace subscribers
        self.workspace_subscribers
            .entry(connection.workspace_id)
            .or_insert_with(HashSet::new)
            .insert(conn_id.clone());
            
        // Add to chat subscribers for each subscribed chat
        for chat_id in &connection.subscribed_chats {
            self.chat_subscribers
                .entry(*chat_id)
                .or_insert_with(HashSet::new)
                .insert(conn_id.clone());
        }
        
        // Store the connection
        self.connections.insert(conn_id, connection);
    }
    
    fn remove_connection(&self, conn_id: &str) {
        if let Some((_, connection)) = self.connections.remove(conn_id) {
            // Remove from workspace subscribers
            if let Some(mut subscribers) = self.workspace_subscribers.get_mut(&connection.workspace_id) {
                subscribers.remove(conn_id);
            }
            
            // Remove from chat subscribers
            for chat_id in &connection.subscribed_chats {
                if let Some(mut subscribers) = self.chat_subscribers.get_mut(chat_id) {
                    subscribers.remove(conn_id);
                }
            }
        }
    }
    
    fn subscribe_to_chat(&self, conn_id: &str, chat_id: i64) -> bool {
        if let Some(mut connection) = self.connections.get_mut(conn_id) {
            connection.subscribed_chats.insert(chat_id);
            
            // Add to chat subscribers
            self.chat_subscribers
                .entry(chat_id)
                .or_insert_with(HashSet::new)
                .insert(conn_id.to_string());
                
            true
        } else {
            false
        }
    }
    
    fn dispatch_chat_message(&self, chat_id: i64, message: &str) {
        if let Some(subscribers) = self.chat_subscribers.get(&chat_id) {
            let ws_message = Message::Text(message.to_string());
            
            for conn_id in subscribers.iter() {
                if let Some(connection) = self.connections.get(conn_id) {
                    let _ = connection.sender.send(ws_message.clone());
                }
            }
        }
    }
}
```

## WebSocket Handler Implementation

Current WebSocket implementation:

```rust
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};

// Client message types
#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    #[serde(rename = "auth")]
    Auth { token: String },
    
    #[serde(rename = "subscribe")]
    Subscribe { chat_id: i64 },
    
    #[serde(rename = "unsubscribe")]
    Unsubscribe { chat_id: i64 },
    
    #[serde(rename = "ping")]
    Ping,
}

// Server message types
#[derive(Serialize)]
#[serde(tag = "type")]
enum ServerMessage {
    #[serde(rename = "auth_result")]
    AuthResult { success: bool, user_id: Option<i64> },
    
    #[serde(rename = "chat_message")]
    ChatMessage { 
        chat_id: i64, 
        message_id: i64, 
        sender_id: i64, 
        content: String,
        timestamp: i64,
    },
    
    #[serde(rename = "error")]
    Error { code: String, message: String },
    
    #[serde(rename = "pong")]
    Pong,
}

async fn handle_websocket_connection(
    conn_mgr: Arc<ConnectionManager>,
    auth_service: Arc<AuthService>,
    socket: TcpStream,
    addr: SocketAddr,
) {
    let conn_id = uuid::Uuid::new_v4().to_string();
    
    // Accept WebSocket connection
    let ws_stream = match accept_async(socket).await {
        Ok(ws) => ws,
        Err(e) => {
            tracing::error!("Error accepting WebSocket connection: {}", e);
            return;
        }
    };
    
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    // Create message channel
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    
    // Spawn task to forward messages from channel to WebSocket
    let forward_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = ws_sender.send(msg).await {
                tracing::error!("Error sending WebSocket message: {}", e);
                break;
            }
        }
    });
    
    // Process incoming messages
    let mut authenticated = false;
    let mut user_id = None;
    let mut workspace_id = None;
    
    while let Some(result) = ws_receiver.next().await {
        match result {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    match serde_json::from_str::<ClientMessage>(&text) {
                        Ok(ClientMessage::Auth { token }) => {
                            // Authenticate the user
                            match auth_service.verify_token(&token).await {
                                Ok(claims) => {
                                    authenticated = true;
                                    user_id = Some(claims.id);
                                    workspace_id = Some(claims.workspace_id);
                                    
                                    // Create client connection
                                    let connection = ClientConnection {
                                        user_id: claims.id,
                                        workspace_id: claims.workspace_id,
                                        subscribed_chats: HashSet::new(),
                                        sender: tx.clone(),
                                    };
                                    
                                    // Add to connection manager
                                    conn_mgr.add_connection(conn_id.clone(), connection);
                                    
                                    // Send success response
                                    let response = ServerMessage::AuthResult {
                                        success: true,
                                        user_id: Some(claims.id),
                                    };
                                    let _ = tx.send(Message::Text(serde_json::to_string(&response).unwrap()));
                                }
                                Err(_) => {
                                    // Send failure response
                                    let response = ServerMessage::AuthResult {
                                        success: false,
                                        user_id: None,
                                    };
                                    let _ = tx.send(Message::Text(serde_json::to_string(&response).unwrap()));
                                }
                            }
                        }
                        Ok(ClientMessage::Subscribe { chat_id }) => {
                            if authenticated {
                                conn_mgr.subscribe_to_chat(&conn_id, chat_id);
                            } else {
                                let response = ServerMessage::Error {
                                    code: "unauthorized".to_string(),
                                    message: "Authentication required".to_string(),
                                };
                                let _ = tx.send(Message::Text(serde_json::to_string(&response).unwrap()));
                            }
                        }
                        Ok(ClientMessage::Ping) => {
                            let _ = tx.send(Message::Text(serde_json::to_string(&ServerMessage::Pong).unwrap()));
                        }
                        Err(e) => {
                            tracing::error!("Error parsing client message: {}", e);
                            let response = ServerMessage::Error {
                                code: "invalid_message".to_string(),
                                message: "Invalid message format".to_string(),
                            };
                            let _ = tx.send(Message::Text(serde_json::to_string(&response).unwrap()));
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
        }
    }
    
    // Connection closed, clean up
    conn_mgr.remove_connection(&conn_id);
    forward_task.abort();
}
```

## Future Message Processor with NATS

When NATS JetStream is implemented, the message processor will look like this:

```rust
use async_nats::jetstream::{consumer::pull::Consumer, Message as NatsMessage};
use std::time::Duration;

#[derive(Deserialize)]
struct ChatMessagePayload {
    message_id: i64,
    chat_id: i64,
    workspace_id: i64,
    sender_id: i64,
    content: String,
    timestamp: i64,
}

async fn process_messages(
    consumer: Consumer,
    conn_mgr: Arc<ConnectionManager>,
) {
    // Create pull subscription
    let mut subscription = consumer.messages().await.unwrap();
    
    // Process messages
    while let Some(msg) = subscription.next().await {
        match process_single_message(msg, &conn_mgr).await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Error processing message: {}", e);
            }
        }
    }
}

async fn process_single_message(
    msg: NatsMessage,
    conn_mgr: &ConnectionManager,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get subject parts to determine message type
    let subject = msg.subject.to_string();
    let subject_parts: Vec<&str> = subject.split('.').collect();
    
    // Parse subject pattern: workspace.{id}.chat.{id}.messages
    if subject_parts.len() >= 5 && subject_parts[0] == "workspace" && subject_parts[2] == "chat" && subject_parts[4] == "messages" {
        // Parse message payload
        let payload: ChatMessagePayload = serde_json::from_slice(&msg.payload)?;
        
        // Create server message
        let server_msg = ServerMessage::ChatMessage {
            chat_id: payload.chat_id,
            message_id: payload.message_id,
            sender_id: payload.sender_id,
            content: payload.content,
            timestamp: payload.timestamp,
        };
        
        // Serialize message
        let message_text = serde_json::to_string(&server_msg)?;
        
        // Dispatch to connected clients
        conn_mgr.dispatch_chat_message(payload.chat_id, &message_text);
    }
    
    // Acknowledge the message
    msg.ack().await?;
    
    Ok(())
}
```

## Current Server Integration

The current server implementation:

```rust
use std::error::Error;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Create connection manager
    let conn_mgr = Arc::new(ConnectionManager::new());
    
    // Create auth service
    let auth_service = Arc::new(AuthService::new(/* ... */));
    
    // Start WebSocket server
    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("WebSocket server listening on: {}", addr);
    
    while let Ok((socket, addr)) = listener.accept().await {
        let conn_mgr = conn_mgr.clone();
        let auth_service = auth_service.clone();
        
        tokio::spawn(async move {
            handle_websocket_connection(conn_mgr, auth_service, socket, addr).await;
        });
    }
    
    Ok(())
}
```

## Future Server Integration with NATS

The future server implementation with NATS:

```rust
use std::error::Error;
use tokio::net::TcpListener;
use async_nats::connect;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Connect to NATS
    let nats_client = connect("nats://localhost:4222").await?;
    let js_context = async_nats::jetstream::new(nats_client.clone());
    
    // Setup JetStream
    let stream = setup_jetstream(&js_context).await?;
    let consumer = stream.get_consumer("notify-server").await?;
    
    // Create connection manager
    let conn_mgr = Arc::new(ConnectionManager::new());
    
    // Create auth service
    let auth_service = Arc::new(AuthService::new(/* ... */));
    
    // Start message processor
    let conn_mgr_clone = conn_mgr.clone();
    tokio::spawn(async move {
        process_messages(consumer, conn_mgr_clone).await;
    });
    
    // Start WebSocket server
    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("WebSocket server listening on: {}", addr);
    
    while let Ok((socket, addr)) = listener.accept().await {
        let conn_mgr = conn_mgr.clone();
        let auth_service = auth_service.clone();
        
        tokio::spawn(async move {
            handle_websocket_connection(conn_mgr, auth_service, socket, addr).await;
        });
    }
    
    Ok(())
}
```

## Horizontal Scaling Considerations

### Multiple Notify Server Instances

To run multiple instances of the notify server:

1. **Unique Consumer Names**: Ensure each server has a unique identifier.
2. **Load Balancing**: Set up a consistent hash load balancer (e.g., using HAProxy or Nginx Plus).
3. **Future Enhancement**: When NATS is implemented, configure consumers in the same queue group for load balancing.

Example HAProxy configuration for consistent hashing:

```
backend notify_servers
    balance url_param client_id check_post
    hash-type consistent
    server notify1 notify1:8080 check
    server notify2 notify2:8080 check
    server notify3 notify3:8080 check
```

### Future NATS Cluster Configuration

For a resilient NATS cluster (future implementation):

1. Run at least 3 NATS servers for quorum
2. Configure with proper routes for server discovery
3. Set up monitoring endpoints

### Connection Migration

When scaling the cluster:

1. **Client Reconnection**: Implement exponential backoff for client reconnects
2. **Session Resumption**: Allow clients to resume subscriptions when reconnecting
3. **Message Replay**: When NATS is implemented, support message sequence tracking for gap detection

## Testing the Implementation

Create integration tests:

```rust
#[tokio::test]
async fn test_message_dispatch() {
    // Setup test environment
    // Create test connection manager
    let conn_mgr = Arc::new(ConnectionManager::new());
    
    // Create mock WebSocket connections
    let (tx1, mut rx1) = mpsc::unbounded_channel::<Message>();
    let (tx2, mut rx2) = mpsc::unbounded_channel::<Message>();
    
    // Add test connections
    conn_mgr.add_connection("conn1".to_string(), ClientConnection {
        user_id: 1,
        workspace_id: 10,
        subscribed_chats: [100].into_iter().collect(),
        sender: tx1,
    });
    
    conn_mgr.add_connection("conn2".to_string(), ClientConnection {
        user_id: 2,
        workspace_id: 10,
        subscribed_chats: [100, 101].into_iter().collect(),
        sender: tx2,
    });
    
    // Create test message
    let message = ChatMessagePayload {
        message_id: 1000,
        chat_id: 100,
        workspace_id: 10,
        sender_id: 1,
        content: "Test message".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };
    
    // Simulate dispatch
    let message_text = serde_json::to_string(&ServerMessage::ChatMessage {
        chat_id: message.chat_id,
        message_id: message.message_id,
        sender_id: message.sender_id,
        content: message.content.clone(),
        timestamp: message.timestamp,
    }).unwrap();
    
    conn_mgr.dispatch_chat_message(message.chat_id, &message_text);
    
    // Verify both clients received the message
    let received1 = tokio::time::timeout(Duration::from_secs(1), rx1.recv()).await.unwrap().unwrap();
    let received2 = tokio::time::timeout(Duration::from_secs(1), rx2.recv()).await.unwrap().unwrap();
    
    if let Message::Text(text1) = received1 {
        let server_msg: ServerMessage = serde_json::from_str(&text1).unwrap();
        // Assert message content is correct
        // ...
    }
    
    if let Message::Text(text2) = received2 {
        let server_msg: ServerMessage = serde_json::from_str(&text2).unwrap();
        // Assert message content is correct
        // ...
    }
}
```

## Production Deployment

For production deployment:

1. **Docker Containers**: Package as Docker containers
2. **Kubernetes**: Use Deployments for notify servers (and StatefulSets for NATS servers when implemented)
3. **Health Checks**: Implement /health endpoint for monitoring
4. **Metrics**: Export Prometheus metrics for tracking
5. **Connection Draining**: Implement graceful shutdown for zero-downtime deployments

## Future Performance Optimization with NATS

For high-scale deployments with NATS:

1. **Connection Pooling**: Batch publish operations to NATS
2. **Message Batching**: Send multiple small messages in a single WebSocket frame
3. **Protocol Buffers**: Consider using ProtoBuf instead of JSON for efficiency
4. **WebSocket Compression**: Enable per-message deflate compression

## Conclusion

This implementation guide provides a foundation for building a scalable notification system. The current implementation focuses on WebSocket handling and connection management, while future enhancements will incorporate NATS JetStream for improved scalability and reliability. Adapt the code examples to your specific requirements and integrate with your existing authentication and authorization systems. 