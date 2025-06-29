# Fechatter API Documentation

A comprehensive guide to the Fechatter API ecosystem, covering REST APIs, real-time communication, authentication, and integration patterns.

## 📋 Table of Contents

1. [API Overview & Authentication](#1-api-overview--authentication)
2. [REST API Endpoints](#2-rest-api-endpoints)
3. [WebSocket/SSE Real-time APIs](#3-websocketsse-real-time-apis)
4. [Message Passing Protocols](#4-message-passing-protocols)
5. [Error Handling & Status Codes](#5-error-handling--status-codes)
6. [SDK Usage Examples](#6-sdk-usage-examples)

---

## 1. API Overview & Authentication

### 🌐 API Architecture

Fechatter implements a microservices architecture with multiple API endpoints:

```
Frontend → API Gateway (:8080) → Services
                 ↓
    ┌─────────────────┬─────────────────┬─────────────────┐
    │                 │                 │                 │
Fechatter Server   Notify Server    Bot Server      Analytics Server
    (:6688)          (:6687)         (:6686)           (:6690)
```

### 🔐 Authentication

#### JWT Token Authentication

All API requests require JWT authentication via the `Authorization` header:

```bash
curl -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  http://localhost:8080/api/chats
```

#### Getting Access Tokens

**Sign In Request:**
```bash
curl -X POST "http://localhost:8080/api/signin" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "password123"
  }'
```

**Response:**
```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user": {
      "id": 1,
      "email": "user@example.com",
      "display_name": "John Doe",
      "workspace_id": 2
    },
    "expires_in": 3600
  }
}
```

#### Token Refresh

```bash
curl -X POST "http://localhost:8080/api/refresh" \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }'
```

### 🌍 Base URLs

| Environment | Base URL | Description |
|------------|----------|-------------|
| Local Development | `http://localhost:8080` | Docker Compose setup |
| Production | `https://your-domain.com` | Production deployment |
| Cloud | `https://hook-nav-attempt-size.trycloudflare.com` | Example cloud deployment |

---

## 2. REST API Endpoints

### 👤 Authentication & User Management

#### User Registration
```bash
POST /api/signup
Content-Type: application/json

{
  "email": "newuser@example.com",
  "password": "securepassword123",
  "display_name": "New User",
  "workspace_id": 1
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "user_id": 5,
    "email": "newuser@example.com",
    "display_name": "New User"
  },
  "message": "User created successfully"
}
```

#### User Sign In
```bash
POST /api/signin
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123"
}
```

#### Get Current User Profile
```bash
GET /api/users/me
Authorization: Bearer {token}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": 1,
    "email": "user@example.com",
    "display_name": "John Doe",
    "avatar_url": null,
    "workspace_id": 2,
    "created_at": "2023-12-01T10:00:00Z"
  }
}
```

### 💬 Chat Management

#### Get Chat List
```bash
GET /api/chats
Authorization: Bearer {token}
```

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": 2,
      "name": "General Discussion",
      "chat_type": "Group",
      "member_count": 5,
      "workspace_id": 2,
      "last_message": {
        "content": "Hello everyone!",
        "created_at": "2023-12-01T10:30:00Z",
        "sender_name": "Jane Doe"
      },
      "unread_count": 3
    }
  ]
}
```

#### Create New Chat
```bash
POST /api/chat
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "Project Alpha Team",
  "chat_type": "Group",
  "initial_members": [1, 2, 3, 4, 5],
  "workspace_id": 2
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": 3,
    "name": "Project Alpha Team",
    "chat_type": "Group",
    "member_count": 5,
    "workspace_id": 2,
    "created_at": "2023-12-01T11:00:00Z"
  },
  "message": "Chat created successfully"
}
```

#### Get Chat Details
```bash
GET /api/chats/{chat_id}
Authorization: Bearer {token}
```

#### Join Chat
```bash
POST /api/chats/{chat_id}/join
Authorization: Bearer {token}
```

#### Leave Chat
```bash
POST /api/chats/{chat_id}/leave
Authorization: Bearer {token}
```

### 📝 Message Management

#### Send Message
```bash
POST /api/chat/{chat_id}/messages
Authorization: Bearer {token}
Content-Type: application/json

{
  "content": "Hello everyone! How's the project going?",
  "message_type": "text"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": 12345,
    "content": "Hello everyone! How's the project going?",
    "chat_id": 2,
    "sender_id": 1,
    "sender_name": "John Doe",
    "message_type": "text",
    "created_at": "2023-12-01T12:00:00Z",
    "status": "sent"
  },
  "message": "Message sent successfully"
}
```

#### Get Chat Messages
```bash
GET /api/chats/{chat_id}/messages?limit=20&offset=0
Authorization: Bearer {token}
```

**Query Parameters:**
- `limit`: Number of messages to fetch (default: 20, max: 100)
- `offset`: Number of messages to skip (for pagination)
- `before_id`: Get messages before specific message ID
- `after_id`: Get messages after specific message ID

**Response:**
```json
{
  "success": true,
  "data": {
    "messages": [
      {
        "id": 12345,
        "content": "Hello everyone!",
        "chat_id": 2,
        "sender_id": 1,
        "sender_name": "John Doe",
        "message_type": "text",
        "created_at": "2023-12-01T12:00:00Z",
        "reactions": [],
        "reply_to": null
      }
    ],
    "total_count": 1,
    "has_more": false
  }
}
```

#### Edit Message
```bash
PUT /api/messages/{message_id}
Authorization: Bearer {token}
Content-Type: application/json

