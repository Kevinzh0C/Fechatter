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

## Key Features

- Multiple chat types (Single, Group, Private Channels, Public Channels)
- JWT-based authentication and security
- Real-time communication using Server-Sent Events (SSE)
- RESTful API design with PostgreSQL persistence

## Upcoming Features

- Meilisearch integration for powerful message search capabilities
- NATS JetStream for enhanced real-time messaging infrastructure
- Backend and frontend integration with UI
- ChatGPT chatbot service integration
- OpenTelemetry monitoring and Pingora gateway integration

For detailed documentation on architecture, API endpoints, chat types, and more, see the [detailed documentation](./docs/detailed_documentation.md).

For information about our development roadmap and upcoming features, see the [Project Roadmap](./docs/roadmap.md).

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

## Development

For development instructions including testing, formatting, and linting, see the [detailed documentation](./docs/detailed_documentation.md).

## License

MIT
