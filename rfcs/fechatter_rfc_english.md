# Fechatter System Design - Key Points

## 1. System Overview

Fechatter is a real-time chat application built with Rust, providing secure and efficient communication within workspaces through different chat types (single, group, and channels).

### Core Components

1. **fechatter_server**: Main chat application server
   - User authentication and registration
   - Chat and workspace management
   - Message handling

2. **notify_server**: Notification service
   - Real-time updates via Server-Sent Events (SSE)
   - Push notification capabilities

3. **PostgreSQL Database**: Persistent data storage

## 2. Architecture

### Component Interactions

- Clients connect to `fechatter_server` for authentication and data operations
- Clients establish SSE connections with `notify_server` for real-time updates
- Both servers interact with the PostgreSQL database

### Authentication Flow

- JWT-based authentication
- Tokens include user ID and expiration time
- Middleware validates tokens for protected endpoints

### Message Flow

- Messages sent to `fechatter_server`
- Server validates user's chat membership
- Messages stored in database
- Notification events trigger updates via `notify_server`

## 3. Data Model

### Core Entities

- **User**: Account information and authentication
- **Chat**: Conversation container with different types
  - Single (1:1)
  - Group
  - Channels (Public/Private)
- **Message**: Content sent within chats
- **ChatMember**: User-chat relationship with roles
- **Workspace**: Organizational unit containing chats

### Entity Relationships

- User to Chat: Many-to-many through ChatMember
- Chat to Message: One-to-many
- Workspace to Chat: One-to-many

## 4. API Design

### Key Endpoints

- **Authentication**: `/api/auth/signup`, `/api/auth/signin`
- **Chat Management**: `/api/chats`, `/api/chats/:id`
- **Messages**: `/api/chats/:id/messages`
- **Chat Members**: `/api/chats/:id/members`
- **Workspaces**: `/api/workspaces`
- **Notifications**: `/api/notifications/events` (SSE)

## 5. Security Considerations

- JWT tokens for stateless authentication
- Argon2 password hashing
- Role-based access control
- Chat membership validation

## 6. Performance Optimizations

- Chat list caching (30-second TTL)
- Message pagination
- Database indexes for frequent queries
- Connection pooling

## 7. Technology Stack

- **Rust**: Core programming language
- **Tokio**: Asynchronous runtime
- **Axum**: Web framework
- **SQLx**: Database interaction
- **PostgreSQL**: Database system