{
  "content": "Updated message content"
}
```

#### Delete Message
```bash
DELETE /api/messages/{message_id}
Authorization: Bearer {token}
```

#### Add Reaction
```bash
POST /api/messages/{message_id}/reactions
Authorization: Bearer {token}
Content-Type: application/json

{
  "emoji": "👍"
}
```

### 📁 File Upload & Management

#### Upload File
```bash
POST /api/upload
Authorization: Bearer {token}
Content-Type: multipart/form-data

# Form data:
# file: [binary file data]
# chat_id: 2
# description: "Project documentation"
```

**Response:**
```json
{
  "success": true,
  "data": {
    "file_id": "f12345",
    "filename": "document.pdf",
    "file_size": 1048576,
    "mime_type": "application/pdf",
    "download_url": "/api/files/f12345/download",
    "preview_url": "/api/files/f12345/preview",
    "chat_id": 2
  },
  "message": "File uploaded successfully"
}
```

#### Download File
```bash
GET /api/files/{file_id}/download
Authorization: Bearer {token}
```

#### Get File Preview
```bash
GET /api/files/{file_id}/preview
Authorization: Bearer {token}
```

### 🔍 Search API

#### Search Messages
```bash
GET /api/search/messages?q=project&chat_id=2&limit=10
Authorization: Bearer {token}
```

**Query Parameters:**
- `q`: Search query string
- `chat_id`: Limit search to specific chat (optional)
- `workspace_id`: Limit search to workspace (optional)
- `limit`: Number of results (default: 10, max: 50)
- `offset`: Pagination offset

**Response:**
```json
{
  "success": true,
  "data": {
    "results": [
      {
        "id": 12345,
        "content": "How's the **project** going?",
        "chat_id": 2,
        "chat_name": "General Discussion",
        "sender_name": "John Doe",
        "created_at": "2023-12-01T12:00:00Z",
        "highlight": "How's the <mark>project</mark> going?"
      }
    ],
    "total_count": 1,
    "query": "project",
    "took_ms": 15
  }
}
```

#### Search Chats
```bash
GET /api/search/chats?q=discussion
Authorization: Bearer {token}
```

#### Search Users
```bash
GET /api/search/users?q=john
Authorization: Bearer {token}
```

### 🤖 Bot/AI Assistant API

#### Get Supported Languages
```bash
GET /api/bot/languages
Authorization: Bearer {token}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "languages": [
      {"code": "en", "name": "English"},
      {"code": "zh", "name": "Chinese"},
      {"code": "ja", "name": "Japanese"},
      {"code": "ko", "name": "Korean"},
      {"code": "es", "name": "Spanish"},
      {"code": "fr", "name": "French"},
      {"code": "de", "name": "German"},
      {"code": "ru", "name": "Russian"},
      {"code": "pt", "name": "Portuguese"},
      {"code": "it", "name": "Italian"}
    ]
  }
}
```

#### Translate Message
```bash
POST /api/bot/translate
Authorization: Bearer {token}
Content-Type: application/json

