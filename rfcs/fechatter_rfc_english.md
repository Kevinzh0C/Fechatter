# Fechatter System Design RFC

## 1. Introduction

### 1.1 Overview

Fechatter is a real-time chat application built with Rust, designed to provide secure, efficient, and scalable communication capabilities. The system enables users to communicate within workspaces through different types of chats (single, group, and channels) while maintaining high performance and security standards.

### 1.2 Motivation

Modern communication tools require robust security, high performance, and excellent user experience. Fechatter aims to leverage Rust's safety guarantees and performance characteristics to build a chat platform that excels in these areas. By using an asynchronous architecture and efficient database design, Fechatter provides a responsive and reliable communication experience.

### 1.3 Goals

- Create a secure, real-time chat application with multiple chat types
- Implement efficient authentication and authorization mechanisms
- Design a scalable architecture that can handle high message throughput
- Provide a clean API for future client implementations
- Ensure data integrity and privacy throughout the system

## 2. Architecture

### 2.1 System Components

Fechatter consists of two main services:

1. **fechatter_server**: The core chat application server
   - Handles user authentication and registration
   - Manages chat rooms, memberships, and messages
   - Implements workspace functionality
   - Processes API requests and database operations

2. **notify_server**: A dedicated notification service
   - Manages real-time notifications using Server-Sent Events (SSE)
   - Provides push notification capabilities
   - Maintains client connections for real-time updates

3. **PostgreSQL Database**: Persistent storage for all application data
   - Stores user accounts, chat information, and messages
   - Maintains relationship data between entities
   - Provides transaction support and data integrity

### 2.2 Component Interactions

The system components interact as follows:

1. Clients connect to `fechatter_server` for authentication and data operations
2. Clients establish SSE connections with `notify_server` for real-time updates
3. When data changes occur in `fechatter_server`, notifications are sent to relevant clients via `notify_server`
4. Both servers interact with the PostgreSQL database for data persistence

### 2.3 Authentication Flow

1. Client sends login credentials to `fechatter_server`
2. Server validates credentials against stored user data
3. Upon successful authentication, server generates a JWT token
4. Client uses the JWT token for subsequent API requests
5. Token validation occurs in middleware for protected endpoints

### 2.4 Message Flow

1. Client sends a message to a specific chat via `fechatter_server`
2. Server validates the user's membership in the chat
3. Message is stored in the database with metadata
4. Server triggers a notification event
5. `notify_server` pushes the notification to all relevant clients
6. Receiving clients update their UI with the new message

## 3. Data Model

### 3.1 Core Entities

#### User
```rust
struct User {
    id: Uuid,
    email: String,
    password_hash: String,
    full_name: String,
    avatar_url: Option<String>,
    status: UserStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

enum UserStatus {
    Active,
    Suspended,
}
```

#### Chat
```rust
struct Chat {
    id: Uuid,
    name: String,
    chat_type: ChatType,
    workspace_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

enum ChatType {
    Single,
    Group,
    PrivateChannel,
    PublicChannel,
}
```

#### Message
```rust
struct Message {
    id: Uuid,
    chat_id: Uuid,
    sender_id: Uuid,
    content: String,
    created_at: DateTime<Utc>,
}
```

#### ChatMember
```rust
struct ChatMember {
    user_id: Uuid,
    chat_id: Uuid,
    role: ChatMemberRole,
    joined_at: DateTime<Utc>,
}

enum ChatMemberRole {
    Member,
    Admin,
    Owner,
}
```

#### Workspace
```rust
struct Workspace {
    id: Uuid,
    name: String,
    description: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

### 3.2 Entity Relationships

- **User to Chat**: Many-to-many relationship through ChatMember
- **Chat to Message**: One-to-many relationship (a chat contains many messages)
- **User to Message**: One-to-many relationship (a user sends many messages)
- **Workspace to Chat**: One-to-many relationship (a workspace contains many chats)
- **User to Workspace**: Many-to-many relationship through WorkspaceMember

### 3.3 Database Schema

The database schema includes the following tables:

- `users`: Stores user account information
- `chats`: Contains chat room metadata
- `messages`: Stores all chat messages
- `chat_members`: Junction table for user-chat relationships
- `workspaces`: Stores workspace information
- `workspace_members`: Junction table for user-workspace relationships

Key indexes:
- Email index on users table for quick user lookup
- Composite index on chat_id and created_at for efficient message retrieval
- Indexes on foreign keys for relationship queries

## 4. API Design

### 4.1 Authentication API

#### User Registration
```
POST /api/auth/signup
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "secure_password",
  "full_name": "John Doe"
}
```

Response:
```
Status: 201 Created
{
  "id": "uuid",
  "email": "user@example.com",
  "full_name": "John Doe",
  "token": "jwt_token"
}
```

#### User Login
```
POST /api/auth/signin
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "secure_password"
}
```

Response:
```
Status: 200 OK
{
  "id": "uuid",
  "email": "user@example.com",
  "full_name": "John Doe",
  "token": "jwt_token"
}
```

### 4.2 Chat API

#### List Chats
```
GET /api/chats
Authorization: Bearer jwt_token
```

Response:
```
Status: 200 OK
{
  "chats": [
    {
      "id": "uuid",
      "name": "General",
      "chat_type": "PublicChannel",
      "last_message": {
        "content": "Hello world",
        "sender_name": "John Doe",
        "created_at": "2023-01-01T12:00:00Z"
      },
      "unread_count": 5
    },
    ...
  ]
}
```

#### Create Chat
```
POST /api/chats
Authorization: Bearer jwt_token
Content-Type: application/json

