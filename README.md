# Fechatter

A real-time chat application built with Rust, Axum, SQLx, and WebSockets.

## Features

- User authentication with JWT
- Real-time messaging with WebSockets
- PostgreSQL database for data persistence
- RESTful API for user and chat room management
- Secure password hashing with Argon2

## Prerequisites

- Rust (latest stable)
- PostgreSQL
- Docker (optional, for containerized deployment)

## Setup

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/fechatter.git
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

4. Build and run the project:
   ```
   cargo run
   ```

## Project Structure

```
src/
├── api/          # API routes and handlers
├── auth/         # Authentication logic
├── chat/         # WebSocket chat implementation
├── db/           # Database connection and queries
└── main.rs       # Application entry point
```

## API Endpoints

### Authentication
- `POST /api/v1/auth/register` - Register a new user
- `POST /api/v1/auth/login` - Login and get a JWT token

### Chat
- `GET /chat/ws/:user_id/:username` - WebSocket endpoint for chat
- `GET /chat/active` - Get active users

## WebSocket Protocol

Messages are sent and received as JSON objects:

```json
{
  "type": "message",
  "user_id": "123",
  "username": "john_doe",
  "message": "Hello, world!",
  "timestamp": "2023-06-15T14:30:00Z"
}
```

## License

MIT