{
  "text": "Hello world",
  "target_language": "zh",
  "message_id": "msg_12345"
}
```

**Response:**
```json
{
  "success": true,
  "translation": "你好，世界",
  "source_language": "en",
  "target_language": "zh",
  "confidence": 0.95,
  "message_id": "msg_12345",
  "quota_used": 1,
  "quota_remaining": 19,
  "quota_limit": 20,
  "provider": "openai_gpt",
  "processing_time_ms": 935
}
```

#### Detect Language
```bash
POST /api/bot/detect-language
Authorization: Bearer {token}
Content-Type: application/json

{
  "text": "Bonjour le monde"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "detected_language": "fr",
    "confidence": 0.98,
    "language_name": "French"
  }
}
```

#### Get Bot Status
```bash
GET /api/bot/status
Authorization: Bearer {token}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "services": {
      "openai": {"status": "healthy", "latency_ms": 150},
      "translation": {"status": "active", "quota_used": 5, "quota_limit": 20},
      "language_detection": {"status": "active"}
    },
    "version": "0.1.0"
  }
}
```

### 📊 Analytics API

#### Get Usage Statistics
```bash
GET /api/analytics/usage?period=7d
Authorization: Bearer {token}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "period": "7d",
    "metrics": {
      "messages_sent": 1250,
      "active_users": 45,
      "files_uploaded": 23,
      "api_requests": 5420
    },
    "daily_breakdown": [
      {"date": "2023-12-01", "messages": 180, "users": 12},
      {"date": "2023-12-02", "messages": 195, "users": 15}
    ]
  }
}
```

### 🏥 Health & Status

#### System Health Check
```bash
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "services": [
    {"name": "database", "status": "healthy", "latency_ms": 5},
    {"name": "redis", "status": "healthy", "latency_ms": 2},
    {"name": "nats", "status": "healthy", "latency_ms": 1},
    {"name": "search", "status": "healthy", "latency_ms": 8}
  ],
  "timestamp": "2023-12-01T12:00:00Z",
  "version": "1.0.0"
}
```

#### Service Metrics
```bash
GET /metrics
```

Returns Prometheus-compatible metrics for monitoring.

---

## 3. WebSocket/SSE Real-time APIs

### 🔄 Server-Sent Events (SSE)

Fechatter uses Server-Sent Events for real-time communication. Connect to the SSE endpoint to receive live updates.

#### Establishing SSE Connection

```bash
curl -N -H "Accept: text/event-stream" \
  "http://localhost:8080/events?access_token=your_jwt_token"
```

**JavaScript Example:**
```javascript
const token = localStorage.getItem('auth_token');
const eventSource = new EventSource(`/events?access_token=${token}`);

eventSource.onopen = function(event) {
  console.log('SSE connection established');
};

eventSource.onmessage = function(event) {
  const data = JSON.parse(event.data);
  console.log('Received event:', data);
};

eventSource.onerror = function(event) {
  console.error('SSE connection error:', event);
};
```

#### Event Types

**Connection Confirmation:**
```json
{
  "type": "connection_confirmed",
  "user_id": 1,
  "timestamp": "2023-12-01T12:00:00Z",
  "connected_chats": [1, 2, 3]
}
```

**New Message:**
```json
{
  "type": "new_message",
  "message": {
    "id": 12345,
    "content": "Hello everyone!",
    "chat_id": 2,
    "sender_id": 3,
    "sender_name": "Jane Doe",
    "created_at": "2023-12-01T12:00:00Z"
  },
  "chat_id": 2
}
```

**Message Status Update:**
```json
{
  "type": "message_status",
  "message_id": 12345,
  "status": "delivered",
  "chat_id": 2,
  "timestamp": "2023-12-01T12:00:01Z"
}
```

**Typing Indicator:**
```json
{
  "type": "typing_started",
  "chat_id": 2,
  "user_id": 3,
  "user_name": "Jane Doe",
  "timestamp": "2023-12-01T12:00:00Z"
}
```

**User Presence:**
```json
{
  "type": "user_presence",
  "user_id": 3,
  "status": "online",
  "last_seen": "2023-12-01T12:00:00Z"
}
```

**Heartbeat/Ping:**
```json
{
  "type": "ping",
  "timestamp": "2023-12-01T12:00:00Z"
}
```

### 🔌 WebSocket Fallback

If SSE is not available, Fechatter provides WebSocket fallback:

```javascript
const ws = new WebSocket(`ws://localhost:8080/ws?access_token=${token}`);

