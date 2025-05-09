# Fechatter Detailed Documentation

This document provides detailed information about the Fechatter chat application, including its architecture, features, API endpoints, and chat types.

## System Architecture

Fechatter consists of two main services:

- **fechatter_server**: Core chat functionality and API
  - Handles user authentication and registration
  - Manages chat rooms, memberships, and messages
  - Implements workspace functionality

- **notify_server**: A separate service for handling notifications
  - Implements Server-Sent Events (SSE) for real-time notifications
  - Shares some infrastructure with the main server

## Technology Stack

- **Backend**: Rust with Axum web framework
- **Database**: PostgreSQL with SQLx
- **Authentication**: JWT (JSON Web Tokens)
- **Real-time**: Server-Sent Events (SSE)
- **Testing**: Comprehensive unit and integration tests
- **CI/CD**: GitHub Actions workflow

## Project Structure

```
fechatter/
├── fechatter_core/         # Core logic and shared functionalities
│   └── src/
├── fechatter_macro/        # Procedural macros
│   └── src/
├── fechatter_server/       # Main chat application server
│   └── src/
│       ├── handlers/       # HTTP request handlers
│       ├── middlewares/    # HTTP middleware components
│       ├── models/         # Data models and database interactions
│       ├── services/       # Business logic services
│       ├── utils/          # Utility functions
│       ├── tests/          # Integration and unit tests for the server
│       ├── config.rs       # Configuration management
│       ├── error.rs        # Error handling
│       ├── lib.rs          # Core application setup
│       └── main.rs         # Application entry point
│
├── notify_server/          # Notification service
│   └── src/
│       ├── config.rs       # Configuration for the notify server
│       ├── error.rs        # Error handling for the notify server
│       ├── lib.rs          # Notification service core
│       ├── main.rs         # Notification service entry point
│       ├── notify.rs       # Core notification logic
│       ├── sse.rs          # Server-Sent Events implementation
│       └── utils.rs        # Utility functions for the notify server
│
├── migrations/             # Database migration files
├── .env.example            # Example environment variables
└── app.yml.example         # Example application configuration
```

## API Endpoints

### Authentication
- `POST /api/signin` - Login and get JWT tokens
- `POST /api/signup` - Register a new user
- `POST /api/refresh` - Refresh authentication token
- `POST /api/logout` - Logout and invalidate token

### Chat Management
- `GET /api/chat` - List all chats for the authenticated user
- `POST /api/chat` - Create a new chat
- `PATCH /api/chat/{id}` - Update chat details
- `DELETE /api/chat/{id}` - Delete a chat

### Chat Members
- `GET /api/chat/{id}/members` - List members of a chat
- `POST /api/chat/{id}/members` - Add members to a chat
- `DELETE /api/chat/{id}/members` - Remove members from a chat
- `PATCH /api/chat/{id}/members/{member_id}` - Transfer chat ownership

### Workspace
- `GET /api/users` - List all users in the workspace

## Chat Types

Fechatter supports four distinct chat types, each with specific membership rules:

1. **Single Chat**: One-on-one private conversations between two users.
   - Must have exactly two members
   - Cannot create a single chat with yourself

2. **Group Chat**: Multi-user conversations.
   - Requires at least three members (including creator)
   - All members can send messages

3. **Private Channel**: Invite-only topic-based channels.
   - Creator is automatically a member
   - Additional members can be added by invitation

4. **Public Channel**: Open topic-based channels.
   - Initially only has the creator as a member
   - Users can join without invitation

## Features

### Multiple Chat Types
- Single: One-on-one private conversations
- Group: Multi-user conversations
- Private Channels: Invite-only topic-based channels
- Public Channels: Open topic-based channels

### Authentication & Security
- JWT-based authentication
- Refresh token support
- Secure password hashing with Argon2
- Permission-based access control

### Real-time Communication
- Server-Sent Events (SSE) for real-time notifications
- Efficient caching for chat lists
- Low-latency message delivery

### Robust Architecture
- RESTful API design
- PostgreSQL database for data persistence
- Comprehensive error handling
- Modular code organization