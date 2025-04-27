# Fechatter

A real-time chat application built with Rust, featuring workspaces, multiple chat types, and secure authentication.

## Overview

Fechatter is a modern chat platform built with Rust that provides a secure and efficient way for users to communicate within workspaces. The system consists of two main services:

- **fechatter_server**: Core chat functionality and API
- **notify_server**: Real-time notification delivery

The application enables users to:
- Create and join workspaces
- Participate in different types of chats (one-on-one, group chats, channels)
- Send and receive messages in real-time
- Manage chat memberships
- Authenticate securely using JWT tokens

## Features

- **Multiple Chat Types**:
  - Single: One-on-one private conversations
  - Group: Multi-user conversations
  - Private Channels: Invite-only topic-based channels
  - Public Channels: Open topic-based channels

- **Authentication & Security**:
  - JWT-based authentication
  - Refresh token support
  - Secure password hashing with Argon2
  - Permission-based access control

- **Real-time Communication**:
  - Server-Sent Events (SSE) for real-time notifications
  - Efficient caching for chat lists
  - Low-latency message delivery

- **Robust Architecture**:
  - RESTful API design
  - PostgreSQL database for data persistence
  - Comprehensive error handling
  - Modular code organization

## Technology Stack

- **Backend**: Rust with Axum web framework
- **Database**: PostgreSQL with SQLx
- **Authentication**: JWT (JSON Web Tokens)
- **Real-time**: Server-Sent Events (SSE)
- **Testing**: Comprehensive unit and integration tests
- **CI/CD**: GitHub Actions workflow

## Prerequisites

- Rust (latest stable)
- PostgreSQL
- Docker (optional, for containerized deployment)

## Setup

1. Clone the repository:
   ```
   git clone https://github.com/Kevinzh0C/fechatter.git
   cd fechatter
   ```

2. Set up the database:
   ```
   createdb fechatter
   ```

3. Configure environment variables:
   Copy the `.env.example` file to `.env` and update the values as needed.
   ```
   cp .env.example .env
   ```

4. Run database migrations:
   ```
   sqlx migrate run
   ```

5. Build and run the project:
   ```
   cargo run
   ```

## Project Structure

```
fechatter/
├── fechatter_server/       # Main chat application server
│   └── src/
│       ├── handlers/       # HTTP request handlers
│       ├── middlewares/    # HTTP middleware components
│       ├── models/         # Data models and database interactions
│       ├── services/       # Business logic services
│       ├── utils/          # Utility functions
│       ├── config.rs       # Configuration management
│       ├── error.rs        # Error handling
│       ├── lib.rs          # Core application setup
│       └── main.rs         # Application entry point
│
├── notify_server/          # Notification service
│   └── src/
│       ├── sse.rs          # Server-Sent Events implementation
│       ├── lib.rs          # Notification service core
│       └── main.rs         # Notification service entry point
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

## Development

### Running Tests

```
cargo test
```

### Formatting Code

```
cargo fmt
```

### Linting

```
cargo clippy
```

## License

MIT