ws.onopen = function(event) {
  console.log('WebSocket connection established');
};

ws.onmessage = function(event) {
  const data = JSON.parse(event.data);
  console.log('Received message:', data);
};

ws.onclose = function(event) {
  console.log('WebSocket connection closed');
};
```

---

## 4. Message Passing Protocols

### 📡 NATS JetStream Integration

Fechatter uses NATS JetStream for reliable message passing between services.

#### Event Subjects

**Message Events:**
```
fechatter.messages.message.created.v1
fechatter.messages.message.edited.v1
fechatter.messages.message.deleted.v1
```

**Chat Events:**
```
fechatter.chats.member.joined.v1
fechatter.chats.member.left.v1
fechatter.chats.created.v1
```

**User Events:**
```
fechatter.users.presence.changed.v1
fechatter.users.typing.started.v1
fechatter.users.typing.stopped.v1
```

**Analytics Events:**
```
fechatter.analytics.bot.*
fechatter.analytics.usage.*
fechatter.analytics.performance.*
```

#### Event Payload Format

**Enhanced Message Event:**
```json
{
  "event_id": "evt_12345",
  "event_type": "message_created",
  "timestamp": "2023-12-01T12:00:00Z",
  "trace_context": {
    "request_id": "req_67890",
    "user_id": "1",
    "workspace_id": "2"
  },
  "message": {
    "id": 12345,
    "content": "Hello everyone!",
    "chat_id": 2,
    "sender_id": 1,
    "sender_name": "John Doe",
    "message_type": "text",
    "created_at": "2023-12-01T12:00:00Z"
  },
  "recipients": [1, 2, 3, 4, 5],
  "chat_name": "General Discussion",
  "workspace_id": 2
}
```

#### Reliable Delivery

NATS JetStream ensures reliable message delivery with:
- **At-least-once delivery**: Messages are delivered at least once
- **Acknowledgment**: Consumers must acknowledge message processing
- **Retry mechanism**: Failed messages are retried automatically
- **Dead letter queue**: Failed messages after retries go to DLQ

---

## 5. Error Handling & Status Codes

### 📊 HTTP Status Codes

| Code | Meaning | Description |
|------|---------|-------------|
| 200 | OK | Request successful |
| 201 | Created | Resource created successfully |
| 400 | Bad Request | Invalid request parameters |
| 401 | Unauthorized | Authentication required or invalid |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Resource not found |
| 409 | Conflict | Resource already exists |
| 422 | Unprocessable Entity | Validation errors |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server error |
| 502 | Bad Gateway | Service unavailable |
| 503 | Service Unavailable | Temporary service unavailability |

### ❌ Error Response Format

All API errors follow a consistent format:

```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid request parameters",
    "details": {
      "field": "email",
      "issue": "Email format is invalid"
    },
    "request_id": "req_12345"
  },
  "timestamp": "2023-12-01T12:00:00Z"
}
```

### 🔍 Common Error Codes

#### Authentication Errors
```json
{
  "success": false,
  "error": {
    "code": "INVALID_TOKEN",
    "message": "JWT token is expired or invalid",
    "details": {
      "token_status": "expired",
      "expires_at": "2023-12-01T11:00:00Z"
    }
  }
}
```

#### Validation Errors
```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Request validation failed",
    "details": {
      "errors": [
        {
          "field": "email",
          "message": "Email is required"
        },
        {
          "field": "password",
          "message": "Password must be at least 8 characters"
        }
      ]
    }
  }
}
```

#### Rate Limit Errors
```json
{
  "success": false,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests",
    "details": {
      "limit": 100,
      "window": "1 hour",
      "reset_at": "2023-12-01T13:00:00Z"
    }
  }
}
```

#### Service Unavailable
```json
{
  "success": false,
  "error": {
    "code": "SERVICE_UNAVAILABLE",
    "message": "Translation service is temporarily unavailable",
    "details": {
      "service": "bot_server",
      "retry_after": 30
    }
  }
}
```

### 🔄 Retry Strategies

**Exponential Backoff for 5xx errors:**
```javascript
async function apiRequest(url, options, maxRetries = 3) {
  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      const response = await fetch(url, options);
      
      if (response.ok) {
        return await response.json();
      }
      
      if (response.status >= 500 && attempt < maxRetries) {
        const delay = Math.pow(2, attempt) * 1000; // 1s, 2s, 4s
        await new Promise(resolve => setTimeout(resolve, delay));
        continue;
      }
      
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    } catch (error) {
      if (attempt === maxRetries) throw error;
    }
  }
}
```

---

## 6. SDK Usage Examples

### 🚀 JavaScript/TypeScript SDK

#### Installation
```bash
npm install @fechatter/sdk
# or
yarn add @fechatter/sdk
```

#### Basic Setup
```typescript
import { FechatterSDK } from '@fechatter/sdk';