{
  "name": "Project Discussion",
  "chat_type": "Group",
  "workspace_id": "uuid",
  "member_ids": ["uuid1", "uuid2"]
}
```

Response:
```
Status: 201 Created
{
  "id": "uuid",
  "name": "Project Discussion",
  "chat_type": "Group",
  "created_at": "2023-01-01T12:00:00Z"
}
```

#### Update Chat
```
PUT /api/chats/:id
Authorization: Bearer jwt_token
Content-Type: application/json

{
  "name": "Updated Name"
}
```

Response:
```
Status: 200 OK
{
  "id": "uuid",
  "name": "Updated Name",
  "chat_type": "Group",
  "updated_at": "2023-01-01T12:30:00Z"
}
```

#### Delete Chat
```
DELETE /api/chats/:id
Authorization: Bearer jwt_token
```

Response:
```
Status: 204 No Content
```

### 4.3 Message API

#### Get Messages
```
GET /api/chats/:id/messages?limit=50&before=message_id
Authorization: Bearer jwt_token
```

Response:
```
Status: 200 OK
{
  "messages": [
    {
      "id": "uuid",
      "content": "Hello everyone!",
      "sender": {
        "id": "uuid",
        "name": "John Doe",
        "avatar_url": "https://example.com/avatar.jpg"
      },
      "created_at": "2023-01-01T12:00:00Z"
    },
    ...
  ],
  "has_more": true
}
```

#### Send Message
```
POST /api/chats/:id/messages
Authorization: Bearer jwt_token
Content-Type: application/json

{
  "content": "Hello world!"
}
```

Response:
```
Status: 201 Created
{
  "id": "uuid",
  "content": "Hello world!",
  "created_at": "2023-01-01T12:00:00Z"
}
```

### 4.4 Notification API

#### SSE Connection
```
GET /api/notifications/events
Authorization: Bearer jwt_token
Accept: text/event-stream
```

Response:
```
Status: 200 OK
Content-Type: text/event-stream

event: message
data: {"chat_id": "uuid", "message_id": "uuid"}

event: chat_update
data: {"chat_id": "uuid", "type": "name_changed"}
```

## 5. Security Considerations

### 5.1 Authentication

- JWT tokens are used for stateless authentication
- Tokens include expiration time to limit session duration
- Passwords are hashed using Argon2 before storage
- Rate limiting is implemented for authentication endpoints

### 5.2 Authorization

- Middleware verifies user permissions for each request
- Chat membership is validated for all chat operations
- Role-based access control for administrative actions
- Workspace membership verification for workspace resources

### 5.3 Data Protection

- All database connections use TLS encryption
- Sensitive data is encrypted at rest
- Input validation prevents injection attacks
- CORS policies restrict cross-origin requests

## 6. Performance Considerations

### 6.1 Caching

- Chat list caching with a 30-second TTL
- User profile caching to reduce database load
- Message pagination to limit response size
- Connection pooling for database efficiency

### 6.2 Database Optimization

- Indexes on frequently queried columns
- Composite indexes for complex queries
- Query optimization for message retrieval
- Connection pooling for efficient resource usage

### 6.3 Concurrency

- Asynchronous request handling with Tokio
- Non-blocking I/O operations
- Database connection pooling
- Efficient resource utilization

## 7. Configuration Management

### 7.1 Configuration Sources

- YAML configuration files (`app.yml`)
- Environment variables for deployment-specific settings
- Sensible defaults for development environments
- Separate configurations for each service

### 7.2 Configuration Parameters

- Server settings (host, port)
- Database connection parameters
- JWT secret and expiration time
- Logging configuration
- Cache TTL values

## 8. Implementation Details

### 8.1 Technology Stack

- **Rust**: Core programming language
- **Tokio**: Asynchronous runtime
- **Axum**: Web framework
- **SQLx**: Database interaction
- **PostgreSQL**: Database system
- **jsonwebtoken**: JWT implementation
- **tracing**: Logging and instrumentation

### 8.2 Project Structure

```
fechatter/
├── Cargo.toml
├── fechatter_server/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── handlers/
│   │   │   ├── auth.rs
│   │   │   ├── chat.rs
│   │   │   ├── messages.rs
│   │   │   └── workspace.rs
│   │   ├── middlewares/
│   │   │   └── bearer_auth.rs
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   ├── user.rs
│   │   │   └── chat.rs
│   │   └── utils/
│   │       └── jwt.rs
│   └── migrations/
│       └── 0001_initial_schema.sql
├── notify_server/
│   └── src/
│       ├── main.rs
│       └── sse.rs
└── app.yml
```

### 8.3 Key Components

- **AppState**: Central state container with shared resources
- **Router**: Axum router with all API endpoints
- **TokenManager**: JWT token generation and validation
- **Error Handling**: Centralized error types and responses

## 9. Future Work

### 9.1 Feature Enhancements

- File sharing capabilities
- End-to-end encryption for messages
- Message reactions and threading
- User presence indicators
- Voice and video chat integration

### 9.2 Technical Improvements

- Horizontal scaling for high availability
- WebSocket support for bidirectional communication
- Message queue integration for reliable delivery
- Full-text search for message content
- Metrics and monitoring infrastructure

## 10. Conclusion

Fechatter provides a robust foundation for a real-time chat application with its focus on security, performance, and scalability. The system's architecture, built on Rust and modern asynchronous patterns, ensures efficient resource utilization while maintaining high throughput. The clear separation of concerns between the main server and notification service allows for independent scaling and maintenance.

By implementing this design, Fechatter will deliver a secure, responsive, and feature-rich chat experience that can evolve to meet future requirements.