const fechatter = new FechatterSDK({
  baseUrl: 'http://localhost:8080',
  apiKey: 'your-api-key', // Optional for server-to-server
  timeout: 10000
});

// Authenticate with user credentials
await fechatter.auth.signIn({
  email: 'user@example.com',
  password: 'password123'
});
```

#### Chat Operations
```typescript
// Get all chats
const chats = await fechatter.chats.list();

// Create a new chat
const newChat = await fechatter.chats.create({
  name: 'Project Alpha',
  type: 'group',
  members: [1, 2, 3, 4]
});

// Send a message
const message = await fechatter.messages.send(chatId, {
  content: 'Hello team!',
  type: 'text'
});

// Get chat messages with pagination
const messages = await fechatter.messages.list(chatId, {
  limit: 20,
  offset: 0
});
```

#### Real-time Events
```typescript
// Connect to real-time events
const eventStream = fechatter.realtime.connect();

eventStream.on('message', (event) => {
  console.log('New message:', event.message);
});

eventStream.on('typing', (event) => {
  console.log(`${event.user_name} is typing...`);
});

eventStream.on('presence', (event) => {
  console.log(`User ${event.user_id} is ${event.status}`);
});

// Send typing indicator
await fechatter.realtime.startTyping(chatId);
await fechatter.realtime.stopTyping(chatId);
```

#### File Upload
```typescript
// Upload a file
const file = document.getElementById('fileInput').files[0];
const uploadResult = await fechatter.files.upload(file, {
  chatId: 2,
  description: 'Project documentation'
});

// Send file as message
await fechatter.messages.send(chatId, {
  type: 'file',
  fileId: uploadResult.file_id,
  content: 'Check out this document!'
});
```

#### Search
```typescript
// Search messages
const searchResults = await fechatter.search.messages({
  query: 'project deadline',
  chatId: 2, // Optional: limit to specific chat
  limit: 10
});

// Search users
const users = await fechatter.search.users({
  query: 'john'
});
```

#### Bot/AI Features
```typescript
// Translate text
const translation = await fechatter.bot.translate({
  text: 'Hello world',
  targetLanguage: 'zh',
  messageId: 'msg_12345'
});

// Detect language
const detection = await fechatter.bot.detectLanguage({
  text: 'Bonjour le monde'
});

// Get supported languages
const languages = await fechatter.bot.getSupportedLanguages();
```

#### Error Handling
```typescript
try {
  const message = await fechatter.messages.send(chatId, {
    content: 'Hello!'
  });
} catch (error) {
  if (error.code === 'RATE_LIMIT_EXCEEDED') {
    console.log(`Rate limited. Retry after: ${error.details.reset_at}`);
  } else if (error.code === 'UNAUTHORIZED') {
    // Refresh token or redirect to login
    await fechatter.auth.refresh();
  } else {
    console.error('Unexpected error:', error.message);
  }
}
```

### 🐍 Python SDK

#### Installation
```bash
pip install fechatter-sdk
```

#### Basic Usage
```python
from fechatter import FechatterClient
import asyncio

# Initialize client
client = FechatterClient(
    base_url="http://localhost:8080",
    timeout=10.0
)

async def main():
    # Authenticate
    auth_result = await client.auth.sign_in(
        email="user@example.com",
        password="password123"
    )
    
    # Get chats
    chats = await client.chats.list()
    
    # Send message
    message = await client.messages.send(
        chat_id=2,
        content="Hello from Python!",
        message_type="text"
    )
    
    # Real-time events
    async for event in client.realtime.stream():
        if event.type == "new_message":
            print(f"New message: {event.message.content}")

asyncio.run(main())
```

### 🦀 Rust SDK

#### Cargo.toml
```toml
[dependencies]
fechatter-sdk = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

#### Usage
```rust
use fechatter_sdk::{FechatterClient, auth::SignInRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = FechatterClient::new("http://localhost:8080")?;
    
    // Authenticate
    let auth_result = client.auth().sign_in(SignInRequest {
        email: "user@example.com".to_string(),
        password: "password123".to_string(),
    }).await?;
    
    // Send message
    let message = client.messages().send(2, "Hello from Rust!").await?;
    
    // Real-time events
    let mut stream = client.realtime().connect().await?;
    while let Some(event) = stream.next().await {
        match event? {
            Event::NewMessage { message, .. } => {
                println!("New message: {}", message.content);
            }
            Event::Typing { user_name, .. } => {
                println!("{} is typing...", user_name);
            }
            _ => {}
        }
    }
    
    Ok(())
}
```

### 🔧 Configuration Examples

#### Environment-based Configuration
```bash
# .env file
FECHATTER_BASE_URL=http://localhost:8080
FECHATTER_API_KEY=your-api-key
FECHATTER_TIMEOUT=10000
FECHATTER_RETRY_ATTEMPTS=3
FECHATTER_DEBUG=true
```

#### Advanced Client Configuration
```typescript
const fechatter = new FechatterSDK({
  baseUrl: process.env.FECHATTER_BASE_URL,
  timeout: 10000,
  retryConfig: {
    maxAttempts: 3,
    baseDelay: 1000,
    maxDelay: 10000,
    retryCondition: (error) => error.status >= 500
  },
  middleware: [
    // Request middleware
    (request) => {
      request.headers['X-Client-Version'] = '1.0.0';
      return request;
    },
    // Response middleware
    (response) => {
      console.log(`API call took ${response.timing}ms`);
      return response;
    }
  ]
});
```

---

## 📚 Additional Resources

### 🔗 Related Documentation
- [System Architecture](./ARCHITECTURE.md) - Technical design and patterns
- [Deployment Guide](./DEPLOYMENT.md) - Production deployment options
- [Quick Start Guide](./QUICK_START.md) - Get running in 2 minutes

### 🛠️ Development Tools
- **Interactive API Explorer**: `http://localhost:8080/docs`
- **Health Monitoring**: `http://localhost:8080/health`
- **Metrics Dashboard**: `http://localhost:8080/metrics`

### 🔍 Testing & Debugging
- Use browser DevTools Network tab to inspect API calls
- Check SSE connection in DevTools EventSource
- Monitor server logs for debugging: `docker logs fechatter-server`
- Test API endpoints with tools like Postman or curl

### 📞 Support
- **Documentation**: Comprehensive guides in `/docs`
- **GitHub Issues**: Report bugs and feature requests
- **Community**: Join our Discord for discussions
- **Enterprise**: Contact sales for enterprise support

---

**API Version**: v1.0  
**Last Updated**: 2024-12-29  
**Generated**: Consolidated from Fechatter documentation archive